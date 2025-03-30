use std::{collections::HashMap, fs::File, path::Path, time::Instant};

use anyhow::{Result, anyhow};
use csv::{ReaderBuilder, Writer, WriterBuilder};
use regex::bytes::RegexBuilder;

use crate::utils::{CsvOptions, Selection};

#[derive(Debug)]
enum SearchMode {
  Equal,
  EqualMulti(Vec<String>),
  Contains,
  ContainsMulti(Vec<String>),
  StartsWith,
  StartsWithMulti(Vec<String>),
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

fn sanitize_condition(condition: &str) -> String {
  condition
    .chars()
    .map(|c| match c {
      '/' | '\\' | '|' | ',' | '.' | '"' | ':' => '-',
      _ => c,
    })
    .collect()
}

async fn generic_search<F, P>(
  path: P,
  sep: u8,
  select_column: String,
  conditions: Vec<String>,
  output_path: String,
  match_fn: F,
) -> Result<String>
where
  F: Fn(&str, &[String]) -> bool + Send + Sync,
  P: AsRef<Path> + Send + Sync,
{
  let mut match_rows: usize = 0;

  let csv_options = CsvOptions::new(&path);

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(csv_options.skip_csv_rows()?);

  let sel = Selection::from_headers(rdr.byte_headers()?, &[select_column.as_str()][..])?;

  let mut wtr = WriterBuilder::new().delimiter(sep).from_path(output_path)?;

  wtr.write_record(rdr.headers()?)?;

  for result in rdr.records() {
    let record = result?;
    if let Some(value) = record.get(sel.first_indices()?) {
      if match_fn(value, &conditions) {
        wtr.write_record(&record)?;
        match_rows += 1;
      }
    }
  }

  Ok(match_rows.to_string())
}

async fn generic_multi_search<F, P>(
  path: P,
  sep: u8,
  select_column: String,
  conditions: Vec<String>,
  match_fn: F,
) -> Result<String>
where
  F: Fn(&str, &String) -> bool + Send + Sync,
  P: AsRef<Path> + Send + Sync,
{
  let mut match_rows: usize = 0;
  let mut writers: HashMap<&str, Writer<File>> = HashMap::new();

  // prepare writers for each condition with sanitized output paths
  for condition in &conditions {
    let sanitized_condition = sanitize_condition(condition);
    let output_path = format!(
      "{}/{}_{}.csv",
      path.as_ref().parent().unwrap().to_str().unwrap(),
      path.as_ref().file_stem().unwrap().to_str().unwrap(),
      sanitized_condition
    );
    let file = File::create(&output_path)?;
    writers.insert(
      condition,
      WriterBuilder::new().delimiter(sep).from_writer(file),
    );
  }

  let csv_options = CsvOptions::new(&path);

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(csv_options.skip_csv_rows()?);

  let headers = rdr.headers()?.clone();
  let sel = Selection::from_headers(rdr.byte_headers()?, &[select_column.as_str()][..])?;

  // write headers to all output files
  for wtr in writers.values_mut() {
    wtr.write_record(&headers)?;
  }

  for result in rdr.records() {
    let record = result?;
    if let Some(value) = record.get(sel.first_indices()?) {
      for condition in &conditions {
        if match_fn(value, condition) {
          if let Some(wtr) = writers.get_mut(condition.as_str()) {
            wtr.write_record(&record)?;
            match_rows += 1;
          }
        }
      }
    }
  }

  // flush all writers
  for wtr in writers.values_mut() {
    wtr.flush()?;
  }

  Ok(match_rows.to_string())
}

pub async fn equal_search<P: AsRef<Path> + Send + Sync>(
  path: P,
  sep: u8,
  select_column: String,
  conditions: Vec<String>,
  output_path: String,
) -> Result<String> {
  generic_search(
    path,
    sep,
    select_column,
    conditions,
    output_path,
    |value, conditions| conditions.contains(&value.to_string()),
  )
  .await
}

pub async fn equal_multi_search<P: AsRef<Path> + Send + Sync>(
  path: P,
  sep: u8,
  select_column: String,
  conditions: Vec<String>,
) -> Result<String> {
  generic_multi_search(path, sep, select_column, conditions, |value, condition| {
    value == condition
  })
  .await
}

pub async fn contains_search<P: AsRef<Path> + Send + Sync>(
  path: P,
  sep: u8,
  select_column: String,
  conditions: Vec<String>,
  output_path: String,
) -> Result<String> {
  generic_search(
    path,
    sep,
    select_column,
    conditions,
    output_path,
    |value, conditions| conditions.iter().any(|cond| value.contains(cond)),
  )
  .await
}

pub async fn contains_multi_search<P: AsRef<Path> + Send + Sync>(
  path: P,
  sep: u8,
  select_column: String,
  conditions: Vec<String>,
) -> Result<String> {
  generic_multi_search(path, sep, select_column, conditions, |value, condition| {
    value.contains(condition)
  })
  .await
}

pub async fn startswith_search<P: AsRef<Path> + Send + Sync>(
  path: P,
  sep: u8,
  select_column: String,
  conditions: Vec<String>,
  output_path: String,
) -> Result<String> {
  generic_search(
    path,
    sep,
    select_column,
    conditions,
    output_path,
    |value, conditions| conditions.iter().any(|cond| value.starts_with(cond)),
  )
  .await
}

pub async fn startswith_multi_search<P: AsRef<Path> + Send + Sync>(
  path: P,
  sep: u8,
  select_column: String,
  conditions: Vec<String>,
) -> Result<String> {
  generic_multi_search(path, sep, select_column, conditions, |value, condition| {
    value.starts_with(condition)
  })
  .await
}

pub async fn regex_search<P: AsRef<Path> + Send + Sync>(
  path: P,
  sep: u8,
  select_column: String,
  regex_char: String,
  output_path: String,
) -> Result<String> {
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

async fn perform_search<P: AsRef<Path> + Send + Sync>(
  path: P,
  select_column: String,
  conditions: String,
  mode: &str,
) -> Result<String> {
  let csv_options = CsvOptions::new(&path);
  let sep = csv_options.detect_separator()?;

  let multi_conditions: Vec<String> = conditions
    .split('|')
    .map(|s| s.trim().to_string())
    .collect();

  let search_mode = match mode {
    "equalmulti" => SearchMode::EqualMulti(multi_conditions),
    "startswithmulti" => SearchMode::StartsWithMulti(multi_conditions),
    "containsmulti" => SearchMode::ContainsMulti(multi_conditions),
    _ => mode.into(),
  };

  match search_mode {
    SearchMode::EqualMulti(conditions) => {
      equal_multi_search(path, sep, select_column, conditions).await
    }
    SearchMode::StartsWithMulti(conditions) => {
      startswith_multi_search(path, sep, select_column, conditions).await
    }
    SearchMode::ContainsMulti(conditions) => {
      contains_multi_search(path, sep, select_column, conditions).await
    }
    _ => {
      let vec_conditions: Vec<String> = conditions
        .split('|')
        .map(|s| s.trim().to_string())
        .collect();
      let parent_path = path.as_ref().parent().unwrap().to_str().unwrap();
      let file_name = path.as_ref().file_stem().unwrap().to_str().unwrap();
      let output_path = format!("{}/{}.search.csv", parent_path, file_name);

      match search_mode {
        SearchMode::Equal => {
          equal_search(path, sep, select_column, vec_conditions, output_path).await
        }
        SearchMode::Contains => {
          contains_search(path, sep, select_column, vec_conditions, output_path).await
        }
        SearchMode::StartsWith => {
          startswith_search(path, sep, select_column, vec_conditions, output_path).await
        }
        SearchMode::Regex => regex_search(path, sep, select_column, conditions, output_path).await,
        _ => Err(anyhow!("Unsupported search mode")),
      }
    }
  }
}

#[tauri::command]
pub async fn search(
  path: String,
  select_column: String,
  mode: String,
  condition: String,
) -> Result<(String, String), String> {
  let start_time = Instant::now();

  match perform_search(path, select_column, condition, mode.as_str()).await {
    Ok(result) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      let runtime = format!("{elapsed_time:.2}");
      Ok((result, runtime))
    }
    Err(err) => Err(format!("{err}")),
  }
}
