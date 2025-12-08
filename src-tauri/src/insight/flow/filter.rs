use std::{collections::HashSet, sync::Arc};

use anyhow::{Result, anyhow};
use csv::StringRecord;

pub fn equal(
  column: Arc<str>,
  value: Arc<str>,
  headers: Arc<Vec<String>>,
) -> Result<Box<dyn Fn(&StringRecord) -> bool + Send + Sync>> {
  let idx = headers
    .iter()
    .position(|h| h == column.as_ref())
    .ok_or_else(|| anyhow!("Column not found: {}", column))?;
  if value.contains('|') {
    let values: Vec<Arc<str>> = value
      .split('|')
      .map(|s| s.trim())
      .filter(|s| !s.is_empty())
      .collect::<HashSet<_>>()
      .into_iter()
      .map(Arc::from)
      .collect();
    Ok(Box::new(move |record: &StringRecord| {
      record
        .get(idx)
        .map(|f| values.iter().any(|val| f == val.as_ref()))
        .unwrap_or(false)
    }))
  } else {
    let val = value.to_string();
    Ok(Box::new(move |record: &StringRecord| {
      record.get(idx).map_or(false, |f| f == val)
    }))
  }
}

pub fn not_equal(
  column: Arc<str>,
  value: Arc<str>,
  headers: Arc<Vec<String>>,
) -> Result<Box<dyn Fn(&StringRecord) -> bool + Send + Sync>> {
  let idx = headers
    .iter()
    .position(|h| h == column.as_ref())
    .ok_or_else(|| anyhow!("Column not found: {}", column))?;
  if value.contains('|') {
    let values: Vec<Arc<str>> = value
      .split('|')
      .map(|s| s.trim())
      .filter(|s| !s.is_empty())
      .collect::<HashSet<_>>()
      .into_iter()
      .map(Arc::from)
      .collect();
    Ok(Box::new(move |record: &StringRecord| {
      record
        .get(idx)
        .map_or(true, |f| values.iter().all(|val| f != val.as_ref()))
    }))
  } else {
    let val = value.to_string();
    Ok(Box::new(move |record: &StringRecord| {
      record.get(idx).map_or(true, |f| f != val)
    }))
  }
}

pub fn contains(
  column: Arc<str>,
  value: Arc<str>,
  headers: Arc<Vec<String>>,
) -> Result<Box<dyn Fn(&StringRecord) -> bool + Send + Sync>> {
  let idx = headers
    .iter()
    .position(|h| h == column.as_ref())
    .ok_or_else(|| anyhow!("Column not found: {}", column))?;
  if value.contains('|') {
    let values: Vec<Arc<str>> = value
      .split('|')
      .map(|s| s.trim())
      .filter(|s| !s.is_empty())
      .collect::<HashSet<_>>()
      .into_iter()
      .map(Arc::from)
      .collect();
    Ok(Box::new(move |record: &StringRecord| {
      record
        .get(idx)
        .map(|f| values.iter().any(|val| f.contains(val.as_ref())))
        .unwrap_or(false)
    }))
  } else {
    let val = value.to_string();
    Ok(Box::new(move |record: &StringRecord| {
      record.get(idx).map(|f| f.contains(&val)).unwrap_or(false)
    }))
  }
}

pub fn not_contains(
  column: Arc<str>,
  value: Arc<str>,
  headers: Arc<Vec<String>>,
) -> Result<Box<dyn Fn(&StringRecord) -> bool + Send + Sync>> {
  let idx = headers
    .iter()
    .position(|h| h == column.as_ref())
    .ok_or_else(|| anyhow!("Column not found: {}", column))?;
  if value.contains('|') {
    let values: Vec<Arc<str>> = value
      .split('|')
      .map(|s| s.trim())
      .filter(|s| !s.is_empty())
      .collect::<HashSet<_>>()
      .into_iter()
      .map(Arc::from)
      .collect();
    Ok(Box::new(move |record: &StringRecord| {
      record
        .get(idx)
        .map(|f| !values.iter().any(|val| f.contains(val.as_ref())))
        .unwrap_or(true)
    }))
  } else {
    let val = value.to_string();
    Ok(Box::new(move |record: &StringRecord| {
      record.get(idx).map(|f| !f.contains(&val)).unwrap_or(true)
    }))
  }
}

