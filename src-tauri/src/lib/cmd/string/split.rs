use std::{
  fs::File,
  io::{BufReader, BufWriter},
  path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};
use csv::{Reader, ReaderBuilder, Writer, WriterBuilder};

use crate::io::csv::{options::CsvOptions, selection::Selection};

#[derive(Debug)]
pub enum SplitMode {
  Nth,
  Nmax,
}

impl From<&str> for SplitMode {
  fn from(mode: &str) -> Self {
    match mode {
      "nth" => SplitMode::Nth,
      "nmax" => SplitMode::Nmax,
      _ => SplitMode::Nth,
    }
  }
}

pub async fn split_nth(
  mut rdr: Reader<BufReader<File>>,
  mut wtr: Writer<BufWriter<File>>,
  select_column: &str,
  n: usize,
  str_sep: &str,
) -> Result<()> {
  let mut headers = rdr.headers()?.clone();

  let sel = Selection::from_headers(rdr.byte_headers()?, &[select_column][..])?;

  let new_column_name = format!("{}_nth", select_column);
  headers.push_field(&new_column_name);
  wtr.write_record(&headers)?;

  for result in rdr.records() {
    let record = result?;
    if let Some(value) = record.get(sel.first_indices()?) {
      let split_parts: Vec<&str> = value.split(str_sep).collect();
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

pub async fn split_nmax(
  mut rdr: Reader<BufReader<File>>,
  mut wtr: Writer<BufWriter<File>>,
  select_column: &str,
  n: usize,
  str_sep: &str,
) -> Result<()> {
  let mut headers = rdr.headers()?.clone();

  let sel = Selection::from_headers(rdr.byte_headers()?, &[select_column][..])?;

  let mut first_record = true;
  for result in rdr.records() {
    let record = result?;
    if let Some(value) = record.get(sel.first_indices()?) {
      let split_parts: Vec<&str> = value.split(str_sep).collect();
      if first_record {
        for i in 1..=n {
          headers.push_field(&format!("{}_nmax{}", select_column, i));
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

pub async fn perform_split<P: AsRef<Path> + Send + Sync>(
  path: P,
  select_column: &str,
  n: i32,
  str_sep: &str,
  mode: SplitMode,
) -> Result<()> {
  let num = n as usize;
  if n < 1 {
    return Err(anyhow!(
      "Number of the split must be greater than or equal 1"
    ));
  }

  let csv_options = CsvOptions::new(&path);
  let sep = csv_options.detect_separator()?;
  let file_stem = csv_options.file_stem()?;
  let mut output_path = PathBuf::from(csv_options.parent_path()?);
  output_path.push(format!("{file_stem}.slice.csv"));

  let rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(csv_options.rdr_skip_rows()?);

  let buf_writer = BufWriter::with_capacity(256_000, File::create(output_path)?);
  let wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_writer);

  match mode {
    SplitMode::Nth => split_nth(rdr, wtr, select_column, num, str_sep).await?,
    SplitMode::Nmax => split_nmax(rdr, wtr, select_column, num, str_sep).await?,
  }

  Ok(())
}
