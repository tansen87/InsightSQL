use std::{collections::HashSet, fs::File, path::Path, time::Instant};

use anyhow::{Result, anyhow};
use smallvec::SmallVec;
use tauri::AppHandle;

use crate::{
  cmd::search::{filters, filters_multi},
  index::Indexed,
  io::csv::{config::CsvConfigBuilder, options::CsvOptions},
  utils::EventEmitter,
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

async fn perform_search<P: AsRef<Path> + Send + Sync + 'static>(
  path: P,
  column: String,
  conditions: String,
  mode: &str,
  progress: bool,
  quoting: bool,
  flexible: bool,
  skiprows: usize,
  threads: Option<usize>,
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
      filters_multi::equal_multi(
        path, column, conditions, skiprows, quoting, progress, emitter,
      )
      .await
    }
    SearchMode::StartsWithMulti(conditions) => {
      filters_multi::starts_with_multi(
        path, column, conditions, skiprows, quoting, progress, emitter,
      )
      .await
    }
    SearchMode::ContainsMulti(conditions) => {
      filters_multi::contains_multi(
        path, column, conditions, skiprows, quoting, progress, emitter,
      )
      .await
    }
    SearchMode::EndsWithMulti(conditions) => {
      filters_multi::ends_with_multi(
        path, column, conditions, skiprows, quoting, progress, emitter,
      )
      .await
    }
    _ => {
      let vec_conditions = multi_conditions.to_vec();
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

      let mut idx: Option<Indexed<File, File>> = None;

      if let Some(threads) = threads {
        if threads <= 1 {
          // 单线程: emit_total_rows
          let total_rows = if progress {
            opts.idx_count_rows().await?
          } else {
            0
          };
          emitter.emit_total_rows(total_rows).await?;
        } else {
          // 多线程: 初始化idx供后续使用
          idx = Some(
            opts
              .indexed()?
              .ok_or_else(|| anyhow!("No indexed file, create index first"))?,
          );
        }
      }

      match search_mode {
        SearchMode::Equal => {
          filters::equal(
            rdr,
            wtr,
            opts,
            idx,
            column,
            vec_conditions,
            progress,
            threads,
            emitter,
          )
          .await
        }
        SearchMode::NotEqual => {
          filters::not_equal(
            rdr,
            wtr,
            opts,
            idx,
            column,
            vec_conditions,
            progress,
            threads,
            emitter,
          )
          .await
        }
        SearchMode::Contains => {
          filters::contains(
            rdr,
            wtr,
            opts,
            idx,
            column,
            vec_conditions,
            progress,
            threads,
            emitter,
          )
          .await
        }
        SearchMode::NotContains => {
          filters::not_contains(
            rdr,
            wtr,
            opts,
            idx,
            column,
            vec_conditions,
            progress,
            threads,
            emitter,
          )
          .await
        }
        SearchMode::StartsWith => {
          filters::starts_with(
            rdr,
            wtr,
            opts,
            idx,
            column,
            vec_conditions,
            progress,
            threads,
            emitter,
          )
          .await
        }
        SearchMode::NotStartsWith => {
          filters::not_starts_with(
            rdr,
            wtr,
            opts,
            idx,
            column,
            vec_conditions,
            progress,
            threads,
            emitter,
          )
          .await
        }
        SearchMode::EndsWith => {
          filters::ends_with(
            rdr,
            wtr,
            opts,
            idx,
            column,
            vec_conditions,
            progress,
            threads,
            emitter,
          )
          .await
        }
        SearchMode::NotEndsWith => {
          filters::not_ends_with(
            rdr,
            wtr,
            opts,
            idx,
            column,
            vec_conditions,
            progress,
            threads,
            emitter,
          )
          .await
        }
        SearchMode::Regex => {
          filters::regex_search(
            rdr, wtr, opts, idx, column, conditions, progress, threads, emitter,
          )
          .await
        }
        SearchMode::IsNull => {
          filters::is_null(
            rdr,
            wtr,
            opts,
            idx,
            column,
            vec![],
            progress,
            threads,
            emitter,
          )
          .await
        }
        SearchMode::IsNotNull => {
          filters::is_not_null(
            rdr,
            wtr,
            opts,
            idx,
            column,
            vec![],
            progress,
            threads,
            emitter,
          )
          .await
        }
        SearchMode::GreaterThan => {
          filters::greater_than(
            rdr, wtr, opts, idx, column, conditions, progress, threads, emitter,
          )
          .await
        }
        SearchMode::GreaterThanEqual => {
          filters::greater_than_or_equal(
            rdr, wtr, opts, idx, column, conditions, progress, threads, emitter,
          )
          .await
        }
        SearchMode::LessThan => {
          filters::less_than(
            rdr, wtr, opts, idx, column, conditions, progress, threads, emitter,
          )
          .await
        }
        SearchMode::LessThanEqual => {
          filters::less_than_or_equal(
            rdr, wtr, opts, idx, column, conditions, progress, threads, emitter,
          )
          .await
        }
        SearchMode::Between => {
          filters::between(
            rdr,
            wtr,
            opts,
            idx,
            column,
            vec_conditions,
            progress,
            threads,
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
  flexible: bool,
  skiprows: usize,
  threads: usize,
  app_handle: AppHandle,
) -> Result<(String, String), String> {
  let start_time = Instant::now();

  match perform_search(
    path,
    column,
    condition,
    &mode,
    progress,
    quoting,
    flexible,
    skiprows,
    Some(threads),
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
