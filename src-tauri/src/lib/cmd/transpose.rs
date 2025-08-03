use std::{
  fs::File,
  io::BufWriter,
  path::{Path, PathBuf},
  time::Instant,
};

use anyhow::Result;
use csv::{ByteRecord, ReaderBuilder, WriterBuilder};
use memmap2::MmapOptions;

use crate::io::csv::options::CsvOptions;

pub async fn in_memory_transpose<P: AsRef<Path> + Send + Sync>(path: P) -> Result<()> {
  let csv_options = CsvOptions::new(&path);
  let sep = csv_options.detect_separator()?;
  let file_stem = csv_options.file_stem()?;
  let mut output_path = PathBuf::from(csv_options.parent_path()?);
  output_path.push(format!("{file_stem}.transpose.csv"));

  let nrows = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(csv_options.rdr_skip_rows()?)
    .byte_headers()?
    .len();

  let file = File::open(path)?;
  let mmap = unsafe { MmapOptions::new().populate().map(&file)? };
  let mut wtr = WriterBuilder::new()
    .delimiter(sep)
    .from_writer(BufWriter::new(File::create(output_path)?));

  let mut record = ByteRecord::with_capacity(1024, nrows);

  for i in 0..nrows {
    record.clear();

    let mut rdr = ReaderBuilder::new().delimiter(sep).from_reader(&mmap[..]);

    for row in rdr.byte_records() {
      let row = row?;
      if i < row.len() {
        record.push_field(&row[i]);
      }
    }
    wtr.write_byte_record(&record)?;
  }

  Ok(wtr.flush()?)
}

pub async fn multipass_transpose<P: AsRef<Path> + Send + Sync>(path: P) -> Result<()> {
  let csv_options = CsvOptions::new(&path);
  let sep = csv_options.detect_separator()?;
  let file_stem = csv_options.file_stem()?;
  let mut output_path = PathBuf::from(csv_options.parent_path()?);
  output_path.push(format!("{file_stem}.transpose.csv"));

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(File::open(&path)?);

  let mut wtr = WriterBuilder::new()
    .delimiter(sep)
    .from_writer(BufWriter::new(File::create(output_path)?));

  let nrows = rdr.byte_headers()?.len();

  let mut record = ByteRecord::with_capacity(1024, nrows);

  for i in 0..nrows {
    record.clear();
    let mut rdr = ReaderBuilder::new()
      .delimiter(sep)
      .from_reader(File::open(&path)?);

    let mut record = ByteRecord::new();
    for row in rdr.byte_records() {
      record.push_field(&row?[i]);
    }
    wtr.write_byte_record(&record)?;
  }

  Ok(wtr.flush()?)
}

#[tauri::command]
pub async fn transpose(path: String, mode: String) -> Result<String, String> {
  let start_time = Instant::now();

  match mode.as_str() {
    "memory" => match in_memory_transpose(path).await {
      Ok(()) => {
        let end_time = Instant::now();
        let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
        Ok(format!("{elapsed_time:.2}"))
      }
      Err(err) => Err(format!("{err}")),
    },
    "multipass" => match multipass_transpose(path).await {
      Ok(()) => {
        let end_time = Instant::now();
        let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
        Ok(format!("{elapsed_time:.2}"))
      }
      Err(err) => Err(format!("{err}")),
    },
    _ => Err(format!("Unknown transpose mode")),
  }
}
