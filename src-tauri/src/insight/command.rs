use std::{
  collections::{HashMap, HashSet},
  path::Path,
};

use anyhow::Result;
use tauri::{Emitter, Window};

use crate::io::csv::options::CsvOptions;
use crate::tojson;

#[tauri::command]
pub async fn from_headers(path: String, skiprows: usize) -> Result<Vec<String>, String> {
  let mut opts = CsvOptions::new(path);
  opts.set_skiprows(skiprows);
  
  async { opts.from_headers().map_err(|e| e.to_string()) }.await
}

#[tauri::command]
pub async fn map_headers(
  path: String,
  skiprows: usize,
) -> Result<Vec<HashMap<String, String>>, String> {
  let mut opts = CsvOptions::new(path);
  opts.set_skiprows(skiprows);

  async { opts.map_headers().map_err(|e| e.to_string()) }.await
}

#[tauri::command]
pub async fn inter_headers(path: String, skiprows: usize) -> Result<HashSet<String>, String> {
  let mut opts = CsvOptions::new(path);
  opts.set_skiprows(skiprows);

  match async { opts.inter_headers() }.await {
    Ok(result) => Ok(result),
    Err(err) => Err(format!("{err}")),
  }
}

#[tauri::command]
pub async fn dupli_headers(
  path: String,
  skiprows: usize,
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

    let mut opts = CsvOptions::new(p);
    opts.set_skiprows(skiprows);

    match opts.dupli_headers() {
      Ok((duplicate_headers, unique_headers)) => {
        window
          .emit(
            "dupler-msg",
            format!("{filename}|{:?}|{:?}", &unique_headers, &duplicate_headers),
          )
          .map_err(|e| e.to_string())?;

        all_unique_headers.extend(unique_headers);
        all_duplicate_headers.extend(duplicate_headers);
      }
      Err(err) => {
        window
          .emit("dupler-err", format!("{filename}|{err}"))
          .map_err(|e| e.to_string())?;
        continue;
      }
    }
  }

  Ok((all_unique_headers, all_duplicate_headers))
}

#[tauri::command]
pub async fn to_json(path: String, skiprows: usize) -> Result<String, String> {
  match async { tojson::csv_to_json(path, skiprows) }.await {
    Ok(result) => Ok(result),
    Err(err) => Err(format!("{err}")),
  }
}
