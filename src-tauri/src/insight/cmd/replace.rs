use std::{
  borrow::Cow,
  path::Path,
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
  time::{Duration, Instant},
};

use anyhow::Result;
use csv::ByteRecord;
use regex::bytes::RegexBuilder;
use tauri::AppHandle;
use tokio::sync::oneshot;

use crate::{
  io::csv::{config::CsvConfigBuilder, options::CsvOptions, selection::Selection},
  utils::EventEmitter,
};

pub async fn regex_replace<E, P>(
  path: P,
  sel: String,
  regex_pattern: String,
  replacement: String,
  quoting: bool,
  progress: bool,
  skiprows: usize,
  flexible: bool,
  emitter: E,
) -> Result<()>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync,
{
  let pattern = RegexBuilder::new(&regex_pattern).build()?;
  let mut opts = CsvOptions::new(path);
  opts.set_skiprows(skiprows);
  let (sep, reader) = opts.skiprows_and_delimiter()?;
  let output_path = opts.output_path(Some("replace"), None)?;

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

  let headers = rdr.byte_headers()?;
  let sel = Selection::from_headers(headers, &[sel.as_str()][..])?;

  wtr.write_record(headers)?;

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
      record = record
        .into_iter()
        .enumerate()
        .map(|(idx, val)| {
          if sel.get_indices().contains(&idx) {
            if pattern.is_match(val) {
              pattern.replace_all(val, replacement.as_bytes())
            } else {
              Cow::Borrowed(val)
            }
          } else {
            Cow::Borrowed(val)
          }
        })
        .collect();
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
pub async fn replace(
  path: String,
  column: String,
  regex_pattern: String,
  replacement: String,
  quoting: bool,
  progress: bool,
  skiprows: usize,
  flexible: bool,
  emitter: AppHandle,
) -> Result<String, String> {
  let start_time = Instant::now();

  match regex_replace(
    path,
    column,
    regex_pattern,
    replacement,
    quoting,
    progress,
    skiprows,
    flexible,
    emitter,
  )
  .await
  {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("{err}")),
  }
}
