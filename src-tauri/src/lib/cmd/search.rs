use std::{
  collections::{HashMap, HashSet},
  fs::File,
  path::{Path, PathBuf},
  sync::{Arc, Mutex},
  time::{Duration, Instant},
};

use anyhow::{Result, anyhow};
use csv::{ReaderBuilder, Writer, WriterBuilder};
use regex::bytes::RegexBuilder;
use tauri::{AppHandle, Emitter};
use tokio::sync::oneshot;

use crate::io::csv::{options::CsvOptions, selection::Selection};

#[derive(Debug)]
enum SearchMode {
  Equal,
  EqualMulti(Vec<String>),
  NotEqual,
  Contains,
  ContainsMulti(Vec<String>),
  NotContains,
  StartsWith,
  StartsWithMulti(Vec<String>),
  NotStartsWith,
  EndsWith,
  EndsWithMulti(Vec<String>),
  NotEndsWith,
  Regex,
  IsNull,
  IsNotNull,
  GreaterThan,
  GreaterThanEqual,
  LessThan,
  LessThanEqual,
  Between,
}

impl From<&str> for SearchMode {
  fn from(mode: &str) -> Self {
    match mode {
      "equal" => SearchMode::Equal,
      "notequal" => SearchMode::NotEqual,
      "contains" => SearchMode::Contains,
      "notcontains" => SearchMode::NotContains,
      "startswith" => SearchMode::StartsWith,
      "notstartswith" => SearchMode::NotStartsWith,
      "endswith" => SearchMode::EndsWith,
      "notendswith" => SearchMode::NotEndsWith,
      "regex" => SearchMode::Regex,
      "isnull" => SearchMode::IsNull,
      "isnotnull" => SearchMode::IsNotNull,
      "gt" => SearchMode::GreaterThan,
      "ge" => SearchMode::GreaterThanEqual,
      "lt" => SearchMode::LessThan,
      "le" => SearchMode::LessThanEqual,
      "between" => SearchMode::Between,
      _ => SearchMode::Equal,
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
  output_path: PathBuf,
  match_fn: F,
  app_handle: AppHandle,
) -> Result<String>
where
  F: Fn(&str, &[String]) -> bool + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync,
{
  let csv_options = CsvOptions::new(&path);

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(csv_options.rdr_skip_rows()?);

  let sel = Selection::from_headers(rdr.byte_headers()?, &[select_column.as_str()][..])?;

  let mut wtr = WriterBuilder::new().delimiter(sep).from_path(output_path)?;

  wtr.write_record(rdr.headers()?)?;

  let rows = Arc::new(Mutex::new(0));
  let rows_clone = Arc::clone(&rows);
  let match_rows = Arc::new(Mutex::new(0));
  let match_rows_clone = Arc::clone(&match_rows);
  let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
  let (done_tx, mut done_rx) = oneshot::channel::<usize>();

  let timer_task = tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_millis(500));
    loop {
      tokio::select! {
        _ = interval.tick() => {
          let current_rows = match rows_clone.lock() {
            Ok(lock) => *lock,
            Err(err) => {
              eprintln!("Failed to lock current rows: {err}");
              0
            }
          };
          if let Err(err) = app_handle.emit("update-rows", current_rows) {
            eprintln!("Failed to emit current rows: {err}");
          }
        },
        Ok(final_rows) = (&mut done_rx) => {
          if let Err(err) = app_handle.emit("update-rows", final_rows) {
            eprintln!("Failed to emit final rows: {err}");
          }
          break;
        },
        _ = (&mut stop_rx) => { break; }
      }
    }
  });

  let counter_task = tokio::task::spawn_blocking(move || {
    for result in rdr.records() {
      let record = result?;
      if let Some(value) = record.get(sel.first_indices()?) {
        if match_fn(value, &conditions) {
          wtr.write_record(&record)?;
          let mut match_rows_cnt = match_rows
            .lock()
            .map_err(|poison| anyhow!("match rows cnt lock poisoned: {poison}"))?;
          *match_rows_cnt += 1;
        }
      }
      let mut cnt = rows
        .lock()
        .map_err(|poison| anyhow!("cnt lock poisoned: {poison}"))?;
      *cnt += 1;
    }
    let final_rows = *rows
      .lock()
      .map_err(|poison| anyhow!("final rows lock poisoned: {poison}"))?;
    let _ = done_tx.send(final_rows);
    Ok::<_, anyhow::Error>(wtr.flush()?)
  });

  counter_task.await??;
  let _ = stop_tx.send(());
  timer_task.await?;

  let final_match_rows = *match_rows_clone
    .lock()
    .map_err(|poison| anyhow!("final match rows lock poisoned: {poison}"))?;
  Ok(final_match_rows.to_string())
}

