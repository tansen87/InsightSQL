use std::{
  fs::File,
  io::{BufRead, BufReader, BufWriter, Read, Write},
  path::PathBuf,
};

use anyhow::{Result, anyhow};
use regex::bytes::RegexBuilder;

use crate::{
  cmd::search::generic::{generic_parallel_search, generic_search},
  index::Indexed,
  io::csv::options::CsvOptions,
  utils::{EventEmitter, WTR_BUFFER_SIZE},
};

pub async fn equal<E>(
  rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  wtr: csv::Writer<BufWriter<File>>,
  opts: CsvOptions<String>,
  idx: Option<Indexed<File, File>>,
  column: String,
  conditions: Vec<String>,
  progress: bool,
  threads: Option<usize>,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
{
  let jobs = threads.unwrap_or(1);
  let match_fn = |value: &str, cond: &[String]| cond.contains(&value.to_string());
  match jobs {
    1 => generic_search(rdr, wtr, column, conditions, progress, match_fn, emitter).await,
    _ => tokio::task::spawn_blocking(move || {
      generic_parallel_search(
        opts,
        &mut idx.unwrap(),
        wtr,
        column,
        conditions,
        jobs,
        match_fn,
      )
    })
    .await
    .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?,
  }
}

pub async fn not_equal<E>(
  rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  wtr: csv::Writer<BufWriter<File>>,
  opts: CsvOptions<String>,
  idx: Option<Indexed<File, File>>,
  column: String,
  conditions: Vec<String>,
  progress: bool,
  threads: Option<usize>,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
{
  let jobs = threads.unwrap_or(1);
  let match_fn = |value: &str, cond: &[String]| !cond.contains(&value.to_string());
  match jobs {
    1 => generic_search(rdr, wtr, column, conditions, progress, match_fn, emitter).await,
    _ => tokio::task::spawn_blocking(move || {
      generic_parallel_search(
        opts,
        &mut idx.unwrap(),
        wtr,
        column,
        conditions,
        jobs,
        match_fn,
      )
    })
    .await
    .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?,
  }
}

pub async fn contains<E>(
  rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  wtr: csv::Writer<BufWriter<File>>,
  opts: CsvOptions<String>,
  idx: Option<Indexed<File, File>>,
  column: String,
  conditions: Vec<String>,
  progress: bool,
  threads: Option<usize>,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
{
  let jobs = threads.unwrap_or(1);
  let match_fn = |value: &str, cond: &[String]| cond.iter().any(|cond| value.contains(cond));
  match jobs {
    1 => generic_search(rdr, wtr, column, conditions, progress, match_fn, emitter).await,
    _ => tokio::task::spawn_blocking(move || {
      generic_parallel_search(
        opts,
        &mut idx.unwrap(),
        wtr,
        column,
        conditions,
        jobs,
        match_fn,
      )
    })
    .await
    .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?,
  }
}

pub async fn not_contains<E>(
  rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  wtr: csv::Writer<BufWriter<File>>,
  opts: CsvOptions<String>,
  idx: Option<Indexed<File, File>>,
  column: String,
  conditions: Vec<String>,
  progress: bool,
  threads: Option<usize>,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
{
  let jobs = threads.unwrap_or(1);
  let match_fn = |value: &str, conds: &[String]| !conds.iter().any(|cond| value.contains(cond));
  match jobs {
    1 => generic_search(rdr, wtr, column, conditions, progress, match_fn, emitter).await,
    _ => tokio::task::spawn_blocking(move || {
      generic_parallel_search(
        opts,
        &mut idx.unwrap(),
        wtr,
        column,
        conditions,
        jobs,
        match_fn,
      )
    })
    .await
    .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?,
  }
}

pub async fn starts_with<E>(
  rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  wtr: csv::Writer<BufWriter<File>>,
  opts: CsvOptions<String>,
  idx: Option<Indexed<File, File>>,
  column: String,
  conditions: Vec<String>,
  progress: bool,
  threads: Option<usize>,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
{
  let jobs = threads.unwrap_or(1);
  let match_fn = |value: &str, cond: &[String]| cond.iter().any(|cond| value.starts_with(cond));
  match jobs {
    1 => generic_search(rdr, wtr, column, conditions, progress, match_fn, emitter).await,
    _ => tokio::task::spawn_blocking(move || {
      generic_parallel_search(
        opts,
        &mut idx.unwrap(),
        wtr,
        column,
        conditions,
        jobs,
        match_fn,
      )
    })
    .await
    .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?,
  }
}

pub async fn not_starts_with<E>(
  rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  wtr: csv::Writer<BufWriter<File>>,
  opts: CsvOptions<String>,
  idx: Option<Indexed<File, File>>,
  column: String,
  conditions: Vec<String>,
  progress: bool,
  threads: Option<usize>,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
{
  let jobs = threads.unwrap_or(1);
  let match_fn = |value: &str, conds: &[String]| !conds.iter().any(|cond| value.starts_with(cond));
  match jobs {
    1 => generic_search(rdr, wtr, column, conditions, progress, match_fn, emitter).await,
    _ => tokio::task::spawn_blocking(move || {
      generic_parallel_search(
        opts,
        &mut idx.unwrap(),
        wtr,
        column,
        conditions,
        jobs,
        match_fn,
      )
    })
    .await
    .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?,
  }
}

pub async fn ends_with<E>(
  rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  wtr: csv::Writer<BufWriter<File>>,
  opts: CsvOptions<String>,
  idx: Option<Indexed<File, File>>,
  column: String,
  conditions: Vec<String>,
  progress: bool,
  threads: Option<usize>,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
{
  let jobs = threads.unwrap_or(1);
  let match_fn = |value: &str, conds: &[String]| conds.iter().any(|cond| value.ends_with(cond));
  match jobs {
    1 => generic_search(rdr, wtr, column, conditions, progress, match_fn, emitter).await,
    _ => tokio::task::spawn_blocking(move || {
      generic_parallel_search(
        opts,
        &mut idx.unwrap(),
        wtr,
        column,
        conditions,
        jobs,
        match_fn,
      )
    })
    .await
    .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?,
  }
}

pub async fn not_ends_with<E>(
  rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  wtr: csv::Writer<BufWriter<File>>,
  opts: CsvOptions<String>,
  idx: Option<Indexed<File, File>>,
  column: String,
  conditions: Vec<String>,
  progress: bool,
  threads: Option<usize>,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
{
  let jobs = threads.unwrap_or(1);
  let match_fn = |value: &str, conds: &[String]| !conds.iter().any(|cond| value.ends_with(cond));
  match jobs {
    1 => generic_search(rdr, wtr, column, conditions, progress, match_fn, emitter).await,
    _ => tokio::task::spawn_blocking(move || {
      generic_parallel_search(
        opts,
        &mut idx.unwrap(),
        wtr,
        column,
        conditions,
        jobs,
        match_fn,
      )
    })
    .await
    .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?,
  }
}

pub async fn regex_search<E>(
  rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  wtr: csv::Writer<BufWriter<File>>,
  opts: CsvOptions<String>,
  idx: Option<Indexed<File, File>>,
  column: String,
  regex_char: String,
  progress: bool,
  threads: Option<usize>,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
{
  let pattern = RegexBuilder::new(&regex_char).build()?;
  let jobs = threads.unwrap_or(1);
  let match_fn = move |value: &str, _: &[String]| pattern.is_match(value.as_bytes());
  match jobs {
    1 => {
      generic_search(
        rdr,
        wtr,
        column,
        vec![regex_char],
        progress,
        match_fn,
        emitter,
      )
      .await
    }
    _ => tokio::task::spawn_blocking(move || {
      generic_parallel_search(
        opts,
        &mut idx.unwrap(),
        wtr,
        column,
        vec![regex_char],
        jobs,
        match_fn,
      )
    })
    .await
    .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?,
  }
}

pub async fn is_null<E>(
  rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  wtr: csv::Writer<BufWriter<File>>,
  opts: CsvOptions<String>,
  idx: Option<Indexed<File, File>>,
  column: String,
  conditions: Vec<String>,
  progress: bool,
  threads: Option<usize>,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
{
  let jobs = threads.unwrap_or(1);
  let match_fn = |value: &str, _: &[String]| value.trim().is_empty();
  match jobs {
    1 => generic_search(rdr, wtr, column, conditions, progress, match_fn, emitter).await,
    _ => tokio::task::spawn_blocking(move || {
      generic_parallel_search(
        opts,
        &mut idx.unwrap(),
        wtr,
        column,
        conditions,
        jobs,
        match_fn,
      )
    })
    .await
    .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?,
  }
}

pub async fn is_not_null<E>(
  rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  wtr: csv::Writer<BufWriter<File>>,
  opts: CsvOptions<String>,
  idx: Option<Indexed<File, File>>,
  column: String,
  conditions: Vec<String>,
  progress: bool,
  threads: Option<usize>,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
{
  let jobs = threads.unwrap_or(1);
  let match_fn = |value: &str, _: &[String]| !value.trim().is_empty();
  match jobs {
    1 => generic_search(rdr, wtr, column, conditions, progress, match_fn, emitter).await,
    _ => tokio::task::spawn_blocking(move || {
      generic_parallel_search(
        opts,
        &mut idx.unwrap(),
        wtr,
        column,
        conditions,
        jobs,
        match_fn,
      )
    })
    .await
    .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?,
  }
}

pub async fn greater_than<E>(
  rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  wtr: csv::Writer<BufWriter<File>>,
  opts: CsvOptions<String>,
  idx: Option<Indexed<File, File>>,
  column: String,
  conditions: String,
  progress: bool,
  threads: Option<usize>,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
{
  let threshold_value = conditions
    .parse::<f64>()
    .map_err(|_| anyhow!("Condition must be a valid number"))?;
  let jobs = threads.unwrap_or(1);
  let match_fn = move |value: &str, _: &[String]| {
    value
      .parse::<f64>()
      .map(|v| v > threshold_value)
      .unwrap_or(false)
  };
  match jobs {
    1 => {
      generic_search(
        rdr,
        wtr,
        column,
        vec![conditions],
        progress,
        match_fn,
        emitter,
      )
      .await
    }
    _ => tokio::task::spawn_blocking(move || {
      generic_parallel_search(
        opts,
        &mut idx.unwrap(),
        wtr,
        column,
        vec![conditions],
        jobs,
        match_fn,
      )
    })
    .await
    .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?,
  }
}

pub async fn greater_than_or_equal<E>(
  rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  wtr: csv::Writer<BufWriter<File>>,
  opts: CsvOptions<String>,
  idx: Option<Indexed<File, File>>,
  column: String,
  conditions: String,
  progress: bool,
  threads: Option<usize>,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
{
  let threshold_value = conditions
    .parse::<f64>()
    .map_err(|_| anyhow!("Condition must be a valid number"))?;
  let jobs = threads.unwrap_or(1);
  let match_fn = move |value: &str, _: &[String]| {
    value
      .parse::<f64>()
      .map(|v| v >= threshold_value)
      .unwrap_or(false)
  };
  match jobs {
    1 => {
      generic_search(
        rdr,
        wtr,
        column,
        vec![conditions],
        progress,
        match_fn,
        emitter,
      )
      .await
    }
    _ => tokio::task::spawn_blocking(move || {
      generic_parallel_search(
        opts,
        &mut idx.unwrap(),
        wtr,
        column,
        vec![conditions],
        jobs,
        match_fn,
      )
    })
    .await
    .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?,
  }
}

pub async fn less_than<E>(
  rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  wtr: csv::Writer<BufWriter<File>>,
  opts: CsvOptions<String>,
  idx: Option<Indexed<File, File>>,
  column: String,
  conditions: String,
  progress: bool,
  threads: Option<usize>,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
{
  let threshold_value = conditions
    .parse::<f64>()
    .map_err(|_| anyhow!("Invalid number: {conditions}"))?;
  let jobs = threads.unwrap_or(1);
  let match_fn = move |value: &str, _: &[String]| {
    value
      .parse::<f64>()
      .map(|v| v < threshold_value)
      .unwrap_or(false)
  };
  match jobs {
    1 => {
      generic_search(
        rdr,
        wtr,
        column,
        vec![conditions],
        progress,
        match_fn,
        emitter,
      )
      .await
    }
    _ => tokio::task::spawn_blocking(move || {
      generic_parallel_search(
        opts,
        &mut idx.unwrap(),
        wtr,
        column,
        vec![conditions],
        jobs,
        match_fn,
      )
    })
    .await
    .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?,
  }
}

pub async fn less_than_or_equal<E>(
  rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  wtr: csv::Writer<BufWriter<File>>,
  opts: CsvOptions<String>,
  idx: Option<Indexed<File, File>>,
  column: String,
  conditions: String,
  progress: bool,
  threads: Option<usize>,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
{
  let threshold_value = conditions
    .parse::<f64>()
    .map_err(|_| anyhow!("Condition must be a valid number"))?;
  let jobs = threads.unwrap_or(1);
  let match_fn = move |value: &str, _: &[String]| {
    value
      .parse::<f64>()
      .map(|v| v <= threshold_value)
      .unwrap_or(false)
  };
  match jobs {
    1 => {
      generic_search(
        rdr,
        wtr,
        column,
        vec![conditions],
        progress,
        match_fn,
        emitter,
      )
      .await
    }
    _ => tokio::task::spawn_blocking(move || {
      generic_parallel_search(
        opts,
        &mut idx.unwrap(),
        wtr,
        column,
        vec![conditions],
        jobs,
        match_fn,
      )
    })
    .await
    .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?,
  }
}

pub async fn between<E>(
  rdr: csv::Reader<BufReader<Box<dyn Read + Send>>>,
  wtr: csv::Writer<BufWriter<File>>,
  opts: CsvOptions<String>,
  idx: Option<Indexed<File, File>>,
  column: String,
  conditions: Vec<String>,
  progress: bool,
  threads: Option<usize>,
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
  let jobs = threads.unwrap_or(1);
  let match_fn = move |value: &str, _: &[String]| {
    value
      .parse::<f64>()
      .map(|v| v >= min_value && v <= max_value)
      .unwrap_or(false)
  };
  match jobs {
    1 => generic_search(rdr, wtr, column, conditions, progress, match_fn, emitter).await,
    _ => tokio::task::spawn_blocking(move || {
      generic_parallel_search(
        opts,
        &mut idx.unwrap(),
        wtr,
        column,
        conditions,
        jobs,
        match_fn,
      )
    })
    .await
    .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?,
  }
}

pub async fn irregular_with_regex(
  reader: BufReader<Box<dyn Read + Send>>,
  output_path: PathBuf,
  pattern: String,
) -> Result<String> {
  let mut wtr = BufWriter::with_capacity(WTR_BUFFER_SIZE, File::create(output_path)?);
  let re = regex::Regex::new(&pattern)?;

  let mut total = 0;
  for line in reader.lines() {
    let line = line?;
    if re.is_match(&line) {
      writeln!(wtr, "{}", line)?;
      total += 1;
    }
  }
  wtr.flush()?;

  Ok(total.to_string())
}
