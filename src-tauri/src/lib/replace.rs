use std::{borrow::Cow, collections::HashMap, fs::File, path::Path, time::Instant};

use anyhow::{anyhow, Result};
use regex::bytes::RegexBuilder;

use crate::detect::detect_separator;

async fn get_header(file_path: String) -> Result<Vec<HashMap<String, String>>> {
  let sep = match detect_separator(file_path.as_str()) {
    Some(separator) => separator as u8,
    None => b',',
  };

  let mut rdr = csv::ReaderBuilder::new()
    .delimiter(sep)
    .has_headers(true)
    .from_reader(File::open(file_path)?);

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

async fn regex_replace(
  file_path: String,
  select_column: String,
  regex_pattern: String,
  replacement: String,
) -> Result<()> {
  let pattern = RegexBuilder::new(&regex_pattern).build()?;

  let sep = match detect_separator(&file_path.as_str()) {
    Some(separator) => separator as u8,
    None => b',',
  };

  let mut rdr = csv::ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(File::open(&file_path)?);

  let parent_path = Path::new(&file_path)
    .parent()
    .map(|path| path.to_string_lossy())
    .unwrap();
  let file_name = Path::new(&file_path).file_stem().unwrap().to_str().unwrap();
  let output_path = format!("{}/{}.replace.csv", parent_path, file_name);
  let mut wtr = csv::WriterBuilder::new()
    .delimiter(sep)
    .from_path(output_path)?;

  let headers = rdr.headers()?.clone();
  let header_idx = match headers.iter().position(|field| field == select_column) {
    Some(idx) => idx,
    None => {
      return Err(anyhow!(
        "The column '{select_column}' was not found in the headers."
      ))
    }
  };

  wtr.write_record(&headers)?;

  let mut record = csv::ByteRecord::new();
  while rdr.read_byte_record(&mut record)? {
    record = record
      .into_iter()
      .enumerate()
      .map(|(idx, val)| {
        if header_idx == idx {
          if pattern.is_match(val) {
            pattern.replace_all(val, replacement.as_bytes())
          } else {
            Cow::Borrowed(val)
          }
        } else {
          Cow::Borrowed(val)
        }
      })
      .collect();
    wtr.write_byte_record(&record)?;
  }

  Ok(wtr.flush()?)
}

#[tauri::command]
pub async fn get_replace_headers(
  file_path: String,
) -> Result<Vec<HashMap<String, String>>, String> {
  match get_header(file_path).await {
    Ok(result) => Ok(result),
    Err(err) => Err(format!("get header error: {err}")),
  }
}

#[tauri::command]
pub async fn replace(
  file_path: String,
  select_column: String,
  regex_pattern: String,
  replacement: String,
) -> Result<String, String> {
  let start_time = Instant::now();

  match regex_replace(file_path, select_column, regex_pattern, replacement).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("replace failed: {err}")),
  }
}

/// for integration test
pub async fn public_replace(
  file_path: String,
  select_column: String,
  regex_pattern: String,
  replacement: String,
) -> Result<()> {
  regex_replace(file_path, select_column, regex_pattern, replacement).await
}
