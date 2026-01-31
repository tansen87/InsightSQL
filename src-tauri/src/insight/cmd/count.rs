use std::{path::Path, time::Instant};

use anyhow::Result;
use csv::{ByteRecord, ReaderBuilder};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tauri::{Emitter, Window};

use crate::io::csv::options::CsvOptions;

pub async fn count_rows<P: AsRef<Path> + Send + Sync>(path: P, skiprows: usize) -> Result<u64> {
  let mut opts = CsvOptions::new(&path);
  opts.set_skiprows(skiprows);

  let count = match opts.indexed()? {
    Some(idx) => idx.count(),
    None => count_record(true, &opts).await?,
  };

  Ok(count)
}

/// Used to check for counting errors caused by double quotation marks in CSV files
pub async fn count_check<P: AsRef<Path> + Clone + Send + Sync>(
  path: P,
  skiprows: usize,
) -> Result<u64> {
  let mut opts = CsvOptions::new(&path);
  opts.set_skiprows(skiprows);

  let (c_false, c_true) = tokio::try_join!(count_record(false, &opts), count_record(true, &opts),)?;

  Ok(c_false.abs_diff(c_true))
}

async fn count_record<P: AsRef<Path> + Send + Sync>(
  quoting: bool,
  opts: &CsvOptions<P>,
) -> Result<u64> {
  let (sep, reader) = opts.skiprows_and_delimiter()?;

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .quoting(quoting)
    .from_reader(reader);

  let mut record = ByteRecord::new();
  let mut count: u64 = 0;
  while rdr.read_byte_record(&mut record)? {
    count += 1;
  }

  Ok(count)
}

async fn single_process(
  file: &str,
  mode: &str,
  skiprows: usize,
  window: &Window,
) -> Result<(), String> {
  let opts = CsvOptions::new(file);
  let filename = opts.file_name().map_err(|e| e.to_string())?;

  window.emit("info", &filename).map_err(|e| e.to_string())?;

  match mode {
    "count" => match count_rows(file, skiprows).await {
      Ok(cnt) => {
        window
          .emit("success", format!("{filename}|{cnt}"))
          .map_err(|e| e.to_string())?;
      }
      Err(err) => {
        window
          .emit("err", format!("{filename}|{err}"))
          .map_err(|e| e.to_string())?;
      }
    },
    _ => match count_check(file, skiprows).await {
      Ok(cnt) => {
        window
          .emit("success", format!("{filename}|{cnt}"))
          .map_err(|e| e.to_string())?;
      }
      Err(err) => {
        window
          .emit("err", format!("{filename}|{err}"))
          .map_err(|e| e.to_string())?;
      }
    },
  }

  Ok(())
}

fn parallel_process(
  file: &str,
  mode: &str,
  skiprows: usize,
  window: &Window,
) -> Result<(), String> {
  let opts = CsvOptions::new(file);
  let filename = opts.file_name().map_err(|e| e.to_string())?;

  window.emit("info", &filename).map_err(|e| e.to_string())?;

  match mode {
    "count" => {
      let _ = tauri::async_runtime::block_on(async {
        match count_rows(file, skiprows).await {
          Ok(cnt) => {
            if let Err(e) = window
              .emit("success", format!("{filename}|{cnt}"))
              .map_err(|e| e.to_string())
            {
              return Err(e);
            }
          }
          Err(err) => {
            if let Err(e) = window
              .emit("err", format!("{filename}|{err}"))
              .map_err(|e| e.to_string())
            {
              return Err(e);
            }
          }
        }
        Ok(())
      });
    }
    _ => {
      let _ = tauri::async_runtime::block_on(async {
        match count_check(file, skiprows).await {
          Ok(cnt) => {
            if let Err(e) = window
              .emit("success", format!("{filename}|{cnt}"))
              .map_err(|e| e.to_string())
            {
              return Err(e);
            }
          }
          Err(err) => {
            if let Err(e) = window
              .emit("err", format!("{filename}|{err}"))
              .map_err(|e| e.to_string())
            {
              return Err(e);
            }
          }
        }
        Ok(())
      });
    }
  }

  Ok(())
}

#[tauri::command]
pub async fn count(
  path: String,
  mode: String,
  skiprows: usize,
  window: Window,
) -> Result<String, String> {
  let start_time = Instant::now();
  let paths: Vec<&str> = path.split('|').collect();

  let result = if paths.len() > 1 {
    paths
      .par_iter()
      .try_for_each(|file| parallel_process(file, &mode, skiprows, &window))
  } else {
    Ok(if let Some(file) = paths.first() {
      single_process(file, &mode, skiprows, &window).await?;
    })
  };

  result?;

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  Ok(format!("{elapsed_time:.0}"))
}
