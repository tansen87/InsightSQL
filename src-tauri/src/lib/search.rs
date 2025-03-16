use std::{collections::HashMap, fs::File, path::Path, time::Instant};

use anyhow::{anyhow, Result};
use csv::{ReaderBuilder, WriterBuilder};
use regex::bytes::RegexBuilder;

use crate::utils::{CsvOptions, Selection};

#[derive(Debug)]
enum SearchMode {
  Equal,
  EqualMulti(Vec<String>),
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

async fn generic_search<F, P>(
  path: P,
  sep: u8,
  select_column: String,
  conditions: Vec<String>,
  skip_rows: usize,
  output_path: String,
  match_fn: F,
) -> Result<String>
where
  F: Fn(&str, &[String]) -> bool + Send + Sync,
  P: AsRef<Path> + Send + Sync,
{
  let mut match_rows: usize = 0;

  let mut csv_options = CsvOptions::new(&path);
  csv_options.set_skip_rows(skip_rows);

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(csv_options.skip_csv_rows()?);

  let headers = rdr.headers()?.clone();

  let sel = Selection::from_headers(rdr.byte_headers()?, &[select_column.as_str()][..])?;

  let mut wtr = WriterBuilder::new().delimiter(sep).from_path(output_path)?;

  wtr.write_record(&headers)?;

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

pub async fn equal_search<P: AsRef<Path> + Send + Sync>(
  path: P,
  sep: u8,
  select_column: String,
  conditions: Vec<String>,
  skip_rows: usize,
  output_path: String,
) -> Result<String> {
  generic_search(
    path,
    sep,
    select_column,
    conditions,
    skip_rows,
    output_path,
    |value, conditions| conditions.contains(&value.to_string()),
  )
  .await
}

fn sanitize_condition(condition: &str) -> String {
  condition
    .chars()
    .map(|c| match c {
      '/' | '\\' | '|' | ',' | '.' | '"' => '-',
      _ => c,
    })
    .collect()
}

pub async fn equal_multi_search<P: AsRef<Path> + Send + Sync>(
  path: P,
  sep: u8,
  select_column: String,
  conditions: Vec<String>,
  skip_rows: usize,
) -> Result<String> {
  let mut match_rows: usize = 0;
  let mut writers: HashMap<&str, csv::Writer<File>> = HashMap::new();

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
      &condition,
      WriterBuilder::new().delimiter(sep).from_writer(file),
    );
  }

  let mut csv_options = CsvOptions::new(&path);
  csv_options.set_skip_rows(skip_rows);

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(csv_options.skip_csv_rows()?);

  let headers = rdr.headers()?.clone();
  let sel = Selection::from_headers(rdr.byte_headers()?, &[select_column.as_str()][..])?;

  // write headers to all output files
  for writer in writers.values_mut() {
    writer.write_record(&headers)?;
  }

  for result in rdr.records() {
    let record = result?;
    if let Some(value) = record.get(sel.first_indices()?) {
      for condition in &conditions {
        if value == condition {
          if let Some(writer) = writers.get_mut(condition.as_str()) {
            writer.write_record(&record)?;
            match_rows += 1;
          }
        }
      }
    }
  }

  // flush all writers
  for writer in writers.values_mut() {
    writer.flush()?;
  }

  Ok(match_rows.to_string())
}

pub async fn contains_search<P: AsRef<Path> + Send + Sync>(
  path: P,
  sep: u8,
  select_column: String,
  conditions: Vec<String>,
  skip_rows: usize,
  output_path: String,
) -> Result<String> {
  generic_search(
    path,
    sep,
    select_column,
    conditions,
    skip_rows,
    output_path,
    |value, conditions| {
      conditions
        .iter()
        .any(|cond| value.to_lowercase().contains(&cond.to_lowercase()))
    },
  )
  .await
}

pub async fn startswith_search<P: AsRef<Path> + Send + Sync>(
  path: P,
  sep: u8,
  select_column: String,
  conditions: Vec<String>,
  skip_rows: usize,
  output_path: String,
) -> Result<String> {
  generic_search(
    path,
    sep,
    select_column,
    conditions,
    skip_rows,
    output_path,
    |value, conditions| conditions.iter().any(|cond| value.starts_with(cond)),
  )
  .await
}

pub async fn regex_search<P: AsRef<Path> + Send + Sync>(
  path: P,
  sep: u8,
  select_column: String,
  regex_char: String,
  skip_rows: usize,
  output_path: String,
) -> Result<String> {
  let pattern = RegexBuilder::new(&regex_char).build()?;

  generic_search(
    path,
    sep,
    select_column,
    vec![regex_char],
    skip_rows,
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
  skip_rows: usize,
) -> Result<String> {
  let mut csv_options = CsvOptions::new(&path);
  csv_options.set_skip_rows(skip_rows);

  let sep = csv_options.detect_separator()?;

  let search_mode = match mode {
    "equalmulti" => {
      let conditions: Vec<String> = conditions
        .split('|')
        .map(|s| s.trim().to_string())
        .collect();
      SearchMode::EqualMulti(conditions)
    }
    _ => mode.into(),
  };

  match search_mode {
    SearchMode::EqualMulti(conditions) => {
      equal_multi_search(path, sep, select_column, conditions, skip_rows).await
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
          equal_search(
            path,
            sep,
            select_column,
            vec_conditions,
            skip_rows,
            output_path,
          )
          .await
        }
        SearchMode::Contains => {
          contains_search(
            path,
            sep,
            select_column,
            vec_conditions,
            skip_rows,
            output_path,
          )
          .await
        }
        SearchMode::StartsWith => {
          startswith_search(
            path,
            sep,
            select_column,
            vec_conditions,
            skip_rows,
            output_path,
          )
          .await
        }
        SearchMode::Regex => {
          regex_search(path, sep, select_column, conditions, skip_rows, output_path).await
        }
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
  skip_rows: String,
) -> Result<(String, String), String> {
  let start_time = Instant::now();

  match perform_search(
    path,
    select_column,
    condition,
    mode.as_str(),
    skip_rows.parse::<usize>().map_err(|e| e.to_string())?,
  )
  .await
  {
    Ok(result) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      let runtime = format!("{elapsed_time:.2}");
      Ok((result, runtime))
    }
    Err(err) => Err(format!("{err}")),
  }
}
