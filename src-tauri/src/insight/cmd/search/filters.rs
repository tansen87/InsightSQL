use std::{
  fs::File,
  io::{BufReader, BufWriter, Read},
};

use anyhow::{Result, anyhow};
use regex::bytes::RegexBuilder;

use crate::{cmd::search::generic::generic_search, utils::EventEmitter};

pub async fn equal<E>(
  rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  wtr: csv::Writer<BufWriter<File>>,
  column: String,
  conditions: Vec<String>,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
{
  generic_search(
    rdr,
    wtr,
    column,
    conditions,
    progress,
    |value, conditions| conditions.contains(&value.to_string()),
    emitter,
  )
  .await
}

pub async fn not_equal<E>(
  rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  wtr: csv::Writer<BufWriter<File>>,
  column: String,
  conditions: Vec<String>,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
{
  generic_search(
    rdr,
    wtr,
    column,
    conditions,
    progress,
    |value, cond| !cond.contains(&value.to_string()),
    emitter,
  )
  .await
}

pub async fn contains<E>(
  rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  wtr: csv::Writer<BufWriter<File>>,
  column: String,
  conditions: Vec<String>,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
{
  generic_search(
    rdr,
    wtr,
    column,
    conditions,
    progress,
    |value, conditions| conditions.iter().any(|cond| value.contains(cond)),
    emitter,
  )
  .await
}

pub async fn not_contains<E>(
  rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  wtr: csv::Writer<BufWriter<File>>,
  column: String,
  conditions: Vec<String>,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
{
  generic_search(
    rdr,
    wtr,
    column,
    conditions,
    progress,
    |value, conds| !conds.iter().any(|cond| value.contains(cond)),
    emitter,
  )
  .await
}

pub async fn starts_with<E>(
  rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  wtr: csv::Writer<BufWriter<File>>,
  column: String,
  conditions: Vec<String>,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
{
  generic_search(
    rdr,
    wtr,
    column,
    conditions,
    progress,
    |value, conditions| conditions.iter().any(|cond| value.starts_with(cond)),
    emitter,
  )
  .await
}

pub async fn not_starts_with<E>(
  rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  wtr: csv::Writer<BufWriter<File>>,
  column: String,
  conditions: Vec<String>,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
{
  generic_search(
    rdr,
    wtr,
    column,
    conditions,
    progress,
    |value, conds| !conds.iter().any(|cond| value.starts_with(cond)),
    emitter,
  )
  .await
}

pub async fn ends_with<E>(
  rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  wtr: csv::Writer<BufWriter<File>>,
  column: String,
  conditions: Vec<String>,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
{
  generic_search(
    rdr,
    wtr,
    column,
    conditions,
    progress,
    |value, conds| conds.iter().any(|cond| value.ends_with(cond)),
    emitter,
  )
  .await
}

pub async fn not_ends_with<E>(
  rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  wtr: csv::Writer<BufWriter<File>>,
  column: String,
  conditions: Vec<String>,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
{
  generic_search(
    rdr,
    wtr,
    column,
    conditions,
    progress,
    |value, conds| !conds.iter().any(|cond| value.ends_with(cond)),
    emitter,
  )
  .await
}

pub async fn regex_search<E>(
  rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  wtr: csv::Writer<BufWriter<File>>,
  column: String,
  regex_char: String,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
{
  let pattern = RegexBuilder::new(&regex_char).build()?;

  generic_search(
    rdr,
    wtr,
    column,
    vec![regex_char],
    progress,
    move |value, _| pattern.is_match(value.as_bytes()),
    emitter,
  )
  .await
}

pub async fn is_null<E>(
  rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  wtr: csv::Writer<BufWriter<File>>,
  column: String,
  conditions: Vec<String>,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
{
  generic_search(
    rdr,
    wtr,
    column,
    conditions,
    progress,
    |value, _c| value.trim().is_empty(),
    emitter,
  )
  .await
}

pub async fn is_not_null<E>(
  rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  wtr: csv::Writer<BufWriter<File>>,
  column: String,
  conditions: Vec<String>,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
{
  generic_search(
    rdr,
    wtr,
    column,
    conditions,
    progress,
    |value, _c| !value.trim().is_empty(),
    emitter,
  )
  .await
}

pub async fn greater_than<E>(
  rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  wtr: csv::Writer<BufWriter<File>>,
  column: String,
  conditions: String,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
{
  let threshold_value = conditions
    .parse::<f64>()
    .map_err(|_| anyhow!("Condition must be a valid number"))?;

  generic_search(
    rdr,
    wtr,
    column,
    vec![conditions],
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

pub async fn greater_than_or_equal<E>(
  rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  wtr: csv::Writer<BufWriter<File>>,
  column: String,
  conditions: String,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
{
  let threshold_value = conditions
    .parse::<f64>()
    .map_err(|_| anyhow!("Condition must be a valid number"))?;

  generic_search(
    rdr,
    wtr,
    column,
    vec![conditions],
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

pub async fn less_than<E>(
  rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  wtr: csv::Writer<BufWriter<File>>,
  column: String,
  conditions: String,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
{
  let threshold_value = conditions
    .parse::<f64>()
    .map_err(|_| anyhow!("Invalid number: {conditions}"))?;

  generic_search(
    rdr,
    wtr,
    column,
    vec![conditions],
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

pub async fn less_than_or_equal<E>(
  rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  wtr: csv::Writer<BufWriter<File>>,
  column: String,
  conditions: String,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
{
  let threshold_value = conditions
    .parse::<f64>()
    .map_err(|_| anyhow!("Condition must be a valid number"))?;

  generic_search(
    rdr,
    wtr,
    column,
    vec![conditions],
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

pub async fn between<E>(
  rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  wtr: csv::Writer<BufWriter<File>>,
  column: String,
  conditions: Vec<String>,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
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
    rdr,
    wtr,
    column,
    conditions,
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
