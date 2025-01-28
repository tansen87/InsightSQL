use std::{path::Path, time::Instant};

use anyhow::Result;
use csv::{ByteRecord, ReaderBuilder, WriterBuilder};
use tauri::Emitter;

use crate::utils::CsvOptions;

async fn drop_headers(path: String, skip_rows: String, window: tauri::Window) -> Result<()> {
  let skip_rows = skip_rows.parse::<usize>()?;
  let paths: Vec<&str> = path.split('|').collect();
  let parent_path = Path::new(&paths[0])
    .parent()
    .map(|parent| parent.to_string_lossy())
    .unwrap();

  let mut count: usize = 0;
  let file_len = paths.len();

  for fp in paths.iter() {
    window.emit("start_convert", fp)?;
    let mut csv_options = CsvOptions::new(fp);
    csv_options.set_skip_rows(skip_rows);
    let sep = match csv_options.detect_separator() {
      Some(separator) => separator as u8,
      None => b',',
    };

    let filename = Path::new(fp).file_stem().unwrap().to_str().unwrap();
    let output_path = format!("{}/{}.behead.csv", parent_path, filename);

    let mut rdr = ReaderBuilder::new()
      .delimiter(sep)
      .has_headers(true)
      .from_reader(csv_options.skip_csv_rows()?);

    let mut wtr = WriterBuilder::new()
      .delimiter(sep)
      .has_headers(false)
      .flexible(true)
      .from_path(output_path)?;

    let mut record = ByteRecord::new();

    while rdr.read_byte_record(&mut record)? {
      wtr.write_byte_record(&record)?;
    }
    wtr.flush()?;

    window.emit("drop_msg", fp)?;

    count += 1;
    let progress = ((count as f32) / (file_len as f32)) * 100.0;
    let drop_progress = format!("{progress:.0}");
    window.emit("drop_progress", drop_progress)?;
  }

  Ok(())
}

#[tauri::command]
pub async fn behead(path: String, skip_rows: String, window: tauri::Window) -> Result<String, String> {
  let start_time = Instant::now();

  match drop_headers(path, skip_rows, window).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("behead failed: {err}")),
  }
}
