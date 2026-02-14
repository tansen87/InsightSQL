use std::path::Path;

use anyhow::{Result, anyhow};

use crate::{
  cmd::search::{generic, perform::ColumnConfig},
  io::csv::{config::CsvConfigBuilder, options::CsvOptions},
  utils::EventEmitter,
};

pub async fn search_with_chain<E, P>(
  path: P,
  configs: Vec<ColumnConfig>,
  logics: Vec<String>,
  skiprows: usize,
  quoting: bool,
  flexible: bool,
  progress: bool,
  emitter: E,
) -> Result<String>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync,
{
  if configs.is_empty() {
    return Err(anyhow!("No filters added"));
  }
  if logics.len() != configs.len().saturating_sub(1) {
    return Err(anyhow!("logics length must be configs.len() - 1"));
  }

  let columns: Vec<String> = configs.iter().map(|c| c.column.clone()).collect();

  let mut opts = CsvOptions::new(path.as_ref().to_string_lossy().to_string());
  opts.set_skiprows(skiprows);
  let (sep, reader) = opts.skiprows_and_delimiter()?;
  let output_path = opts.output_path(Some("search"), None)?;
  let config = CsvConfigBuilder::new()
    .flexible(flexible)
    .delimiter(sep)
    .quoting(quoting)
    .build();
  let rdr = config.build_reader(reader);
  let wtr = config.build_writer(&output_path)?;

  let total_rows = if progress {
    opts.idx_count_rows().await?
  } else {
    0
  };
  emitter.emit_total_rows(total_rows).await?;

  // 预解析每个 condition
  let parsed_configs: Vec<(String, Vec<String>)> = configs
    .into_iter()
    .map(|cfg| {
      let sub_conds = cfg
        .condition
        .split('|')
        .map(|s| s.trim().to_string())
        .collect::<Vec<_>>();
      (cfg.mode, sub_conds)
    })
    .collect();

  // 构造 match_fn
  let match_fn = move |values: &[&str]| -> bool {
    // 每列独立判断
    let col_results: Vec<bool> = values
      .iter()
      .enumerate()
      .map(|(i, &value)| {
        let (mode, sub_conds) = &parsed_configs[i];
        match mode.as_str() {
          "equal" => sub_conds.iter().any(|c| value == c),
          "not_equal" => sub_conds.iter().all(|c| value != c),
          "contains" => sub_conds.iter().any(|c| value.contains(c)),
          "not_contains" => sub_conds.iter().all(|c| !value.contains(c)),
          "starts_with" => sub_conds.iter().any(|c| value.starts_with(c)),
          "not_starts_with" => sub_conds.iter().all(|c| !value.starts_with(c)),
          "ends_with" => sub_conds.iter().any(|c| value.ends_with(c)),
          "not_ends_with" => sub_conds.iter().all(|c| !value.ends_with(c)),
          "regex" => sub_conds.iter().any(|pattern| {
            regex::Regex::new(pattern)
              .ok()
              .map_or(false, |re| re.is_match(value))
          }),
          "is_null" => value.is_empty(),
          "is_not_null" => !value.is_empty(),
          "gt" => {
            if let (Some(threshold), Ok(val)) = (sub_conds.get(0), value.parse::<f64>()) {
              if let Ok(t) = threshold.parse::<f64>() {
                val > t
              } else {
                false
              }
            } else {
              false
            }
          }
          "ge" => {
            if let (Some(threshold), Ok(val)) = (sub_conds.get(0), value.parse::<f64>()) {
              if let Ok(t) = threshold.parse::<f64>() {
                val >= t
              } else {
                false
              }
            } else {
              false
            }
          }
          "lt" => {
            if let (Some(threshold), Ok(val)) = (sub_conds.get(0), value.parse::<f64>()) {
              if let Ok(t) = threshold.parse::<f64>() {
                val < t
              } else {
                false
              }
            } else {
              false
            }
          }
          "le" => {
            if let (Some(threshold), Ok(val)) = (sub_conds.get(0), value.parse::<f64>()) {
              if let Ok(t) = threshold.parse::<f64>() {
                val <= t
              } else {
                false
              }
            } else {
              false
            }
          }
          "between" => {
            if sub_conds.len() == 2 {
              if let (Ok(low), Ok(high)) =
                (sub_conds[0].parse::<f64>(), sub_conds[1].parse::<f64>())
              {
                if let Ok(val) = value.parse::<f64>() {
                  return val >= low && val <= high;
                }
              }
            }
            false
          }
          _ => false,
        }
      })
      .collect();

    // 链式组合
    let mut result = col_results[0];
    for i in 1..col_results.len() {
      match logics[i - 1].as_str() {
        "and" => result = result && col_results[i],
        "or" => result = result || col_results[i],
        _ => {} // fallback to AND
      }
    }
    result
  };

  let match_count =
    generic::generic_search_chain(rdr, wtr, columns, progress, match_fn, emitter).await?;

  Ok(match_count)
}
