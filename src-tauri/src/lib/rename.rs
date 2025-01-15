use std::{fs::File, path::Path, time::Instant};

use anyhow::Result;

use crate::utils::detect_separator;

async fn get_header(file_path: &str) -> Result<Vec<String>> {
  let sep = match detect_separator(file_path, 0) {
    Some(separator) => separator as u8,
    None => b',',
  };

  let mut rdr = csv::ReaderBuilder::new()
    .delimiter(sep)
    .has_headers(true)
    .from_reader(File::open(file_path)?);

  let headers = rdr.headers()?.clone();
  let vec_headers: Vec<String> = headers.iter().map(|h| h.to_string()).collect();

  Ok(vec_headers)
}

async fn rename_headers(file_path: &str, r_header: String) -> Result<()> {
  let sep = match detect_separator(file_path, 0) {
    Some(separator) => separator as u8,
    None => b',',
  };

  let mut rdr = csv::ReaderBuilder::new()
    .delimiter(sep)
    .has_headers(true)
    .from_reader(File::open(file_path)?);

  let mut new_rdr = csv::Reader::from_reader(r_header.as_bytes());

  let new_headers = new_rdr.byte_headers()?;

  let file_name = Path::new(&file_path).file_stem().unwrap().to_str().unwrap();
  let parent_path = Path::new(&file_path)
    .parent()
    .map(|parent| parent.to_string_lossy())
    .unwrap();
  let output_path = format!("{}/{}.rename.csv", parent_path, file_name);

  let mut wtr = csv::WriterBuilder::new()
    .delimiter(sep)
    .from_path(output_path)?;

  wtr.write_record(new_headers)?;

  let mut record = csv::ByteRecord::new();
  while rdr.read_byte_record(&mut record)? {
    wtr.write_record(&record)?;
  }

  wtr.flush()?;

  Ok(())
}

#[tauri::command]
pub async fn get_rename_headers(file_path: String) -> Result<Vec<String>, String> {
  match get_header(file_path.as_str()).await {
    Ok(headers) => Ok(headers),
    Err(err) => Err(format!("get header error: {err}")),
  }
}

#[tauri::command]
pub async fn rename(file_path: String, headers: String) -> Result<String, String> {
  let start_time = Instant::now();

  match rename_headers(file_path.as_str(), headers).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("rename failed: {err}")),
  }
}

/// for integration test
pub async fn public_rename(file_path: &str, r_header: String) -> Result<()> {
  rename_headers(file_path, r_header).await
}
