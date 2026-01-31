use std::{fs::File, io::BufWriter, path::Path, str::FromStr, time::Instant};

use anyhow::{Result, anyhow};
use csv::{ByteRecord, ReaderBuilder, WriterBuilder};

use crate::{
  index::Indexed,
  io::csv::options::CsvOptions,
  utils::{WTR_BUFFER_SIZE, parse_usize},
};

pub enum SliceMode {
  Index,
  Lines,
}

impl FromStr for SliceMode {
  type Err = &'static str;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_ascii_lowercase().as_ref() {
      "index" => Ok(SliceMode::Index),
      "lines" => Ok(SliceMode::Lines),
      _ => Err("expected 'index' or 'lines'"),
    }
  }
}

fn create_writer<P: AsRef<Path>>(
  output_path: P,
  sep: u8,
  flexible: bool,
) -> Result<csv::Writer<BufWriter<File>>> {
  let buf_wtr = BufWriter::with_capacity(WTR_BUFFER_SIZE, File::create(output_path)?);
  Ok(
    WriterBuilder::new()
      .delimiter(sep)
      .flexible(flexible)
      .from_writer(buf_wtr),
  )
}

pub async fn slice_csv_by_lines<P>(
  path: P,
  quoting: bool,
  flexible: bool,
  start: usize,
  end: usize,
  skiprows: usize,
) -> Result<()>
where
  P: AsRef<Path> + Send + Sync,
{
  if start < 1 {
    return Err(anyhow!("start must be at least 1"));
  }
  if start > end {
    return Err(anyhow!("start must be <= end"));
  }

  let mut opts = CsvOptions::new(&path);
  opts.set_skiprows(skiprows);
  let (sep, reader) = opts.skiprows_and_delimiter()?;
  let output_path = opts.output_path(Some("slice"), None)?;

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .quoting(quoting)
    .flexible(flexible)
    .from_reader(reader);

  let mut wtr = create_writer(output_path, sep, flexible)?;
  wtr.write_byte_record(rdr.byte_headers()?)?;

  let mut n = 0;
  let mut record = ByteRecord::new();
  while rdr.read_byte_record(&mut record)? {
    n += 1;
    if n < start {
      continue;
    }
    if n > end {
      break;
    }
    wtr.write_byte_record(&record)?;
  }

  Ok(wtr.flush()?)
}

pub async fn with_index(
  mut indexed_file: Indexed<File, File>,
  path: &str,
  flexible: bool,
  start: usize,
  end: usize,
) -> Result<()> {
  if start < 1 {
    return Err(anyhow!("start must be at least 1"));
  }
  if start > end {
    return Err(anyhow!("start must be <= end"));
  }

  let opts = CsvOptions::new(path);
  let sep = opts.get_delimiter()?;
  let output_path = opts.output_path(Some("slice"), None)?;

  let mut wtr = create_writer(output_path, sep, flexible)?;
  wtr.write_byte_record(indexed_file.byte_headers()?)?;

  indexed_file.seek((start - 1) as u64)?;

  let num_rows = end - start + 1;
  for r in indexed_file.byte_records().take(num_rows) {
    wtr.write_byte_record(&r?)?;
  }

  Ok(wtr.flush()?)
}

#[tauri::command]
pub async fn slice(
  path: String,
  quoting: bool,
  flexible: bool,
  start: String,
  end: String,
  skiprows: usize,
  mode: String,
) -> Result<String, String> {
  let start_time = Instant::now();

  let mode: SliceMode = mode
    .as_str()
    .parse()
    .map_err(|e| format!("invalid mode '{}': {}", mode, e))?;

  let start = parse_usize(&start, "start")?;
  let end = parse_usize(&end, "end")?;

  let res = match mode {
    SliceMode::Index => {
      let opts = CsvOptions::new(&path);
      let indexed_file = opts
        .indexed()
        .map_err(|e| format!("failed to open indexed file: {}", e))?
        .ok_or_else(|| "no index file found; please create an index first".to_string())?;

      with_index(indexed_file, &path, flexible, start, end).await
    }
    SliceMode::Lines => slice_csv_by_lines(path, quoting, flexible, start, end, skiprows).await,
  };

  res.map_err(|e| format!("slice failed: {e}"))?;

  let elapsed = Instant::now().duration_since(start_time).as_secs_f64();
  Ok(format!("{:.0}", elapsed))
}
