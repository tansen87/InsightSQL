use std::{
  collections::HashMap, fs::File, io::BufWriter, ops::Neg, path::Path, sync::OnceLock,
  time::Instant,
};

use anyhow::{Result, anyhow};
use cpc::{eval, units::Unit};
use csv::{ReaderBuilder, WriterBuilder};
use dynfmt::Format;
use rayon::{
  iter::{IndexedParallelIterator, ParallelIterator},
  prelude::IntoParallelRefIterator,
};
use regex::Regex;
use smallvec::SmallVec;

use crate::{io::csv::options::CsvOptions, utils::WTR_BUFFER_SIZE};

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
  Squeeze,
  Strip,
  Reverse,
  Abs,
  Neg,
  Normalize,
}

impl Operations {
  fn from_str(op: &str) -> Result<Self> {
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
      "squeeze" => Ok(Operations::Squeeze),
      "strip" => Ok(Operations::Strip),
      "reverse" => Ok(Operations::Reverse),
      "abs" => Ok(Operations::Abs),
      "neg" => Ok(Operations::Neg),
      "normalize" => Ok(Operations::Normalize),
      _ => Ok(Operations::Copy),
    }
  }
}

#[derive(PartialEq)]
enum ApplyCmd {
  Operations,
  CalcConv,
  Cat,
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

  if places == 9999 {
    return ryu::Buffer::new().format(dec_f64).to_owned();
  }

  let Some(dec_num) = Decimal::from_f64_retain(dec_f64) else {
    return String::new();
  };

  dec_num
    .round_dp_with_strategy(places, RoundingStrategy::MidpointNearestEven)
    .normalize()
    .to_string()
}

fn validate_operations(
  operations: &Vec<&str>,
  comparand: &str,
  new_column: Option<&String>,
  formatstr: &str,
) -> Result<SmallVec<[Operations; 4]>> {
  let mut ops_vec = SmallVec::with_capacity(operations.len());

  for op in operations {
    let operation = Operations::from_str(op)?;
    match operation {
      Operations::Copy => {
        if new_column.is_none() {
          return Err(anyhow!("new_column is required for copy operation."));
        }
      }
      Operations::Len => {
        if new_column.is_none() {
          return Err(anyhow!("new_column is required for len operation."));
        }
      }
      Operations::Reverse => {
        if new_column.is_none() {
          return Err(anyhow!("new_column is required for reverse operation."));
        }
      }
      Operations::Replace => {
        if comparand.is_empty() {
          return Err(anyhow!("comparand is required for replace operation."));
        }
      }
      Operations::Round => {
        if ROUND_PLACES
          .set(formatstr.parse::<u32>().unwrap_or(2))
          .is_err()
        {
          return Err(anyhow!("Cannot initialize Round precision."));
        };
      }
      _ => {}
    }
    ops_vec.push(operation);
  }

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
      Operations::Squeeze => {
        let squeezer: &'static Regex = regex_oncelock!(r"\s+");
        *cell = squeezer.replace_all(cell, " ").into_owned();
      }
      Operations::Strip => {
        let striper: &'static Regex = regex_oncelock!(r"[\r\n]+");
        *cell = striper.replace_all(cell, " ").into_owned();
      }
      Operations::Normalize => {
        let normalizer: &'static Regex = regex_oncelock!(r"^(\d+(?:\.\d+)?)([+-])$");
        if let Some(caps) = normalizer.captures(cell) {
          let number = &caps[1];
          let sign = &caps[2];
          *cell = match sign {
            "-" => format!("-{number}"),
            _ => format!("{number}"), // "+"
          };
        }
      }
      Operations::Reverse => {
        *cell = cell.as_str().chars().rev().collect::<String>();
      }
      Operations::Abs => {
        if let Ok(num) = cell.parse::<f64>() {
          *cell = num.abs().to_string()
        }
      }
      Operations::Neg => {
        if let Ok(num) = cell.parse::<f64>() {
          *cell = num.neg().to_string()
        }
      }
      Operations::Copy => {} // copy is a noop
    }
  }
}

