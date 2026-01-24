use std::{
  fs::File,
  io::BufWriter,
  path::Path,
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
  time::{Duration, Instant},
};

use anyhow::Result;
use csv::{ByteRecord, ReaderBuilder, WriterBuilder};
use tauri::AppHandle;
use tokio::sync::oneshot;

use crate::{
  io::csv::options::CsvOptions,
  utils::{EventEmitter, WTR_BUFFER_SIZE},
};

pub async fn enumerate_index<E, P>(path: P, progress: bool, quoting: bool, emitter: E) -> Result<()>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync,
{
  let opts = CsvOptions::new(&path);
  let sep = opts.detect_separator()?;
  let output_path = opts.output_path(Some("enumer"), None)?;

  let total_rows = match progress {
    true => opts.idx_count_rows().await?,
    false => 0,
  };
  emitter.emit_total_rows(total_rows).await?;

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .quoting(quoting)
    .from_reader(opts.rdr_skip_rows()?);

  let output_file = File::create(output_path)?;
  let buf_wtr = BufWriter::with_capacity(WTR_BUFFER_SIZE, output_file);
  let mut wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_wtr);

  let headers = rdr.headers()?;
  let mut new_headers = vec![String::from("enumerate_idx")];
  new_headers.extend(headers.into_iter().map(String::from));
  wtr.write_record(&new_headers)?;

  let rows = Arc::new(AtomicUsize::new(0));
  let rows_clone = Arc::clone(&rows);
  let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
  let (done_tx, mut done_rx) = oneshot::channel::<usize>();

  let timer_task = tokio::spawn(async move {
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
  });

  let counter_task = tokio::task::spawn_blocking(move || {
    let mut record = ByteRecord::new();
    while rdr.read_byte_record(&mut record)? {
      let i = {
        let count = rows.fetch_add(1, Ordering::Relaxed);
        count
      };

      let mut new_record = vec![i.to_string()];
      new_record.extend(
        record
          .iter()
          .map(|field| String::from_utf8_lossy(field).into_owned()),
      );
      wtr.write_record(&new_record)?;
    }

    let final_rows = rows.load(Ordering::Relaxed);
    let _ = done_tx.send(final_rows);
    Ok::<_, anyhow::Error>(wtr.flush()?)
  });

  counter_task.await??;
  let _ = stop_tx.send(());
  timer_task.await?;

  Ok(())
}

#[tauri::command]
pub async fn enumer(
  path: String,
  progress: bool,
  quoting: bool,
  app_handle: AppHandle,
) -> Result<String, String> {
  let start_time = Instant::now();

  match enumerate_index(path, progress, quoting, app_handle).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("{err}")),
  }
}
