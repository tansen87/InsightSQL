use std::path::Path;

use anyhow::Result;

use crate::{cmd::search::generic::generic_multi_search, utils::EventEmitter};

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
