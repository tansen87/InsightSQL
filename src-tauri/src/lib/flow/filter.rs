use anyhow::{Result, anyhow};
use csv::StringRecord;

pub fn equal(
  column: &str,
  value: &str,
  headers: &[String],
) -> Result<impl Fn(&StringRecord) -> bool + use<>> {
  let idx = headers
    .iter()
    .position(|h| h == column)
    .ok_or_else(|| anyhow!("Column not found: {}", column))?;
  let value = value.to_string();
  Ok(move |record: &StringRecord| record.get(idx).map_or(false, |field| field == value))
}

pub fn not_equal(
  column: &str,
  value: &str,
  headers: &[String],
) -> Result<impl Fn(&StringRecord) -> bool + use<>> {
  let idx = headers
    .iter()
    .position(|h| h == column)
    .ok_or_else(|| anyhow!("Column not found: {}", column))?;
  let value = value.to_string();
  Ok(move |record: &StringRecord| record.get(idx).map_or(true, |field| field != value))
}

pub fn is_in(
  column: &str,
  value: &str,
  headers: &[String],
) -> Result<impl Fn(&StringRecord) -> bool + use<>> {
  let idx = headers
    .iter()
    .position(|h| h == column)
    .ok_or_else(|| anyhow!("Column not found: {}", column))?;
  let values: Vec<String> = value.split('|').map(|s| s.to_string()).collect();
  Ok(move |record: &StringRecord| {
    record
      .get(idx)
      .map_or(false, |field| values.contains(&field.to_string()))
  })
}

pub fn contains(
  column: &str,
  substring: &str,
  headers: &[String],
) -> Result<impl Fn(&StringRecord) -> bool + use<>> {
  let column_idx = headers
    .iter()
    .position(|h| h == column)
    .ok_or_else(|| anyhow!("Column not found: {}", column))?;
  let substring = substring.to_string();
  Ok(move |record: &StringRecord| {
    record
      .get(column_idx)
      .map_or(false, |field| field.contains(&substring))
  })
}

pub fn not_contains(
  column: &str,
  substring: &str,
  headers: &[String],
) -> Result<impl Fn(&StringRecord) -> bool + use<>> {
  let idx = headers
    .iter()
    .position(|h| h == column)
    .ok_or_else(|| anyhow!("Column not found: {}", column))?;
  let substring = substring.to_string();
  Ok(move |record: &StringRecord| {
    record
      .get(idx)
      .map_or(true, |field| !field.contains(&substring))
  })
}

pub fn starts_with(
  column: &str,
  prefix: &str,
  headers: &[String],
) -> Result<impl Fn(&StringRecord) -> bool + use<>> {
  let idx = headers
    .iter()
    .position(|h| h == column)
    .ok_or_else(|| anyhow!("Column not found: {}", column))?;
  let prefix = prefix.to_string();
  Ok(move |record: &StringRecord| {
    record
      .get(idx)
      .map_or(false, |field| field.starts_with(&prefix))
  })
}

pub fn not_starts_with(
  column: &str,
  prefix: &str,
  headers: &[String],
) -> Result<impl Fn(&StringRecord) -> bool + use<>> {
  let idx = headers
    .iter()
    .position(|h| h == column)
    .ok_or_else(|| anyhow!("Column not found: {}", column))?;
  let prefix = prefix.to_string();
  Ok(move |record: &StringRecord| {
    record
      .get(idx)
      .map_or(true, |field| !field.starts_with(&prefix))
  })
}

pub fn ends_with(
  column: &str,
  suffix: &str,
  headers: &[String],
) -> Result<impl Fn(&StringRecord) -> bool + use<>> {
  let idx = headers
    .iter()
    .position(|h| h == column)
    .ok_or_else(|| anyhow!("Column not found: {}", column))?;
  let suffix = suffix.to_string();
  Ok(move |record: &StringRecord| {
    record
      .get(idx)
      .map_or(false, |field| field.ends_with(&suffix))
  })
}

pub fn not_ends_with(
  column: &str,
  suffix: &str,
  headers: &[String],
) -> Result<impl Fn(&StringRecord) -> bool + use<>> {
  let idx = headers
    .iter()
    .position(|h| h == column)
    .ok_or_else(|| anyhow!("Column not found: {}", column))?;
  let suffix = suffix.to_string();
  Ok(move |record: &StringRecord| {
    record
      .get(idx)
      .map_or(true, |field| !field.ends_with(&suffix))
  })
}

