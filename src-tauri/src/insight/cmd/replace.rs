use std::{borrow::Cow, fs::File, io::BufWriter, path::Path, time::Instant};

use anyhow::Result;
use csv::{ByteRecord, ReaderBuilder, WriterBuilder};
use regex::bytes::RegexBuilder;

use crate::{
  io::csv::{options::CsvOptions, selection::Selection},
  utils::WTR_BUFFER_SIZE,
};

pub async fn regex_replace<P: AsRef<Path> + Send + Sync>(
  path: P,
  sel: String,
  regex_pattern: String,
  replacement: String,
) -> Result<()> {
  let pattern = RegexBuilder::new(&regex_pattern).build()?;
  let opts = CsvOptions::new(&path);
  let sep = opts.detect_separator()?;
  let output_path = opts.output_path(Some("replace"), None)?;

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(opts.rdr_skip_rows()?);

  let output_file = File::create(output_path)?;
  let buf_wtr = BufWriter::with_capacity(WTR_BUFFER_SIZE, output_file);
  let mut wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_wtr);

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
  column: String,
  regex_pattern: String,
  replacement: String,
) -> Result<String, String> {
  let start_time = Instant::now();

  match regex_replace(path, column, regex_pattern, replacement).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("{err}")),
  }
}
