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

use anyhow::{Result, anyhow};
use csv::{ReaderBuilder, Writer, WriterBuilder};
use rayon::ThreadPoolBuilder;
use tokio::sync::oneshot;

use crate::{
  index::Indexed,
  io::csv::{options::CsvOptions, selection::Selection},
  utils::{self, EventEmitter},
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

pub(crate) async fn generic_parallel_search<F>(
  opts: CsvOptions<String>,
  idx: &mut Indexed<File, File>,
  mut wtr: csv::Writer<BufWriter<File>>,
  column: String,
  conditions: Vec<String>,
  jobs: usize,
  match_fn: F,
) -> Result<String>
where
  F: Fn(&str, &[String]) -> bool + Send + Sync + 'static,
{
  let idx_count = idx.count() as usize;
  if idx_count == 0 {
    return Ok("0".to_string());
  }

  let njobs = utils::njobs(Some(jobs));
  let chunk_size = utils::chunk_size(idx_count, njobs);
  let nchunks = utils::num_of_chunks(idx_count, chunk_size);

  let headers = idx.headers()?.clone();
  let sel = Selection::from_headers(&idx.byte_headers()?.clone(), &[column.as_str()])?;

  wtr.write_record(&headers)?;

  let (send, recv) = crossbeam_channel::bounded(nchunks);
  let pool = ThreadPoolBuilder::new()
    .num_threads(njobs)
    .build()
    .map_err(|e| anyhow!("thread pool: {e}"))?;

  let value = Arc::new(match_fn);
  for i in 0..nchunks {
    let send = send.clone();
    let sel = sel.clone();
    let opts = opts.clone();
    let conditions = conditions.clone();
    let match_fn = value.clone();

    pool.spawn(move || {
      let mut local_idx = match opts.indexed() {
        Ok(Some(idx)) => idx,
        _ => {
          // 无法打开索引,发送空结果避免死锁
          let _ = send.send(Vec::new());
          return;
        }
      };

      let start = (i * chunk_size) as u64;
      let end = ((i + 1) * chunk_size).min(idx_count) as u64;
      let count = (end - start) as usize;

      if local_idx.seek(start).is_err() {
        let _ = send.send(Vec::new());
        return;
      }

      let mut matched = Vec::new();
      for record_result in local_idx.byte_records().take(count) {
        let record = match record_result {
          Ok(r) => r,
          Err(_) => continue, // 跳过坏记录
        };
        let field_index = match sel.first_indices() {
          Ok(idx) => idx,
          Err(_) => continue,
        };

        if let Some(value) = record.get(field_index) {
          if let Ok(value_str) = std::str::from_utf8(value) {
            if match_fn(value_str, &conditions) {
              matched.push(record);
            }
          }
        }
      }
      let _ = send.send(matched);
    });
  }

  drop(send);

  let mut total_matches = 0;
  while let Ok(records) = recv.recv() {
    total_matches += records.len();
    for rec in records {
      wtr.write_byte_record(&rec)?;
    }
  }
  wtr.flush()?;

  Ok(total_matches.to_string())
}
