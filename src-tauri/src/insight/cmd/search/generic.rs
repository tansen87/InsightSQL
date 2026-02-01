use std::{
  collections::{HashMap, HashSet},
  fs::File,
  io::{BufReader, BufWriter, Read},
  path::Path,
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
  time::Duration,
};

use anyhow::Result;
use csv::{ReaderBuilder, Writer, WriterBuilder};
use tokio::sync::oneshot;

use crate::{
  io::csv::{options::CsvOptions, selection::Selection},
  utils::EventEmitter,
};

fn sanitize_condition(condition: &str) -> String {
  condition
    .chars()
    .map(|c| match c {
      '/' | '\\' | '|' | ',' | '.' | '"' | ':' => '-',
      _ => c,
    })
    .collect()
}

pub(crate) async fn generic_search<E, F>(
  mut rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  mut wtr: csv::Writer<BufWriter<File>>,
  column: String,
  conditions: Vec<String>,
  progress: bool,
  match_fn: F,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
  F: Fn(&str, &[String]) -> bool + Send + Sync + 'static,
{
  let sel = Selection::from_headers(rdr.byte_headers()?, &[column.as_str()][..])?;

  wtr.write_record(rdr.headers()?)?;

  let rows = Arc::new(AtomicUsize::new(0));
  let match_rows = Arc::new(AtomicUsize::new(0));
  let match_rows_clone = Arc::clone(&match_rows);
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
    for result in rdr.records() {
      let record = result?;
      if let Some(value) = record.get(sel.first_indices()?) {
        if match_fn(value, &conditions) {
          wtr.write_record(&record)?;

          match_rows.fetch_add(1, Ordering::Relaxed);
        }
      }
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

  let final_match_rows = match_rows_clone.load(Ordering::Relaxed);
  Ok(final_match_rows.to_string())
}

pub(crate) async fn generic_multi_search<E, F, P>(
  path: P,
  column: String,
  conditions: Vec<String>,
  skiprows: usize,
  quoting: bool,
  progress: bool,
  match_fn: F,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
  F: Fn(&str, &String) -> bool + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync + 'static,
{
  let unique_conditions = conditions
    .into_iter()
    .collect::<HashSet<_>>()
    .into_iter()
    .collect::<Vec<_>>();
  let conditions = Arc::new(unique_conditions);
  let match_fn = Arc::new(match_fn);
  let mut opts = CsvOptions::new(&path);
  opts.set_skiprows(skiprows);
  let (sep, reader) = opts.skiprows_and_delimiter()?;

  let total_rows = match progress {
    true => opts.idx_count_rows().await?,
    false => 0,
  };
  emitter.emit_total_rows(total_rows).await?;

  // prepare writers for each condition with sanitized output paths
  let parent_path = opts.parent_path()?;
  let file_stem = opts.file_stem()?;
  let output_paths: HashMap<String, String> = conditions
    .iter()
    .map(|cond| {
      let sanitized = sanitize_condition(cond);
      let path = format!("{parent_path}/{file_stem}_{sanitized}.csv");
      (cond.clone(), path)
    })
    .collect();

  let rows = Arc::new(AtomicUsize::new(0));
  let match_rows = Arc::new(AtomicUsize::new(0));
  let match_rows_clone = Arc::clone(&match_rows);
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
    let mut writers: HashMap<String, Writer<std::fs::File>> = HashMap::new();

    for (cond, path) in &output_paths {
      let file = File::create(path)?;
      writers.insert(
        cond.clone(),
        WriterBuilder::new().delimiter(sep).from_writer(file),
      );
    }

    let mut rdr = ReaderBuilder::new()
      .delimiter(sep)
      .quoting(quoting)
      .from_reader(reader);
    let headers = rdr.headers()?.clone();

    for wtr in writers.values_mut() {
      wtr.write_record(&headers)?;
    }

    let sel = Selection::from_headers(rdr.byte_headers()?, &[column.as_str()][..])?;

    for result in rdr.records() {
      let record = result?;
      if let Some(value) = record.get(sel.first_indices()?) {
        for condition in conditions.iter() {
          if match_fn(value, condition) {
            if let Some(wtr) = writers.get_mut(condition) {
              wtr.write_record(&record)?;

              match_rows.fetch_add(1, Ordering::Relaxed);
            }
          }
        }
      }
      rows.fetch_add(1, Ordering::Relaxed);
    }
    let final_rows = rows.load(Ordering::Relaxed);
    let _ = done_tx.send(final_rows);

    // flush all writers
    for wtr in writers.values_mut() {
      wtr.flush()?;
    }
    Ok::<_, anyhow::Error>(())
  });

  counter_task.await??;
  let _ = stop_tx.send(());
  if let Some(task) = timer_task {
    task.await?;
  }

  let final_match_rows = match_rows_clone.load(Ordering::Relaxed);
  Ok(final_match_rows.to_string())
}
