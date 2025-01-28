use std::{fs::File, path::Path, time::Instant};

use anyhow::Result;
use csv::{ByteRecord, ReaderBuilder};
use tauri::{Emitter, Window};

use crate::utils::CsvOptions;

async fn count_rows<P: AsRef<Path>>(path: P) -> Result<u64> {
  let csv_options = CsvOptions::new(&path);
  let sep = match csv_options.detect_separator() {
    Some(separator) => separator as u8,
    None => b',',
  };

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(File::open(&path)?);

  let mut record = ByteRecord::new();
  let mut count: u64 = 1;
  while rdr.read_byte_record(&mut record)? {
    count += 1;
  }

  Ok(count)
}

#[tauri::command]
pub async fn count(path: String, window: Window) -> Result<String, String> {
  let start_time = Instant::now();
  let paths: Vec<&str> = path.split('|').collect();

  for file in paths.iter() {
    let filename = Path::new(file)
      .file_name()
      .unwrap()
      .to_str()
      .unwrap()
      .to_string();
    window
      .emit("start_convert", &filename)
      .map_err(|e| e.to_string())?;
    match count_rows(file).await {
      Ok(cnt) => {
        window
          .emit("count_msg", format!("{}|{}", &filename, cnt))
          .map_err(|e| e.to_string())?;
      }
      Err(e) => {
        window
          .emit("count_err", format!("{}|{}", &filename, e))
          .map_err(|e| e.to_string())?;
      }
    }
  }

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  Ok(format!("{:.2}", elapsed_time))
}

/// for integration test
pub async fn public_count(path: String) -> Result<u64> {
  count_rows(path).await
}