async fn generic_multi_search<F, P>(
  path: P,
  sep: u8,
  select_column: String,
  conditions: Vec<String>,
  match_fn: F,
  app_handle: AppHandle,
) -> Result<String>
where
  F: Fn(&str, &String) -> bool + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync + 'static,
{
  let unique_conditions = conditions
    .into_iter()
    .collect::<HashSet<_>>()
    .into_iter()
    .collect::<Vec<_>>();
  let conditions = Arc::new(unique_conditions);
  let match_fn = Arc::new(match_fn);

  // prepare writers for each condition with sanitized output paths
  let parent_path = path.as_ref().parent().unwrap().to_str().unwrap();
  let file_stem = path.as_ref().file_stem().unwrap().to_str().unwrap();
  let output_paths: HashMap<String, String> = conditions
    .iter()
    .map(|cond| {
      let sanitized = sanitize_condition(cond);
      let path = format!("{parent_path}/{file_stem}_{sanitized}.csv");
      (cond.clone(), path)
    })
    .collect();

  let rows = Arc::new(Mutex::new(0));
  let rows_clone = Arc::clone(&rows);
  let match_rows = Arc::new(Mutex::new(0));
  let match_rows_clone = Arc::clone(&match_rows);
  let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
  let (done_tx, mut done_rx) = oneshot::channel::<usize>();

  let timer_task = tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_millis(500));
    loop {
      tokio::select! {
        _ = interval.tick() => {
          let current_rows = match rows_clone.lock() {
            Ok(lock) => *lock,
            Err(err) => {
              eprintln!("Failed to lock current rows: {err}");
              0
            }
          };
          if let Err(err) = app_handle.emit("update-rows", current_rows) {
            eprintln!("Failed to emit current rows: {err:?}");
          }
        },
        Ok(final_rows) = (&mut done_rx) => {
          if let Err(err) = app_handle.emit("update-rows", final_rows) {
            eprintln!("Failed to emit final rows: {err:?}");
          }
          break;
        },
        _ = (&mut stop_rx) => { break; }
      }
    }
  });

  let counter_task = tokio::task::spawn_blocking(move || {
    let mut writers: HashMap<String, Writer<std::fs::File>> = HashMap::new();

    for (cond, path) in &output_paths {
      let file = File::create(path)?;
      writers.insert(
        cond.clone(),
        WriterBuilder::new().delimiter(sep).from_writer(file),
      );
    }

    let csv_options = CsvOptions::new(&path);
    let mut rdr = ReaderBuilder::new()
      .delimiter(sep)
      .from_reader(csv_options.rdr_skip_rows()?);
    let headers = rdr.headers()?.clone();

    for wtr in writers.values_mut() {
      wtr.write_record(&headers)?;
    }

    let sel = Selection::from_headers(rdr.byte_headers()?, &[select_column.as_str()][..])?;

    for result in rdr.records() {
      let record = result?;
      if let Some(value) = record.get(sel.first_indices()?) {
        for condition in conditions.iter() {
          if match_fn(value, condition) {
            if let Some(wtr) = writers.get_mut(condition) {
              wtr.write_record(&record)?;
              let mut match_rows_cnt = match_rows
                .lock()
                .map_err(|poison| anyhow!("match rows cnt lock poisoned: {poison}"))?;
              *match_rows_cnt += 1;
            }
          }
        }
      }
      let mut cnt = rows
        .lock()
        .map_err(|poison| anyhow!("cnt lock poisoned: {poison}"))?;
      *cnt += 1;
    }
    let final_rows = *rows
      .lock()
      .map_err(|poison| anyhow!("final rows lock poisoned: {poison}"))?;
    let _ = done_tx.send(final_rows);

    // flush all writers
    for wtr in writers.values_mut() {
      wtr.flush()?;
    }
    Ok::<_, anyhow::Error>(())
  });

  counter_task.await??;
  let _ = stop_tx.send(());
  timer_task.await?;

  let final_match_rows = *match_rows_clone
    .lock()
    .map_err(|poison| anyhow!("final match rows lock poisoned: {poison}"))?;
  Ok(final_match_rows.to_string())
}

