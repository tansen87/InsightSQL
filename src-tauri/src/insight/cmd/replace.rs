use std::{
  borrow::Cow,
  fs::File,
  io::{BufReader, BufWriter, Cursor, Read},
  path::PathBuf,
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
  time::{Duration, Instant},
};

use anyhow::Result;
use csv::{ByteRecord, ReaderBuilder, WriterBuilder};
use rayon::{
  ThreadPoolBuilder,
  iter::{IntoParallelIterator, ParallelIterator},
};
use regex::bytes::RegexBuilder;
use tauri::AppHandle;
use tempfile::TempDir;
use tokio::sync::oneshot;

use crate::{
  io::csv::{config::CsvConfigBuilder, options::CsvOptions, selection::Selection},
  utils::{self, EventEmitter, MmapOffsets},
};

pub async fn regex_replace<E>(
  mut rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  mut wtr: csv::Writer<BufWriter<File>>,
  opts: CsvOptions<String>,
  column: String,
  regex_pattern: String,
  replacement: String,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
{
  let pattern = RegexBuilder::new(&regex_pattern).build()?;

  let total_rows = match progress {
    true => opts.idx_count_rows().await?,
    false => 0,
  };
  emitter.emit_total_rows(total_rows).await?;

  let headers = rdr.byte_headers()?;
  let sel = Selection::from_headers(headers, &[column.as_str()][..])?;

  wtr.write_record(headers)?;

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
    let mut record = ByteRecord::new();
    while rdr.read_byte_record(&mut record)? {
      record = record
        .into_iter()
        .enumerate()
        .map(|(idx, val)| {
          if sel.get_indices().contains(&idx) {
            if pattern.is_match(val) {
              match_rows.fetch_add(1, Ordering::Relaxed);
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

  let final_match_rows = match_rows_clone.load(Ordering::Relaxed);
  Ok(final_match_rows.to_string())
}

pub fn regex_replace_parallel(
  mut wtr: csv::Writer<BufWriter<File>>,
  opts: CsvOptions<String>,
  column: String,
  pattern: String,
  replacement: String,
  threads: usize,
) -> Result<String> {
  let pattern = RegexBuilder::new(&pattern).build()?;
  let sep = opts.get_delimiter()?;
  let total_data_rows = opts
    .indexed()?
    .ok_or_else(|| anyhow::anyhow!("No indexed file, create index first"))?
    .count() as usize;
  if total_data_rows == 0 {
    return Ok("0".to_string());
  }

  let offsets = Arc::new(MmapOffsets::from_file(opts.idx_path())?);
  let total_records = offsets.len();
  if total_records != total_data_rows + 1 {
    anyhow::bail!(
      "Index inconsistency: total_records={} vs data_rows={}",
      total_records,
      total_data_rows
    );
  }

  let csv_file = File::open(opts.file_path()?)?;
  let csv_mmap = unsafe { memmap2::Mmap::map(&csv_file)? };
  let csv_mmap = Arc::new(csv_mmap);

  // Parse header
  let header_start = offsets.get(0) as usize;
  let header_end = offsets.get(1) as usize;
  if header_start > header_end || header_end > csv_mmap.len() {
    anyhow::bail!(
      "Invalid header range: [{}..{}) in file of size {}",
      header_start,
      header_end,
      csv_mmap.len()
    );
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
  let true_header = utils::clean_header(&raw_header);
  let sel = Selection::from_headers(&true_header, &[column.as_str()])?;
  let field_index = sel.first_indices()?;

  // Write header to output
  wtr.write_byte_record(&true_header)?;

  let njobs = utils::njobs(Some(threads));
  let chunk_size_bytes = 256 * 1024 * 1024; // 256MB
  let file_size = csv_mmap.len();
  let num_chunks = (file_size + chunk_size_bytes - 1) / chunk_size_bytes;

  let temp_dir = TempDir::new()?;

  let pool = ThreadPoolBuilder::new()
    .num_threads(njobs)
    .build()
    .map_err(|e| anyhow::anyhow!("Failed to create thread pool: {}", e))?;

  let results: Result<Vec<(PathBuf, usize)>> = pool.install(|| {
    (0..num_chunks)
      .into_par_iter()
      .map(|chunk_id| -> Result<(PathBuf, usize)> {
        let start_byte = chunk_id * chunk_size_bytes;
        if start_byte >= file_size {
          return Ok((PathBuf::new(), 0));
        }

        // Find physical row indices for this chunk
        let mut phys_start_idx = 1; // skip header
        if offsets.len() <= 1 {
          return Ok((PathBuf::new(), 0));
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
        let mut local_wtr = WriterBuilder::new().delimiter(sep).from_path(&out_path)?;

        let mut match_count = 0;
        for record_result in reader.into_byte_records() {
          let record = record_result?;

          let mut new_fields: Vec<Vec<u8>> = Vec::with_capacity(record.len());
          let mut matched = false;

          for (idx, field) in record.iter().enumerate() {
            if idx == field_index {
              if pattern.is_match(field) {
                matched = true;
                let replaced = pattern.replace_all(field, replacement.as_bytes());
                new_fields.push(replaced.into_owned());
              } else {
                new_fields.push(field.to_vec());
              }
            } else {
              new_fields.push(field.to_vec());
            }
          }

          if matched {
            match_count += 1;
          }

          let new_record = ByteRecord::from(new_fields);
          local_wtr.write_byte_record(&new_record)?;
        }

        local_wtr.flush()?;
        Ok((out_path, match_count))
      })
      .collect()
  });

  let mut total_matches = 0;
  for (path, count) in results? {
    if path.as_os_str().is_empty() {
      continue;
    }
    total_matches += count;

    let mut part_file = File::open(&path)?;
    let mut part_reader = ReaderBuilder::new()
      .delimiter(sep)
      .has_headers(false)
      .from_reader(&mut part_file);

    for record_result in part_reader.byte_records() {
      let record = record_result?;
      wtr.write_byte_record(&record)?;
    }
  }

  wtr.flush()?;
  Ok(total_matches.to_string())
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
  threads: usize,
  emitter: AppHandle,
) -> Result<(String, String), String> {
  let start_time = Instant::now();
  let mut opts = CsvOptions::new(path.clone());
  opts.set_skiprows(skiprows);
  let (sep, reader) = opts.skiprows_and_delimiter().map_err(|e| format!("{e}"))?;
  let output_path = opts
    .output_path(Some("replace"), None)
    .map_err(|e| format!("create output falied: {e}"))?;
  let config = CsvConfigBuilder::new()
    .flexible(flexible)
    .delimiter(sep)
    .quoting(quoting)
    .build();
  let rdr = config.build_reader(reader);
  let wtr = config
    .build_writer(&output_path)
    .map_err(|e| format!("build writer failed: {e}"))?;

  let replaced_rows = match threads {
    1 => regex_replace(
      rdr,
      wtr,
      opts,
      column,
      regex_pattern,
      replacement,
      progress,
      emitter,
    )
    .await
    .map_err(|e| format!("{e}"))?,
    _ => tokio::task::spawn_blocking(move || {
      regex_replace_parallel(wtr, opts, column, regex_pattern, replacement, threads)
    })
    .await
    .map_err(|e| format!("parallel replace error: {e}"))?
    .map_err(|e| format!("Task join error: {}", e))?,
  };

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  Ok((replaced_rows, format!("{elapsed_time:.0}")))
}
