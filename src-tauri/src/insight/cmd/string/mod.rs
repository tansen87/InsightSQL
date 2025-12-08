use std::time::Instant;

pub mod pad;
pub mod slice;
pub mod split;

#[tauri::command]
pub async fn str_slice(
  path: String,
  column: String,
  n: String,
  length: String,
  reverse: bool,
  mode: String,
) -> Result<String, String> {
  let start_time = Instant::now();

  let slice_mode: slice::SliceMode = mode.as_str().into();

  match slice::perform_slice(
    path,
    column.as_str(),
    n.parse::<i32>().map_err(|e| e.to_string())?,
    length.parse::<usize>().map_err(|e| e.to_string())?,
    reverse,
    slice_mode,
  )
  .await
  {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("{err}")),
  }
}

#[tauri::command]
pub async fn str_split(
  path: String,
  column: String,
  n: String,
  by: String,
  mode: String,
) -> Result<String, String> {
  let start_time = Instant::now();

  let split_mode: split::SplitMode = mode.as_str().into();

  match split::split(
    path,
    column.as_str(),
    n.parse::<i32>().map_err(|e| e.to_string())?,
    by.as_str(),
    split_mode,
  )
  .await
  {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("{err}")),
  }
}

#[tauri::command]
pub async fn str_pad(
  path: String,
  column: String,
  length: String,
  fill_char: String,
  mode: String,
) -> Result<String, String> {
  let start_time = Instant::now();

  match pad::pad(path, &column, length, &fill_char, &mode).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("{err}")),
  }
}