pub async fn equal<P: AsRef<Path> + Send + Sync>(
  path: P,
  sep: u8,
  select_column: String,
  conditions: Vec<String>,
  output_path: PathBuf,
  app_handle: AppHandle,
) -> Result<String> {
  generic_search(
    path,
    sep,
    select_column,
    conditions,
    output_path,
    |value, conditions| conditions.contains(&value.to_string()),
    app_handle,
  )
  .await
}

pub async fn equal_multi<P: AsRef<Path> + Send + Sync + 'static>(
  path: P,
  sep: u8,
  select_column: String,
  conditions: Vec<String>,
  app_handle: AppHandle,
) -> Result<String> {
  generic_multi_search(
    path,
    sep,
    select_column,
    conditions,
    |value, condition| value == condition,
    app_handle,
  )
  .await
}

pub async fn not_equal<P: AsRef<Path> + Send + Sync>(
  path: P,
  sep: u8,
  select_column: String,
  conditions: Vec<String>,
  output_path: PathBuf,
  app_handle: AppHandle,
) -> Result<String> {
  generic_search(
    path,
    sep,
    select_column,
    conditions,
    output_path,
    |value, cond| !cond.contains(&value.to_string()),
    app_handle,
  )
  .await
}

pub async fn contains<P: AsRef<Path> + Send + Sync>(
  path: P,
  sep: u8,
  select_column: String,
  conditions: Vec<String>,
  output_path: PathBuf,
  app_handle: AppHandle,
) -> Result<String> {
  generic_search(
    path,
    sep,
    select_column,
    conditions,
    output_path,
    |value, conditions| conditions.iter().any(|cond| value.contains(cond)),
    app_handle,
  )
  .await
}

pub async fn contains_multi<P: AsRef<Path> + Send + Sync + 'static>(
  path: P,
  sep: u8,
  select_column: String,
  conditions: Vec<String>,
  app_handle: AppHandle,
) -> Result<String> {
  generic_multi_search(
    path,
    sep,
    select_column,
    conditions,
    |value, condition| value.contains(condition),
    app_handle,
  )
  .await
}

pub async fn not_contains<P: AsRef<Path> + Send + Sync>(
  path: P,
  sep: u8,
  select_column: String,
  conditions: Vec<String>,
  output_path: PathBuf,
  app_handle: AppHandle,
) -> Result<String> {
  generic_search(
    path,
    sep,
    select_column,
    conditions,
    output_path,
    |value, conds| !conds.iter().any(|cond| value.contains(cond)),
    app_handle,
  )
  .await
}

pub async fn starts_with<P: AsRef<Path> + Send + Sync>(
  path: P,
  sep: u8,
  select_column: String,
  conditions: Vec<String>,
  output_path: PathBuf,
  app_handle: AppHandle,
) -> Result<String> {
  generic_search(
    path,
    sep,
    select_column,
    conditions,
    output_path,
    |value, conditions| conditions.iter().any(|cond| value.starts_with(cond)),
    app_handle,
  )
  .await
}

pub async fn starts_with_multi<P: AsRef<Path> + Send + Sync + 'static>(
  path: P,
  sep: u8,
  select_column: String,
  conditions: Vec<String>,
  app_handle: AppHandle,
) -> Result<String> {
  generic_multi_search(
    path,
    sep,
    select_column,
    conditions,
    |value, condition| value.starts_with(condition),
    app_handle,
  )
  .await
}

