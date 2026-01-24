use std::{
  fs::File,
  io::{BufReader, BufWriter},
  path::Path,
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
  time::Duration,
};

use anyhow::{Result, anyhow};
use csv::{Reader, ReaderBuilder, Writer, WriterBuilder};
use tokio::sync::oneshot;

use crate::{
  io::csv::{options::CsvOptions, selection::Selection},
  utils::EventEmitter,
};

#[derive(Debug)]
pub enum SplitMode {
  Nth,
  Max,
}

impl From<&str> for SplitMode {
  fn from(mode: &str) -> Self {
    match mode {
      "split_n" => SplitMode::Nth,
      "split_max" => SplitMode::Max,
      _ => SplitMode::Nth,
    }
  }
}

pub async fn split_n<E>(
  mut rdr: Reader<BufReader<File>>,
  mut wtr: Writer<BufWriter<File>>,
  column: &str,
  n: usize,
  by: String,
  emitter: E,
) -> Result<()>
where
  E: EventEmitter + Send + Sync + 'static,
{
  let mut headers = rdr.headers()?.clone();

  let sel = Selection::from_headers(rdr.byte_headers()?, &[column][..])?;

  let new_column_name = format!("{}_nth", column);
  headers.push_field(&new_column_name);
  wtr.write_record(&headers)?;

  let rows = Arc::new(AtomicUsize::new(0));
  let rows_clone = Arc::clone(&rows);
  let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
  let (done_tx, mut done_rx) = oneshot::channel::<usize>();

  let timer_task = tokio::spawn(async move {
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
  });

  let counter_task = tokio::task::spawn_blocking(move || {
    for result in rdr.records() {
      let record = result?;
      if let Some(value) = record.get(sel.first_indices()?) {
        let split_parts: Vec<&str> = value.split(by.as_str()).collect();
        let selected_part = if split_parts.len() >= n {
          split_parts[n - 1]
        } else {
          ""
        };

        let mut new_record = record.clone();
        new_record.push_field(selected_part);
        wtr.write_record(&new_record)?;
      }

      rows.fetch_add(1, Ordering::Relaxed);
    }

    let final_rows = rows.load(Ordering::Relaxed);
    let _ = done_tx.send(final_rows);
    Ok::<_, anyhow::Error>(wtr.flush()?)
  });

  counter_task.await??;
  let _ = stop_tx.send(());
  timer_task.await?;

  Ok(())
}

pub async fn split_max<E>(
  mut rdr: Reader<BufReader<File>>,
  mut wtr: Writer<BufWriter<File>>,
  column: String,
  n: usize,
  by: String,
  emitter: E,
) -> Result<()>
where
  E: EventEmitter + Send + Sync + 'static,
{
  let mut headers = rdr.headers()?.clone();

  let sel = Selection::from_headers(rdr.byte_headers()?, &[column.as_str()][..])?;

  let rows = Arc::new(AtomicUsize::new(0));
  let rows_clone = Arc::clone(&rows);
  let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
  let (done_tx, mut done_rx) = oneshot::channel::<usize>();

  let timer_task = tokio::spawn(async move {
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
  });

  let counter_task = tokio::task::spawn_blocking(move || {
    let mut first_record = true;
    for result in rdr.records() {
      let record = result?;
      if let Some(value) = record.get(sel.first_indices()?) {
        let split_parts: Vec<&str> = value.split(&by).collect();
        if first_record {
          for i in 1..=n {
            headers.push_field(&format!("{}_max{}", column, i));
          }
          wtr.write_record(&headers)?;
          first_record = false;
        }

        let mut new_record = record.clone();
        for i in 0..n {
          if i < split_parts.len() {
            new_record.push_field(split_parts[i]);
          } else {
            new_record.push_field("");
          }
        }

        wtr.write_record(&new_record)?;
      }

      rows.fetch_add(1, Ordering::Relaxed);
    }

    let final_rows = rows.load(Ordering::Relaxed);
    let _ = done_tx.send(final_rows);
    Ok::<_, anyhow::Error>(wtr.flush()?)
  });

  counter_task.await??;
  let _ = stop_tx.send(());
  timer_task.await?;

  Ok(())
}

pub async fn split<E, P>(
  path: P,
  column: String,
  n: i32,
  by: String,
  mode: SplitMode,
  quoting: bool,
  progress: bool,
  emitter: E,
) -> Result<()>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync,
{
  let num = n as usize;
  if n < 1 {
    return Err(anyhow!(
      "Number of the split must be greater than or equal 1"
    ));
  }
  if by.chars().count() != 1 {
    return Err(anyhow!("by must be a single character"));
  }

  let opts = CsvOptions::new(&path);
  let sep = opts.detect_separator()?;
  let output_path = opts.output_path(Some("split"), None)?;

  let total_rows = match progress {
    true => opts.idx_count_rows().await?,
    false => 0,
  };
  emitter.emit_total_rows(total_rows).await?;

  let rdr = ReaderBuilder::new()
    .delimiter(sep)
    .quoting(quoting)
    .from_reader(opts.rdr_skip_rows()?);

  let buf_writer = BufWriter::with_capacity(256_000, File::create(output_path)?);
  let wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_writer);

  match mode {
    SplitMode::Nth => split_n(rdr, wtr, &column, num, by, emitter).await?,
    SplitMode::Max => split_max(rdr, wtr, column, num, by, emitter).await?,
  }

  Ok(())
}
