use std::{borrow::Cow, path::Path, time::Instant};

use anyhow::Result;
use csv::{ByteRecord, ReaderBuilder, WriterBuilder};
use regex::bytes::RegexBuilder;

use crate::utils::{CsvOptions, Selection};

pub async fn regex_replace<P: AsRef<Path> + Send + Sync>(
  path: P,
  sel: String,
  regex_pattern: String,
  replacement: String,
  skip_rows: String,
) -> Result<()> {
  let pattern = RegexBuilder::new(&regex_pattern).build()?;

  let mut csv_options = CsvOptions::new(&path);
  csv_options.set_skip_rows(skip_rows.parse::<usize>()?);

  let sep = csv_options.detect_separator()?;

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(csv_options.skip_csv_rows()?);

  let parent_path = path.as_ref().parent().unwrap().to_str().unwrap();
  let file_name = path.as_ref().file_stem().unwrap().to_str().unwrap();
  let output_path = format!("{parent_path}/{file_name}.replace.csv");

  let mut wtr = WriterBuilder::new().delimiter(sep).from_path(output_path)?;

  let headers = rdr.headers()?.clone();
  let sel = Selection::from_headers(rdr.byte_headers()?, &[sel.as_str()][..])?;

  wtr.write_record(&headers)?;

  let mut record = ByteRecord::new();
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
pub async fn replace(
  path: String,
  select_column: String,
  regex_pattern: String,
  replacement: String,
  skip_rows: String,
) -> Result<String, String> {
  let start_time = Instant::now();

  match regex_replace(path, select_column, regex_pattern, replacement, skip_rows).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("{err}")),
  }
}
