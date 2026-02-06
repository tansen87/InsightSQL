use std::{
  collections::{HashMap, HashSet},
  fs::File,
  io::{BufReader, BufWriter, Cursor, Read},
  path::{Path, PathBuf},
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
  time::Duration,
};

use anyhow::Result;
use csv::{ByteRecord, ReaderBuilder, Writer, WriterBuilder};
use rayon::{
  ThreadPoolBuilder,
  iter::{IntoParallelIterator, ParallelIterator},
};
use tempfile::TempDir;
use tokio::sync::oneshot;

use crate::{
  index::Indexed,
  io::csv::{options::CsvOptions, selection::Selection},
  utils::{EventEmitter, MmapOffsets},
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

fn clean_header(header: &ByteRecord) -> ByteRecord {
  let mut cleaned = ByteRecord::new();
  for field in header {
    if field.len() >= 3 && &field[0..3] == b"\xEF\xBB\xBF" {
      cleaned.push_field(&field[3..]);
    } else {
      cleaned.push_field(field);
    }
  }
  cleaned
}

pub(crate) fn generic_parallel_search<F>(
  opts: CsvOptions<String>,
  idx: &mut Indexed<File, File>,
  mut wtr: Writer<BufWriter<File>>,
  column: String,
  conditions: Vec<String>,
  jobs: usize,
  match_fn: F,
) -> Result<String>
where
  F: Fn(&str, &[String]) -> bool + Send + Sync + 'static,
{
  let total_data_rows = idx.count() as usize;
  if total_data_rows == 0 {
    return Ok("0".to_string());
  }

  let offsets = Arc::new(MmapOffsets::from_file(opts.idx_path())?);
  let total_records = offsets.len();
  let sep = opts.get_delimiter()?;

  if total_records != total_data_rows + 1 {
    return Err(anyhow::anyhow!(
      "Index inconsistency: total_records={} vs data_rows={}",
      total_records,
      total_data_rows
    ));
  }

  let csv_path = opts.file_path()?;
  let csv_file = File::open(&csv_path)?;
  let csv_mmap = unsafe { memmap2::Mmap::map(&csv_file)? };
  let csv_mmap = Arc::new(csv_mmap);

  // Parse header
  let header_start = offsets.get(0) as usize;
  let header_end = offsets.get(1) as usize;
  if header_start > header_end || header_end > csv_mmap.len() {
    return Err(anyhow::anyhow!(
      "Invalid header range: [{}..{}) in file of size {}",
      header_start,
      header_end,
      csv_mmap.len()
    ));
  }

  let header_slice = &csv_mmap[header_start..header_end];
  let mut header_reader = ReaderBuilder::new()
    .has_headers(false)
    .delimiter(sep)
    .from_reader(Cursor::new(header_slice));

  let raw_header = header_reader
    .byte_records()
    .next()
    .ok_or_else(|| anyhow::anyhow!("Failed to parse header"))??;

  let true_header = clean_header(&raw_header);
  let header_debug: Vec<_> = true_header
    .iter()
    .map(|b| String::from_utf8_lossy(b).into_owned())
    .collect();
  log::debug!("Cleaned header: {:?}", header_debug);

  let sel = Selection::from_headers(&true_header, &[column.as_str()])?;
  let field_index = sel.first_indices()?;

  wtr.write_byte_record(&true_header)?;

  // Configure thread count
  let effective_jobs = jobs.max(1).min(num_cpus::get());
  let chunk_size_bytes = 256 * 1024 * 1024; // 256MB
  let file_size = csv_mmap.len();
  let num_chunks = (file_size + chunk_size_bytes - 1) / chunk_size_bytes;

  let temp_dir = TempDir::new()?;

  let pool = ThreadPoolBuilder::new()
    .num_threads(effective_jobs)
    .build()
    .map_err(|e| anyhow::anyhow!("Failed to create thread pool: {}", e))?;

  let results: Result<Vec<(PathBuf, usize)>> = Ok(pool.install(|| {
    (0..num_chunks)
      .into_par_iter()
      .map(|chunk_id| -> Result<(PathBuf, usize)> {
        let start_byte = chunk_id * chunk_size_bytes;
        if start_byte >= file_size {
          return Ok((PathBuf::new(), 0));
        }

        // Skip header: data rows start at index 1
        let mut phys_start_idx = 1;
        if offsets.len() <= 1 {
          return Ok((PathBuf::new(), 0)); // no data rows
        }
        for i in 1..offsets.len() {
          if offsets.get(i) as usize >= start_byte {
            phys_start_idx = i;
            break;
          }
        }

        let next_chunk_start = (chunk_id + 1) * chunk_size_bytes;
        let mut phys_end_idx = offsets.len();
        for i in phys_start_idx..offsets.len() {
          if offsets.get(i) as usize >= next_chunk_start {
            phys_end_idx = i;
            break;
          }
        }

        if phys_start_idx >= phys_end_idx {
          return Ok((PathBuf::new(), 0));
        }

        let chunk_start = offsets.get(phys_start_idx) as usize;
        let chunk_end = if phys_end_idx < offsets.len() {
          offsets.get(phys_end_idx) as usize
        } else {
          file_size
        };

        if chunk_start >= file_size || chunk_start >= chunk_end {
          return Ok((PathBuf::new(), 0));
        }

        let slice = &csv_mmap[chunk_start..chunk_end];
        let reader = ReaderBuilder::new()
          .has_headers(false)
          .delimiter(sep)
          .from_reader(Cursor::new(slice));

        let out_path = temp_dir.path().join(format!("part_{}.csv", chunk_id));

        let mut local_wtr = WriterBuilder::new()
          .delimiter(sep) // or Never, but be consistent
          .from_path(&out_path)?;

        let mut count = 0;
        for record_result in reader.into_byte_records() {
          let record = record_result?;
          if let Some(value) = record.get(field_index) {
            if let Ok(s) = std::str::from_utf8(value) {
              if match_fn(s, &conditions) {
                local_wtr.write_byte_record(&record)?;
                count += 1;
              }
            }
          }
        }
        local_wtr.flush()?;
        Ok((out_path, count))
      })
      .collect::<Result<Vec<_>, _>>()
  })?);

  let mut total = 0;
  for (path, count) in results? {
    if count == 0 || path.as_os_str().is_empty() {
      continue;
    }

    let mut part_file = File::open(&path)?;
    let part_reader = ReaderBuilder::new()
      .delimiter(sep)
      .has_headers(false) // temp files have no header
      .from_reader(&mut part_file);

    for record_result in part_reader.into_byte_records() {
      let record = record_result?;
      wtr.write_byte_record(&record)?;
      total += 1;
    }
  }

  wtr.flush()?;
  Ok(total.to_string())
}
