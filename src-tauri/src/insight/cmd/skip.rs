use std::{
  fmt::Display,
  fs::File,
  io::{BufRead, BufReader, BufWriter, Write},
  path::Path,
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
  time::{Duration, Instant},
};

use anyhow::{Result, anyhow};
use tauri::{AppHandle, Emitter};
use tokio::sync::oneshot;

use crate::{
  io::csv::options::CsvOptions,
  utils::{self, EventEmitter, RDR_BUFFER_SIZE, WTR_BUFFER_SIZE},
};

pub async fn skip_csv<E, F, P>(
  path: P,
  file_name: F,
  skip_rows: usize,
  progress: bool,
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

  let rdr = BufReader::with_capacity(RDR_BUFFER_SIZE, File::open(&path)?);

  let opts = CsvOptions::new(&path);
  let output_path = opts.output_path(Some("skip"), None)?;
  let mut wtr = BufWriter::with_capacity(WTR_BUFFER_SIZE, File::create(output_path)?);

  let total_rows = match progress {
    true => opts.count_lines()?.saturating_sub(skip_rows) + 1,
    false => 0,
  };
  emitter
    .emit_total_msg(&format!("{file_name}|{total_rows}"))
    .await?;

  let mut lines = rdr.lines();

  for _ in 0..skip_rows {
    let _ = lines.next();
  }

  let rows = Arc::new(AtomicUsize::new(0));
  // 创建一次性通道用于控制定时器结束
  let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
  let (done_tx, mut done_rx) = oneshot::channel::<usize>();

  // 启动定时器任务
  let timer_task = if progress {
    let rows_clone = Arc::clone(&rows);

    Some(tokio::spawn(async move {
      let mut interval = tokio::time::interval(Duration::from_millis(500));
      loop {
        tokio::select! {
          _ = interval.tick() => {
            let current_rows = rows_clone.load(Ordering::Relaxed);
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
    }))
  } else {
    None
  };

  let counter_task = tokio::task::spawn_blocking(move || {
    for line in lines {
      writeln!(wtr, "{}", line?)?;

      rows.fetch_add(1, Ordering::Relaxed);
    }

    // 发送最终行数
    let final_rows = rows.load(Ordering::Relaxed);
    let _ = done_tx.send(final_rows);
    Ok::<_, anyhow::Error>(wtr.flush()?)
  });

  counter_task.await??;
  let _ = stop_tx.send(());
  if let Some(task) = timer_task {
    task.await?;
  }

  Ok(())
}

#[tauri::command]
pub async fn skip(
  path: String,
  skip_rows: String,
  progress: bool,
  emitter: AppHandle,
) -> Result<String, String> {
  let start_time = Instant::now();

  let paths: Vec<&str> = path.split('|').collect();
  let skip_rows = utils::parse_usize(&skip_rows, "skiprows")?;

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
      progress,
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
