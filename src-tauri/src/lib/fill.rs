use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::time::Instant;

use anyhow::{anyhow, Result};

use crate::detect::detect_separator;

async fn get_header(path: String) -> Result<Vec<HashMap<String, String>>> {
  let sep = match detect_separator(path.as_str(), 0) {
    Some(separator) => separator as u8,
    None => b',',
  };

  let mut rdr = csv::ReaderBuilder::new()
    .delimiter(sep)
    .has_headers(true)
    .from_reader(File::open(path)?);

  let headers = rdr.headers()?.clone();
  let vec_headers: Vec<String> = headers.iter().map(|h| h.to_string()).collect();

  let hs = vec_headers
    .into_iter()
    .enumerate()
    .map(|(_index, value)| {
      let mut map = HashMap::new();
      map.insert("value".to_string(), value.clone());
      map.insert("label".to_string(), value);
      map
    })
    .collect();

  Ok(hs)
}

async fn fill_values(input_file: String, fill_column: String, fill_value: String) -> Result<()> {
  let sep = match detect_separator(input_file.as_str(), 0) {
    Some(separator) => separator as u8,
    None => b',',
  };

  let fill_columns: Vec<&str> = fill_column.split('|').collect();

  let mut rdr = csv::ReaderBuilder::new()
    .delimiter(sep)
    .has_headers(true)
    .from_reader(File::open(&input_file)?);

  let headers = rdr.headers()?;

  let mut header_indices = Vec::new();
  for column in &fill_columns {
    let index = match headers.iter().position(|field| &field == column) {
      Some(idx) => idx,
      None => {
        return Err(anyhow!(
          "The column '{}' was not found in the headers.",
          column
        ));
      }
    };
    header_indices.push(index);
  }

  let parent_path = Path::new(&input_file)
    .parent()
    .map(|parent| parent.to_string_lossy())
    .unwrap();
  let file_name = Path::new(&input_file)
    .file_stem()
    .unwrap()
    .to_str()
    .unwrap();
  let output_file = format!("{}/{}.fill.csv", parent_path, file_name);

  let mut wtr = csv::WriterBuilder::new()
    .delimiter(sep)
    .from_writer(BufWriter::new(File::create(output_file)?));

  wtr.write_record(headers)?;

  for result in rdr.deserialize() {
    let mut record: Vec<String> = result?;
    for &index in &header_indices {
      if record.get(index).map_or(true, |s| s.is_empty()) {
        record[index] = fill_value.clone();
      }
    }
    wtr.write_record(&record)?;
  }

  Ok(wtr.flush()?)
}

#[tauri::command]
pub async fn get_fill_headers(path: String) -> Result<Vec<HashMap<String, String>>, String> {
  match get_header(path).await {
    Ok(result) => Ok(result),
    Err(err) => Err(format!("get header error: {err}")),
  }
}

#[tauri::command]
pub async fn fill(path: String, columns: String, values: String) -> Result<String, String> {
  let start_time = Instant::now();

  match fill_values(path, columns, values).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("fill failed: {err}")),
  }
}

/// for integration test
pub async fn public_fill(
  input_file: String,
  fill_column: String,
  fill_value: String,
) -> Result<()> {
  fill_values(input_file, fill_column, fill_value).await
}