async fn apply_perform<P: AsRef<Path> + Send + Sync>(
  path: P,
  columns: String,
  mode: String,
  operations: &str,
  comparand: String,
  replacement: String,
  formatstr: String,
  new_column_flag: bool,
  quoting: bool,
  skiprows: usize,
) -> Result<()> {
  let columns: Vec<&str> = columns.split('|').collect();
  if columns.is_empty() {
    return Err(anyhow!("At least one column must be specified."));
  }

  let mut opts = CsvOptions::new(&path);
  opts.set_skiprows(skiprows);
  let (sep, reader) = opts.skiprows_and_delimiter()?;
  let sep_char = sep as char;
  let output_path = opts.output_path(Some("apply"), None)?;

  let force_new_column = mode == "cat" || mode == "calcconv";
  let effective_new_column = new_column_flag || force_new_column;

  let new_column: Option<String> = if effective_new_column {
    let suffix = if mode == "cat" {
      "_dynfmt"
    } else if mode == "calcconv" {
      "_calcconv"
    } else {
      "_new"
    };

    Some(
      columns
        .iter()
        .map(|col| format!("{}{}", col, suffix))
        .collect::<Vec<_>>()
        .join(&sep_char.to_string()),
    )
  } else {
    None
  };

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .quoting(quoting)
    .from_reader(reader);

  let output_file = File::create(output_path)?;
  let buf_wtr = BufWriter::with_capacity(WTR_BUFFER_SIZE, output_file);
  let mut wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_wtr);

  let headers = rdr.byte_headers()?;

  let header_map: HashMap<_, _> = headers
    .iter()
    .enumerate()
    .map(|(i, field)| (field.to_vec(), i))
    .collect();
  let select_column_bytes: Vec<_> = columns.iter().map(|&col| col.as_bytes().to_vec()).collect();
  let column_index = select_column_bytes
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
    .collect::<Result<Vec<usize>, _>>()
    .map_err(|err| anyhow!(err))?;
  let column_index_next = *column_index.iter().next().unwrap(); // safe due to columns.is_empty() check

  let mut headers = rdr.headers()?.clone();

  if let Some(ref new_column) = new_column {
    for col in new_column.split(sep_char) {
      headers.push_field(col);
    }
  }
  wtr.write_record(&headers)?;

  let dynfmt_template = if (mode == "calcconv") || (mode == "cat") {
    let mut dynfmt_template_wrk = formatstr.clone();
    let mut dynfmt_fields = Vec::new();

    let formatstr_re: &'static Regex = crate::regex_oncelock!(r"\{(?P<key>\w+)?\}");
    for format_fields in formatstr_re.captures_iter(&formatstr) {
      dynfmt_fields.push(format_fields.name("key").unwrap().as_str());
    }
    dynfmt_fields.sort_unstable();

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

  let apply_cmd = if mode == "operations" {
    match validate_operations(
      &operations.split('|').collect(),
      &comparand,
      new_column.as_ref(),
      &formatstr,
    ) {
      Ok(operations_vec) => ops_vec = operations_vec,
      Err(e) => return Err(e),
    }
    ApplyCmd::Operations
  } else if mode == "calcconv" {
    ApplyCmd::CalcConv
  } else {
    ApplyCmd::Cat
  };

  let mut batch_record = csv::StringRecord::new();
  let batchsize = 50_000;
  let mut batch = Vec::with_capacity(batchsize);
  let mut batch_results = Vec::with_capacity(batchsize);

  'batch_loop: loop {
    for _ in 0..batchsize {
      match rdr.read_record(&mut batch_record) {
        Ok(true) => batch.push(std::mem::take(&mut batch_record)),
        Ok(false) => break,
        Err(err) => return Err(anyhow!("Error reading file: {err}")),
      }
    }

    if batch.is_empty() {
      break 'batch_loop;
    }

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
              let record_vec: Vec<String> = record.iter().map(|f| f.to_string()).collect();
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
                Err(e) => format!("ERROR: {e}"),
              }
            };
            record.push_field(&result);
          }
          ApplyCmd::Cat => {
            let mut cell = record[column_index_next].to_owned();
            if !cell.is_empty() {
              let record_vec: Vec<String> = record.iter().map(|f| f.to_string()).collect();
              if let Ok(formatted) = dynfmt::SimpleCurlyFormat.format(&dynfmt_template, record_vec)
              {
                cell = formatted.to_string();
              }
            }
            record.push_field(&cell);
          }
        }
        record
      })
      .collect_into_vec(&mut batch_results);

    for result_record in &batch_results {
      wtr.write_record(result_record)?;
    }

    batch.clear();
  }

  Ok(wtr.flush()?)
}

#[tauri::command]
pub async fn apply(
  path: String,
  columns: String,
  mode: String,
  operations: String,
  comparand: String,
  replacement: String,
  formatstr: String,
  new_column: bool,
  quoting: bool,
  skiprows: usize,
) -> Result<String, String> {
  let start_time = Instant::now();

  match apply_perform(
    path,
    columns,
    mode,
    &operations,
    comparand,
    replacement,
    formatstr,
    new_column,
    quoting,
    skiprows,
  )
  .await
  {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("{err}")),
  }
}
