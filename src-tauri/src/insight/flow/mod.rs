use std::{
  path::{Path, PathBuf},
  time::Instant,
};

use anyhow::Context;

pub mod filter;
pub mod operation;
pub mod process;
pub mod str;
pub mod utils;

#[tauri::command]
pub async fn flow(
  path: String,
  json_config: String,
  quoting: bool,
) -> anyhow::Result<String, String> {
  let start_time = Instant::now();

  let operations: Vec<utils::Operation> =
    serde_json::from_str(&json_config).map_err(|e| e.to_string())?;

  let parent = Path::new(&path)
    .parent()
    .context("Path is null")
    .map_err(|e| e.to_string())?;
  let parent_path = parent.to_str().context("").map_err(|e| e.to_string())?;
  let stem = Path::new(&path)
    .file_stem()
    .context("File stem is null")
    .map_err(|e| e.to_string())?;
  let file_stem = stem.to_str().context("").map_err(|e| e.to_string())?;
  let mut output_path = PathBuf::from(parent_path);
  output_path.push(format!("{file_stem}.flow.csv"));

  match process::process_operations(path, &operations, output_path, quoting).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(err.to_string()),
  }
}
