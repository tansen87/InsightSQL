use std::{fs::File, io::BufWriter, path::Path, time::Instant};

use anyhow::Result;
use csv::{ByteRecord, ReaderBuilder, WriterBuilder};
use memmap2::MmapOptions;

use crate::{io::csv::options::CsvOptions, utils::WTR_BUFFER_SIZE};

pub async fn in_memory_transpose<P: AsRef<Path> + Send + Sync>(
  path: P,
  quoting: bool,
) -> Result<()> {
  let opts = CsvOptions::new(&path);
  let sep = opts.detect_separator()?;
  let output_path = opts.output_path(Some("transpose"), None)?;

  let nrows = ReaderBuilder::new()
    .delimiter(sep)
    .quoting(quoting)
    .from_reader(opts.rdr_skip_rows()?)
    .byte_headers()?
    .len();

  let file = File::open(path)?;
  let mmap = unsafe { MmapOptions::new().populate().map(&file)? };

  let output_file = File::create(output_path)?;
  let buf_wtr = BufWriter::with_capacity(WTR_BUFFER_SIZE, output_file);
  let mut wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_wtr);

  let mut record = ByteRecord::with_capacity(1024, nrows);

  for i in 0..nrows {
    record.clear();

    let mut rdr = ReaderBuilder::new()
      .delimiter(sep)
      .quoting(quoting)
      .from_reader(&mmap[..]);

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

pub async fn multipass_transpose<P: AsRef<Path> + Send + Sync>(path: P, quoting: bool) -> Result<()> {
  let opts = CsvOptions::new(&path);
  let sep = opts.detect_separator()?;
  let output_path = opts.output_path(Some("transpose"), None)?;

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .quoting(quoting)
    .from_reader(File::open(&path)?);

  let output_file = File::create(output_path)?;
  let buf_wtr = BufWriter::with_capacity(WTR_BUFFER_SIZE, output_file);
  let mut wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_wtr);

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
pub async fn transpose(path: String, mode: String, quoting: bool) -> Result<String, String> {
  let start_time = Instant::now();

  match mode.as_str() {
    "memory" => match in_memory_transpose(path, quoting).await {
      Ok(()) => {
        let end_time = Instant::now();
        let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
        Ok(format!("{elapsed_time:.2}"))
      }
      Err(err) => Err(format!("{err}")),
    },
    "multipass" => match multipass_transpose(path, quoting).await {
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
