use std::{
  collections::{HashMap, HashSet},
  ffi::OsStr,
  path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};
use calamine::{Data, HeaderRow, Range, Reader};
use csv::{StringRecord, WriterBuilder};
use rayon::{iter::ParallelIterator, slice::ParallelSlice};

use crate::utils::num_cpus;

/// convert excel to csv
pub async fn excel_to_csv<P: AsRef<Path>>(
  path: P,
  skip_rows: u32,
  sheet_name: Option<String>,
  output_path: &PathBuf,
) -> Result<()> {
  let mut wtr = WriterBuilder::new().from_path(output_path)?;

  let mut workbook = calamine::open_workbook_auto(&path)?;

  let range = match sheet_name {
    None => {
      match workbook
        .with_header_row(HeaderRow::Row(skip_rows))
        .worksheet_range_at(0)
      {
        Some(Ok(range)) => range,
        Some(Err(_)) | None => Range::empty(),
      }
    }
    Some(ref name) => {
      match workbook
        .with_header_row(HeaderRow::Row(skip_rows))
        .worksheet_range(name)
      {
        Ok(range) => range,
        Err(_) => Range::empty(),
      }
    }
  };

  let (row_count, col_count) = range.get_size();
  // check row count
  if row_count == 0 {
    return Err(anyhow!("file is empty, skipping processing"));
  }

  let mut rows_iter = range.rows();

  // amortize allocations
  let mut record = StringRecord::with_capacity(500, col_count);
  let mut col_name: String;

  // get the first row as header
  let first_row = match rows_iter.next() {
    Some(first_row) => first_row,
    None => &[Data::Empty],
  };

  for cell in first_row {
    col_name = match *cell {
      Data::String(ref s) => s.to_string(),
      Data::Empty => String::new(),
      Data::Error(ref _e) => String::new(),
      Data::Int(ref i) => i.to_string(),
      Data::Float(ref f) => f.to_string(),
      Data::DateTime(ref edt) => edt.to_string(),
      Data::Bool(ref b) => b.to_string(),
      Data::DateTimeIso(ref dt) => dt.to_string(),
      Data::DurationIso(ref d) => d.to_string(),
    };
    record.push_field(&col_name);
  }

  wtr.write_record(&record)?;

  let mut rows = Vec::with_capacity(row_count);
  // process rest of the rows
  for row in rows_iter {
    rows.push(row);
  }

  // set RAYON_NUM_THREADS
  let ncpus = num_cpus();
  // set chunk_size to number of rows per core/thread
  // let chunk_size = row_count.div_ceil(ncpus);
  let chunk_size = (row_count + ncpus - 1) / ncpus;

  let processed_rows: Vec<Vec<StringRecord>> = rows
    .par_chunks(chunk_size)
    .map(|chunk| {
      let mut record = StringRecord::with_capacity(500, col_count);
      let mut float_val;
      let mut work_date = String::new();
      let mut ryu_buffer = ryu::Buffer::new();
      let mut itoa_buffer = itoa::Buffer::new();
      let mut formatted_date = String::new();
      let mut processed_chunk = Vec::with_capacity(chunk_size);

      for row in chunk {
        for cell in *row {
          match *cell {
            Data::Empty => record.push_field(""),
            Data::String(ref s) => record.push_field(s),
            Data::Int(ref i) => record.push_field(itoa_buffer.format(*i)),
            Data::Float(ref f) => {
              float_val = *f;

              #[allow(clippy::cast_precision_loss)]
              if float_val.fract().abs() > f64::EPSILON
                || float_val > (i64::MAX as f64)
                || float_val < (i64::MIN as f64)
              {
                record.push_field(ryu_buffer.format_finite(float_val));
              } else {
                record.push_field(itoa_buffer.format(float_val as i64));
              }
            }
            Data::DateTime(ref edt) => {
              if edt.is_datetime() {
                work_date.clear();
                if let Some(dt) = edt.as_datetime() {
                  formatted_date.clear();
                  work_date = dt.to_string();
                }
              } else {
                work_date = edt.as_duration().unwrap().to_string();
              }

              record.push_field(&work_date);
            }
            Data::Error(ref e) => record.push_field(&format!("{e:?}")),
            Data::Bool(ref b) => {
              record.push_field(if *b { "true" } else { "false" });
            }
            Data::DateTimeIso(ref dt) => record.push_field(dt),
            Data::DurationIso(ref d) => record.push_field(d),
          };
        }

        processed_chunk.push(record.clone());
        record.clear();
      }
      processed_chunk
    })
    .collect();

  for processed_chunk in processed_rows {
    for processed_row in processed_chunk {
      wtr.write_record(&processed_row)?;
    }
  }

  Ok(wtr.flush()?)
}

pub fn get_sheetname_by_filename(
  records: &Vec<HashMap<String, String>>,
  filename: &str,
) -> Option<String> {
  for record in records.iter() {
    if let Some(file) = record.get("filename") {
      if file == filename {
        return record.get("sheetname").cloned();
      }
    }
  }
  None
}

pub async fn get_all_sheetnames<P: AsRef<Path>>(path: P) -> HashSet<String> {
  let (sheets, _) = map_excel_sheets(path.as_ref().to_str().unwrap().to_string()).await;
  let mut sheetnames = HashSet::new();

  for (_, sheet_list) in sheets.iter() {
    for sheet_name in sheet_list.iter() {
      sheetnames.insert(sheet_name.clone());
    }
  }

  sheetnames
}

#[tauri::command]
pub async fn map_excel_sheets(
  path: String,
) -> (HashMap<String, Vec<String>>, HashMap<String, String>) {
  let mut map_sheets = HashMap::new();
  let mut errors = HashMap::new();

  let paths: Vec<&str> = path.split('|').filter(|&x| !x.is_empty()).collect();
  if paths.is_empty() {
    return (map_sheets, errors);
  }

  for file in paths.iter() {
    let filename = Path::new(file)
      .file_name()
      .unwrap_or_else(|| OsStr::new(""))
      .to_str()
      .unwrap_or("")
      .to_string();
    match calamine::open_workbook_auto(file).map_err(|e| e.to_string()) {
      Ok(workbook) => {
        let sheets = workbook.sheet_names();
        map_sheets.insert(filename, sheets);
      }
      Err(err) => {
        errors.insert(filename, format!("get sheets error|{}", err));
      }
    }
  }

  (map_sheets, errors)
}
