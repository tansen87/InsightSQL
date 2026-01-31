use std::{
  fs::File,
  io::BufWriter,
  path::{Path, PathBuf},
  time::Instant,
};

use anyhow::Result;
use csv::ReaderBuilder;
use csv_index::RandomAccessSimple;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tauri::{Emitter, Window};

use crate::{io::csv::options::CsvOptions, utils::WTR_BUFFER_SIZE};

pub async fn create_index<P: AsRef<Path> + Send + Sync>(
  path: P,
  quoting: bool,
  skiprows: usize,
) -> Result<()> {
  let mut opts = CsvOptions::new(&path);
  opts.set_skiprows(skiprows);
  let (sep, reader) = opts.skiprows_and_delimiter()?;
  let file_name = opts.file_name()?;
  let mut output_path = PathBuf::from(opts.parent_path()?);
  output_path.push(format!("{file_name}.idx"));

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .quoting(quoting)
    .from_reader(reader);

  let mut wtr = BufWriter::with_capacity(WTR_BUFFER_SIZE, File::create(output_path)?);

  RandomAccessSimple::create(&mut rdr, &mut wtr)?;

  Ok(())
}

async fn single_process(
  file: &str,
  quoting: bool,
  skiprows: usize,
  start_time: Instant,
  window: &Window,
) -> Result<(), String> {
  let opts = CsvOptions::new(file);
  let filename = opts.file_name().map_err(|e| e.to_string())?;

  window.emit("info", filename).map_err(|e| e.to_string())?;

  match create_index(file, quoting, skiprows).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      window
        .emit("success", format!("{filename}|{elapsed_time:.0} s"))
        .map_err(|e| e.to_string())?;
    }
    Err(err) => {
      window
        .emit("err", format!("{filename}|{err}"))
        .map_err(|e| e.to_string())?;
    }
  }

  Ok(())
}

fn parallel_process(
  file: &str,
  quoting: bool,
  skiprows: usize,
  start_time: Instant,
  window: &Window,
) -> Result<(), String> {
  let opts = CsvOptions::new(file);
  let filename = opts.file_name().map_err(|e| e.to_string())?;

  window.emit("info", filename).map_err(|e| e.to_string())?;

  let _ = tauri::async_runtime::block_on(async {
    match create_index(file, quoting, skiprows).await {
      Ok(_) => {
        let elapsed_time = start_time.elapsed().as_secs_f64();
        if let Err(e) = window
          .emit("success", format!("{filename}|{elapsed_time:.0} s"))
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

  Ok(())
}

#[tauri::command]
pub async fn csv_idx(
  path: String,
  quoting: bool,
  skiprows: usize,
  window: Window,
) -> Result<String, String> {
  let start_time = Instant::now();
  let paths: Vec<&str> = path.split('|').collect();

  let result = if paths.len() > 1 {
    paths
      .par_iter()
      .try_for_each(|file| parallel_process(file, quoting, skiprows, start_time, &window))
  } else {
    Ok(if let Some(file) = paths.first() {
      single_process(file, quoting, skiprows, start_time, &window).await?;
    })
  };

  result?;

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  Ok(format!("{elapsed_time:.0}"))
}
