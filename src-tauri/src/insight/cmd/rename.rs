use std::{
  path::Path,
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
  time::{Duration, Instant},
};

use anyhow::Result;
use csv::{ByteRecord, Reader};
use tauri::AppHandle;
use tokio::sync::oneshot;

use crate::{
  io::csv::{config::CsvConfigBuilder, options::CsvOptions},
  utils::EventEmitter,
};

pub async fn rename_headers<E, P>(
  path: P,
  r_header: String,
  progress: bool,
  quoting: bool,
  skiprows: usize,
  flexible: bool,
  emitter: E,
) -> Result<()>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync,
{
  let mut opts = CsvOptions::new(&path);
  opts.set_skiprows(skiprows);
  let (sep, reader) = opts.skiprows_and_delimiter()?;
  let output_path = opts.output_path(Some("rename"), None)?;

  let total_rows = match progress {
    true => opts.idx_count_rows().await?,
    false => 0,
  };
  emitter.emit_total_rows(total_rows).await?;

  let config = CsvConfigBuilder::new()
    .flexible(flexible)
    .delimiter(sep)
    .quoting(quoting)
    .build();

  let mut rdr = config.build_reader(reader);
  let mut wtr = config.build_writer(&output_path)?;

  let mut new_rdr = Reader::from_reader(r_header.as_bytes());
  let new_headers = new_rdr.byte_headers()?;

  wtr.write_record(new_headers)?;

  let rows = Arc::new(AtomicUsize::new(0));
  let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
  let (done_tx, mut done_rx) = oneshot::channel::<usize>();

  let timer_task = if progress {
    let rows_clone = Arc::clone(&rows);

    Some(tokio::spawn(async move {
      let mut interval = tokio::time::interval(Duration::from_millis(500));
      loop {
        tokio::select! {
          _ = interval.tick() => {
            let current_rows = rows_clone.load(Ordering::Relaxed);
            if let Err(err) = emitter.emit_update_rows(current_rows).await {
              let _ = emitter.emit_err(&format!("failed to emit current rows: {err}")).await;
            }
          },
          Ok(final_rows) = (&mut done_rx) => {
            if let Err(err) = emitter.emit_update_rows(final_rows).await {
              let _ = emitter.emit_err(&format!("failed to emit final rows: {err}")).await;
            }
            break;
          },
          _ = (&mut stop_rx) => { break; }
        }
      }
    }))
  } else {
    None
  };

  let counter_task = tokio::task::spawn_blocking(move || {
    let mut record = ByteRecord::new();
    while rdr.read_byte_record(&mut record)? {
      wtr.write_byte_record(&record)?;

      rows.fetch_add(1, Ordering::Relaxed);
    }

    let final_rows = rows.load(Ordering::Relaxed);
    let _ = done_tx.send(final_rows);
    Ok::<_, anyhow::Error>(wtr.flush()?)
  });

  counter_task.await??;
  let _ = stop_tx.send(());
  if let Some(task) = timer_task {
    task.await?;
  }

  Ok(())
}

#[tauri::command]
pub async fn rename(
  path: String,
  headers: String,
  progress: bool,
  quoting: bool,
  skiprows: usize,
  flexible: bool,
  app_handle: AppHandle,
) -> Result<String, String> {
  let start_time = Instant::now();

  match rename_headers(
    path, headers, progress, quoting, skiprows, flexible, app_handle,
  )
  .await
  {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.0}"))
    }
    Err(err) => Err(format!("{err}")),
  }
}
