use std::{
  fs::File,
  io::BufWriter,
  path::{Path, PathBuf},
  sync::{Arc, Mutex},
  time::{Duration, Instant},
};

use anyhow::Result;
use csv::{ByteRecord, ReaderBuilder, WriterBuilder};
use tauri::{AppHandle, Emitter};
use tokio::sync::oneshot;

use crate::utils::CsvOptions;

pub async fn enumerate_index<P: AsRef<Path> + Send + Sync>(
  path: P,
  mode: &str,
  app_handle: AppHandle,
) -> Result<()> {
  let csv_options = CsvOptions::new(&path);
  let sep = csv_options.detect_separator()?;
  let parent_path = path.as_ref().parent().unwrap().to_str().unwrap();
  let file_stem = path.as_ref().file_stem().unwrap().to_str().unwrap();
  let mut output_path = PathBuf::from(parent_path);
  output_path.push(format!("{file_stem}.enumer.csv"));

  let total_rows = match mode {
    "idx" => csv_options.idx_csv_rows().await?,
    "std" => csv_options.std_csv_rows()?,
    _ => 0,
  };
  app_handle.emit("total-rows", total_rows)?;

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(csv_options.skip_csv_rows()?);

  let buf_writer = BufWriter::with_capacity(256_000, File::create(output_path)?);
  let mut wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_writer);

  let headers = rdr.headers()?;
  let mut new_headers = vec![String::from("enumerate_idx")];
  new_headers.extend(headers.into_iter().map(String::from));
  wtr.write_record(&new_headers)?;

  let rows = Arc::new(Mutex::new(0));
  let rows_clone = Arc::clone(&rows);

  let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
  let (done_tx, mut done_rx) = oneshot::channel::<usize>();

  let timer_task = tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_millis(500));
    loop {
      tokio::select! {
        _ = interval.tick() => {
          let current_rows = *rows_clone.lock().unwrap();
          if let Err(err) = app_handle.emit("update-rows", current_rows) {
            eprintln!("Failed to emit current rows: {err:?}");
          }
        },
        Ok(final_rows) = (&mut done_rx) => {
          if let Err(err) = app_handle.emit("update-rows", final_rows) {
            eprintln!("Failed to emit final rows: {err:?}");
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
      let i = {
        let mut count = rows.lock().unwrap();
        *count += 1;
        *count
      };

      let mut new_record = vec![i.to_string()];
      new_record.extend(
        record
          .iter()
          .map(|field| String::from_utf8_lossy(field).into_owned()),
      );
      wtr.write_record(&new_record)?;
    }

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
pub async fn enumer(path: String, mode: String, app_handle: AppHandle) -> Result<String, String> {
  let start_time = Instant::now();

  match enumerate_index(path, mode.as_str(), app_handle).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("{err}")),
  }
}
