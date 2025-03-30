use std::{path::Path, time::Instant};

use anyhow::Result;
use csv::{ByteRecord, Reader, ReaderBuilder, WriterBuilder};

use crate::utils::CsvOptions;

pub async fn rename_headers<P: AsRef<Path> + Send + Sync>(path: P, r_header: String) -> Result<()> {
  let csv_options = CsvOptions::new(&path);
  let sep = csv_options.detect_separator()?;

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(csv_options.skip_csv_rows()?);

  let mut new_rdr = Reader::from_reader(r_header.as_bytes());

  let new_headers = new_rdr.byte_headers()?;

  let parent_path = path.as_ref().parent().unwrap().to_str().unwrap();
  let file_stem = path.as_ref().file_stem().unwrap().to_str().unwrap();
  let output_path = format!("{parent_path}/{file_stem}.rename.csv");

  let mut wtr = WriterBuilder::new().delimiter(sep).from_path(output_path)?;

  wtr.write_record(new_headers)?;

  let mut record = ByteRecord::new();
  while rdr.read_byte_record(&mut record)? {
    wtr.write_record(&record)?;
  }

  Ok(wtr.flush()?)
}

#[tauri::command]
pub async fn get_rename_headers(path: String) -> Result<Vec<String>, String> {
  let csv_options = CsvOptions::new(path);

  async { csv_options.from_headers().map_err(|e| e.to_string()) }.await
}

#[tauri::command]
pub async fn rename(path: String, headers: String) -> Result<String, String> {
  let start_time = Instant::now();

  match rename_headers(path, headers).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("{err}")),
  }
}
