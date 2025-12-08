use std::{
  fs::File,
  io::{BufReader, BufWriter},
  path::Path,
};

use anyhow::{Result, anyhow};
use csv::{Reader, ReaderBuilder, Writer, WriterBuilder};

use crate::io::csv::{options::CsvOptions, selection::Selection};

#[derive(Debug)]
pub enum SplitMode {
  Nth,
  Max,
}

impl From<&str> for SplitMode {
  fn from(mode: &str) -> Self {
    match mode {
      "split_n" => SplitMode::Nth,
      "split_max" => SplitMode::Max,
      _ => SplitMode::Nth,
    }
  }
}

pub async fn split_n(
  mut rdr: Reader<BufReader<File>>,
  mut wtr: Writer<BufWriter<File>>,
  column: &str,
  n: usize,
  by: &str,
) -> Result<()> {
  let mut headers = rdr.headers()?.clone();

  let sel = Selection::from_headers(rdr.byte_headers()?, &[column][..])?;

  let new_column_name = format!("{}_nth", column);
  headers.push_field(&new_column_name);
  wtr.write_record(&headers)?;

  for result in rdr.records() {
    let record = result?;
    if let Some(value) = record.get(sel.first_indices()?) {
      let split_parts: Vec<&str> = value.split(by).collect();
      let selected_part = if split_parts.len() >= n {
        split_parts[n - 1]
      } else {
        ""
      };

      let mut new_record = record.clone();
      new_record.push_field(selected_part);
      wtr.write_record(&new_record)?;
    }
  }

  Ok(wtr.flush()?)
}

pub async fn split_max(
  mut rdr: Reader<BufReader<File>>,
  mut wtr: Writer<BufWriter<File>>,
  column: &str,
  n: usize,
  by: &str,
) -> Result<()> {
  let mut headers = rdr.headers()?.clone();

  let sel = Selection::from_headers(rdr.byte_headers()?, &[column][..])?;

  let mut first_record = true;
  for result in rdr.records() {
    let record = result?;
    if let Some(value) = record.get(sel.first_indices()?) {
      let split_parts: Vec<&str> = value.split(by).collect();
      if first_record {
        for i in 1..=n {
          headers.push_field(&format!("{}_max{}", column, i));
        }
        wtr.write_record(&headers)?;
        first_record = false;
      }

      let mut new_record = record.clone();
      for i in 0..n {
        if i < split_parts.len() {
          new_record.push_field(split_parts[i]);
        } else {
          new_record.push_field("");
        }
      }

      wtr.write_record(&new_record)?;
    }
  }

  Ok(wtr.flush()?)
}

pub async fn split<P: AsRef<Path> + Send + Sync>(
  path: P,
  column: &str,
  n: i32,
  by: &str,
  mode: SplitMode,
) -> Result<()> {
  let num = n as usize;
  if n < 1 {
    return Err(anyhow!(
      "Number of the split must be greater than or equal 1"
    ));
  }
  if by.chars().count() != 1 {
    return Err(anyhow!("by must be a single character"));
  }

  let opts = CsvOptions::new(&path);
  let sep = opts.detect_separator()?;
  let output_path = opts.output_path(Some("split"), None)?;

  let rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(opts.rdr_skip_rows()?);

  let buf_writer = BufWriter::with_capacity(256_000, File::create(output_path)?);
  let wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_writer);

  match mode {
    SplitMode::Nth => split_n(rdr, wtr, column, num, by).await?,
    SplitMode::Max => split_max(rdr, wtr, column, num, by).await?,
  }

  Ok(())
}
