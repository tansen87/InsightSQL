use std::{
  fs::File,
  io::{BufReader, BufWriter},
  path::Path,
};

use anyhow::{Result, anyhow};
use csv::{Reader, ReaderBuilder, Writer, WriterBuilder};

use crate::io::csv::{options::CsvOptions, selection::Selection};

#[derive(Debug)]
pub enum SliceMode {
  Left,
  Right,
  Slice,
}

impl From<&str> for SliceMode {
  fn from(mode: &str) -> Self {
    match mode {
      "left" => SliceMode::Left,
      "right" => SliceMode::Right,
      "slice" => SliceMode::Slice,
      _ => SliceMode::Left,
    }
  }
}

impl SliceMode {
  fn to_str(&self) -> &'static str {
    match self {
      SliceMode::Left => "left",
      SliceMode::Right => "right",
      SliceMode::Slice => "slice",
    }
  }
}

pub async fn slice_nchar(
  mut rdr: Reader<BufReader<File>>,
  mut wtr: Writer<BufWriter<File>>,
  column: &str,
  n: usize,
  reverse: bool,
  mode: &str,
) -> Result<()> {
  let headers = rdr.headers()?.clone();

  let sel = Selection::from_headers(rdr.byte_headers()?, &[column][..])?;

  let mut new_headers = headers.clone();
  let new_column_name = format!("{}_nchar", column);
  new_headers.push_field(&new_column_name);

  wtr.write_record(&new_headers)?;

  for result in rdr.records() {
    let record = result?;

    if let Some(value) = record.get(sel.first_indices()?) {
      let slice_n = {
        let chars: Vec<char> = value.chars().collect();

        let slice = if mode == "left" {
          &chars[..n.min(chars.len())]
        } else {
          // mode == "right"
          let len = chars.len();
          &chars[len.saturating_sub(n)..]
        };

        let mut result: String = slice.iter().collect();

        if reverse {
          result = result.chars().rev().collect();
        }

        result
      };

      let mut new_record = record.clone();
      new_record.push_field(&slice_n);

      wtr.write_record(&new_record)?;
    }
  }

  Ok(wtr.flush()?)
}

pub async fn slice(
  mut rdr: Reader<BufReader<File>>,
  mut wtr: Writer<BufWriter<File>>,
  column: &str,
  start_idx: i32,
  length: usize,
  reverse: bool,
) -> Result<()> {
  let headers = rdr.headers()?.clone();
  let sel = Selection::from_headers(rdr.byte_headers()?, &[column][..])?;

  let mut new_headers = headers.clone();
  let new_column_name = format!("{}_slice", column);
  new_headers.push_field(&new_column_name);

  wtr.write_record(&new_headers)?;

  for result in rdr.records() {
    let record = result?;

    if let Some(value) = record.get(sel.first_indices()?) {
      let slice_sl = {
        let chars: Vec<char> = value.chars().collect();

        let (start, is_reversed) = if start_idx > 0 {
          ((start_idx - 1).try_into()?, false)
        } else if start_idx < 0 {
          let start = chars.len().saturating_sub((-start_idx - 1).try_into()?);
          (start, true)
        } else {
          return Err(anyhow!("Number of the slice cannot be equal to 0"));
        };

        // determine the indices of the slice
        let end = start + length;
        let slice = if is_reversed {
          chars
            .iter()
            .rev()
            .skip(chars.len().saturating_sub(end))
            .take(length)
            .cloned()
            .collect::<Vec<char>>()
        } else {
          chars
            .get(start..end)
            .map(|r| r.to_vec())
            .unwrap_or_default()
        };

        let mut result: String = slice.into_iter().collect();

        // warning: 不需要将以下两个if合并到一起
        if start_idx < 0 {
          result = result.chars().rev().collect();
        }
        if reverse {
          result = result.chars().rev().collect();
        }

        result
      };

      let mut new_record = record.clone();
      new_record.push_field(&slice_sl);

      wtr.write_record(&new_record)?;
    }
  }

  Ok(wtr.flush()?)
}

pub async fn perform_slice<P: AsRef<Path> + Send + Sync>(
  path: P,
  column: &str,
  n: i32,
  length: usize,
  reverse: bool,
  mode: SliceMode,
) -> Result<()> {
  let num = n as usize;
  if n < 1 && mode.to_str() != "slice" {
    return Err(anyhow!(
      "Number of the slice must be greater than or equal 1"
    ));
  }
  if n == 0 {
    return Err(anyhow!("Number of the slice cannot be equal to 0"));
  }

  let opts = CsvOptions::new(&path);
  let sep = opts.detect_separator()?;
  let output_path = opts.output_path(Some("slice"), None)?;

  let rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(opts.rdr_skip_rows()?);

  let buf_writer = BufWriter::with_capacity(256_000, File::create(output_path)?);
  let wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_writer);

  match mode {
    SliceMode::Left => {
      slice_nchar(rdr, wtr, column, num, reverse, SliceMode::Left.to_str()).await?
    }
    SliceMode::Right => {
      slice_nchar(rdr, wtr, column, num, reverse, SliceMode::Right.to_str()).await?
    }
    SliceMode::Slice => slice(rdr, wtr, column, n, length, reverse).await?,
  }

  Ok(())
}