pub fn is_null(column: &str, headers: &[String]) -> Result<impl Fn(&StringRecord) -> bool + use<>> {
  let idx = headers
    .iter()
    .position(|h| h == column)
    .ok_or_else(|| anyhow!("Column not found: {}", column))?;
  Ok(move |record: &StringRecord| {
    record
      .get(idx)
      .map_or(true, |field| field.trim().is_empty())
  })
}

pub fn is_not_null(
  column: &str,
  headers: &[String],
) -> Result<impl Fn(&StringRecord) -> bool + use<>> {
  let idx = headers
    .iter()
    .position(|h| h == column)
    .ok_or_else(|| anyhow!("Column not found: {}", column))?;
  Ok(move |record: &StringRecord| {
    record
      .get(idx)
      .map_or(false, |field| !field.trim().is_empty())
  })
}

pub fn gt(
  column: &str,
  value: &str,
  headers: &[String],
) -> Result<impl Fn(&StringRecord) -> bool + use<>> {
  let idx = headers
    .iter()
    .position(|h| h == column)
    .ok_or_else(|| anyhow!("Column not found: {}", column))?;
  let value = value
    .parse::<f64>()
    .map_err(|e| anyhow!("filter value 不是有效数字: {}", e))?;
  Ok(move |record: &StringRecord| {
    record
      .get(idx)
      .and_then(|field| field.parse::<f64>().ok())
      .map_or(false, |v| v > value)
  })
}

pub fn ge(
  column: &str,
  value: &str,
  headers: &[String],
) -> Result<impl Fn(&StringRecord) -> bool + use<>> {
  let idx = headers
    .iter()
    .position(|h| h == column)
    .ok_or_else(|| anyhow!("Column not found: {}", column))?;
  let value = value
    .parse::<f64>()
    .map_err(|e| anyhow!("filter value 不是有效数字: {}", e))?;
  Ok(move |record: &StringRecord| {
    record
      .get(idx)
      .and_then(|field| field.parse::<f64>().ok())
      .map_or(false, |v| v >= value)
  })
}

pub fn lt(
  column: &str,
  value: &str,
  headers: &[String],
) -> Result<impl Fn(&StringRecord) -> bool + use<>> {
  let idx = headers
    .iter()
    .position(|h| h == column)
    .ok_or_else(|| anyhow!("Column not found: {}", column))?;
  let value = value
    .parse::<f64>()
    .map_err(|e| anyhow!("filter value 不是有效数字: {}", e))?;
  Ok(move |record: &StringRecord| {
    record
      .get(idx)
      .and_then(|field| field.parse::<f64>().ok())
      .map_or(false, |v| v < value)
  })
}

pub fn le(
  column: &str,
  value: &str,
  headers: &[String],
) -> Result<impl Fn(&StringRecord) -> bool + use<>> {
  let idx = headers
    .iter()
    .position(|h| h == column)
    .ok_or_else(|| anyhow!("Column not found: {}", column))?;
  let value = value
    .parse::<f64>()
    .map_err(|e| anyhow!("filter value 不是有效数字: {}", e))?;
  Ok(move |record: &StringRecord| {
    record
      .get(idx)
      .and_then(|field| field.parse::<f64>().ok())
      .map_or(false, |v| v <= value)
  })
}

pub fn between(
  column: &str,
  value: &str,
  headers: &[String],
) -> Result<impl Fn(&StringRecord) -> bool + use<>> {
  let idx = headers
    .iter()
    .position(|h| h == column)
    .ok_or_else(|| anyhow!("Column not found: {}", column))?;
  let parts: Vec<&str> = value.split('|').collect();
  let min = parts
    .get(0)
    .and_then(|s| s.parse::<f64>().ok())
    .unwrap_or(f64::MIN);
  let max = parts
    .get(1)
    .and_then(|s| s.parse::<f64>().ok())
    .unwrap_or(f64::MAX);
  Ok(move |record: &StringRecord| {
    record
      .get(idx)
      .and_then(|field| field.parse::<f64>().ok())
      .map_or(false, |v| v >= min && v <= max)
  })
}
