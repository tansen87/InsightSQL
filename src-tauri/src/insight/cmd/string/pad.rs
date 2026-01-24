use std::{
  fs::File,
  io::BufWriter,
  path::Path,
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
  time::Duration,
};

use anyhow::{Result, anyhow};
use csv::{ReaderBuilder, WriterBuilder};
use tokio::sync::oneshot;

use crate::{
  io::csv::{options::CsvOptions, selection::Selection},
  utils::EventEmitter,
};

#[derive(Debug)]
pub enum PadMode {
  Left,
  Right,
  Both,
}

impl From<&str> for PadMode {
  fn from(mode: &str) -> Self {
    match mode {
      "pad_left" => PadMode::Left,
      "pad_right" => PadMode::Right,
      "pad_both" => PadMode::Both,
      _ => PadMode::Left,
    }
  }
}

fn pad_string(cell: &str, length: usize, fill_char: &str, mode: PadMode) -> Result<String> {
  if fill_char.chars().count() != 1 {
    return Err(anyhow!("fill char must be a single character"));
  }

  let cell_len = cell.chars().count();
  if cell_len >= length {
    return Ok(cell.to_string());
  }

  let total_pad = length - cell_len;
  match mode {
    PadMode::Left => {
      let pad = fill_char.repeat(total_pad);
      Ok(format!("{pad}{cell}"))
    }
    PadMode::Right => {
      let pad = fill_char.repeat(total_pad);
      Ok(format!("{cell}{pad}"))
    }
    PadMode::Both => {
      let left = total_pad / 2;
      let right = total_pad - left;
      let left_pad = fill_char.repeat(left);
      let right_pad = fill_char.repeat(right);
      Ok(format!("{left_pad}{cell}{right_pad}"))
    }
  }
}

pub async fn pad<E, P>(
  path: P,
  column: &str,
  length: String,
  fill_char: String,
  mode: String,
  quoting: bool,
  progress: bool,
  emitter: E,
) -> Result<()>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync,
{
  let opts = CsvOptions::new(&path);
  let sep = opts.detect_separator()?;
  let output_path = opts.output_path(Some("pad"), None)?;
  let length = length.parse::<usize>()?;

  let total_rows = match progress {
    true => opts.idx_count_rows().await?,
    false => 0,
  };
  emitter.emit_total_rows(total_rows).await?;

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .quoting(quoting)
    .from_reader(opts.rdr_skip_rows()?);
  let sel = Selection::from_headers(rdr.byte_headers()?, &[column][..])?;

  let buf_writer = BufWriter::with_capacity(256_000, File::create(output_path)?);
  let mut wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_writer);
  wtr.write_record(rdr.headers()?)?;

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
      let mut row_fields: Vec<String> = record.iter().map(|s| s.to_string()).collect();
      let idx = sel.first_indices()?;
      let cell = &row_fields[idx];
      let pad_cell = pad_string(cell, length, &fill_char, mode.as_str().into())?;
      row_fields[idx] = pad_cell;
      wtr.write_record(&row_fields)?;

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
