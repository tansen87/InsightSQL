use std::{fs::File, io::BufWriter, path::Path, time::Instant};

use anyhow::Result;
use csv::{ByteRecord, ReaderBuilder, Writer, WriterBuilder};

use crate::utils::CsvOptions;

fn new_writer(
  headers: &ByteRecord,
  start: i32,
  output_path: String,
  sep: u8,
) -> Result<Writer<BufWriter<File>>> {
  let spath = format!("{output_path}.split_{start}.csv");

  let mut wtr = WriterBuilder::new()
    .delimiter(sep)
    .from_writer(BufWriter::new(File::create(spath)?));
  wtr.write_record(headers)?;

  Ok(wtr)
}

async fn split_csv<P: AsRef<Path>>(path: P, size: u32, skip_rows: String) -> Result<()> {
  let mut csv_options = CsvOptions::new(&path);
  csv_options.set_skip_rows(skip_rows.parse::<usize>()?);
  let sep = match csv_options.detect_separator() {
    Some(separator) => separator as u8,
    None => b',',
  };

  let parent_path = &path
    .as_ref()
    .parent()
    .map(|path| path.to_string_lossy())
    .unwrap();
  let file_name = &path.as_ref().file_stem().unwrap().to_str().unwrap();
  let output_path = format!("{}/{}", parent_path, file_name);

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(csv_options.skip_csv_rows()?);

  let headers = rdr.byte_headers()?.clone();

  let mut wtr = new_writer(&headers, 0, output_path.clone(), sep)?;
  let mut i = 0;
  let mut cnt = 1;
  let mut row = ByteRecord::new();
  while rdr.read_byte_record(&mut row)? {
    if i > 0 && i % size == 0 {
      wtr.flush()?;
      wtr = new_writer(&headers, cnt, output_path.clone(), sep)?;
      cnt += 1;
    }
    wtr.write_byte_record(&row)?;
    i += 1;
  }

  Ok(wtr.flush()?)
}

#[tauri::command]
pub async fn split(path: String, size: u32, skip_rows: String) -> Result<String, String> {
  let start_time = Instant::now();

  match split_csv(path, size, skip_rows).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("split failed: {err}")),
  }
}

/// for integration test
pub async fn public_split(path: String, size: u32, skip_rows: String) -> Result<()> {
  split_csv(path, size, skip_rows).await
}
