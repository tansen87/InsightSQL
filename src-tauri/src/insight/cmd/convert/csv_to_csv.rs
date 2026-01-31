use std::{
  collections::HashMap,
  fs::File,
  io::{BufRead, BufReader, BufWriter},
  path::Path,
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
  time::Duration,
};

use anyhow::Result;
use csv::{ByteRecord, ReaderBuilder, WriterBuilder};
use encoding_rs_io::DecodeReaderBytesBuilder;
use tokio::sync::oneshot;

use crate::{
  io::csv::{config::CsvConfigBuilder, options::CsvOptions},
  utils::EventEmitter,
};

/// Reformat a CSV with different delimiters, quoting rules
pub async fn csv_to_csv<E, P>(
  path: P,
  wtr_sep: &str,
  quote: &str,
  quote_style: &str,
  quoting: bool,
  filename: String,
  progress: bool,
  skiprows: usize,
  flexible: bool,
  emitter: E,
) -> Result<()>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync,
{
  let mut opts = CsvOptions::new(path);
  opts.set_skiprows(skiprows);
  let (rdr_sep, reader) = opts.skiprows_and_delimiter()?;
  let output_path = opts.output_path(Some("fmt"), None)?;
  let sep = if wtr_sep == "\\t" {
    b'\t'
  } else {
    wtr_sep.as_bytes().get(0).copied().unwrap_or(b',')
  };
  let quote = quote.as_bytes().get(0).copied().unwrap_or(b'"');
  let quote_style = match quote_style {
    "always" => csv::QuoteStyle::Always,
    "non_numeric" => csv::QuoteStyle::NonNumeric,
    "never" => csv::QuoteStyle::Never,
    _ => csv::QuoteStyle::Necessary,
  };

  let total_rows = match progress {
    true => opts.idx_count_rows().await?,
    false => 0,
  };
  emitter
    .emit_total_msg(&format!("{filename}|{total_rows}"))
    .await?;

  let config = CsvConfigBuilder::new()
    .flexible(flexible)
    .read_delimiter(rdr_sep)
    .quoting(quoting)
    .write_delimiter(sep)
    .quote(quote)
    .quote_style(quote_style)
    .build();

  let mut rdr = config.build_reader(reader);
  let mut wtr = config.build_writer(&output_path)?;

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
            if let Err(err) = emitter.emit_update_msg(&format!("{filename}|{current_rows}")).await {
              let _ = emitter.emit_err(&format!("{filename}|{err}")).await;
            }
          },
          Ok(final_rows) = (&mut done_rx) => {
            if let Err(err) = emitter.emit_update_msg(&format!("{filename}|{final_rows}")).await {
              let _ = emitter.emit_err(&format!("{filename}|{err}")).await;
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
    let mut record = ByteRecord::new();
    while rdr.read_byte_record(&mut record)? {
      wtr.write_byte_record(&record)?;

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

pub async fn encoding_to_utf8<P>(path: P, bom: bool, quoting: bool) -> Result<()>
where
  P: AsRef<Path> + Send + Sync,
{
  let opts = CsvOptions::new(&path);
  let encoding = opts.detect_encoding(bom)?;
  log::info!("{encoding:?}");

  // 检测分隔符
  let separator = {
    let file = File::open(&path)?;
    let decoder = DecodeReaderBytesBuilder::new()
      .encoding(Some(encoding))
      .build(file);
    let mut lines = BufReader::new(decoder).lines();

    let candidates = [b',', b';', b'\t', b'|', b'^'];
    let mut counts: HashMap<u8, usize> = candidates.iter().map(|&c| (c, 0)).collect();

    if let Some(Ok(line)) = lines.next() {
      for &b in line.as_bytes() {
        if counts.contains_key(&b) {
          *counts.get_mut(&b).unwrap() += 1;
        }
      }
    }

    let (&best, &max) = counts
      .iter()
      .max_by_key(|&(_, v)| v)
      .expect("Candidate separators map is empty");
    if max > 0 { best } else { b',' }
  };
  log::info!("Separator: {:?}", separator as char);

  let file = File::open(&path)?;
  let decoder = DecodeReaderBytesBuilder::new()
    .encoding(Some(encoding))
    .build(file);
  let buf_reader = BufReader::new(decoder);

  let mut rdr = ReaderBuilder::new()
    .delimiter(separator)
    .quoting(quoting)
    .from_reader(buf_reader);

  let output_path = opts.output_path(Some("encoding"), None)?;
  let wtr = BufWriter::new(File::create(&output_path)?);
  let mut wtr = WriterBuilder::new().delimiter(separator).from_writer(wtr);

  if let Ok(headers) = rdr.headers() {
    wtr.write_record(headers)?;
  }

  for result in rdr.records() {
    let record = result?;
    wtr.write_record(&record)?;
  }

  Ok(wtr.flush()?)
}
