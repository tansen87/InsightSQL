use std::{
  collections::HashMap,
  fs::File,
  io::{BufRead, BufReader, BufWriter},
  path::Path,
  sync::{Arc, Mutex},
  time::Duration,
};

use anyhow::{Result, anyhow};
use csv::{ByteRecord, ReaderBuilder, WriterBuilder};
use encoding_rs_io::DecodeReaderBytesBuilder;
use tokio::sync::oneshot;

use crate::{
  io::csv::options::CsvOptions,
  utils::{EventEmitter, WTR_BUFFER_SIZE},
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
  emitter: E,
) -> Result<()>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync,
{
  let opts = CsvOptions::new(&path);
  let rdr_sep = opts.detect_separator()?;
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

  let mut rdr = ReaderBuilder::new()
    .delimiter(rdr_sep)
    .quoting(quoting)
    .from_reader(opts.rdr_skip_rows()?);

  let buf_writer = BufWriter::with_capacity(WTR_BUFFER_SIZE, File::create(output_path)?);
  let mut wtr = WriterBuilder::new()
    .delimiter(sep)
    .quote(quote)
    .quote_style(quote_style)
    .from_writer(buf_writer);

  wtr.write_record(rdr.headers()?)?;

  let rows = Arc::new(Mutex::new(0));
  let rows_clone = Arc::clone(&rows);
  let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
  let (done_tx, mut done_rx) = oneshot::channel::<usize>();

  let timer_task = tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_millis(300));
    loop {
      tokio::select! {
        _ = interval.tick() => {
          let current_rows = match rows_clone.lock() {
            Ok(lock) => *lock,
            Err(err) => {
              log::error!("Failed to lock current rows: {err}");
              0
            }
          };
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
  wtr.flush()?;

  Ok(())
}
