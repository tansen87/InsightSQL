use std::{
  path::Path,
  sync::{Arc, Mutex},
  time::{Duration, Instant},
};

use anyhow::{Result, anyhow};
use csv::{ByteRecord, ReaderBuilder, WriterBuilder};
use tauri::{AppHandle, Emitter, Window};

use crate::utils::CsvOptions;

pub async fn skip_csv<P: AsRef<Path> + Send + Sync>(
  path: P,
  filename: String,
  skip_rows: usize,
  parent_path: &str,
  mode: &str,
  app_handle: AppHandle,
) -> Result<()> {
  if skip_rows < 1 {
    return Err(anyhow!("The skip rows must be greater than or equal to 1"));
  }

  let mut csv_options = CsvOptions::new(&path);
  csv_options.set_skip_rows(skip_rows);

  let sep = csv_options.detect_separator()?;

  let total_rows = match mode {
    "idx" => csv_options.idx_csv_rows().await? - skip_rows,
    "std" => csv_options.std_csv_rows()? - skip_rows,
    _ => 0,
  };
  app_handle.emit("total-rows", format!("{filename}|{total_rows}"))?;

  let file_stem = path.as_ref().file_stem().unwrap().to_str().unwrap();
  let output_path = format!("{parent_path}/{file_stem}.skiprows.csv");

  let mut rdr = ReaderBuilder::new()
    .has_headers(false)
    .delimiter(sep)
    .from_reader(csv_options.skip_csv_rows()?);

  let mut wtr = WriterBuilder::new().delimiter(sep).from_path(output_path)?;

  // 创建一个Arc<Mutex<usize>>用于跨线程安全地共享rows计数
  let rows = Arc::new(Mutex::new(0));
  let rows_clone = Arc::clone(&rows);

  // 创建一次性通道用于控制定时器结束
  let (stop_tx, mut stop_rx) = tokio::sync::oneshot::channel::<()>();
  let (done_tx, mut done_rx) = tokio::sync::oneshot::channel::<usize>();

  // 启动定时器任务
  let timer_task = tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_millis(500));
    loop {
      tokio::select! {
        _ = interval.tick() => {
          let current_rows = *rows_clone.lock().unwrap();
          let emit_msg = format!("{filename}|{current_rows}");
          if let Err(err) = app_handle.emit("update-rows", emit_msg) {
            eprintln!("Failed to emit update-rows event: {err:?}");
          }
        },
        Ok(final_rows) = (&mut done_rx) => {
          let emit_msg = format!("{filename}|{final_rows}");
          if let Err(err) = app_handle.emit("update-rows", emit_msg) {
            eprintln!("Failed to emit final rows count: {err:?}");
          }
          break;
        },
        _ = (&mut stop_rx) => {
          break;
        }
      }
    }
  });

  let counter_task = tokio::task::spawn_blocking(move || {
    let mut record = ByteRecord::new();
    while rdr.read_byte_record(&mut record)? {
      wtr.write_byte_record(&record)?;
      let mut count = rows.lock().unwrap();
      *count += 1;
    }

    // 发送最终行数
    let final_rows = *rows.lock().unwrap();
    let _ = done_tx.send(final_rows);

    Ok::<_, anyhow::Error>(wtr.flush()?)
  });

  counter_task.await??;
  let _ = stop_tx.send(());
  timer_task.await?;

  Ok(())
}

#[tauri::command]
pub async fn skip(
  path: String,
  skip_rows: String,
  mode: String,
  window: Window,
  app_handle: AppHandle,
) -> Result<String, String> {
  let start_time = Instant::now();

  let paths: Vec<&str> = path.split('|').collect();
  let parent_path = Path::new(&paths[0]).parent().unwrap().to_str().unwrap();
  let skip_rows = skip_rows.parse::<usize>().map_err(|e| e.to_string())?;

  for fp in paths.iter() {
    let filename = Path::new(fp).file_name().unwrap().to_str().unwrap();
    window
      .emit("start_convert", filename)
      .map_err(|e| e.to_string())?;
    match skip_csv(
      fp,
      filename.to_string(),
      skip_rows,
      parent_path,
      mode.as_str(),
      app_handle.clone(),
    )
    .await
    {
      Ok(_) => {
        window
          .emit("skip_msg", filename)
          .map_err(|e| e.to_string())?;
      }
      Err(err) => {
        window
          .emit("skip_err", format!("{filename}|{err}"))
          .map_err(|e| e.to_string())?;
        continue;
      }
    }
  }

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  Ok(format!("{:.2}", elapsed_time))
}
