use std::{
  fs::File,
  io::{BufRead, BufReader, BufWriter, Write},
  path::Path,
  time::Instant,
};

use anyhow::{Result, anyhow};
use csv::{ReaderBuilder, WriterBuilder};

use crate::io::csv::options::CsvOptions;
use crate::utils::WTR_BUFFER_SIZE;

/// 将CSV文件拆分为good行和bad行
///
/// # 参数
/// - `path`: 输入CSV路径
/// - `expected_columns`: 可手动指定期望列数:若为 None,则以第一行为准
pub async fn separate_csv<P>(path: P, quoting: bool, expected_columns: Option<usize>) -> Result<()>
where
  P: AsRef<Path> + Send + Sync,
{
  let reader = BufReader::new(File::open(&path)?);
  let opts = CsvOptions::new(&path);
  let sep = opts.detect_separator()?;
  let good_path = opts.output_path(Some("good"), None)?;
  let bad_path = opts.output_path(Some("bad"), None)?;

  let good_buf_wtr = BufWriter::with_capacity(WTR_BUFFER_SIZE, File::create(good_path)?);
  let mut good_wtr = WriterBuilder::new()
    .delimiter(sep)
    .from_writer(good_buf_wtr);
  let mut bad_wtr = BufWriter::with_capacity(WTR_BUFFER_SIZE, File::create(bad_path)?);

  let mut lines = reader.lines().enumerate();

  let (_header_line_num, header) = match lines.next() {
    Some((_, line)) => (1, line?),
    None => return Err(anyhow!("Input file is empty (missing header)")),
  };

  // 解析 header
  let mut header_parser = ReaderBuilder::new()
    .has_headers(false)
    .flexible(true)
    .delimiter(sep)
    .quoting(quoting)
    .from_reader(header.as_bytes());

  let inferred_columns = if let Some(Ok(record)) = header_parser.records().next() {
    // 如果expected_columns是Some(n)且 n > 0,则使用它;否则用header的列数
    let expected_len = match expected_columns {
      Some(n) if n > 0 => n,
      _ => record.len(), // 包括None, Some(0), Some(负数)
    };
    good_wtr.write_record(&record)?;
    expected_len
  } else {
    log::error!("Failed to parse header line. Writing to bad file.");
    writeln!(bad_wtr, "{}", header)?;
    bad_wtr.flush()?;
    return Err(anyhow!("Header line is invalid CSV; cannot proceed"));
  };

  log::info!(
    "Using {} columns (from header{})",
    inferred_columns,
    if expected_columns.is_some() {
      " (overridden)"
    } else {
      ""
    }
  );

  // 处理数据行
  for (idx, line_result) in lines {
    let physical_line_num = idx + 1;
    let line = match line_result {
      Ok(l) => l,
      Err(e) => {
        log::error!("I/O error on line {}: {}", physical_line_num, e);
        continue;
      }
    };

    let mut parser = ReaderBuilder::new()
      .has_headers(false)
      .delimiter(sep)
      .flexible(false)
      .quoting(quoting)
      .from_reader(line.as_bytes());

    match parser.records().next() {
      Some(Ok(record)) if record.len() == inferred_columns => {
        good_wtr.write_record(&record)?;
      }
      Some(Ok(record)) => {
        log::error!(
          "Line {}: expected {} fields, got {} → writing to bad file",
          physical_line_num,
          inferred_columns,
          record.len()
        );
        writeln!(bad_wtr, "{}", line)?;
      }
      Some(Err(e)) => {
        log::error!(
          "Line {}: CSV parse error: {} → writing to bad file",
          physical_line_num,
          e
        );
        writeln!(bad_wtr, "{}", line)?;
      }
      None => {
        log::warn!(
          "Line {}: empty or unreadable → writing to bad file",
          physical_line_num
        );
        writeln!(bad_wtr, "{}", line)?;
      }
    }
  }

  good_wtr.flush()?;
  bad_wtr.flush()?;

  Ok(())
}

#[tauri::command]
pub async fn separate(
  path: String,
  quoting: bool,
  expected_columns: String,
) -> Result<String, String> {
  let start_time = Instant::now();

  match separate_csv(
    path,
    quoting,
    Some(expected_columns.parse::<usize>().unwrap_or(0)),
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