pub fn starts_with(
  column: Arc<str>,
  value: Arc<str>,
  headers: Arc<Vec<String>>,
) -> Result<Box<dyn Fn(&StringRecord) -> bool + Send + Sync>> {
  let idx = headers
    .iter()
    .position(|h| h == column.as_ref())
    .ok_or_else(|| anyhow!("Column not found: {}", column))?;
  if value.contains('|') {
    let values: Vec<Arc<str>> = value
      .split('|')
      .map(|s| s.trim())
      .filter(|s| !s.is_empty())
      .collect::<HashSet<_>>()
      .into_iter()
      .map(Arc::from)
      .collect();
    Ok(Box::new(move |record: &StringRecord| {
      record
        .get(idx)
        .map(|f| values.iter().any(|val| f.starts_with(val.as_ref())))
        .unwrap_or(false)
    }))
  } else {
    let val = value.to_string();
    Ok(Box::new(move |record: &StringRecord| {
      record
        .get(idx)
        .map(|f| f.starts_with(&val))
        .unwrap_or(false)
    }))
  }
}

pub fn not_starts_with(
  column: Arc<str>,
  value: Arc<str>,
  headers: Arc<Vec<String>>,
) -> Result<Box<dyn Fn(&StringRecord) -> bool + Send + Sync>> {
  let idx = headers
    .iter()
    .position(|h| h == column.as_ref())
    .ok_or_else(|| anyhow!("Column not found: {}", column))?;
  if value.contains('|') {
    let values: Vec<Arc<str>> = value
      .split('|')
      .map(|s| s.trim())
      .filter(|s| !s.is_empty())
      .collect::<HashSet<_>>()
      .into_iter()
      .map(Arc::from)
      .collect();
    Ok(Box::new(move |record: &StringRecord| {
      record
        .get(idx)
        .map(|f| values.iter().all(|val| !f.starts_with(val.as_ref())))
        .unwrap_or(true)
    }))
  } else {
    let val = value.to_string();
    Ok(Box::new(move |record: &StringRecord| {
      record
        .get(idx)
        .map(|f| !f.starts_with(&val))
        .unwrap_or(true)
    }))
  }
}

pub fn ends_with(
  column: Arc<str>,
  value: Arc<str>,
  headers: Arc<Vec<String>>,
) -> Result<Box<dyn Fn(&StringRecord) -> bool + Send + Sync>> {
  let idx = headers
    .iter()
    .position(|h| h == column.as_ref())
    .ok_or_else(|| anyhow!("Column not found: {}", column))?;
  if value.contains('|') {
    let values: Vec<Arc<str>> = value
      .split('|')
      .map(|s| s.trim())
      .filter(|s| !s.is_empty())
      .collect::<HashSet<_>>()
      .into_iter()
      .map(Arc::from)
      .collect();
    Ok(Box::new(move |record: &StringRecord| {
      record
        .get(idx)
        .map(|f| values.iter().any(|val| f.ends_with(val.as_ref())))
        .unwrap_or(false)
    }))
  } else {
    let val = value.to_string();
    Ok(Box::new(move |record: &StringRecord| {
      record.get(idx).map(|f| f.ends_with(&val)).unwrap_or(false)
    }))
  }
}

pub fn not_ends_with(
  column: Arc<str>,
  value: Arc<str>,
  headers: Arc<Vec<String>>,
) -> Result<Box<dyn Fn(&StringRecord) -> bool + Send + Sync>> {
  let idx = headers
    .iter()
    .position(|h| h == column.as_ref())
    .ok_or_else(|| anyhow!("Column not found: {}", column))?;
  if value.contains('|') {
    let values: Vec<Arc<str>> = value
      .split('|')
      .map(|s| s.trim())
      .filter(|s| !s.is_empty())
      .collect::<HashSet<_>>()
      .into_iter()
      .map(Arc::from)
      .collect();
    Ok(Box::new(move |record: &StringRecord| {
      record
        .get(idx)
        .map(|f| values.iter().all(|val| !f.ends_with(val.as_ref())))
        .unwrap_or(true)
    }))
  } else {
    let val = value.to_string();
    Ok(Box::new(move |record: &StringRecord| {
      record.get(idx).map(|f| !f.ends_with(&val)).unwrap_or(true)
    }))
  }
}

pub fn is_null(
  column: Arc<str>,
  headers: Arc<Vec<String>>,
) -> Result<Box<dyn Fn(&StringRecord) -> bool + Send + Sync>> {
  let idx = headers
    .iter()
    .position(|h| h == column.as_ref())
    .ok_or_else(|| anyhow!("Column not found: {}", column))?;
  Ok(Box::new(move |record: &StringRecord| {
    record.get(idx).map_or(true, |f| f.trim().is_empty())
  }))
}

