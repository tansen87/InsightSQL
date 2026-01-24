use std::{fs::File, io::BufWriter, path::Path};

use anyhow::{Result, anyhow};
use csv::{ReaderBuilder, WriterBuilder};

use crate::io::csv::{options::CsvOptions, selection::Selection};

#[derive(Debug)]
pub enum PadMode {
  Left,
  Right,
  Both,
}

impl From<&str> for PadMode {
  fn from(mode: &str) -> Self {
    match mode {
      "pad_left" => PadMode::Left,
      "pad_right" => PadMode::Right,
      "pad_both" => PadMode::Both,
      _ => PadMode::Left,
    }
  }
}

fn pad_string(cell: &str, length: usize, fill_char: &str, mode: PadMode) -> Result<String> {
  if fill_char.chars().count() != 1 {
    return Err(anyhow!("fill char must be a single character"));
  }

  let cell_len = cell.chars().count();
  if cell_len >= length {
    return Ok(cell.to_string());
  }

  let total_pad = length - cell_len;
  match mode {
    PadMode::Left => {
      let pad = fill_char.repeat(total_pad);
      Ok(format!("{pad}{cell}"))
    }
    PadMode::Right => {
      let pad = fill_char.repeat(total_pad);
      Ok(format!("{cell}{pad}"))
    }
    PadMode::Both => {
      let left = total_pad / 2;
      let right = total_pad - left;
      let left_pad = fill_char.repeat(left);
      let right_pad = fill_char.repeat(right);
      Ok(format!("{left_pad}{cell}{right_pad}"))
    }
  }
}

pub async fn pad<P: AsRef<Path> + Send + Sync>(
  path: P,
  column: &str,
  length: String,
  fill_char: &str,
  mode: &str,
  quoting: bool,
) -> Result<()> {
  let opts = CsvOptions::new(&path);
  let sep = opts.detect_separator()?;
  let output_path = opts.output_path(Some("pad"), None)?;
  let length = length.parse::<usize>()?;

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .quoting(quoting)
    .from_reader(opts.rdr_skip_rows()?);
  let sel = Selection::from_headers(rdr.byte_headers()?, &[column][..])?;

  let buf_writer = BufWriter::with_capacity(256_000, File::create(output_path)?);
  let mut wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_writer);
  wtr.write_record(rdr.headers()?)?;

  for result in rdr.records() {
    let record = result?;
    let mut row_fields: Vec<String> = record.iter().map(|s| s.to_string()).collect();
    let idx = sel.first_indices()?;
    let cell = &row_fields[idx];
    let pad_cell = pad_string(cell, length, fill_char, mode.into())?;
    row_fields[idx] = pad_cell;
    wtr.write_record(&row_fields)?;
  }

  Ok(wtr.flush()?)
}
