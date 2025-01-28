use std::{fs, path::Path, time::Instant};

use anyhow::Result;

async fn modify_filenames(path: String, file_name: String) -> Result<()> {
  let paths: Vec<&str> = path.split('|').collect();
  let vec_name: Vec<&str> = file_name.split('|').collect();
  let parent_path = Path::new(paths[0])
    .parent()
    .map(|parent| parent.to_string_lossy())
    .unwrap();

  for (idx, fp) in paths.iter().enumerate() {
    let modify_path = format!("{parent_path}/{}", vec_name[idx]);
    fs::rename(fp, modify_path)?;
  }

  Ok(())
}

#[tauri::command]
pub async fn modify(path: String, file_name: String) -> Result<String, String> {
  let start_time = Instant::now();

  match modify_filenames(path, file_name).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("modify failed: {err}")),
  }
}