pub fn is_not_null(
  column: Arc<str>,
  headers: Arc<Vec<String>>,
) -> Result<Box<dyn Fn(&StringRecord) -> bool + Send + Sync>> {
  let idx = headers
    .iter()
    .position(|h| h == column.as_ref())
    .ok_or_else(|| anyhow!("Column not found: {}", column))?;
  Ok(Box::new(move |record: &StringRecord| {
    record.get(idx).map_or(false, |f| !f.trim().is_empty())
  }))
}

pub fn gt(
  column: Arc<str>,
  value: Arc<str>,
  headers: Arc<Vec<String>>,
) -> Result<Box<dyn Fn(&StringRecord) -> bool + Send + Sync>> {
  let idx = headers
    .iter()
    .position(|h| h == column.as_ref())
    .ok_or_else(|| anyhow!("Column not found: {}", column))?;
  let val = value
    .parse::<f64>()
    .map_err(|e| anyhow!("filter value is invalid number: {}", e))?;
  Ok(Box::new(move |record: &StringRecord| {
    record
      .get(idx)
      .and_then(|f| f.parse::<f64>().ok())
      .map(|f| f > val)
      .unwrap_or(false)
  }))
}

pub fn ge(
  column: Arc<str>,
  value: Arc<str>,
  headers: Arc<Vec<String>>,
) -> Result<Box<dyn Fn(&StringRecord) -> bool + Send + Sync>> {
  let idx = headers
    .iter()
    .position(|h| h == column.as_ref())
    .ok_or_else(|| anyhow!("Column not found: {}", column))?;
  let val = value
    .parse::<f64>()
    .map_err(|e| anyhow!("filter value is invalid number: {}", e))?;
  Ok(Box::new(move |record: &StringRecord| {
    record
      .get(idx)
      .and_then(|f| f.parse::<f64>().ok())
      .map(|f| f >= val)
      .unwrap_or(false)
  }))
}

pub fn lt(
  column: Arc<str>,
  value: Arc<str>,
  headers: Arc<Vec<String>>,
) -> Result<Box<dyn Fn(&StringRecord) -> bool + Send + Sync>> {
  let idx = headers
    .iter()
    .position(|h| h == column.as_ref())
    .ok_or_else(|| anyhow!("Column not found: {}", column))?;
  let val = value
    .parse::<f64>()
    .map_err(|e| anyhow!("filter value is invalid number: {}", e))?;
  Ok(Box::new(move |record: &StringRecord| {
    record
      .get(idx)
      .and_then(|f| f.parse::<f64>().ok())
      .map(|f| f < val)
      .unwrap_or(false)
  }))
}

pub fn le(
  column: Arc<str>,
  value: Arc<str>,
  headers: Arc<Vec<String>>,
) -> Result<Box<dyn Fn(&StringRecord) -> bool + Send + Sync>> {
  let idx = headers
    .iter()
    .position(|h| h == column.as_ref())
    .ok_or_else(|| anyhow!("Column not found: {}", column))?;
  let val = value
    .parse::<f64>()
    .map_err(|e| anyhow!("filter value is invalid number: {}", e))?;
  Ok(Box::new(move |record: &StringRecord| {
    record
      .get(idx)
      .and_then(|field| field.parse::<f64>().ok())
      .map(|f| f <= val)
      .unwrap_or(false)
  }))
}

pub fn between(
  column: Arc<str>,
  value: Arc<str>,
  headers: Arc<Vec<String>>,
) -> Result<Box<dyn Fn(&StringRecord) -> bool + Send + Sync>> {
  let idx = headers
    .iter()
    .position(|h| h == column.as_ref())
    .ok_or_else(|| anyhow!("Column not found: {}", column))?;
  let values: Vec<Arc<str>> = value
    .split('|')
    .map(|s| s.trim())
    .filter(|s| !s.is_empty())
    .collect::<HashSet<_>>()
    .into_iter()
    .map(Arc::from)
    .collect();
  if values.len() != 2 {
    return Err(anyhow!(
      "Between value must have two values separated by vertical line"
    ));
  }
  let min = values[0].clone();
  let max = values[1].clone();
  Ok(Box::new(move |record: &StringRecord| {
    record
      .get(idx)
      .map_or(false, |f| f >= min.as_ref() && f <= max.as_ref())
  }))
}
