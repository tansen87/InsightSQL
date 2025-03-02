use std::{path::Path, time::Instant};

use anyhow::{anyhow, Result};
use csv::{ByteRecord, ReaderBuilder, WriterBuilder};
use tauri::{Emitter, Window};

use crate::utils::CsvOptions;

pub async fn skip_csv<P: AsRef<Path> + Send + Sync>(
  path: P,
  skip_rows: usize,
  parent_path: &str,
) -> Result<()> {
  if skip_rows < 1 {
    return Err(anyhow!("The skip rows must be greater than or equal to 1"));
  }

  let mut csv_options = CsvOptions::new(&path);
  csv_options.set_skip_rows(skip_rows - 1);

  let sep = csv_options.detect_separator()?;

  let filename = path.as_ref().file_stem().unwrap().to_str().unwrap();
  let output_path = format!("{parent_path}/{filename}.skiprows.csv");

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(csv_options.skip_csv_rows()?);

  let mut wtr = WriterBuilder::new().delimiter(sep).from_path(output_path)?;

  let mut record = ByteRecord::new();

  while rdr.read_byte_record(&mut record)? {
    wtr.write_byte_record(&record)?;
  }

  Ok(wtr.flush()?)
}

#[tauri::command]
pub async fn skip(path: String, skip_rows: String, window: Window) -> Result<String, String> {
  let start_time = Instant::now();

  let paths: Vec<&str> = path.split('|').collect();
  let parent_path = Path::new(&paths[0]).parent().unwrap().to_str().unwrap();
  let skip_rows = skip_rows.parse::<usize>().map_err(|e| e.to_string())?;

  for fp in paths.iter() {
    let filename = Path::new(fp).file_name().unwrap().to_str().unwrap();
    window
      .emit("start_convert", filename)
      .map_err(|e| e.to_string())?;
    match skip_csv(fp, skip_rows, parent_path).await {
      Ok(_) => {
        window
          .emit("skip_msg", filename)
          .map_err(|e| e.to_string())?;
      }
      Err(err) => {
        window
          .emit("skip_err", format!("{filename}|{err}"))
          .map_err(|e| e.to_string())?;
        continue;
      }
    }
  }

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  Ok(format!("{:.2}", elapsed_time))
}
