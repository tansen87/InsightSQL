use std::{fs::File, path::Path, time::Instant};

use anyhow::Result;
use tauri::Emitter;

use crate::detect::detect_separator;

async fn count_rows(path: String, window: tauri::Window) -> Result<()> {
  /* count csv rows */
  let vec_path: Vec<&str> = path.split('|').collect();
  let mut countf: usize = 0;
  let file_len = vec_path.len();

  for file in vec_path.iter() {
    window.emit("start_convert", file)?;

    let sep = match detect_separator(file, 0) {
      Some(separator) => separator as u8,
      None => b',',
    };

    let mut rdr = csv::ReaderBuilder::new()
      .delimiter(sep)
      .has_headers(false)
      .from_reader(File::open(file)?);

    let mut record = csv::ByteRecord::new();
    rdr.read_byte_record(&mut record)?;
    let mut count: u64 = 1;
    while rdr.read_byte_record(&mut record)? {
      count += 1;
    }

    let filename = Path::new(file).file_name().unwrap().to_str().unwrap();
    window.emit("count_msg", format!("{}|{}", filename, count))?;

    countf += 1;
    let progress = ((countf as f32) / (file_len as f32)) * 100.0;
    window.emit("count_progress", format!("{progress:.0}"))?;
  }

  Ok(())
}

#[tauri::command]
pub async fn count(path: String, window: tauri::Window) -> Result<String, String> {
  let start_time = Instant::now();

  match count_rows(path, window).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("count failed: {err}")),
  }
}
