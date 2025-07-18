use std::{
  collections::{BTreeMap, HashSet},
  fs::File,
  io::BufWriter,
  path::{Path, PathBuf},
  sync::{Arc, Mutex},
  time::{Duration, Instant},
};

use anyhow::Result;
use csv::{ByteRecord, ReaderBuilder, WriterBuilder};
use tauri::{Emitter, Window};
use tokio::sync::oneshot;

use crate::io::csv::options::CsvOptions;

#[derive(Debug, Clone, Copy)]
pub enum SelectMode {
  Include,
  Exclude,
}

impl From<&str> for SelectMode {
  fn from(mode: &str) -> Self {
    match mode {
      "include" => SelectMode::Include,
      "exclude" => SelectMode::Exclude,
      _ => SelectMode::Include,
    }
  }
}

pub async fn select_columns<P: AsRef<Path> + Send + Sync>(
  path: P,
  sel_cols: String,
  sel_mode: SelectMode,
  pgs_mode: &str,
  window: Window,
) -> Result<()> {
  let csv_options = CsvOptions::new(&path);
  let sep = csv_options.detect_separator()?;
  let parent_path = path.as_ref().parent().unwrap().to_str().unwrap();
  let file_stem = path.as_ref().file_stem().unwrap().to_str().unwrap();
  let mut output_path = PathBuf::from(parent_path);
  output_path.push(format!("{file_stem}.select.csv"));
  let col_names: HashSet<&str> = sel_cols.split('|').collect();

  let total_rows = match pgs_mode {
    "idx" => csv_options.idx_count_rows().await?,
    "std" => csv_options.std_count_rows()?,
    _ => 0,
  };
  window.emit("total-rows", total_rows)?;

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(csv_options.rdr_skip_rows()?);

  let headers: Vec<String> = rdr.headers()?.iter().map(|s| s.to_string()).collect();

  // 构建一个 header -> index 的映射表,便于快速查找
  let header_to_index: BTreeMap<&str, usize> = headers
    .iter()
    .enumerate()
    .map(|(i, h)| (h.as_str(), i))
    .collect();

  let (col_indices, output_headers): (Vec<usize>, Vec<String>) = match sel_mode {
    SelectMode::Include => {
      let mut indices = Vec::new();
      let mut out_headers = Vec::new();
      for col_name in sel_cols.split('|') {
        if let Some(&index) = header_to_index.get(col_name) {
          indices.push(index);
          out_headers.push(col_name.to_string());
        }
      }
      (indices, out_headers)
    }
    SelectMode::Exclude => {
      let mut indices = Vec::new();
      let mut out_headers = Vec::new();
      for (i, header) in headers.iter().enumerate() {
        if !col_names.contains(header.as_str()) {
          indices.push(i);
          out_headers.push(header.clone());
        }
      }
      (indices, out_headers)
    }
  };

  let mut wtr = WriterBuilder::new()
    .delimiter(sep)
    .from_writer(BufWriter::new(File::create(output_path)?));

  wtr.write_record(output_headers.iter())?;

  let mut record = ByteRecord::new();

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
          if let Err(err) = window.emit("update-rows", current_rows) {
            eprintln!("Failed to emit current rows: {err:?}");
          }
        },
        Ok(final_rows) = (&mut done_rx) => {
          if let Err(err) = window.emit("update-rows", final_rows) {
            eprintln!("Failed to emit final rows: {err:?}");
          }
          break;
        },
        _ = (&mut stop_rx) => { break; }
      }
    }
  });

  let counter_task = tokio::task::spawn_blocking(move || {
    while rdr.read_byte_record(&mut record)? {
      let selected_data: Vec<&[u8]> = col_indices
        .iter()
        .map(|&i| {
          if i < record.len() {
            &record[i]
          } else {
            "".as_bytes()
          }
        })
        .collect();
      wtr.write_record(selected_data.iter())?;

      let mut cnt = rows.lock().unwrap();
      *cnt += 1;
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
pub async fn select(
  path: String,
  sel_cols: String,
  sel_mode: String,
  pgs_mode: String,
  window: Window,
) -> Result<String, String> {
  let start_time = Instant::now();

  let sel_mode: SelectMode = sel_mode.as_str().into();

  match select_columns(path, sel_cols, sel_mode, &pgs_mode, window).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("{err}")),
  }
}
