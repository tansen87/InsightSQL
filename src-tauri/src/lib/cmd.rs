use std::{
  collections::{HashMap, HashSet},
  path::Path,
};

use anyhow::Result;
use tauri::{Emitter, Window};

use crate::{tojson, utils::CsvOptions};

#[tauri::command]
pub async fn from_headers(path: String) -> Result<Vec<String>, String> {
  let csv_options = CsvOptions::new(path);

  async { csv_options.from_headers().map_err(|e| e.to_string()) }.await
}

#[tauri::command]
pub async fn map_headers(
  path: String,
  skip_rows: String,
) -> Result<Vec<HashMap<String, String>>, String> {
  let mut csv_options = CsvOptions::new(path);
  csv_options.set_skip_rows(skip_rows.parse::<usize>().map_err(|e| e.to_string())?);

  async { csv_options.map_headers().map_err(|e| e.to_string()) }.await
}

#[tauri::command]
pub async fn inter_headers(path: String, skip_rows: String) -> Result<HashSet<String>, String> {
  let mut csv_options = CsvOptions::new(path);
  csv_options.set_skip_rows(skip_rows.parse::<usize>().map_err(|e| e.to_string())?);

  match async { csv_options.inter_headers() }.await {
    Ok(result) => Ok(result),
    Err(err) => Err(format!("{err}")),
  }
}

#[tauri::command]
pub async fn dupli_headers(
  path: String,
  skip_rows: String,
  window: Window,
) -> Result<(HashSet<String>, HashSet<String>), String> {
  let paths: Vec<&str> = path.split('|').collect();
  let mut all_unique_headers: HashSet<String> = HashSet::new();
  let mut all_duplicate_headers: HashSet<String> = HashSet::new();

  for p in paths.iter() {
    let filename = Path::new(p)
      .file_name()
      .and_then(|f| f.to_str())
      .unwrap_or("None");

    window.emit("dupler", filename).map_err(|e| e.to_string())?;

    let mut csv_options = CsvOptions::new(p);
    csv_options.set_skip_rows(skip_rows.parse::<usize>().map_err(|e| e.to_string())?);

    match csv_options.dupli_headers() {
      Ok((duplicate_headers, unique_headers)) => {
        window
          .emit(
            "dupler_msg",
            format!("{filename}|{:?}|{:?}", &unique_headers, &duplicate_headers),
          )
          .map_err(|e| e.to_string())?;

        all_unique_headers.extend(unique_headers);
        all_duplicate_headers.extend(duplicate_headers);
      }
      Err(err) => {
        window
          .emit("dupler_err", format!("{filename}|{err}"))
          .map_err(|e| e.to_string())?;
        continue;
      }
    }
  }

  Ok((all_unique_headers, all_duplicate_headers))
}

#[tauri::command]
pub async fn to_json(path: String) -> Result<String, String> {
  match async { tojson::csv_to_json(path) }.await {
    Ok(result) => Ok(result),
    Err(err) => Err(format!("{err}")),
  }
}