pub async fn not_starts_with<P: AsRef<Path> + Send + Sync>(
  path: P,
  sep: u8,
  select_column: String,
  conditions: Vec<String>,
  output_path: PathBuf,
  app_handle: AppHandle,
) -> Result<String> {
  generic_search(
    path,
    sep,
    select_column,
    conditions,
    output_path,
    |value, conds| !conds.iter().any(|cond| value.starts_with(cond)),
    app_handle,
  )
  .await
}

pub async fn ends_with<P: AsRef<Path> + Send + Sync>(
  path: P,
  sep: u8,
  select_column: String,
  conditions: Vec<String>,
  output_path: PathBuf,
  app_handle: AppHandle,
) -> Result<String> {
  generic_search(
    path,
    sep,
    select_column,
    conditions,
    output_path,
    |value, conds| conds.iter().any(|cond| value.ends_with(cond)),
    app_handle,
  )
  .await
}

pub async fn ends_with_multi<P: AsRef<Path> + Send + Sync + 'static>(
  path: P,
  sep: u8,
  select_column: String,
  conditions: Vec<String>,
  app_handle: AppHandle,
) -> Result<String> {
  generic_multi_search(
    path,
    sep,
    select_column,
    conditions,
    |value, conds| value.ends_with(conds),
    app_handle,
  )
  .await
}

pub async fn not_ends_with<P: AsRef<Path> + Send + Sync>(
  path: P,
  sep: u8,
  select_column: String,
  conditions: Vec<String>,
  output_path: PathBuf,
  app_handle: AppHandle,
) -> Result<String> {
  generic_search(
    path,
    sep,
    select_column,
    conditions,
    output_path,
    |value, conds| !conds.iter().any(|cond| value.ends_with(cond)),
    app_handle,
  )
  .await
}

pub async fn regex_search<P: AsRef<Path> + Send + Sync>(
  path: P,
  sep: u8,
  select_column: String,
  regex_char: String,
  output_path: PathBuf,
  app_handle: AppHandle,
) -> Result<String> {
  let pattern = RegexBuilder::new(&regex_char).build()?;

  generic_search(
    path,
    sep,
    select_column,
    vec![regex_char],
    output_path,
    move |value, _| pattern.is_match(value.as_bytes()),
    app_handle,
  )
  .await
}

pub async fn is_null<P: AsRef<Path> + Send + Sync>(
  path: P,
  sep: u8,
  select_column: String,
  conditions: Vec<String>,
  output_path: PathBuf,
  app_handle: AppHandle,
) -> Result<String> {
  generic_search(
    path,
    sep,
    select_column,
    conditions,
    output_path,
    |value, _c| value.trim().is_empty(),
    app_handle,
  )
  .await
}

pub async fn is_not_null<P: AsRef<Path> + Send + Sync>(
  path: P,
  sep: u8,
  select_column: String,
  conditions: Vec<String>,
  output_path: PathBuf,
  app_handle: AppHandle,
) -> Result<String> {
  generic_search(
    path,
    sep,
    select_column,
    conditions,
    output_path,
    |value, _c| !value.trim().is_empty(),
    app_handle,
  )
  .await
}

pub async fn greater_than<P: AsRef<Path> + Send + Sync>(
  path: P,
  sep: u8,
  select_column: String,
  conditions: String,
  output_path: PathBuf,
  app_handle: AppHandle,
) -> Result<String> {
  let threshold_value = conditions
    .parse::<f64>()
    .map_err(|_| anyhow!("Condition must be a valid number"))?;

  generic_search(
    path,
    sep,
    select_column,
    vec![conditions],
    output_path,
    move |value, _| {
      value
        .parse::<f64>()
        .map(|v| v > threshold_value)
        .unwrap_or(false)
    },
    app_handle,
  )
  .await
}

