use std::{fs::File, io::BufWriter, path::Path, time::Instant};

use anyhow::{Result, anyhow};
use csv::{ByteRecord, ReaderBuilder, WriterBuilder};

use crate::{io::csv::options::CsvOptions, utils::WTR_BUFFER_SIZE};

pub async fn slice_csv_by_lines<P>(
  path: P,
  quoting: bool,
  flexible: bool,
  start: usize,
  end: usize,
) -> Result<()>
where
  P: AsRef<Path> + Send + Sync,
{
  if start > end {
    return Err(anyhow!("start must less than end"));
  }

  let opts = CsvOptions::new(path);
  let sep = opts.detect_separator()?;
  let output_path = opts.output_path(Some("slice"), None)?;

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .quoting(quoting)
    .flexible(flexible)
    .from_reader(opts.rdr_skip_rows()?);

  let buf_wtr = BufWriter::with_capacity(WTR_BUFFER_SIZE, File::create(output_path)?);
  let mut wtr = WriterBuilder::new()
    .delimiter(sep)
    .flexible(flexible)
    .from_writer(buf_wtr);

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

  Ok(())
}

#[tauri::command]
pub async fn slice(
  path: String,
  quoting: bool,
  flexible: bool,
  start: String,
  end: String,
) -> Result<String, String> {
  let start_time = Instant::now();

  match slice_csv_by_lines(
    path,
    quoting,
    flexible,
    start
      .parse::<usize>()
      .map_err(|e| format!("parse start error: {e}"))?,
    end
      .parse::<usize>()
      .map_err(|e| format!("parse end error: {e}"))?,
  )
  .await
  {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("{err}")),
  }
}
