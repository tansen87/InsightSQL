use std::{borrow::Cow, collections::HashMap, fs::File, path::Path, time::Instant};

use anyhow::Result;
use regex::bytes::RegexBuilder;

use crate::utils::{get_same_headers, CsvOptions, Selection};

async fn regex_replace<P: AsRef<Path>>(
  path: P,
  sel: String,
  regex_pattern: String,
  replacement: String,
) -> Result<()> {
  let pattern = RegexBuilder::new(&regex_pattern).build()?;

  let csv_options = CsvOptions::new(&path);
  let sep = match csv_options.detect_separator() {
    Some(separator) => separator as u8,
    None => b',',
  };

  let mut rdr = csv::ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(File::open(&path)?);

  let parent_path = &path
    .as_ref()
    .parent()
    .map(|path| path.to_string_lossy())
    .unwrap();
  let file_name = &path.as_ref().file_stem().unwrap().to_str().unwrap();
  let output_path = format!("{}/{}.replace.csv", parent_path, file_name);
  let mut wtr = csv::WriterBuilder::new()
    .delimiter(sep)
    .from_path(output_path)?;

  let headers = rdr.headers()?.clone();
  let sel = Selection::from_headers(rdr.byte_headers()?, &[sel.as_str()][..])?;

  wtr.write_record(&headers)?;

  let mut record = csv::ByteRecord::new();
  while rdr.read_byte_record(&mut record)? {
    record = record
      .into_iter()
      .enumerate()
      .map(|(idx, val)| {
        if sel.get_indices().contains(&idx) {
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
  match get_same_headers(file_path).await {
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
  sel: String,
  regex_pattern: String,
  replacement: String,
) -> Result<()> {
  regex_replace(file_path, sel, regex_pattern, replacement).await
}
