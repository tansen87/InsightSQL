use std::{collections::HashMap, error::Error, fs::File, sync::OnceLock, time::Instant};

use cpc::{eval, units::Unit};
use dynfmt::Format;
use rayon::{
  iter::{IndexedParallelIterator, ParallelIterator},
  prelude::IntoParallelRefIterator,
};
use regex::Regex;
use smallvec::SmallVec;

use crate::detect::detect_separator;

#[macro_export]
macro_rules! regex_oncelock {
  ($re:literal $(,)?) => {{
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    #[allow(clippy::regex_creation_in_loops)] // false positive as we use oncelock
    RE.get_or_init(|| regex::Regex::new($re).expect("Invalid regex"))
  }};
}

static ROUND_PLACES: OnceLock<u32> = OnceLock::new();

#[derive(Clone, PartialEq)]
enum Operations {
  Copy,
  Len,
  Lower,
  Upper,
  Trim,
  Ltrim,
  Rtrim,
  Replace,
  Round,
}

impl Operations {
  fn from_str(op: &str) -> Result<Self, Box<dyn Error>> {
    match op.to_lowercase().as_str() {
      "copy" => Ok(Operations::Copy),
      "len" => Ok(Operations::Len),
      "lower" => Ok(Operations::Lower),
      "upper" => Ok(Operations::Upper),
      "trim" => Ok(Operations::Trim),
      "ltrim" => Ok(Operations::Ltrim),
      "rtrim" => Ok(Operations::Rtrim),
      "replace" => Ok(Operations::Replace),
      "round" => Ok(Operations::Round),
      _ => Err(Box::<dyn Error>::from(format!("Unknown '{op}' operation"))),
    }
  }
}

#[derive(PartialEq)]
enum ApplyCmd {
  Operations,
  CalcConv,
  DynFmt,
}

async fn get_header(file_path: String) -> Result<Vec<HashMap<String, String>>, Box<dyn Error>> {
  let sep = match detect_separator(&file_path) {
    Some(separator) => separator as u8,
    None => b',',
  };

  let mut rdr = csv::ReaderBuilder::new()
    .delimiter(sep)
    .has_headers(true)
    .from_reader(File::open(&file_path)?);

  let headers = rdr.headers()?;

  let hs: Vec<HashMap<String, String>> = headers
    .iter()
    .map(|header| {
      let mut map = HashMap::new();
      let header_str = header.to_string();
      map.insert("value".to_string(), header_str.clone());
      map.insert("label".to_string(), header_str);
      map
    })
    .collect();

  Ok(hs)
}

fn replace_column_value(
  record: &csv::StringRecord,
  column_index: usize,
  new_value: &str,
) -> csv::StringRecord {
  record
    .into_iter()
    .enumerate()
    .map(|(i, v)| if i == column_index { new_value } else { v })
    .collect()
}

fn round_num(dec_f64: f64, places: u32) -> String {
  use rust_decimal::{Decimal, RoundingStrategy};

  // if places is the sentinel value 9999, we don't round, just return the number as is
  if places == 9999 {
    return ryu::Buffer::new().format(dec_f64).to_owned();
  }

  // use from_f64_retain, so we have all the excess bits before rounding with
  // round_dp_with_strategy as from_f64 will prematurely round when it drops the excess bits
  let Some(dec_num) = Decimal::from_f64_retain(dec_f64) else {
    return String::new();
  };

  // round using Midpoint Nearest Even Rounding Strategy AKA "Bankers Rounding."
  // https://docs.rs/rust_decimal/latest/rust_decimal/enum.RoundingStrategy.html#variant.MidpointNearestEven
  // we also normalize to remove trailing zeroes and to change -0.0 to 0.0.
  dec_num
    .round_dp_with_strategy(places, RoundingStrategy::MidpointNearestEven)
    .normalize()
    .to_string()
}

fn validate_operations(
  operations: &Vec<&str>,
  comparand: &str,
  replacement: &str,
  new_column: Option<&String>,
  formatstr: &str,
) -> Result<SmallVec<[Operations; 4]>, Box<dyn Error>> {
  let mut copy_invokes = 0_u8;
  let mut replace_invokes = 0_u8;

  let mut ops_vec = SmallVec::with_capacity(operations.len());

  for op in operations {
    let operation = Operations::from_str(op)?;
    match operation {
      Operations::Copy => {
        if new_column.is_none() {
          return Err("new_column is required for copy operation.".into());
        }
        copy_invokes = copy_invokes.saturating_add(1);
      }
      Operations::Replace => {
        if comparand.is_empty() || replacement.is_empty() {
          return Err("comparand and replacement are required for replace operation.".into());
        }
        replace_invokes = replace_invokes.saturating_add(1);
      }
      Operations::Round => {
        if ROUND_PLACES
          .set(formatstr.parse::<u32>().unwrap_or(2))
          .is_err()
        {
          return Err("Cannot initialize Round precision.".into());
        };
      }
      _ => {}
    }
    ops_vec.push(operation);
  }
  if copy_invokes > 1 || replace_invokes > 1 {
    return Err(
            "you can only use copy({copy_invokes}), replace({replace_invokes}), ONCE per operation series.".into()
        );
  };

  Ok(ops_vec) // no validation errors
}

