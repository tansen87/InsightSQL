use std::{
  collections::{HashMap, HashSet},
  fs::File,
  io::BufWriter,
  path::Path,
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
  time::{Duration, Instant},
};

use anyhow::{Result, anyhow};
use csv::{ReaderBuilder, Writer, WriterBuilder};
use regex::bytes::RegexBuilder;
use smallvec::SmallVec;
use tauri::AppHandle;
use tokio::sync::oneshot;

use crate::{
  io::csv::{options::CsvOptions, selection::Selection},
  utils::{EventEmitter, WTR_BUFFER_SIZE},
};

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
      "not_equal" => SearchMode::NotEqual,
      "contains" => SearchMode::Contains,
      "not_contains" => SearchMode::NotContains,
      "starts_with" => SearchMode::StartsWith,
      "not_starts_with" => SearchMode::NotStartsWith,
      "ends_with" => SearchMode::EndsWith,
      "not_ends_with" => SearchMode::NotEndsWith,
      "regex" => SearchMode::Regex,
      "is_null" => SearchMode::IsNull,
      "is_not_null" => SearchMode::IsNotNull,
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

async fn generic_search<E, F, P>(
  path: P,
  column: String,
  conditions: Vec<String>,
  skiprows: usize,
  quoting: bool,
  progress: bool,
  match_fn: F,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
  F: Fn(&str, &[String]) -> bool + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync,
{
  let mut opts = CsvOptions::new(&path);
  opts.set_skiprows(skiprows);
  let (sep, reader) = opts.skiprows_and_delimiter()?;
  let output_path = opts.output_path(Some("search"), None)?;

  let total_rows = match progress {
    true => opts.idx_count_rows().await?,
    false => 0,
  };
  emitter.emit_total_rows(total_rows).await?;

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .quoting(quoting)
    .from_reader(reader);

  let sel = Selection::from_headers(rdr.byte_headers()?, &[column.as_str()][..])?;

  let output_file = File::create(output_path)?;
  let buf_wtr = BufWriter::with_capacity(WTR_BUFFER_SIZE, output_file);
  let mut wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_wtr);

  wtr.write_record(rdr.headers()?)?;

  let rows = Arc::new(AtomicUsize::new(0));
  let match_rows = Arc::new(AtomicUsize::new(0));
  let match_rows_clone = Arc::clone(&match_rows);
  let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
  let (done_tx, mut done_rx) = oneshot::channel::<usize>();

  let timer_task = if progress {
    let rows_clone = Arc::clone(&rows);

    Some(tokio::spawn(async move {
      let mut interval = tokio::time::interval(Duration::from_millis(500));
      loop {
        tokio::select! {
          _ = interval.tick() => {
            let current_rows = rows_clone.load(Ordering::Relaxed);
            if let Err(err) = emitter.emit_update_rows(current_rows).await {
              let _ = emitter.emit_err(&format!("failed to emit current rows: {err}")).await;
            }
          },
          Ok(final_rows) = (&mut done_rx) => {
            if let Err(err) = emitter.emit_update_rows(final_rows).await {
              let _ = emitter.emit_err(&format!("failed to emit final rows: {err}")).await;
            }
            break;
          },
          _ = (&mut stop_rx) => { break; }
        }
      }
    }))
  } else {
    None
  };

  let counter_task = tokio::task::spawn_blocking(move || {
    for result in rdr.records() {
      let record = result?;
      if let Some(value) = record.get(sel.first_indices()?) {
        if match_fn(value, &conditions) {
          wtr.write_record(&record)?;

          match_rows.fetch_add(1, Ordering::Relaxed);
        }
      }
      rows.fetch_add(1, Ordering::Relaxed);
    }

    let final_rows = rows.load(Ordering::Relaxed);
    let _ = done_tx.send(final_rows);
    Ok::<_, anyhow::Error>(wtr.flush()?)
  });

  counter_task.await??;
  let _ = stop_tx.send(());
  if let Some(task) = timer_task {
    task.await?;
  }

  let final_match_rows = match_rows_clone.load(Ordering::Relaxed);
  Ok(final_match_rows.to_string())
}

