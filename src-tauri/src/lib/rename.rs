use std::{error::Error, fs::File, path::Path};

fn get_header(path: &str, sep: String) -> Result<Vec<String>, Box<dyn Error>> {
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

  Ok(vec_headers)
}

fn rename_headers(
  path: &str,
  sep: String,
  r_header: String,
  window: tauri::Window,
) -> Result<(), Box<dyn Error>> {
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

  let mut new_rdr = csv::Reader::from_reader(r_header.as_bytes());

  let new_headers = new_rdr.byte_headers()?;

  let file_path = Path::new(&path);
  let file_name = match file_path.file_name() {
    Some(name) => match name.to_str() {
      Some(name_str) => name_str.split('.').collect::<Vec<&str>>(),
      None => vec![],
    },
    None => vec![],
  };
  let current_time = chrono::Local::now();
  let file_path_copy = file_path
    .parent()
    .map(|parent| parent.to_string_lossy())
    .unwrap_or_else(|| "Default Path".to_string().into());
  let output_path = format!(
    "{}/{}_rename {}.csv",
    file_path_copy,
    file_name[0],
    current_time.format("%Y-%m-%d-%H%M%S")
  );

  let mut wtr = csv::WriterBuilder::new()
    .delimiter(separator[0])
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
pub async fn get_headers(path: String, sep: String, window: tauri::Window) -> Vec<String> {
  let headers = match (async { get_header(path.as_str(), sep) }).await {
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
pub async fn rename(path: String, sep: String, headers: String, window: tauri::Window) {
  let rname_window = window.clone();
  match (async { rename_headers(path.as_str(), sep, headers, rname_window) }).await {
    Ok(result) => result,
    Err(err) => {
      eprintln!("rename headers error: {err}");
      window.emit("rename_err", &err.to_string()).unwrap();
    }
  }
}
