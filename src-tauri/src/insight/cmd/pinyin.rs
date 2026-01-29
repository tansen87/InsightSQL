use std::{
  fs::File,
  io::BufWriter,
  path::Path,
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
  time::{Duration, Instant},
};

use anyhow::Result;
use csv::{ByteRecord, ReaderBuilder, WriterBuilder};
use pinyin::ToPinyin;
use tauri::AppHandle;
use tokio::sync::oneshot;

use crate::{
  io::csv::{options::CsvOptions, selection::Selection},
  utils::{EventEmitter, WTR_BUFFER_SIZE},
};

enum PinyinStyle {
  Upper,
  Lower,
  None,
}

pub async fn chinese_to_pinyin<E, P>(
  path: P,
  columns: String,
  progress: bool,
  pinyin_style: &str,
  quoting: bool,
  skiprows: usize,
  emitter: E,
) -> Result<()>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync,
{
  let mut opts = CsvOptions::new(&path);
  opts.set_skiprows(skiprows);
  let (sep, reader) = opts.skiprows_and_delimiter()?;
  let output_path = opts.output_path(Some("pinyin"), None)?;

  let total_rows = match progress {
    true => opts.idx_count_rows().await?,
    false => 0,
  };
  emitter.emit_total_rows(total_rows).await?;

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .quoting(quoting)
    .from_reader(reader);

  let cols: Vec<&str> = columns.split('|').collect();
  let sel = Selection::from_headers(rdr.byte_headers()?, &cols[..])?;

  let output_file = File::create(output_path)?;
  let buf_wtr = BufWriter::with_capacity(WTR_BUFFER_SIZE, output_file);
  let mut wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_wtr);

  wtr.write_record(rdr.headers()?)?;

  let rows = Arc::new(AtomicUsize::new(0));
  let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
  let (done_tx, mut done_rx) = oneshot::channel::<usize>();

  let timer_task = if progress {
    let rows_clone = Arc::clone(&rows);

    Some(tokio::spawn(async move {
      let mut interval = tokio::time::interval(Duration::from_millis(500));
      loop {
        tokio::select! {
          _ = interval.tick() => {
            let current_rows = rows_clone.load(Ordering::Relaxed);
            if let Err(err) = emitter.emit_update_rows(current_rows).await {
              let _ = emitter.emit_err(&format!("failed to emit current rows: {err}")).await;
            }
          },
          Ok(final_rows) = (&mut done_rx) => {
            if let Err(err) = emitter.emit_update_rows(final_rows).await {
              let _ = emitter.emit_err(&format!("failed to emit final rows: {err}")).await;
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

      rows.fetch_add(1, Ordering::Relaxed);
      record.clear();
    }

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
pub async fn pinyin(
  path: String,
  columns: String,
  progress: bool,
  pinyin_style: String,
  quoting: bool,
  skiprows: usize,
  app_handle: AppHandle,
) -> Result<String, String> {
  let start_time = Instant::now();

  match chinese_to_pinyin(
    path,
    columns,
    progress,
    &pinyin_style,
    quoting,
    skiprows,
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
