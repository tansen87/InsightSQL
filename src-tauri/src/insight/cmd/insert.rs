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
use tauri::AppHandle;
use tokio::sync::oneshot;

use crate::io::csv::{config::CsvConfigBuilder, options::CsvOptions, selection::Selection};
use crate::utils::EventEmitter;

fn is_relative_position(pos: &str) -> bool {
  matches!(
    pos.trim().to_lowercase().as_str(),
    "before" | "b" | "left" | "l" | "after" | "a" | "right" | "r"
  )
}

pub async fn insert_columns<E, P>(
  path: P,
  column: String,
  position: String,
  values: String,
  skiprows: usize,
  quoting: bool,
  flexible: bool,
  progress: bool,
  emitter: E,
) -> Result<(), anyhow::Error>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync,
{
  let mut opts = CsvOptions::new(path);
  opts.set_skiprows(skiprows);
  let (sep, reader) = opts.skiprows_and_delimiter()?;
  let output_path = opts.output_path(Some("insert"), None)?;

  let total_rows = if progress {
    opts.idx_count_rows().await?
  } else {
    0
  };
  emitter.emit_total_rows(total_rows).await?;

  let config = CsvConfigBuilder::new()
    .flexible(flexible)
    .delimiter(sep)
    .quoting(quoting)
    .build();

  let mut rdr = config.build_reader(reader);
  let mut wtr = config.build_writer(&output_path)?;

  let headers = rdr.byte_headers()?.clone();

  // Parse positions and check if relative positioning is used
  let pos_parts: Vec<&str> = position.split('|').collect();
  let val_parts: Vec<&str> = values.split('|').collect();

  if pos_parts.len() != val_parts.len() {
    anyhow::bail!(
      "number of positions ({}) must match number of values ({})",
      pos_parts.len(),
      val_parts.len()
    );
  }

  let has_relative = pos_parts.iter().any(|&p| is_relative_position(p));
  let n = headers.len();

  // Only resolve sel_idx if needed
  let sel_idx = if has_relative {
    if column.is_empty() {
      anyhow::bail!(
        "'column' must be specified when using relative positions (before/after/left/right)"
      );
    }
    let sel_selection = Selection::from_headers(&headers, &[column.as_str()][..])?;
    *sel_selection
      .get_indices()
      .first()
      .ok_or_else(|| anyhow::anyhow!("selected column '{}' not found", column))?
  } else {
    0 // dummy value, won't be used
  };

  // Build insertion plan
  let mut insertions: Vec<(usize, Vec<u8>)> = Vec::new();

  for (pos_str, val_str) in pos_parts.into_iter().zip(val_parts) {
    let trimmed = pos_str.trim();
    let lower = trimmed.to_lowercase();
    let target_idx = match lower.as_str() {
      "before" | "b" | "left" | "l" => sel_idx,
      "after" | "a" | "right" | "r" => sel_idx + 1,
      _ => {
        let raw_num = trimmed
          .parse::<isize>()
          .map_err(|_| anyhow::anyhow!("invalid position: '{}'", trimmed))?;

        if raw_num > 0 {
          let one_based = raw_num as usize;
          if one_based > n + 1 { n } else { one_based - 1 }
        } else if raw_num < 0 {
          let adjusted = (n as isize) + raw_num + 1;
          if adjusted < 0 {
            0
          } else if adjusted > n as isize {
            n
          } else {
            adjusted as usize
          }
        } else {
          anyhow::bail!("position index 0 is invalid; column numbering starts at 1");
        }
      }
    };

    let target_idx = target_idx.min(n).max(0);
    let value_bytes = if val_str.is_empty() {
      Vec::new()
    } else {
      val_str.as_bytes().to_vec()
    };
    insertions.push((target_idx, value_bytes));
  }

  // Sort descending to avoid index shift
  insertions.sort_by(|a, b| b.0.cmp(&a.0));

  // Write new header
  let mut new_headers: Vec<Cow<[u8]>> = headers.iter().map(|h| Cow::Borrowed(&h[..])).collect();
  for &(idx, ref val) in &insertions {
    new_headers.insert(idx, Cow::Owned(val.clone()));
  }
  wtr.write_byte_record(&ByteRecord::from(new_headers))?;

  // Process rows
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
      let mut new_record: Vec<Cow<[u8]>> = record.iter().map(|f| Cow::Borrowed(f)).collect();
      for &(idx, ref val) in &insertions {
        new_record.insert(idx, Cow::Owned(val.clone()));
      }
      wtr.write_byte_record(&ByteRecord::from(new_record))?;
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
pub async fn insert(
  path: String,
  column: String,
  position: String,
  values: String,
  skiprows: usize,
  quoting: bool,
  flexible: bool,
  progress: bool,
  emitter: AppHandle,
) -> Result<String, String> {
  let start_time = Instant::now();

  match insert_columns(
    path, column, position, values, skiprows, quoting, flexible, progress, emitter,
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
