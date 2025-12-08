use std::{
  fs::File,
  io::BufWriter,
  path::Path,
  sync::{Arc, Mutex},
  time::{Duration, Instant},
};

use anyhow::{Result, anyhow};
use csv::{ByteRecord, ReaderBuilder, WriterBuilder};
use tauri::AppHandle;
use tokio::sync::oneshot;

use crate::{io::csv::options::CsvOptions, utils::EventEmitter};

pub async fn enumerate_index<E, P>(path: P, mode: &str, emitter: E) -> Result<()>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync,
{
  let opts = CsvOptions::new(&path);
  let sep = opts.detect_separator()?;
  let output_path = opts.output_path(Some("enumer"), None)?;

  let total_rows = match mode {
    "idx" => opts.idx_count_rows().await?,
    _ => 0,
  };
  emitter.emit_total_rows(total_rows).await?;

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(opts.rdr_skip_rows()?);

  let buf_writer = BufWriter::with_capacity(256_000, File::create(output_path)?);
  let mut wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_writer);

  let headers = rdr.headers()?;
  let mut new_headers = vec![String::from("enumerate_idx")];
  new_headers.extend(headers.into_iter().map(String::from));
  wtr.write_record(&new_headers)?;

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
  });

  let counter_task = tokio::task::spawn_blocking(move || {
    let mut record = ByteRecord::new();
    while rdr.read_byte_record(&mut record)? {
      let i = {
        let mut count = rows
          .lock()
          .map_err(|poison| anyhow!("cnt lock poisoned: {poison}"))?;
        *count += 1;
        *count
      };

      let mut new_record = vec![i.to_string()];
      new_record.extend(
        record
          .iter()
          .map(|field| String::from_utf8_lossy(field).into_owned()),
      );
      wtr.write_record(&new_record)?;
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
pub async fn enumer(path: String, mode: String, app_handle: AppHandle) -> Result<String, String> {
  let start_time = Instant::now();

  match enumerate_index(path, mode.as_str(), app_handle).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("{err}")),
  }
}
