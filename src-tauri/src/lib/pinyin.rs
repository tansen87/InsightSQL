use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::time::Instant;

use csv::{ReaderBuilder, WriterBuilder};
use pinyin::ToPinyin;
use tauri::Emitter;

use crate::detect::detect_separator;

fn get_header(file_path: String) -> Result<Vec<HashMap<String, String>>, Box<dyn Error>> {
  let sep = match detect_separator(file_path.as_str()) {
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

  let hs = vec_headers
    .into_iter()
    .enumerate()
    .map(|(_index, value)| {
      let mut map = HashMap::new();
      map.insert("value".to_string(), value.clone());
      map.insert("label".to_string(), value);
      map
    })
    .collect();

  Ok(hs)
}

fn chinese_to_pinyin(
  file_path: String,
  columns: String,
  output_path: String,
) -> Result<(), Box<dyn std::error::Error>> {
  let cols: Vec<&str> = columns.split('|').collect();
  let cols_set: HashSet<&str> = cols.into_iter().collect();

  let sep = match detect_separator(file_path.as_str()) {
    Some(separator) => {
      let separator_u8: u8 = separator as u8;
      separator_u8
    }
    None => b',',
  };

  let mut rdr = ReaderBuilder::new()
    .has_headers(true)
    .delimiter(sep)
    .from_path(Path::new(&file_path))?;
  let mut rdr_header = ReaderBuilder::new()
    .has_headers(true)
    .delimiter(sep)
    .from_path(Path::new(&file_path))?;

  let buf_writer = BufWriter::with_capacity(256_000, File::create(output_path)?);
  let mut wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_writer);

  let headers = rdr_header.headers()?;
  let header_map: Vec<&str> = headers.into_iter().collect();

  wtr.write_record(headers)?;

  for result in rdr.records() {
    let record = result?;

    let mut new_record = Vec::new();

    for (i, field) in record.iter().enumerate() {
      let mut new_field = String::from(field);

      if cols_set.contains(&header_map[i]) {
        new_field = new_field
          .chars()
          .map(|c| {
            c.to_pinyin()
              .map_or_else(|| c.into(), |py| py.plain().to_string().to_uppercase())
          })
          .collect::<String>();
      }

      new_record.push(new_field);
    }

    wtr.write_record(&new_record)?;
  }

  Ok(())
}

#[tauri::command]
pub async fn get_pinyin_headers(
  file_path: String,
  window: tauri::Window,
) -> Vec<HashMap<String, String>> {
  let headers = match (async { get_header(file_path) }).await {
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
pub async fn pinyin(
  file_path: String,
  columns: String,
  output_path: String,
  window: tauri::Window,
) {
  let start_time = Instant::now();

  match (async { chinese_to_pinyin(file_path, columns, output_path) }).await {
    Ok(result) => result,
    Err(err) => {
      eprintln!("pinyin error: {err}");
      window.emit("pinyin_err", &err.to_string()).unwrap();
    }
  }

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  let runtime = format!("{elapsed_time:.2} s");
  window.emit("runtime", runtime).unwrap();
}