pub async fn greater_than_or_equal<P: AsRef<Path> + Send + Sync>(
  path: P,
  sep: u8,
  select_column: String,
  conditions: String,
  output_path: PathBuf,
  app_handle: AppHandle,
) -> Result<String> {
  let threshold_value = conditions
    .parse::<f64>()
    .map_err(|_| anyhow!("Condition must be a valid number"))?;

  generic_search(
    path,
    sep,
    select_column,
    vec![conditions],
    output_path,
    move |value, _| {
      value
        .parse::<f64>()
        .map(|v| v >= threshold_value)
        .unwrap_or(false)
    },
    app_handle,
  )
  .await
}

pub async fn less_than<P: AsRef<Path> + Send + Sync>(
  path: P,
  sep: u8,
  select_column: String,
  conditions: String,
  output_path: PathBuf,
  app_handle: AppHandle,
) -> Result<String> {
  let threshold_value = conditions
    .parse::<f64>()
    .map_err(|_| anyhow!("Invalid number: {conditions}"))?;

  generic_search(
    path,
    sep,
    select_column,
    vec![conditions],
    output_path,
    move |value, _| {
      value
        .parse::<f64>()
        .map(|v| v < threshold_value)
        .unwrap_or(false)
    },
    app_handle,
  )
  .await
}

pub async fn less_than_or_equal<P: AsRef<Path> + Send + Sync>(
  path: P,
  sep: u8,
  select_column: String,
  conditions: String,
  output_path: PathBuf,
  app_handle: AppHandle,
) -> Result<String> {
  let threshold_value = conditions
    .parse::<f64>()
    .map_err(|_| anyhow!("Condition must be a valid number"))?;

  generic_search(
    path,
    sep,
    select_column,
    vec![conditions],
    output_path,
    move |value, _| {
      value
        .parse::<f64>()
        .map(|v| v <= threshold_value)
        .unwrap_or(false)
    },
    app_handle,
  )
  .await
}

pub async fn between<P: AsRef<Path> + Send + Sync>(
  path: P,
  sep: u8,
  select_column: String,
  conditions: Vec<String>,
  output_path: PathBuf,
  app_handle: AppHandle,
) -> Result<String> {
  if conditions.len() != 2 {
    return Err(anyhow!(
      "Exactly two values required for between: min and max"
    ));
  }

  let val1 = conditions[0]
    .parse::<f64>()
    .map_err(|_| anyhow!("Invalid number: {}", conditions[0]))?;

  let val2 = conditions[1]
    .parse::<f64>()
    .map_err(|_| anyhow!("Invalid number: {}", conditions[1]))?;

  let (min_value, max_value) = if val1 <= val2 {
    (val1, val2)
  } else {
    (val2, val1)
  };

  generic_search(
    path,
    sep,
    select_column,
    conditions,
    output_path,
    move |value, _| {
      value
        .parse::<f64>()
        .map(|v| v >= min_value && v <= max_value)
        .unwrap_or(false)
    },
    app_handle,
  )
  .await
}