async fn generic_multi_search<E, F, P>(
  path: P,
  column: String,
  conditions: Vec<String>,
  skiprows: usize,
  quoting: bool,
  progress: bool,
  match_fn: F,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
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
  let mut opts = CsvOptions::new(&path);
  opts.set_skiprows(skiprows);
  let (sep, reader) = opts.skiprows_and_delimiter()?;

  let total_rows = match progress {
    true => opts.idx_count_rows().await?,
    false => 0,
  };
  emitter.emit_total_rows(total_rows).await?;

  // prepare writers for each condition with sanitized output paths
  let parent_path = opts.parent_path()?;
  let file_stem = opts.file_stem()?;
  let output_paths: HashMap<String, String> = conditions
    .iter()
    .map(|cond| {
      let sanitized = sanitize_condition(cond);
      let path = format!("{parent_path}/{file_stem}_{sanitized}.csv");
      (cond.clone(), path)
    })
    .collect();

  let rows = Arc::new(AtomicUsize::new(0));
  let match_rows = Arc::new(AtomicUsize::new(0));
  let match_rows_clone = Arc::clone(&match_rows);
  let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
  let (done_tx, mut done_rx) = oneshot::channel::<usize>();

  let timer_task = if progress {
    let rows_clone = Arc::clone(&rows);

    Some(tokio::spawn(async move {
      let mut interval = tokio::time::interval(Duration::from_millis(500));
      loop {
        tokio::select! {
          _ = interval.tick() => {
            let current_rows = rows_clone.load(Ordering::Relaxed);
            if let Err(err) = emitter.emit_update_rows(current_rows).await {
              let _ = emitter.emit_err(&format!("failed to emit current rows: {err}")).await;
            }
          },
          Ok(final_rows) = (&mut done_rx) => {
            if let Err(err) = emitter.emit_update_rows(final_rows).await {
              let _ = emitter.emit_err(&format!("failed to emit final rows: {err}")).await;
            }
            break;
          },
          _ = (&mut stop_rx) => { break; }
        }
      }
    }))
  } else {
    None
  };

  let counter_task = tokio::task::spawn_blocking(move || {
    let mut writers: HashMap<String, Writer<std::fs::File>> = HashMap::new();

    for (cond, path) in &output_paths {
      let file = File::create(path)?;
      writers.insert(
        cond.clone(),
        WriterBuilder::new().delimiter(sep).from_writer(file),
      );
    }

    let mut rdr = ReaderBuilder::new()
      .delimiter(sep)
      .quoting(quoting)
      .from_reader(reader);
    let headers = rdr.headers()?.clone();

    for wtr in writers.values_mut() {
      wtr.write_record(&headers)?;
    }

    let sel = Selection::from_headers(rdr.byte_headers()?, &[column.as_str()][..])?;

    for result in rdr.records() {
      let record = result?;
      if let Some(value) = record.get(sel.first_indices()?) {
        for condition in conditions.iter() {
          if match_fn(value, condition) {
            if let Some(wtr) = writers.get_mut(condition) {
              wtr.write_record(&record)?;

              match_rows.fetch_add(1, Ordering::Relaxed);
            }
          }
        }
      }
      rows.fetch_add(1, Ordering::Relaxed);
    }
    let final_rows = rows.load(Ordering::Relaxed);
    let _ = done_tx.send(final_rows);

    // flush all writers
    for wtr in writers.values_mut() {
      wtr.flush()?;
    }
    Ok::<_, anyhow::Error>(())
  });

  counter_task.await??;
  let _ = stop_tx.send(());
  if let Some(task) = timer_task {
    task.await?;
  }

  let final_match_rows = match_rows_clone.load(Ordering::Relaxed);
  Ok(final_match_rows.to_string())
}

