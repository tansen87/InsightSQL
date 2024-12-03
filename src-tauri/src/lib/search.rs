use std::{collections::HashMap, error::Error, fs::File, io::BufReader, time::Instant};

use regex::bytes::RegexBuilder;
use tauri::Emitter;

use crate::detect::detect_separator;

#[derive(Debug)]
enum SearchMode {
  Equal,
  Contains,
  StartsWith,
  Regex,
}

impl From<&str> for SearchMode {
  fn from(mode: &str) -> Self {
    match mode {
      "equal" => SearchMode::Equal,
      "contains" => SearchMode::Contains,
      "startswith" => SearchMode::StartsWith,
      _ => SearchMode::Regex,
    }
  }
}

fn get_header(path: &str) -> Result<Vec<HashMap<String, String>>, Box<dyn Error>> {
  let sep = match detect_separator(path) {
    Some(separator) => separator as u8,
    None => b',',
  };

  let mut rdr = csv::ReaderBuilder::new()
    .delimiter(sep)
    .has_headers(true)
    .from_reader(File::open(path)?);

  let headers = rdr.headers()?;

  let hs: Vec<HashMap<String, String>> = headers
    .iter()
    .map(|header| {
      let mut map = HashMap::new();
      let header_str = header.to_string();
      map.insert("value".to_string(), header_str.clone());
      map.insert("label".to_string(), header_str);
      map
    })
    .collect();

  Ok(hs)
}

async fn generic_search<F>(
  path: String,
  sep: u8,
  select_column: String,
  conditions: Vec<String>,
  output_path: String,
  match_fn: F,
) -> Result<String, Box<dyn Error>>
where
  F: Fn(&str, &[String]) -> bool + Send + Sync,
{
  let mut match_rows: usize = 0;
  let mut rdr = csv::ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(BufReader::new(File::open(path)?));
  let headers = rdr.headers()?.clone();

  let name_idx = match headers.iter().position(|field| field == select_column) {
    Some(idx) => idx,
    None => {
      return Err(
        format!(
          "The column '{}' was not found in the headers.",
          select_column
        )
        .into(),
      );
    }
  };

  let mut wtr = csv::WriterBuilder::new()
    .delimiter(sep)
    .from_path(output_path)?;

  wtr.write_record(&headers)?;

  for result in rdr.records() {
    let record = result?;
    if let Some(value) = record.get(name_idx) {
      if match_fn(value, &conditions) {
        wtr.write_record(&record)?;
        match_rows += 1;
      }
    }
  }

  Ok(match_rows.to_string())
}

async fn equal_search(
  path: String,
  sep: u8,
  select_column: String,
  conditions: Vec<String>,
  output_path: String,
) -> Result<String, Box<dyn Error>> {
  generic_search(
    path,
    sep,
    select_column,
    conditions.clone(),
    output_path,
    |value, conditions| conditions.contains(&value.to_string()),
  )
  .await
}

async fn contains_search(
  path: String,
  sep: u8,
  select_column: String,
  conditions: Vec<String>,
  output_path: String,
) -> Result<String, Box<dyn Error>> {
  generic_search(
    path,
    sep,
    select_column,
    conditions.clone(),
    output_path,
    |value, conditions| {
      conditions
        .iter()
        .any(|cond| value.to_lowercase().contains(&cond.to_lowercase()))
    },
  )
  .await
}

async fn startswith_search(
  path: String,
  sep: u8,
  select_column: String,
  conditions: Vec<String>,
  output_path: String,
) -> Result<String, Box<dyn Error>> {
  generic_search(
    path,
    sep,
    select_column,
    conditions.clone(),
    output_path,
    |value, conditions| conditions.iter().any(|cond| value.starts_with(cond)),
  )
  .await
}

async fn regex_search(
  path: String,
  sep: u8,
  select_column: String,
  regex_char: String,
  output_path: String,
) -> Result<String, Box<dyn Error>> {
  let pattern = RegexBuilder::new(&regex_char).build()?;

  generic_search(
    path,
    sep,
    select_column,
    vec![regex_char],
    output_path,
    move |value, _| pattern.is_match(value.as_bytes()),
  )
  .await
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

async fn perform_search(
  path: String,
  sep: u8,
  select_column: String,
  conditions: String,
  output_path: String,
  mode: SearchMode,
) -> Result<String, Box<dyn Error>> {
  let vec_conditions: Vec<String> = conditions
    .split('|')
    .map(|s| s.trim().to_string())
    .collect();

  match mode {
    SearchMode::Equal => equal_search(path, sep, select_column, vec_conditions, output_path).await,
    SearchMode::Contains => {
      contains_search(path, sep, select_column, vec_conditions, output_path).await
    }
    SearchMode::StartsWith => {
      startswith_search(path, sep, select_column, vec_conditions, output_path).await
    }
    SearchMode::Regex => regex_search(path, sep, select_column, conditions, output_path).await,
  }
}

#[tauri::command]
pub async fn search(
  path: String,
  select_column: String,
  mode: String,
  condition: String,
  output_path: String,
  window: tauri::Window,
) -> Result<String, String> {
  let start_time = Instant::now();

  let sep = match detect_separator(path.as_str()) {
    Some(separator) => separator as u8,
    None => b',',
  };

  let search_mode: SearchMode = mode.as_str().into();

  match perform_search(
    path,
    sep,
    select_column,
    condition,
    output_path,
    search_mode,
  )
  .await
  {
    Ok(result) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      let runtime = format!("{elapsed_time:.2} s");
      window.emit("runtime", runtime).unwrap();
      Ok(result)
    }
    Err(err) => Err(format!("Search failed: {err}")),
  }
}
