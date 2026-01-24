use std::{
  fmt::Display,
  fs::File,
  io::BufWriter,
  path::Path,
  sync::{Arc, Mutex},
  time::{Duration, Instant},
};

use anyhow::{Result, anyhow};
use csv::{ByteRecord, ReaderBuilder, WriterBuilder};
use tauri::{AppHandle, Emitter};
use tokio::sync::oneshot;

use crate::{
  io::csv::options::CsvOptions,
  utils::{EventEmitter, WTR_BUFFER_SIZE},
};

pub async fn skip_csv<E, F, P>(
  path: P,
  file_name: F,
  skip_rows: usize,
  progress: &str,
  quoting: bool,
  emitter: E,
) -> Result<()>
where
  E: EventEmitter + Send + Sync + 'static,
  F: AsRef<str> + Send + Sync + Display + 'static,
  P: AsRef<Path> + Send + Sync,
{
  if skip_rows < 1 {
    return Err(anyhow!("The skip rows must be greater than or equal to 1"));
  }

  let mut opts = CsvOptions::new(&path);
  opts.set_skip_rows(skip_rows);
  let sep = opts.detect_separator()?;
  let output_path = opts.output_path(Some("skip"), None)?;

  let total_rows = match progress {
    "idx" => opts.idx_count_rows().await?.saturating_sub(skip_rows) + 1,
    "std" => opts.std_count_rows()?.saturating_sub(skip_rows) + 1,
    _ => 0,
  };
  emitter
    .emit_total_msg(&format!("{file_name}|{total_rows}"))
    .await?;

  let mut rdr = ReaderBuilder::new()
    .has_headers(false)
    .delimiter(sep)
    .quoting(quoting)
    .from_reader(opts.rdr_skip_rows()?);

  let output_file = File::create(output_path)?;
  let buf_wtr = BufWriter::with_capacity(WTR_BUFFER_SIZE, output_file);
  let mut wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_wtr);

  // 创建一个Arc<Mutex<usize>>用于跨线程安全地共享rows计数
  let rows = Arc::new(Mutex::new(0));
  let rows_clone = Arc::clone(&rows);
  // 创建一次性通道用于控制定时器结束
  let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
  let (done_tx, mut done_rx) = oneshot::channel::<usize>();

  // 启动定时器任务
  let timer_task = tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_millis(500));
    loop {
      tokio::select! {
        _ = interval.tick() => {
          let current_rows = match rows_clone.lock() {
            Ok(lock) => *lock,
            Err(err) => {
              eprintln!("Failed to lock current rows: {err}");
              0
            }
          };
          if let Err(err) = emitter.emit_update_msg(&format!("{file_name}|{current_rows}")).await {
            let _ = emitter.emit_err(&format!("{file_name}|{err}")).await;
          }
        },
        Ok(final_rows) = (&mut done_rx) => {
          if let Err(err) = emitter.emit_update_msg(&format!("{file_name}|{final_rows}")).await {
            let _ = emitter.emit_err(&format!("{file_name}|{err}")).await;
          }
          break;
        },
        _ = (&mut stop_rx) => { break; }
      }
    }
  });

  let counter_task = tokio::task::spawn_blocking(move || {
    let mut record = ByteRecord::new();
    while rdr.read_byte_record(&mut record)? {
      wtr.write_byte_record(&record)?;

      let mut cnt = rows
        .lock()
        .map_err(|poison| anyhow!("cnt lock poisoned: {poison}"))?;
      *cnt += 1;
    }

    // 发送最终行数
    let final_rows = *rows
      .lock()
      .map_err(|poison| anyhow!("final rows lock poisoned: {poison}"))?;
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
  progress: String,
  quoting: bool,
  emitter: AppHandle,
) -> Result<String, String> {
  let start_time = Instant::now();

  let paths: Vec<&str> = path.split('|').collect();
  let skip_rows = skip_rows
    .parse::<usize>()
    .map_err(|e| format!("parse skip rows error: {e}"))?;

  for fp in paths.iter() {
    let file_name = Path::new(fp)
      .file_name()
      .ok_or(format!("get file_name failed"))?
      .to_str()
      .ok_or(format!("file_name to str failed"))?;
    emitter
      .emit("info", file_name)
      .map_err(|e| format!("emit info error: {e}"))?;
    match skip_csv(
      fp,
      file_name.to_string(),
      skip_rows,
      &progress,
      quoting,
      emitter.clone(),
    )
    .await
    {
      Ok(_) => {
        emitter
          .emit("success", file_name)
          .map_err(|e| format!("emit success error: {e}"))?;
      }
      Err(err) => {
        emitter
          .emit("err", format!("{file_name}|{err}"))
          .map_err(|e| format!("emit err error: {e}"))?;
        continue;
      }
    }
  }

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  Ok(format!("{:.2}", elapsed_time))
}
