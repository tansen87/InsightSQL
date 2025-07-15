use std::{fs::File, path::Path, time::Instant};

use anyhow::Result;
use csv::{ByteRecord, ReaderBuilder};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tauri::{Emitter, Window};

use crate::utils::CsvOptions;

pub async fn count_rows<P: AsRef<Path> + Send + Sync>(path: P) -> Result<u64> {
  let csv_options = CsvOptions::new(&path);

  let count = match csv_options.indexed()? {
    Some(idx) => idx.count(),
    None => count_record(&path, true, &csv_options)?,
  };

  Ok(count)
}

/// Used to check for counting errors caused by double quotation marks in CSV files
pub async fn count_check<P: AsRef<Path> + Send + Sync>(path: P) -> Result<u64> {
  let csv_options = CsvOptions::new(&path);
  let quoting_true = count_record(&path, true, &csv_options)?;
  let quoting_false = count_record(&path, false, &csv_options)?;

  let max_count = std::cmp::max(quoting_true, quoting_false);
  let min_count = std::cmp::min(quoting_true, quoting_false);

  Ok(max_count - min_count)
}

fn count_record<P: AsRef<Path> + Send + Sync>(
  path: P,
  quoting: bool,
  csv_options: &CsvOptions<P>,
) -> Result<u64> {
  let mut rdr = ReaderBuilder::new()
    .delimiter(csv_options.detect_separator()?)
    .quoting(quoting)
    .from_reader(File::open(&path)?);

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
  start_time: Instant,
  window: &Window,
) -> Result<(), String> {
  let filename = Path::new(file).file_name().unwrap().to_str().unwrap();

  if let Err(e) = window
    .emit("start-count", &filename)
    .map_err(|e| e.to_string())
  {
    return Err(e);
  }

  match mode {
    "index" => match crate::cmd::idx::create_index(file).await {
      Ok(_) => {
        let end_time = Instant::now();
        let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
        window
          .emit("count-msg", format!("{filename}|{elapsed_time:.2} s"))
          .map_err(|e| e.to_string())?;
      }
      Err(err) => {
        window
          .emit("count-err", format!("{filename}|{err}"))
          .map_err(|e| e.to_string())?;
      }
    },
    "count" => match count_rows(file).await {
      Ok(cnt) => {
        window
          .emit("count-msg", format!("{filename}|{cnt}"))
          .map_err(|e| e.to_string())?;
      }
      Err(err) => {
        window
          .emit("count-err", format!("{filename}|{err}"))
          .map_err(|e| e.to_string())?;
      }
    },
    _ => match count_check(file).await {
      Ok(cnt) => {
        window
          .emit("count-msg", format!("{filename}|{cnt}"))
          .map_err(|e| e.to_string())?;
      }
      Err(err) => {
        window
          .emit("count-err", format!("{filename}|{err}"))
          .map_err(|e| e.to_string())?;
      }
    },
  }

  Ok(())
}

fn parallel_process(
  file: &str,
  mode: &str,
  start_time: Instant,
  window: &Window,
) -> Result<(), String> {
  let filename = Path::new(file).file_name().unwrap().to_str().unwrap();

  if let Err(e) = window
    .emit("start-count", &filename)
    .map_err(|e| e.to_string())
  {
    return Err(e);
  }

  match mode {
    "index" => {
      let _ = tauri::async_runtime::block_on(async {
        match crate::cmd::idx::create_index(file).await {
          Ok(_) => {
            let elapsed_time = start_time.elapsed().as_secs_f64();
            if let Err(e) = window
              .emit("count-msg", format!("{filename}|{elapsed_time:.2} s"))
              .map_err(|e| e.to_string())
            {
              return Err(e);
            }
          }
          Err(err) => {
            if let Err(e) = window
              .emit("count-err", format!("{filename}|{err}"))
              .map_err(|e| e.to_string())
            {
              return Err(e);
            }
          }
        }
        Ok(())
      });
    }
    "count" => {
      let _ = tauri::async_runtime::block_on(async {
        match count_rows(file).await {
          Ok(cnt) => {
            if let Err(e) = window
              .emit("count-msg", format!("{filename}|{cnt}"))
              .map_err(|e| e.to_string())
            {
              return Err(e);
            }
          }
          Err(err) => {
            if let Err(e) = window
              .emit("count-err", format!("{filename}|{err}"))
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
        match count_check(file).await {
          Ok(cnt) => {
            if let Err(e) = window
              .emit("count-msg", format!("{filename}|{cnt}"))
              .map_err(|e| e.to_string())
            {
              return Err(e);
            }
          }
          Err(err) => {
            if let Err(e) = window
              .emit("count-err", format!("{filename}|{err}"))
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
pub async fn count(path: String, mode: String, window: Window) -> Result<String, String> {
  let start_time = Instant::now();
  let paths: Vec<&str> = path.split('|').collect();

  let result = if paths.len() > 1 {
    paths
      .par_iter()
      .try_for_each(|file| parallel_process(file, &mode, start_time, &window))
  } else {
    Ok(if let Some(file) = paths.first() {
      single_process(file, &mode, start_time, &window).await?;
    })
  };

  if let Err(e) = result {
    return Err(e);
  }

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  Ok(format!("{elapsed_time:.2}"))
}
