use std::{
  fs::File,
  io::BufWriter,
  path::{Path, PathBuf},
  sync::{Arc, Mutex},
  time::{Duration, Instant},
};

use anyhow::Result;
use csv::{ByteRecord, ReaderBuilder, WriterBuilder};
use pinyin::ToPinyin;
use tauri::{AppHandle, Emitter};
use tokio::sync::oneshot;

use crate::io::csv::{options::CsvOptions, selection::Selection};

enum PinyinStyle {
  Upper,
  Lower,
  None,
}

pub async fn chinese_to_pinyin<P: AsRef<Path> + Send + Sync>(
  path: P,
  columns: String,
  mode: &str,
  pinyin_style: &str,
  app_handle: AppHandle,
) -> Result<()> {
  let csv_options = CsvOptions::new(&path);
  let sep = csv_options.detect_separator()?;
  let parent_path = path.as_ref().parent().unwrap().to_str().unwrap();
  let file_stem = path.as_ref().file_stem().unwrap().to_str().unwrap();
  let mut output_path = PathBuf::from(parent_path);
  output_path.push(format!("{file_stem}.pinyin.csv"));

  let total_rows = match mode {
    "idx" => csv_options.idx_count_rows().await?,
    "std" => csv_options.std_count_rows()?,
    _ => 0,
  };
  app_handle.emit("total-rows", total_rows)?;

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(csv_options.rdr_skip_rows()?);

  let cols: Vec<&str> = columns.split('|').collect();
  let sel = Selection::from_headers(rdr.byte_headers()?, &cols[..])?;

  let buf_writer = BufWriter::with_capacity(256_000, File::create(output_path)?);
  let mut wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_writer);

  wtr.write_record(rdr.headers()?)?;

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

  let style = match pinyin_style {
    "upper" => PinyinStyle::Upper,
    "lower" => PinyinStyle::Lower,
    _ => PinyinStyle::None,
  };

  let counter_task = tokio::task::spawn_blocking(move || {
    let mut record = ByteRecord::new();
    while rdr.read_byte_record(&mut record)? {
      let mut new_record = Vec::new();
      for (i, field) in record.iter().enumerate() {
        let mut new_field = String::from_utf8_lossy(field).to_string();

        if sel.get_indices().contains(&i) {
          new_field = new_field
            .chars()
            .map(|c| {
              c.to_pinyin().map_or_else(
                || c.into(),
                |py| {
                  let s = py.plain().to_string();
                  match style {
                    PinyinStyle::Upper => s.to_uppercase(),
                    PinyinStyle::Lower => s.to_lowercase(),
                    PinyinStyle::None => s,
                  }
                },
              )
            })
            .collect::<String>();
        }

        new_record.push(new_field);
      }

      wtr.write_record(&new_record)?;
      *rows.lock().unwrap() += 1;
      record.clear();
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
pub async fn pinyin(
  path: String,
  columns: String,
  mode: String,
  pinyin_style: String,
  app_handle: AppHandle,
) -> Result<String, String> {
  let start_time = Instant::now();

  match chinese_to_pinyin(
    path,
    columns,
    mode.as_str(),
    pinyin_style.as_str(),
    app_handle,
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
