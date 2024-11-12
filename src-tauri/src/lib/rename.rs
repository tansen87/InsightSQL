use std::{error::Error, fs::File, path::Path, time::Instant};

use tauri::Emitter;

use crate::detect::detect_separator;

async fn get_header(file_path: &str) -> Result<Vec<String>, Box<dyn Error>> {
  let sep = match detect_separator(file_path) {
    Some(separator) => {
      let separator_u8: u8 = separator as u8;
      separator_u8
    }
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

async fn rename_headers(file_path: &str, r_header: String) -> Result<u64, Box<dyn Error>> {
  let sep = match detect_separator(file_path) {
    Some(separator) => {
      let separator_u8: u8 = separator as u8;
      separator_u8
    }
    None => b',',
  };

  let mut rdr = csv::ReaderBuilder::new()
    .delimiter(sep)
    .has_headers(true)
    .from_reader(File::open(file_path)?);

  let mut new_rdr = csv::Reader::from_reader(r_header.as_bytes());

  let new_headers = new_rdr.byte_headers()?;

  let file_path = Path::new(&file_path);
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

  Ok(count)
}

#[tauri::command]
pub async fn get_rename_headers(file_path: String) -> Result<Vec<String>, String> {
  match get_header(file_path.as_str()).await {
    Ok(headers) => Ok(headers),
    Err(e) => {
      eprintln!("Error occurred: {}", e); // 打印错误信息到控制台
      Err(e.to_string()) // 将错误转换为字符串并返回
    },
  }
}

#[tauri::command]
pub async fn rename(
  file_path: String,
  headers: String,
  window: tauri::Window,
) -> Result<u64, String> {
  let start_time = Instant::now();

  match rename_headers(file_path.as_str(), headers).await {
    Ok(result) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      let runtime = format!("{elapsed_time:.2} s");
      window.emit("runtime", runtime).unwrap();
      Ok(result)
    }
    Err(e) => Err(format!("{e}")),
  }
}