async fn perform_search<P: AsRef<Path> + Send + Sync + 'static>(
  path: P,
  select_column: String,
  conditions: String,
  mode: &str,
  count_mode: &str,
  app_handle: AppHandle,
) -> Result<String> {
  let csv_options = CsvOptions::new(&path);
  let sep = csv_options.detect_separator()?;

  let total_rows = match count_mode {
    "idx" => csv_options.idx_count_rows().await?,
    "std" => csv_options.std_count_rows()?,
    _ => 0,
  };
  app_handle.emit("total-rows", total_rows)?;

  let multi_conditions: Vec<String> = conditions
    .split('|')
    .map(|s| s.trim().to_string())
    .collect::<HashSet<_>>() // de duplication
    .into_iter()
    .collect();

  let search_mode = match mode {
    "equalmulti" => SearchMode::EqualMulti(multi_conditions),
    "startswithmulti" => SearchMode::StartsWithMulti(multi_conditions),
    "containsmulti" => SearchMode::ContainsMulti(multi_conditions),
    "endswithmulti" => SearchMode::EndsWithMulti(multi_conditions),
    _ => mode.into(),
  };

  match search_mode {
    SearchMode::EqualMulti(conditions) => {
      equal_multi(path, sep, select_column, conditions, app_handle).await
    }
    SearchMode::StartsWithMulti(conditions) => {
      starts_with_multi(path, sep, select_column, conditions, app_handle).await
    }
    SearchMode::ContainsMulti(conditions) => {
      contains_multi(path, sep, select_column, conditions, app_handle).await
    }
    SearchMode::EndsWithMulti(conditions) => {
      ends_with_multi(path, sep, select_column, conditions, app_handle).await
    }
    _ => {
      let vec_conditions: Vec<String> = conditions
        .split('|')
        .map(|s| s.trim().to_string())
        .collect::<HashSet<_>>() // de duplication
        .into_iter()
        .collect();
      let parent_path = path.as_ref().parent().unwrap().to_str().unwrap();
      let file_stem = path.as_ref().file_stem().unwrap().to_str().unwrap();
      let mut output_path = PathBuf::from(parent_path);
      output_path.push(format!("{file_stem}.search.csv"));

      match search_mode {
        SearchMode::Equal => {
          equal(
            path,
            sep,
            select_column,
            vec_conditions,
            output_path,
            app_handle,
          )
          .await
        }
        SearchMode::NotEqual => {
          not_equal(
            path,
            sep,
            select_column,
            vec_conditions,
            output_path,
            app_handle,
          )
          .await
        }
        SearchMode::Contains => {
          contains(
            path,
            sep,
            select_column,
            vec_conditions,
            output_path,
            app_handle,
          )
          .await
        }
        SearchMode::NotContains => {
          not_contains(
            path,
            sep,
            select_column,
            vec_conditions,
            output_path,
            app_handle,
          )
          .await
        }
        SearchMode::StartsWith => {
          starts_with(
            path,
            sep,
            select_column,
            vec_conditions,
            output_path,
            app_handle,
          )
          .await
        }
        SearchMode::NotStartsWith => {
          not_starts_with(
            path,
            sep,
            select_column,
            vec_conditions,
            output_path,
            app_handle,
          )
          .await
        }
        SearchMode::EndsWith => {
          ends_with(
            path,
            sep,
            select_column,
            vec_conditions,
            output_path,
            app_handle,
          )
          .await
        }
        SearchMode::NotEndsWith => {
          not_ends_with(
            path,
            sep,
            select_column,
            vec_conditions,
            output_path,
            app_handle,
          )
          .await
        }
        SearchMode::Regex => {
          regex_search(
            path,
            sep,
            select_column,
            conditions,
            output_path,
            app_handle,
          )
          .await
        }
        SearchMode::IsNull => {
          is_null(path, sep, select_column, vec![], output_path, app_handle).await
        }
        SearchMode::IsNotNull => {
          is_not_null(path, sep, select_column, vec![], output_path, app_handle).await
        }
        SearchMode::GreaterThan => {
          greater_than(
            path,
            sep,
            select_column,
            conditions,
            output_path,
            app_handle,
          )
          .await
        }
        SearchMode::GreaterThanEqual => {
          greater_than_or_equal(
            path,
            sep,
            select_column,
            conditions,
            output_path,
            app_handle,
          )
          .await
        }
        SearchMode::LessThan => {
          less_than(
            path,
            sep,
            select_column,
            conditions,
            output_path,
            app_handle,
          )
          .await
        }
        SearchMode::LessThanEqual => {
          less_than_or_equal(
            path,
            sep,
            select_column,
            conditions,
            output_path,
            app_handle,
          )
          .await
        }
        SearchMode::Between => {
          between(
            path,
            sep,
            select_column,
            vec_conditions,
            output_path,
            app_handle,
          )
          .await
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
  count_mode: String,
  app_handle: AppHandle,
) -> Result<(String, String), String> {
  let start_time = Instant::now();

  match perform_search(
    path,
    select_column,
    condition,
    mode.as_str(),
    count_mode.as_str(),
    app_handle,
  )
  .await
  {
    Ok(match_rows) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok((match_rows, format!("{elapsed_time:.2}")))
    }
    Err(err) => Err(format!("{err}")),
  }
}
