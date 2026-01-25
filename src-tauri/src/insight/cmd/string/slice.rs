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
  utils::{EventEmitter, RDR_BUFFER_SIZE},
};

#[derive(Debug)]
pub enum SliceMode {
  Left,
  Right,
  Slice,
}

impl From<&str> for SliceMode {
  fn from(mode: &str) -> Self {
    match mode {
      "left" => SliceMode::Left,
      "right" => SliceMode::Right,
      "slice" => SliceMode::Slice,
      _ => SliceMode::Left,
    }
  }
}

impl SliceMode {
  fn to_str(&self) -> &'static str {
    match self {
      SliceMode::Left => "left",
      SliceMode::Right => "right",
      SliceMode::Slice => "slice",
    }
  }
}

pub async fn slice_nchar<E>(
  mut rdr: Reader<BufReader<File>>,
  mut wtr: Writer<BufWriter<File>>,
  column: &str,
  n: usize,
  reverse: bool,
  mode: String,
  progress: bool,
  emitter: E,
) -> Result<()>
where
  E: EventEmitter + Send + Sync + 'static,
{
  let headers = rdr.headers()?.clone();

  let sel = Selection::from_headers(rdr.byte_headers()?, &[column][..])?;

  let mut new_headers = headers.clone();
  let new_column_name = format!("{}_nchar", column);
  new_headers.push_field(&new_column_name);

  wtr.write_record(&new_headers)?;

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

  let counter_task = tokio::task::spawn_blocking(move || {
    for result in rdr.records() {
      let record = result?;

      if let Some(value) = record.get(sel.first_indices()?) {
        let slice_n = {
          let chars: Vec<char> = value.chars().collect();

          let slice = if mode == "left" {
            &chars[..n.min(chars.len())]
          } else {
            // mode == "right"
            let len = chars.len();
            &chars[len.saturating_sub(n)..]
          };

          let mut result: String = slice.iter().collect();

          if reverse {
            result = result.chars().rev().collect();
          }

          result
        };

        let mut new_record = record.clone();
        new_record.push_field(&slice_n);

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
  if let Some(task) = timer_task {
    task.await?;
  }

  Ok(())
}

pub async fn slice(
  mut rdr: Reader<BufReader<File>>,
  mut wtr: Writer<BufWriter<File>>,
  column: &str,
  start_idx: i32,
  length: usize,
  reverse: bool,
) -> Result<()> {
  let headers = rdr.headers()?.clone();
  let sel = Selection::from_headers(rdr.byte_headers()?, &[column][..])?;

  let mut new_headers = headers.clone();
  let new_column_name = format!("{}_slice", column);
  new_headers.push_field(&new_column_name);

  wtr.write_record(&new_headers)?;

  for result in rdr.records() {
    let record = result?;

    if let Some(value) = record.get(sel.first_indices()?) {
      let slice_sl = {
        let chars: Vec<char> = value.chars().collect();

        let (start, is_reversed) = if start_idx > 0 {
          ((start_idx - 1).try_into()?, false)
        } else if start_idx < 0 {
          let start = chars.len().saturating_sub((-start_idx - 1).try_into()?);
          (start, true)
        } else {
          return Err(anyhow!("Number of the slice cannot be equal to 0"));
        };

        // determine the indices of the slice
        let end = start + length;
        let slice = if is_reversed {
          chars
            .iter()
            .rev()
            .skip(chars.len().saturating_sub(end))
            .take(length)
            .cloned()
            .collect::<Vec<char>>()
        } else {
          chars
            .get(start..end)
            .map(|r| r.to_vec())
            .unwrap_or_default()
        };

        let mut result: String = slice.into_iter().collect();

        // warning: 不需要将以下两个if合并到一起
        if start_idx < 0 {
          result = result.chars().rev().collect();
        }
        if reverse {
          result = result.chars().rev().collect();
        }

        result
      };

      let mut new_record = record.clone();
      new_record.push_field(&slice_sl);

      wtr.write_record(&new_record)?;
    }
  }

  Ok(wtr.flush()?)
}

pub async fn perform_slice<E, P>(
  path: P,
  column: &str,
  n: i32,
  length: usize,
  reverse: bool,
  mode: SliceMode,
  quoting: bool,
  progress: bool,
  emitter: E,
) -> Result<()>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync,
{
  let num = n as usize;
  if n < 1 && mode.to_str() != "slice" {
    return Err(anyhow!(
      "Number of the slice must be greater than or equal 1"
    ));
  }
  if n == 0 {
    return Err(anyhow!("Number of the slice cannot be equal to 0"));
  }

  let opts = CsvOptions::new(&path);
  let sep = opts.detect_separator()?;
  let output_path = opts.output_path(Some("slice"), None)?;

  let total_rows = match progress {
    true => opts.idx_count_rows().await?,
    false => 0,
  };
  emitter.emit_total_rows(total_rows).await?;

  let rdr = ReaderBuilder::new()
    .delimiter(sep)
    .quoting(quoting)
    .from_reader(opts.rdr_skip_rows()?);

  let buf_writer = BufWriter::with_capacity(RDR_BUFFER_SIZE, File::create(output_path)?);
  let wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_writer);

  match mode {
    SliceMode::Left => {
      slice_nchar(
        rdr,
        wtr,
        column,
        num,
        reverse,
        SliceMode::Left.to_str().to_string(),
        progress,
        emitter,
      )
      .await?
    }
    SliceMode::Right => {
      slice_nchar(
        rdr,
        wtr,
        column,
        num,
        reverse,
        SliceMode::Right.to_str().to_string(),
        progress,
        emitter,
      )
      .await?
    }
    SliceMode::Slice => slice(rdr, wtr, column, n, length, reverse).await?,
  }

  Ok(())
}
