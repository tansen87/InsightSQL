use std::{
  collections::HashMap,
  fs::File,
  io::BufWriter,
  path::Path,
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
  time::{Duration, Instant},
};

use anyhow::{Result, anyhow};
use csv::{ReaderBuilder, WriterBuilder};
use tauri::AppHandle;
use tokio::sync::oneshot;

use crate::{
  io::csv::{options::CsvOptions, selection::Selection},
  utils::{EventEmitter, WTR_BUFFER_SIZE},
};

pub async fn fill_null<E, P>(
  path: P,
  fill_column: String,
  fill_value: String,
  mode: String,
  quoting: bool,
  progress: bool,
  emitter: E,
) -> Result<()>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync,
{
  let opts = CsvOptions::new(&path);
  let sep = opts.detect_separator()?;
  let output_path = opts.output_path(Some("fill"), None)?;

  let total_rows = match progress {
    true => opts.idx_count_rows().await?,
    false => 0,
  };
  emitter.emit_total_rows(total_rows).await?;

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .quoting(quoting)
    .from_reader(opts.rdr_skip_rows()?);

  let fill_columns: Vec<&str> = fill_column.split('|').collect();
  let sel = Selection::from_headers(rdr.byte_headers()?, &fill_columns[..])?;

  let output_file = File::create(output_path)?;
  let buf_wtr = BufWriter::with_capacity(WTR_BUFFER_SIZE, output_file);
  let mut wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_wtr);

  wtr.write_record(rdr.headers()?)?;

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
          _ = (&mut stop_rx) => {
            break;
          }
      }
    }
  });

  let counter_task = tokio::task::spawn_blocking(move || {
    // initialize forward filled cache if needed
    let mut forward_fill_cache: HashMap<usize, String> = HashMap::new();

    for record in rdr.deserialize() {
      let mut row: Vec<String> = record?;
      for &index in sel.get_indices() {
        match mode.as_str() {
          // Fill null values
          "fill" => {
            if row.get(index).map_or(true, |s| s.is_empty()) {
              row[index] = fill_value.clone();
            }
          }
          // Fill null values by propagating the last valid observation to next valid
          // just like `pandas.Series.ffill`
          "ffill" => {
            if row.get(index).map_or(true, |s| s.is_empty()) {
              if let Some(fill_val) = forward_fill_cache.get(&index) {
                row[index] = fill_val.clone();
              }
            } else {
              forward_fill_cache.insert(index, row[index].clone());
            }
          }
          _ => return Err(anyhow!("Not supported fill mode")),
        }
      }
      wtr.write_record(&row)?;

      rows.fetch_add(1, Ordering::Relaxed);
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
pub async fn fill(
  path: String,
  columns: String,
  values: String,
  mode: String,
  quoting: bool,
  progress: bool,
  app_handle: AppHandle,
) -> Result<String, String> {
  let start_time = Instant::now();

  match fill_null(path, columns, values, mode, quoting, progress, app_handle).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("{err}")),
  }
}
