use std::{
  fs::File,
  io::BufWriter,
  path::{Path, PathBuf},
  time::Instant,
};

use anyhow::Result;
use csv::ReaderBuilder;
use csv_index::RandomAccessSimple;

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

#[tauri::command]
pub async fn idx(path: String, quoting: bool, skiprows: usize) -> Result<String, String> {
  let start_time = Instant::now();

  match create_index(path, quoting, skiprows).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("{err}")),
  }
}
