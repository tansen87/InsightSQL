use std::{error::Error, fs::File, path::Path, time::Instant};

use tauri::Emitter;

use crate::detect::detect_separator;

fn get_header(path: &str) -> Result<Vec<String>, Box<dyn Error>> {
  let sep = match detect_separator(path) {
    Some(separator) => {
      let separator_u8: u8 = separator as u8;
      separator_u8
    }
    None => b',',
  };

  let mut rdr = csv::ReaderBuilder::new()
    .delimiter(sep)
    .has_headers(true)
    .from_reader(File::open(path)?);

  let headers = rdr.headers()?.clone();
  let vec_headers: Vec<String> = headers.iter().map(|h| h.to_string()).collect();

  Ok(vec_headers)
}

fn rename_headers(
  path: &str,
  r_header: String,
  window: tauri::Window,
) -> Result<(), Box<dyn Error>> {
  let sep = match detect_separator(path) {
    Some(separator) => {
      let separator_u8: u8 = separator as u8;
      separator_u8
    }
    None => b',',
  };

  let mut rdr = csv::ReaderBuilder::new()
    .delimiter(sep)
    .has_headers(true)
    .from_reader(File::open(path)?);

  let mut new_rdr = csv::Reader::from_reader(r_header.as_bytes());

  let new_headers = new_rdr.byte_headers()?;

  let file_path = Path::new(&path);
  let file_name = file_path.file_name().unwrap().to_str().unwrap();
  let current_time = chrono::Local::now();
  let parent_path = file_path
    .parent()
    .map(|parent| parent.to_string_lossy())
    .unwrap();
  let output_path = format!(
    "{}/{}_rename_{}.csv",
    parent_path,
    file_name,
    current_time.format("%Y-%m-%d-%H%M%S")
  );

  let mut wtr = csv::WriterBuilder::new()
    .delimiter(sep)
    .from_path(output_path)?;

  wtr.write_record(new_headers)?;

  let mut count: u64 = 0;

  let mut record = csv::ByteRecord::new();
  while rdr.read_byte_record(&mut record)? {
    wtr.write_record(&record)?;

    count += 1;
  }

  wtr.flush()?;
  window.emit("count_rows", count)?;

  Ok(())
}

#[tauri::command]
pub async fn get_rename_headers(path: String, window: tauri::Window) -> Vec<String> {
  let headers = match (async { get_header(path.as_str()) }).await {
    Ok(result) => result,
    Err(err) => {
      eprintln!("get headers error: {err}");
      window.emit("get_err", &err.to_string()).unwrap();
      return Vec::new();
    }
  };

  headers
}

#[tauri::command]
pub async fn rename(path: String, headers: String, window: tauri::Window) {
  let start_time = Instant::now();
  let rename_window = window.clone();

  match (async { rename_headers(path.as_str(), headers, rename_window) }).await {
    Ok(result) => result,
    Err(err) => {
      eprintln!("rename headers error: {err}");
      window.emit("rename_err", &err.to_string()).unwrap();
    }
  }

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  let runtime = format!("{elapsed_time:.2} s");
  window.emit("runtime", runtime).unwrap();
}
