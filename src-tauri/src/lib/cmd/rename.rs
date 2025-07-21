use std::{
  path::{Path, PathBuf},
  sync::{Arc, Mutex},
  time::{Duration, Instant},
};

use anyhow::{Result, anyhow};
use csv::{ByteRecord, Reader, ReaderBuilder, WriterBuilder};
use tauri::{AppHandle, Emitter};
use tokio::sync::oneshot;

use crate::io::csv::options::CsvOptions;

pub async fn rename_headers<P: AsRef<Path> + Send + Sync>(
  path: P,
  r_header: String,
  mode: &str,
  app_handle: AppHandle,
) -> Result<()> {
  let csv_options = CsvOptions::new(&path);
  let sep = csv_options.detect_separator()?;
  let parent_path = path.as_ref().parent().unwrap().to_str().unwrap();
  let file_stem = path.as_ref().file_stem().unwrap().to_str().unwrap();
  let mut output_path = PathBuf::from(parent_path);
  output_path.push(format!("{file_stem}.rename.csv"));

  let total_rows = match mode {
    "idx" => csv_options.idx_count_rows().await?,
    "std" => csv_options.std_count_rows()?,
    _ => 0,
  };
  app_handle.emit("total-rows", total_rows)?;

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(csv_options.rdr_skip_rows()?);
  let mut new_rdr = Reader::from_reader(r_header.as_bytes());
  let new_headers = new_rdr.byte_headers()?;
  let mut wtr = WriterBuilder::new().delimiter(sep).from_path(output_path)?;
  wtr.write_record(new_headers)?;

  let rows = Arc::new(Mutex::new(0));
  let rows_clone = Arc::clone(&rows);

  let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
  let (done_tx, mut done_rx) = oneshot::channel::<usize>();

  let timer_task = tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_millis(500));
    loop {
      tokio::select! {
        _ = interval.tick() => {
          let current_rows = match rows_clone.lock() {
            Ok(lock) => *lock,
            Err(err) => {
              eprintln!("Failed to lock current rows: {err}");
              0
            }
          };
          if let Err(err) = app_handle.emit("update-rows", current_rows) {
            eprintln!("Failed to emit current rows: {err:?}");
          }
        },
        Ok(final_rows) = (&mut done_rx) => {
          if let Err(err) = app_handle.emit("update-rows", final_rows) {
            eprintln!("Failed to emit final rows: {err:?}");
          }
          break;
        },
        _ = (&mut stop_rx) => { break; }
      }
    }
  });

  let counter_task = tokio::task::spawn_blocking(move || {
    let mut record = ByteRecord::new();
    while rdr.read_byte_record(&mut record)? {
      wtr.write_byte_record(&record)?;

      let mut cnt = rows
        .lock()
        .map_err(|poison| anyhow!("cnt lock poisoned: {poison}"))?;
      *cnt += 1;
    }

    let final_rows = *rows
      .lock()
      .map_err(|poison| anyhow!("final rows lock poisoned: {poison}"))?;
    let _ = done_tx.send(final_rows);
    Ok::<_, anyhow::Error>(wtr.flush()?)
  });

  counter_task.await??;
  let _ = stop_tx.send(());
  timer_task.await?;

  Ok(())
}

#[tauri::command]
pub async fn rename(
  path: String,
  headers: String,
  mode: String,
  app_handle: AppHandle,
) -> Result<String, String> {
  let start_time = Instant::now();

  match rename_headers(path, headers, mode.as_str(), app_handle).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("{err}")),
  }
}
