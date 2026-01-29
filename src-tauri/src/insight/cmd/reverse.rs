use std::{path::Path, time::Instant};

use anyhow::Result;
use csv::{ByteRecord, ReaderBuilder, WriterBuilder};

use crate::io::csv::options::CsvOptions;

pub async fn reverse_csv<P: AsRef<Path> + Send + Sync>(
  path: P,
  quoting: bool,
  skiprows: usize,
) -> Result<()> {
  let mut opts = CsvOptions::new(path);
  opts.set_skiprows(skiprows);
  let (sep, reader) = opts.skiprows_and_delimiter()?;
  let output_path = opts.output_path(Some("reverse"), None)?;

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .quoting(quoting)
    .from_reader(reader);

  let mut wtr = WriterBuilder::new().delimiter(sep).from_path(output_path)?;

  if let Some(mut idx_file) = opts.indexed()? {
    // we have an index, no need to check avail mem,
    // we're reading the file in reverse streaming
    wtr.write_record(rdr.byte_headers()?)?;
    let mut record = ByteRecord::new();
    let mut pos = idx_file.count().saturating_sub(1);

    while idx_file.seek(pos).is_ok() {
      idx_file.read_byte_record(&mut record)?;
      wtr.write_byte_record(&record)?;
      if pos == 0 {
        break;
      }
      pos -= 1;
    }
  } else {
    // we don't have an index, we need to read the entire file into memory
    // we're loading the entire file into memory, we need to check avail mem
    let mut all = rdr.byte_records().collect::<Result<Vec<_>, _>>()?;
    all.reverse();

    wtr.write_record(rdr.byte_headers()?)?;
    for r in all {
      wtr.write_byte_record(&r)?;
    }
  }

  Ok(wtr.flush()?)
}

#[tauri::command]
pub async fn reverse(path: String, quoting: bool, skiprows: usize) -> Result<String, String> {
  let start_time = Instant::now();

  match reverse_csv(path, quoting, skiprows).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      let rtime = format!("{elapsed_time:.2}");
      Ok(rtime)
    }
    Err(err) => Err(format!("{err}")),
  }
}
