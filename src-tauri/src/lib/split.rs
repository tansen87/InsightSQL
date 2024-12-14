use std::{fs::File, io::BufWriter, path::Path, time::Instant};

use anyhow::Result;

use crate::detect::detect_separator;

fn new_writer(
  headers: &csv::ByteRecord,
  start: i32,
  output_path: String,
  sep: u8,
) -> Result<csv::Writer<BufWriter<File>>> {
  let spath = format!("{output_path}.split_{start}.csv");

  let mut wtr = csv::WriterBuilder::new()
    .delimiter(sep)
    .from_writer(BufWriter::new(File::create(spath)?));
  wtr.write_record(headers)?;

  Ok(wtr)
}

async fn split_csv(file_path: String, size: u32) -> Result<()> {
  let sep = match detect_separator(&file_path.as_str()) {
    Some(separator) => separator as u8,
    None => b',',
  };

  let parent_path = Path::new(&file_path)
    .parent()
    .map(|path| path.to_string_lossy())
    .unwrap();
  let file_name = Path::new(&file_path).file_stem().unwrap().to_str().unwrap();
  let output_path = format!("{}/{}", parent_path, file_name);

  let mut rdr = csv::ReaderBuilder::new()
    .delimiter(sep)
    .has_headers(true)
    .from_reader(File::open(&file_path)?);

  let headers = rdr.byte_headers()?.clone();

  let mut wtr = new_writer(&headers, 0, output_path.clone(), sep)?;
  let mut i = 0;
  let mut cnt = 1;
  let mut row = csv::ByteRecord::new();
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
pub async fn split(file_path: String, size: u32) -> Result<String, String> {
  let start_time = Instant::now();

  match split_csv(file_path, size).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("split failed: {err}")),
  }
}

/// for integration test
pub async fn public_split_csv(file_path: String, size: u32) -> Result<()> {
  split_csv(file_path, size).await
}
