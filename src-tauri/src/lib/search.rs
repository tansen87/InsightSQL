use std::{collections::HashMap, error::Error, fs::File, io::BufReader, time::Instant};

use tauri::Emitter;

use crate::detect::detect_separator;

fn get_header(path: &str) -> Result<Vec<HashMap<String, String>>, Box<dyn Error>> {
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

  let hs = vec_headers
    .into_iter()
    .enumerate()
    .map(|(_index, name)| {
      let mut map = HashMap::new();
      map.insert("value".to_string(), name.clone());
      map.insert("label".to_string(), name);
      map
    })
    .collect();

  Ok(hs)
}

pub fn read_csv(path: String, sep: u8) -> Result<csv::Reader<BufReader<File>>, Box<dyn Error>> {
  let file = File::open(path)?;

  let rdr = csv::ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(BufReader::new(file));

  Ok(rdr)
}

pub fn write_csv(sep: u8, output_path: String) -> Result<csv::Writer<File>, Box<dyn Error>> {
  let wtr = csv::WriterBuilder::new()
    .delimiter(sep)
    .from_path(output_path)?;

  Ok(wtr)
}

fn equal_search(
  path: String,
  sep: u8,
  column: String,
  conditions: Vec<String>,
  output_path: String,
  window: tauri::Window,
) -> Result<(), Box<dyn Error>> {
  let mut count: usize = 0;
  let mut rdr = read_csv(path, sep)?;

  let headers = rdr.headers()?.clone();

  let name_idx = match headers.iter().position(|field| field == column) {
    Some(idx) => idx,
    None => {
      return Err(format!("The column '{}' was not found in the headers.", column).into());
    }
  };

  let mut wtr = write_csv(sep, output_path)?;

  wtr.write_record(&headers)?;

  for result in rdr.records() {
    let record = result?;
    let value = record.get(name_idx).unwrap();
    if conditions.contains(&value.to_string()) {
      wtr.write_record(&record)?;
      count += 1;
    }
  }

  window.emit("equal_count", count)?;

  Ok(())
}

fn contains_search(
  path: String,
  sep: u8,
  column: String,
  conditions: Vec<String>,
  output_path: String,
  window: tauri::Window,
) -> Result<(), Box<dyn Error>> {
  let mut count: usize = 0;
  let mut rdr = read_csv(path, sep)?;

  let headers = rdr.headers()?.clone();

  let name_idx = match headers.iter().position(|field| field == column) {
    Some(idx) => idx,
    None => {
      return Err(format!("The column '{}' was not found in the headers.", column).into());
    }
  };

  let mut wtr = write_csv(sep, output_path)?;

  wtr.write_record(&headers)?;

  for result in rdr.records() {
    let record = result?;
    let value = record.get(name_idx).unwrap().to_string();
    let mut found = false;
    for condition in &conditions {
      if value.to_lowercase().contains(&condition.to_lowercase()) {
        found = true;
        break;
      }
    }

    if found {
      wtr.write_record(&record)?;
      count += 1;
    }
  }

  window.emit("contains_count", count)?;

  Ok(())
}

fn startswith_search(
  path: String,
  sep: u8,
  column: String,
  conditions: Vec<String>,
  output_path: String,
  window: tauri::Window,
) -> Result<(), Box<dyn Error>> {
  let mut count: usize = 0;
  let mut rdr = read_csv(path, sep)?;

  let headers = rdr.headers()?.clone();

  let name_idx = match headers.iter().position(|field| field == column) {
    Some(idx) => idx,
    None => {
      return Err(format!("The column '{}' was not found in the headers.", column).into());
    }
  };

  let mut wtr = write_csv(sep, output_path)?;

  wtr.write_record(&headers)?;

  for result in rdr.records() {
    let record = result?;
    let value = record.get(name_idx).unwrap();
    // Check if any condition matches
    if conditions.iter().any(|cond| value.starts_with(cond)) {
      wtr.write_record(&record)?;
      count += 1;
    }
  }

  window.emit("startswith_count", count)?;

  Ok(())
}

#[tauri::command]
pub async fn get_search_headers(
  path: String,
  window: tauri::Window,
) -> Vec<HashMap<String, String>> {
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
pub async fn search(
  path: String,
  column: String,
  mode: String,
  condition: String,
  output_path: String,
  window: tauri::Window,
) {
  let start_time = Instant::now();
  let equal_window = window.clone();
  let contains_window = window.clone();
  let startswith_window = window.clone();

  let vec_conditions: Vec<String> = condition
    .split('|')
    .map(|s| s.replace("\r", "").replace("\n", ""))
    .collect();
  let vec_strings: Vec<String> = vec_conditions
    .into_iter()
    .map(|condition| condition)
    .collect();
  let sep = match detect_separator(path.as_str()) {
    Some(separator) => {
      let separator_u8: u8 = separator as u8;
      separator_u8
    }
    None => b',',
  };

  if mode == "equal" {
    match (async { equal_search(path, sep, column, vec_strings, output_path, equal_window) }).await
    {
      Ok(result) => result,
      Err(error) => {
        eprintln!("equal_search error: {error}");
        window.emit("equal_err", &error.to_string()).unwrap();
        return ();
      }
    };
  } else if mode == "contains" {
    match (async { contains_search(path, sep, column, vec_strings, output_path, contains_window) })
      .await
    {
      Ok(result) => result,
      Err(error) => {
        eprintln!("contains_search error: {error}");
        window.emit("contains_err", &error.to_string()).unwrap();
        return ();
      }
    };
  } else if mode == "startswith" {
    match (async {
      startswith_search(
        path,
        sep,
        column,
        vec_strings,
        output_path,
        startswith_window,
      )
    })
    .await
    {
      Ok(result) => result,
      Err(error) => {
        eprintln!("startswith_search error: {error}");
        window.emit("startswith_err", &error.to_string()).unwrap();
        return ();
      }
    }
  }

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  let runtime = format!("{elapsed_time:.2} s");
  window.emit("runtime", runtime).unwrap();
}
