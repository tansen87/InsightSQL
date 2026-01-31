use std::{
  collections::{BTreeMap, HashSet},
  path::Path,
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
  time::{Duration, Instant},
};

use anyhow::Result;
use csv::ByteRecord;
use tauri::AppHandle;
use tokio::sync::oneshot;

use crate::{
  io::csv::{config::CsvConfigBuilder, options::CsvOptions},
  utils::EventEmitter,
};

#[derive(Debug, Clone, Copy)]
pub enum SelectMode {
  Include,
  Exclude,
}

impl From<&str> for SelectMode {
  fn from(mode: &str) -> Self {
    match mode {
      "include" => SelectMode::Include,
      "exclude" => SelectMode::Exclude,
      _ => SelectMode::Include,
    }
  }
}

pub async fn select_columns<E, P>(
  path: P,
  sel_cols: String,
  sel_mode: SelectMode,
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
  let mut opts = CsvOptions::new(path);
  opts.set_skiprows(skiprows);
  let (sep, reader) = opts.skiprows_and_delimiter()?;
  let output_path = opts.output_path(Some("select"), None)?;
  let col_names: HashSet<&str> = sel_cols.split('|').collect();

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

  let headers: Vec<String> = rdr.headers()?.iter().map(|s| s.to_string()).collect();

  // 构建一个 header -> index 的映射表,便于快速查找
  let header_to_index: BTreeMap<&str, usize> = headers
    .iter()
    .enumerate()
    .map(|(i, h)| (h.as_str(), i))
    .collect();

  let (col_indices, output_headers): (Vec<usize>, Vec<String>) = match sel_mode {
    SelectMode::Include => {
      let mut indices = Vec::new();
      let mut out_headers = Vec::new();
      for col_name in sel_cols.split('|') {
        if let Some(&index) = header_to_index.get(col_name) {
          indices.push(index);
          out_headers.push(col_name.to_string());
        }
      }
      (indices, out_headers)
    }
    SelectMode::Exclude => {
      let mut indices = Vec::new();
      let mut out_headers = Vec::new();
      for (i, header) in headers.iter().enumerate() {
        if !col_names.contains(header.as_str()) {
          indices.push(i);
          out_headers.push(header.clone());
        }
      }
      (indices, out_headers)
    }
  };

  wtr.write_record(output_headers.iter())?;

  let mut record = ByteRecord::new();

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
    while rdr.read_byte_record(&mut record)? {
      let selected_data: Vec<&[u8]> = col_indices
        .iter()
        .map(|&i| {
          if i < record.len() {
            &record[i]
          } else {
            "".as_bytes()
          }
        })
        .collect();
      wtr.write_record(selected_data.iter())?;

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
pub async fn select(
  path: String,
  sel_cols: String,
  sel_mode: String,
  progress: bool,
  quoting: bool,
  skiprows: usize,
  flexible: bool,
  app_handle: AppHandle,
) -> Result<String, String> {
  let start_time = Instant::now();

  let sel_mode: SelectMode = sel_mode.as_str().into();

  match select_columns(
    path, sel_cols, sel_mode, progress, quoting, skiprows, flexible, app_handle,
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
