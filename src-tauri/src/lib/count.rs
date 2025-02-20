use std::{fs::File, path::Path, time::Instant};

use anyhow::Result;
use csv::{ByteRecord, ReaderBuilder};
use tauri::{Emitter, Window};

use crate::utils::CsvOptions;

pub async fn count_rows<P: AsRef<Path>>(path: P) -> Result<u64> {
  let csv_options = CsvOptions::new(&path);

  let sep = match csv_options.detect_separator() {
    Some(separator) => separator as u8,
    None => b',',
  };

  let count = match csv_options.indexed()? {
    Some(idx) => idx.count(),
    None => {
      let mut rdr = ReaderBuilder::new()
        .delimiter(sep)
        .from_reader(File::open(&path)?);

      let mut record = ByteRecord::new();
      let mut count: u64 = 1;
      while rdr.read_byte_record(&mut record)? {
        count += 1;
      }
      count
    }
  };

  Ok(count)
}

#[tauri::command]
pub async fn count(path: String, mode: String, window: Window) -> Result<String, String> {
  let start_time = Instant::now();
  let paths: Vec<&str> = path.split('|').collect();

  for file in paths.iter() {
    let filename = Path::new(file).file_name().unwrap().to_str().unwrap();
    window
      .emit("start_convert", &filename)
      .map_err(|e| e.to_string())?;
    let inner_time = Instant::now();
    match mode.as_str() {
      "index" => match crate::idx::create_index(file).await {
        Ok(_) => {
          let end_time = Instant::now();
          let elapsed_time = end_time.duration_since(inner_time).as_secs_f64();
          window
            .emit("count_msg", format!("{filename}|{elapsed_time:.2} s"))
            .map_err(|e| e.to_string())?;
        }
        Err(err) => {
          window
            .emit("count_err", format!("{filename}|{err}"))
            .map_err(|e| e.to_string())?;
        }
      },
      _ => match count_rows(file).await {
        Ok(cnt) => {
          window
            .emit("count_msg", format!("{filename}|{cnt}"))
            .map_err(|e| e.to_string())?;
        }
        Err(err) => {
          window
            .emit("count_err", format!("{filename}|{err}"))
            .map_err(|e| e.to_string())?;
        }
      },
    }
  }

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  Ok(format!("{elapsed_time:.2}"))
}
