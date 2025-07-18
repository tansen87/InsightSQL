use std::{
  path::{Path, PathBuf},
  sync::{Arc, Mutex},
  time::Duration,
};

use anyhow::Result;
use csv::{ByteRecord, ReaderBuilder, WriterBuilder};
use tauri::{Emitter, Window};
use tokio::sync::oneshot;

use crate::io::csv::options::CsvOptions;

/// convert csv to csv (only replace the delimiter)
pub async fn csv_to_csv<P: AsRef<Path> + Send + Sync>(
  path: P,
  write_sep: String,
  filename: String,
  mode: &str,
  window: Window,
) -> Result<()> {
  let csv_options = CsvOptions::new(&path);
  let sep = csv_options.detect_separator()?;
  let write_sep = if write_sep == "\\t" {
    b'\t'
  } else {
    write_sep.into_bytes()[0]
  };
  let parent_path = path.as_ref().parent().unwrap().to_str().unwrap();
  let file_stem = path.as_ref().file_stem().unwrap().to_str().unwrap();
  let mut output_path = PathBuf::from(parent_path);
  output_path.push(format!("{file_stem}.fmt.csv"));

  let total_rows = match mode {
    "idx" => csv_options.idx_count_rows().await?,
    "std" => csv_options.std_count_rows()?,
    _ => 0,
  };
  window.emit("total-rows", format!("{filename}|{total_rows}"))?;

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(csv_options.rdr_skip_rows()?);

  let mut wtr = WriterBuilder::new()
    .delimiter(write_sep)
    .from_path(output_path)?;
  wtr.write_record(rdr.headers()?)?;

  let rows = Arc::new(Mutex::new(0));
  let rows_clone = Arc::clone(&rows);
  let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
  let (done_tx, mut done_rx) = oneshot::channel::<usize>();

  let timer_task = tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_millis(300));
    loop {
      tokio::select! {
        _ = interval.tick() => {
          let current_rows = *rows_clone.lock().unwrap();
          if let Err(err) = window.emit("update-rows", format!("{filename}|{current_rows}")) {
            let _ = window.emit("to-err", format!("{filename}|{err}"));
          }
        },
        Ok(final_rows) = (&mut done_rx) => {
          if let Err(err) = window.emit("update-rows", format!("{filename}|{final_rows}")) {
            let _ = window.emit("to-err", format!("{filename}|{err}"));
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
      let mut count = rows.lock().unwrap();
      *count += 1;
    }

    let final_rows = *rows.lock().unwrap();
    let _ = done_tx.send(final_rows);

    Ok::<_, anyhow::Error>(wtr.flush()?)
  });

  counter_task.await??;
  let _ = stop_tx.send(());
  timer_task.await?;

  Ok(())
}
