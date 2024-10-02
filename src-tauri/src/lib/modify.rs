use std::{error::Error, fs, path::Path, time::Instant};


fn modify_filenames(file_path: String, file_name: String) -> Result<(), Box<dyn Error>> {
    let vec_path: Vec<&str> = file_path.split('|').collect();
    let vec_name: Vec<&str> = file_name.split('|').collect();
    let parent_path = Path::new(vec_path[0])
    .parent()
    .map(|parent| parent.to_string_lossy()).unwrap();

    for (idx, fp) in vec_path.iter().enumerate() {
        let modify_path = format!("{parent_path}/{}", vec_name[idx]);
        fs::rename(fp, modify_path)?;
    }

    Ok(())
}

#[tauri::command]
pub async fn modify(file_path: String, file_name: String, window: tauri::Window) {
  let start_time = Instant::now();

  match (async { modify_filenames(file_path, file_name) }).await {
    Ok(result) => result,
    Err(err) => {
      eprintln!("modify error: {err}");
      window.emit("modify_err", &err.to_string()).unwrap();
    }
  }

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  let runtime = format!("{elapsed_time:.2} s");
  window.emit("runtime", runtime).unwrap();
}