pub async fn equal<E, P>(
  path: P,
  column: String,
  conditions: Vec<String>,
  skiprows: usize,
  quoting: bool,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync,
{
  generic_search(
    path,
    column,
    conditions,
    skiprows,
    quoting,
    progress,
    |value, conditions| conditions.contains(&value.to_string()),
    emitter,
  )
  .await
}

pub async fn equal_multi<E, P>(
  path: P,
  column: String,
  conditions: Vec<String>,
  skiprows: usize,
  quoting: bool,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync + 'static,
{
  generic_multi_search(
    path,
    column,
    conditions,
    skiprows,
    quoting,
    progress,
    |value, condition| value == condition,
    emitter,
  )
  .await
}

pub async fn not_equal<E, P>(
  path: P,
  column: String,
  conditions: Vec<String>,
  skiprows: usize,
  quoting: bool,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync,
{
  generic_search(
    path,
    column,
    conditions,
    skiprows,
    quoting,
    progress,
    |value, cond| !cond.contains(&value.to_string()),
    emitter,
  )
  .await
}

pub async fn contains<E, P>(
  path: P,
  column: String,
  conditions: Vec<String>,
  skiprows: usize,
  quoting: bool,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync,
{
  generic_search(
    path,
    column,
    conditions,
    skiprows,
    quoting,
    progress,
    |value, conditions| conditions.iter().any(|cond| value.contains(cond)),
    emitter,
  )
  .await
}

pub async fn contains_multi<E, P>(
  path: P,
  column: String,
  conditions: Vec<String>,
  skiprows: usize,
  quoting: bool,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync + 'static,
{
  generic_multi_search(
    path,
    column,
    conditions,
    skiprows,
    quoting,
    progress,
    |value, condition| value.contains(condition),
    emitter,
  )
  .await
}

pub async fn not_contains<E, P>(
  path: P,
  column: String,
  conditions: Vec<String>,
  skiprows: usize,
  quoting: bool,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync,
{
  generic_search(
    path,
    column,
    conditions,
    skiprows,
    quoting,
    progress,
    |value, conds| !conds.iter().any(|cond| value.contains(cond)),
    emitter,
  )
  .await
}

pub async fn starts_with<E, P>(
  path: P,
  column: String,
  conditions: Vec<String>,
  skiprows: usize,
  quoting: bool,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync,
{
  generic_search(
    path,
    column,
    conditions,
    skiprows,
    quoting,
    progress,
    |value, conditions| conditions.iter().any(|cond| value.starts_with(cond)),
    emitter,
  )
  .await
}

pub async fn starts_with_multi<E, P>(
  path: P,
  column: String,
  conditions: Vec<String>,
  skiprows: usize,
  quoting: bool,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync + 'static,
{
  generic_multi_search(
    path,
    column,
    conditions,
    skiprows,
    quoting,
    progress,
    |value, condition| value.starts_with(condition),
    emitter,
  )
  .await
}

pub async fn not_starts_with<E, P>(
  path: P,
  column: String,
  conditions: Vec<String>,
  skiprows: usize,
  quoting: bool,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync,
{
  generic_search(
    path,
    column,
    conditions,
    skiprows,
    quoting,
    progress,
    |value, conds| !conds.iter().any(|cond| value.starts_with(cond)),
    emitter,
  )
  .await
}

pub async fn ends_with<E, P>(
  path: P,
  column: String,
  conditions: Vec<String>,
  skiprows: usize,
  quoting: bool,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync,
{
  generic_search(
    path,
    column,
    conditions,
    skiprows,
    quoting,
    progress,
    |value, conds| conds.iter().any(|cond| value.ends_with(cond)),
    emitter,
  )
  .await
}

pub async fn ends_with_multi<E, P>(
  path: P,
  column: String,
  conditions: Vec<String>,
  skiprows: usize,
  quoting: bool,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync + 'static,
{
  generic_multi_search(
    path,
    column,
    conditions,
    skiprows,
    quoting,
    progress,
    |value, conds| value.ends_with(conds),
    emitter,
  )
  .await
}

pub async fn not_ends_with<E, P>(
  path: P,
  column: String,
  conditions: Vec<String>,
  skiprows: usize,
  quoting: bool,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync,
{
  generic_search(
    path,
    column,
    conditions,
    skiprows,
    quoting,
    progress,
    |value, conds| !conds.iter().any(|cond| value.ends_with(cond)),
    emitter,
  )
  .await
}

pub async fn regex_search<E, P>(
  path: P,
  column: String,
  regex_char: String,
  skiprows: usize,
  quoting: bool,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync,
{
  let pattern = RegexBuilder::new(&regex_char).build()?;

  generic_search(
    path,
    column,
    vec![regex_char],
    skiprows,
    quoting,
    progress,
    move |value, _| pattern.is_match(value.as_bytes()),
    emitter,
  )
  .await
}

pub async fn is_null<E, P>(
  path: P,
  column: String,
  conditions: Vec<String>,
  skiprows: usize,
  quoting: bool,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync,
{
  generic_search(
    path,
    column,
    conditions,
    skiprows,
    quoting,
    progress,
    |value, _c| value.trim().is_empty(),
    emitter,
  )
  .await
}

pub async fn is_not_null<E, P>(
  path: P,
  column: String,
  conditions: Vec<String>,
  skiprows: usize,
  quoting: bool,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync,
{
  generic_search(
    path,
    column,
    conditions,
    skiprows,
    quoting,
    progress,
    |value, _c| !value.trim().is_empty(),
    emitter,
  )
  .await
}

pub async fn greater_than<E, P>(
  path: P,
  column: String,
  conditions: String,
  skiprows: usize,
  quoting: bool,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync,
{
  let threshold_value = conditions
    .parse::<f64>()
    .map_err(|_| anyhow!("Condition must be a valid number"))?;

  generic_search(
    path,
    column,
    vec![conditions],
    skiprows,
    quoting,
    progress,
    move |value, _| {
      value
        .parse::<f64>()
        .map(|v| v > threshold_value)
        .unwrap_or(false)
    },
    emitter,
  )
  .await
}

pub async fn greater_than_or_equal<E, P>(
  path: P,
  column: String,
  conditions: String,
  skiprows: usize,
  quoting: bool,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync,
{
  let threshold_value = conditions
    .parse::<f64>()
    .map_err(|_| anyhow!("Condition must be a valid number"))?;

  generic_search(
    path,
    column,
    vec![conditions],
    skiprows,
    quoting,
    progress,
    move |value, _| {
      value
        .parse::<f64>()
        .map(|v| v >= threshold_value)
        .unwrap_or(false)
    },
    emitter,
  )
  .await
}

pub async fn less_than<E, P>(
  path: P,
  column: String,
  conditions: String,
  skiprows: usize,
  quoting: bool,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync,
{
  let threshold_value = conditions
    .parse::<f64>()
    .map_err(|_| anyhow!("Invalid number: {conditions}"))?;

  generic_search(
    path,
    column,
    vec![conditions],
    skiprows,
    quoting,
    progress,
    move |value, _| {
      value
        .parse::<f64>()
        .map(|v| v < threshold_value)
        .unwrap_or(false)
    },
    emitter,
  )
  .await
}

pub async fn less_than_or_equal<E, P>(
  path: P,
  column: String,
  conditions: String,
  skiprows: usize,
  quoting: bool,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync,
{
  let threshold_value = conditions
    .parse::<f64>()
    .map_err(|_| anyhow!("Condition must be a valid number"))?;

  generic_search(
    path,
    column,
    vec![conditions],
    skiprows,
    quoting,
    progress,
    move |value, _| {
      value
        .parse::<f64>()
        .map(|v| v <= threshold_value)
        .unwrap_or(false)
    },
    emitter,
  )
  .await
}

pub async fn between<E, P>(
  path: P,
  column: String,
  conditions: Vec<String>,
  skiprows: usize,
  quoting: bool,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync,
{
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
    column,
    conditions,
    skiprows,
    quoting,
    progress,
    move |value, _| {
      value
        .parse::<f64>()
        .map(|v| v >= min_value && v <= max_value)
        .unwrap_or(false)
    },
    emitter,
  )
  .await
}

async fn perform_search<P: AsRef<Path> + Send + Sync + 'static>(
  path: P,
  column: String,
  conditions: String,
  mode: &str,
  progress: bool,
  quoting: bool,
  skiprows: usize,
  emitter: AppHandle,
) -> Result<String> {
  let multi_conditions = if conditions.contains('|') {
    conditions
      .split('|')
      .map(|s| s.trim().to_string())
      .collect::<HashSet<_>>()
      .into_iter()
      .collect()
  } else {
    let mut v = SmallVec::<[String; 4]>::new();
    v.push(conditions.trim().to_string());
    v
  };

  let search_mode = match mode {
    "equal_multi" => SearchMode::EqualMulti(multi_conditions.to_vec()),
    "starts_with_multi" => SearchMode::StartsWithMulti(multi_conditions.to_vec()),
    "contains_multi" => SearchMode::ContainsMulti(multi_conditions.to_vec()),
    "ends_with_multi" => SearchMode::EndsWithMulti(multi_conditions.to_vec()),
    _ => mode.into(),
  };

  match search_mode {
    SearchMode::EqualMulti(conditions) => {
      equal_multi(
        path, column, conditions, skiprows, quoting, progress, emitter,
      )
      .await
    }
    SearchMode::StartsWithMulti(conditions) => {
      starts_with_multi(
        path, column, conditions, skiprows, quoting, progress, emitter,
      )
      .await
    }
    SearchMode::ContainsMulti(conditions) => {
      contains_multi(
        path, column, conditions, skiprows, quoting, progress, emitter,
      )
      .await
    }
    SearchMode::EndsWithMulti(conditions) => {
      ends_with_multi(
        path, column, conditions, skiprows, quoting, progress, emitter,
      )
      .await
    }
    _ => {
      let vec_conditions = multi_conditions.to_vec();

      match search_mode {
        SearchMode::Equal => {
          equal(
            path,
            column,
            vec_conditions,
            skiprows,
            quoting,
            progress,
            emitter,
          )
          .await
        }
        SearchMode::NotEqual => {
          not_equal(
            path,
            column,
            vec_conditions,
            skiprows,
            quoting,
            progress,
            emitter,
          )
          .await
        }
        SearchMode::Contains => {
          contains(
            path,
            column,
            vec_conditions,
            skiprows,
            quoting,
            progress,
            emitter,
          )
          .await
        }
        SearchMode::NotContains => {
          not_contains(
            path,
            column,
            vec_conditions,
            skiprows,
            quoting,
            progress,
            emitter,
          )
          .await
        }
        SearchMode::StartsWith => {
          starts_with(
            path,
            column,
            vec_conditions,
            skiprows,
            quoting,
            progress,
            emitter,
          )
          .await
        }
        SearchMode::NotStartsWith => {
          not_starts_with(
            path,
            column,
            vec_conditions,
            skiprows,
            quoting,
            progress,
            emitter,
          )
          .await
        }
        SearchMode::EndsWith => {
          ends_with(
            path,
            column,
            vec_conditions,
            skiprows,
            quoting,
            progress,
            emitter,
          )
          .await
        }
        SearchMode::NotEndsWith => {
          not_ends_with(
            path,
            column,
            vec_conditions,
            skiprows,
            quoting,
            progress,
            emitter,
          )
          .await
        }
        SearchMode::Regex => {
          regex_search(
            path, column, conditions, skiprows, quoting, progress, emitter,
          )
          .await
        }
        SearchMode::IsNull => {
          is_null(path, column, vec![], skiprows, quoting, progress, emitter).await
        }
        SearchMode::IsNotNull => {
          is_not_null(path, column, vec![], skiprows, quoting, progress, emitter).await
        }
        SearchMode::GreaterThan => {
          greater_than(
            path, column, conditions, skiprows, quoting, progress, emitter,
          )
          .await
        }
        SearchMode::GreaterThanEqual => {
          greater_than_or_equal(
            path, column, conditions, skiprows, quoting, progress, emitter,
          )
          .await
        }
        SearchMode::LessThan => {
          less_than(
            path, column, conditions, skiprows, quoting, progress, emitter,
          )
          .await
        }
        SearchMode::LessThanEqual => {
          less_than_or_equal(
            path, column, conditions, skiprows, quoting, progress, emitter,
          )
          .await
        }
        SearchMode::Between => {
          between(
            path,
            column,
            vec_conditions,
            skiprows,
            quoting,
            progress,
            emitter,
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
  column: String,
  mode: String,
  condition: String,
  progress: bool,
  quoting: bool,
  skiprows: usize,
  app_handle: AppHandle,
) -> Result<(String, String), String> {
  let start_time = Instant::now();

  match perform_search(
    path, column, condition, &mode, progress, quoting, skiprows, app_handle,
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
