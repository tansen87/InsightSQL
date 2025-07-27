use std::{
  borrow::Cow,
  path::{Path, PathBuf},
  time::Instant,
};

use anyhow::Result;
use csv::{ByteRecord, ReaderBuilder, WriterBuilder};
use regex::bytes::RegexBuilder;

use crate::io::csv::{options::CsvOptions, selection::Selection};

pub async fn regex_replace<P: AsRef<Path> + Send + Sync>(
  path: P,
  sel: String,
  regex_pattern: String,
  replacement: String,
) -> Result<()> {
  let pattern = RegexBuilder::new(&regex_pattern).build()?;
  let csv_options = CsvOptions::new(&path);
  let sep = csv_options.detect_separator()?;
  let file_stem = csv_options.file_stem()?;
  let mut output_path = PathBuf::from(csv_options.parent_path()?);
  output_path.push(format!("{file_stem}.replace.csv"));

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(csv_options.rdr_skip_rows()?);

  let mut wtr = WriterBuilder::new().delimiter(sep).from_path(output_path)?;

  let headers = rdr.byte_headers()?;

  let sel = Selection::from_headers(headers, &[sel.as_str()][..])?;

  wtr.write_record(headers)?;

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
) -> Result<String, String> {
  let start_time = Instant::now();

  match regex_replace(path, select_column, regex_pattern, replacement).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("{err}")),
  }
}
