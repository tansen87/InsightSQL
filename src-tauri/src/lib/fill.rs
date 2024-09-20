use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::time::Instant;

fn get_header(path: String, sep: String) -> Result<Vec<HashMap<String, String>>, Box<dyn Error>> {
  let mut separator = Vec::new();
  let sep = if sep == "\\t" {
    b'\t'
  } else {
    sep.into_bytes()[0]
  };
  separator.push(sep);

  let mut rdr = csv::ReaderBuilder::new()
    .delimiter(separator[0])
    .has_headers(true)
    .from_reader(File::open(path)?);

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

fn fill_values(
  input_file: String,
  sep: String,
  fill_column: String,
  fill_value: String,
  window: tauri::Window
) -> Result<(), Box<dyn Error>> {
  let mut separator = Vec::new();
  let sep = if sep == "\\t" {
    b'\t'
  } else {
    sep.into_bytes()[0]
  };
  separator.push(sep);

  let fill_columns: Vec<&str> = fill_column.split('|').collect();

  let mut rdr = csv::ReaderBuilder::new()
    .delimiter(separator[0])
    .has_headers(true)
    .from_reader(File::open(&input_file)?);

  let headers = rdr.headers()?;

  let mut header_indices = Vec::new();
  for column in &fill_columns {
    let index = match headers.iter().position(|field| &field == column) {
      Some(idx) => idx,
      None => {
        return Err(format!("The column '{}' was not found in the headers.", column).into());
      }
    };
    header_indices.push(index);
  }

  let current_time = chrono::Local::now().format("%Y-%m-%d-%H%M%S");
  let parent_path = Path::new(&input_file)
    .parent()
    .map(|parent| parent.to_string_lossy())
    .unwrap_or_else(|| "Default Path".to_string().into());
  let output_file = format!("{}/fill{}.csv", parent_path, current_time);

  let mut wtr = csv::WriterBuilder::new()
    .delimiter(separator[0])
    .from_writer(BufWriter::new(File::create(output_file)?));

  wtr.write_record(headers)?;

  let mut count_rows: u64 = 0;

  for result in rdr.deserialize() {
    let mut record: Vec<String> = result?;
    for &index in &header_indices {
      if record.get(index).map_or(true, |s| s.is_empty()) {
        record[index] = fill_value.clone();
        count_rows += 1;
      }
    }
    wtr.write_record(&record)?;
  }
  wtr.flush()?;

  window.emit("fill_rows", count_rows)?;

  Ok(())
}

#[tauri::command]
pub async fn get_fill_headers(path: String, sep: String, window: tauri::Window) -> Vec<HashMap<String, String>> {
  let headers = match (async { get_header(path, sep) }).await {
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
pub async fn fill(
  path: String,
  sep: String,
  columns: String,
  values: String,
  window: tauri::Window,
) {
  let start_time = Instant::now();
  let cnt_window = window.clone();

  match (async { fill_values(path, sep, columns, values, cnt_window) }).await {
    Ok(result) => result,
    Err(err) => {
      eprintln!("fill value error: {err}");
      window.emit("fill_err", &err.to_string()).unwrap();
    }
  }

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  let runtime = format!("{elapsed_time:.2} s");
  window.emit("runtime", runtime).unwrap();
}
