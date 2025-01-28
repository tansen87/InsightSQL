use std::{collections::HashMap, fs::File, io::BufWriter, path::Path, time::Instant};

use anyhow::Result;
use csv::{ReaderBuilder, WriterBuilder};

use crate::utils::{CsvOptions, Selection};

async fn fill_values<P: AsRef<Path>>(
  path: P,
  fill_column: String,
  fill_value: String,
  skip_rows: String
) -> Result<()> {
  let mut csv_options = CsvOptions::new(&path);
  csv_options.set_skip_rows(skip_rows.parse::<usize>()?);
  let sep = match csv_options.detect_separator() {
    Some(separator) => separator as u8,
    None => b',',
  };

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(csv_options.skip_csv_rows()?);

  let headers = rdr.headers()?.clone();

  let fill_columns: Vec<&str> = fill_column.split('|').collect();
  let sel = Selection::from_headers(rdr.byte_headers()?, &fill_columns[..])?;

  let parent_path = &path
    .as_ref()
    .parent()
    .map(|parent| parent.to_string_lossy())
    .unwrap();
  let file_name = path.as_ref().file_stem().unwrap().to_str().unwrap();
  let output_file = format!("{}/{}.fill.csv", parent_path, file_name);

  let mut wtr = WriterBuilder::new()
    .delimiter(sep)
    .from_writer(BufWriter::new(File::create(output_file)?));

  wtr.write_record(&headers)?;

  for record in rdr.deserialize() {
    let mut row: Vec<String> = record?;
    for &index in sel.get_indices() {
      if row.get(index).map_or(true, |s| s.is_empty()) {
        row[index] = fill_value.clone();
      }
    }
    wtr.write_record(&row)?;
  }

  Ok(wtr.flush()?)
}

#[tauri::command]
pub async fn get_fill_headers(
  path: String,
  skip_rows: String,
) -> Result<Vec<HashMap<String, String>>, String> {
  let mut csv_options = CsvOptions::new(&path);
  csv_options.set_skip_rows(skip_rows.parse::<usize>().map_err(|e| e.to_string())?);
  match csv_options.map_headers().await {
    Ok(result) => Ok(result),
    Err(err) => Err(format!("get header error: {err}")),
  }
}

#[tauri::command]
pub async fn fill(path: String, columns: String, values: String, skip_rows: String) -> Result<String, String> {
  let start_time = Instant::now();

  match fill_values(path, columns, values, skip_rows).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("fill failed: {err}")),
  }
}

/// for integration test
pub async fn public_fill(path: String, fill_column: String, fill_value: String, skip_rows: String) -> Result<()> {
  fill_values(path, fill_column, fill_value, skip_rows).await
}