fn apply_operations(
  ops_vec: &SmallVec<[Operations; 4]>,
  cell: &mut String,
  comparand: &str,
  replacement: &str,
) {
  for op in ops_vec {
    match op {
      Operations::Len => {
        itoa::Buffer::new().format(cell.len()).clone_into(cell);
      }
      Operations::Lower => {
        *cell = cell.to_lowercase();
      }
      Operations::Upper => {
        *cell = cell.to_uppercase();
      }
      Operations::Trim => {
        *cell = String::from(cell.trim());
      }
      Operations::Ltrim => {
        *cell = String::from(cell.trim_start());
      }
      Operations::Rtrim => {
        *cell = String::from(cell.trim_end());
      }
      Operations::Replace => {
        *cell = cell.replace(comparand, replacement);
      }
      Operations::Round => {
        if let Ok(num) = cell.parse::<f64>() {
          // safety: we set ROUND_PLACES in validate_operations()
          *cell = round_num(num, *ROUND_PLACES.get().unwrap());
        }
      }
      Operations::Copy => {} // copy is a noop
    }
  }
}

async fn apply_perform(
  file_path: String,
  select_columns: String,
  apply_mode: String,
  operations: &str,
  comparand: String,
  replacement: String,
  formatstr: String,
  new_column: bool,
  output_path: String,
) -> Result<(), Box<dyn Error>> {
  let select_columns: Vec<&str> = select_columns.split('|').collect();
  let sep = match detect_separator(&file_path) {
    Some(separator) => separator as u8,
    None => b',',
  };

  let new_column: Option<String> = if new_column {
    Some(
      select_columns
        .iter()
        .map(|col| format!("{col}_new"))
        .collect(),
    )
  } else {
    None
  };

  let mut rdr = csv::ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(File::open(file_path)?);
  let mut wtr = csv::WriterBuilder::new()
    .delimiter(sep)
    .from_path(output_path)?;

  let headers = rdr.byte_headers()?;

  let header_map: HashMap<_, _> = headers
    .iter()
    .enumerate()
    .map(|(i, field)| (field.to_vec(), i))
    .collect();
  let select_column_bytes: Vec<_> = select_columns
    .iter()
    .map(|&col| col.as_bytes().to_vec())
    .collect();
  let column_index_res: Result<Vec<usize>, String> = select_column_bytes
    .iter()
    .map(|col_bytes| {
      header_map
        .get(col_bytes)
        .ok_or_else(|| {
          format!(
            "The column for {:?} was not found in the headers.",
            String::from_utf8_lossy(col_bytes)
          )
        })
        .map(|&idx| idx)
    })
    .collect();

  let column_index = column_index_res?;
  let column_index_next = *column_index.iter().next().unwrap();

  let mut headers = rdr.headers()?.clone();

  if let Some(new_column) = &new_column {
    headers.push_field(new_column);
  }
  wtr.write_record(&headers)?;

  let dynfmt_template =
    if (apply_mode.to_lowercase() == "calcconv") || (apply_mode.to_lowercase() == "dynfmt") {
      let mut dynfmt_template_wrk = formatstr.clone();
      let mut dynfmt_fields = Vec::new();

      // first, get the fields used in the dynfmt template
      let formatstr_re: &'static Regex = crate::regex_oncelock!(r"\{(?P<key>\w+)?\}");
      for format_fields in formatstr_re.captures_iter(&formatstr) {
        // safety: we already checked that the regex match is valid
        dynfmt_fields.push(format_fields.name("key").unwrap().as_str());
      }
      // we sort the fields so we can do binary_search
      dynfmt_fields.sort_unstable();

      // now, get the indices of the columns for the lookup vec
      for (i, field) in headers.iter().enumerate() {
        if dynfmt_fields.binary_search(&field).is_ok() {
          let field_with_curly = format!("{{{field}}}");
          let field_index = format!("{{{i}}}");
          dynfmt_template_wrk = dynfmt_template_wrk.replace(&field_with_curly, &field_index);
        }
      }

      dynfmt_template_wrk.to_string()
    } else {
      String::new()
    };

  let mut ops_vec = SmallVec::<[Operations; 4]>::new();

  let apply_cmd = if apply_mode.to_lowercase() == "operations" {
    match validate_operations(
      &operations.split('|').collect(),
      &comparand,
      &replacement,
      new_column.as_ref(),
      &formatstr,
    ) {
      Ok(operations_vec) => ops_vec = operations_vec,
      Err(e) => return Err(e),
    }
    ApplyCmd::Operations
  } else if apply_mode.to_lowercase() == "calcconv" {
    ApplyCmd::CalcConv
  } else {
    ApplyCmd::DynFmt
  };

  #[allow(unused_assignments)]
  let mut batch_record = csv::StringRecord::new();

  // reuse batch buffers
  let batchsize = 50_000;
  let mut batch = Vec::with_capacity(batchsize);
  let mut batch_results = Vec::with_capacity(batchsize);

  // main loop to read CSV and construct batches for parallel processing.
  // each batch is processed via Rayon parallel iterator.
  // loop exits when batch is empty.
  'batch_loop: loop {
    for _ in 0..batchsize {
      match rdr.read_record(&mut batch_record) {
        Ok(true) => batch.push(std::mem::take(&mut batch_record)),
        Ok(false) => break, // nothing else to add to batch
        Err(e) => {
          return Err(format!("Error reading file: {e}").into());
        }
      }
    }

    if batch.is_empty() {
      // break out of infinite loop when at EOF
      break 'batch_loop;
    }

    // do actual apply command via Rayon parallel iterator
    batch
      .par_iter()
      .with_min_len(1024)
      .map(|record_item| {
        let mut record = record_item.clone();
        match apply_cmd {
          ApplyCmd::Operations => {
            let mut cell = String::new();
            for col_index in &*column_index {
              record[*col_index].clone_into(&mut cell);
              apply_operations(&ops_vec, &mut cell, &comparand, &replacement);
              if new_column.is_some() {
                record.push_field(&cell);
              } else {
                record = replace_column_value(&record, *col_index, &cell);
              }
            }
          }
          ApplyCmd::CalcConv => {
            let result = if record[column_index_next].is_empty() {
              String::new()
            } else {
              let mut cell = record[column_index_next].to_owned();
              let mut record_vec: Vec<String> = Vec::with_capacity(record.len());
              for field in &record {
                record_vec.push(field.to_string());
              }
              if let Ok(formatted) = dynfmt::SimpleCurlyFormat.format(&dynfmt_template, record_vec)
              {
                cell = formatted.to_string();
              }

              let mut append_unit = false;
              let cell_for_eval = if cell.ends_with("<UNIT>") {
                append_unit = true;
                cell.trim_end_matches("<UNIT>")
              } else {
                &cell
              };
              match eval(cell_for_eval, true, Unit::Celsius, false) {
                Ok(answer) => {
                  if append_unit {
                    format!("{} {:?}", answer.value, answer.unit)
                  } else {
                    answer.value.to_string()
                  }
                }
                Err(e) => {
                  format!("ERROR: {e}")
                }
              }
            };

            if new_column.is_some() {
              record.push_field(&result);
            } else {
              record = replace_column_value(&record, column_index_next, &result);
            }
          }
          ApplyCmd::DynFmt => {
            let mut cell = record[column_index_next].to_owned();
            if !cell.is_empty() {
              let mut record_vec: Vec<String> = Vec::with_capacity(record.len());
              for field in &record {
                record_vec.push(field.to_string());
              }
              if let Ok(formatted) = dynfmt::SimpleCurlyFormat.format(&dynfmt_template, record_vec)
              {
                cell = formatted.to_string();
              }
            }
            if new_column.is_some() {
              record.push_field(&cell);
            } else {
              record = replace_column_value(&record, column_index_next, &cell);
            }
          }
        }

        record
      })
      .collect_into_vec(&mut batch_results);

    // rayon collect() guarantees original order, so we can just append results each batch
    for result_record in &batch_results {
      wtr.write_record(result_record)?;
    }

    batch.clear();
  } // end batch loop

  Ok(wtr.flush()?)
}

#[tauri::command]
pub async fn get_apply_headers(file_path: String) -> Result<Vec<HashMap<String, String>>, String> {
  match get_header(file_path).await {
    Ok(result) => Ok(result),
    Err(err) => Err(format!("get_apply_header: {err}")),
  }
}

#[tauri::command]
pub async fn apply(
  file_path: String,
  select_columns: String,
  apply_mode: String,
  operations: String,
  comparand: String,
  replacement: String,
  formatstr: String,
  new_column: bool,
  output_path: String,
) -> Result<String, String> {
  let start_time = Instant::now();

  match apply_perform(
    file_path,
    select_columns,
    apply_mode,
    operations.as_str(),
    comparand,
    replacement,
    formatstr,
    new_column,
    output_path,
  )
  .await
  {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      let runtime = format!("{elapsed_time:.2}");
      Ok(runtime)
    }
    Err(err) => Err(format!("apply failed: {err}")),
  }
}
