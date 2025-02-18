use std::{
  fs::File,
  io::{BufRead, BufWriter, Write},
  path::Path,
  time::Instant,
};

use anyhow::Result;
use csv::{ByteRecord, ReaderBuilder, Writer, WriterBuilder};

use crate::utils::CsvOptions;

fn new_writer(
  headers: &ByteRecord,
  index: i32,
  output_path: &str,
  sep: u8,
) -> Result<Writer<BufWriter<File>>> {
  let spath = format!("{output_path}.split_{index}.csv");

  let mut wtr = WriterBuilder::new()
    .delimiter(sep)
    .from_writer(BufWriter::new(File::create(spath)?));

  wtr.write_record(headers)?;

  Ok(wtr)
}

pub async fn split_rows<P: AsRef<Path>>(path: P, size: u32, skip_rows: usize) -> Result<()> {
  let mut csv_options = CsvOptions::new(&path);
  csv_options.set_skip_rows(skip_rows);

  let sep = match csv_options.detect_separator() {
    Some(separator) => separator as u8,
    None => b',',
  };

  let parent_path = path.as_ref().parent().unwrap().to_str().unwrap();
  let file_stem = path.as_ref().file_stem().unwrap().to_str().unwrap();
  let output_path = format!("{parent_path}/{file_stem}");

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(csv_options.skip_csv_rows()?);

  let headers = rdr.byte_headers()?.clone();

  let mut wtr = new_writer(&headers, 0, &output_path, sep)?;
  let mut i = 0;
  let mut cnt = 1;
  let mut row = ByteRecord::new();
  while rdr.read_byte_record(&mut row)? {
    if i > 0 && i % size == 0 {
      wtr.flush()?;
      wtr = new_writer(&headers, cnt, &output_path, sep)?;
      cnt += 1;
    }
    wtr.write_byte_record(&row)?;
    i += 1;
  }

  Ok(wtr.flush()?)
}

fn new_lines_writer(
  headers: &Option<String>,
  index: usize,
  output_path: &str,
) -> Result<BufWriter<File>> {
  let output_file = format!("{output_path}.split_{index}.csv");
  let file = File::create(output_file)?;
  let mut wtr = BufWriter::new(file);
  if let Some(header) = headers {
    writeln!(wtr, "{}", header)?;
  }

  Ok(wtr)
}

pub async fn split_lines<P: AsRef<Path>>(path: P, size: u32, skip_rows: usize) -> Result<()> {
  let mut csv_options = CsvOptions::new(&path);
  csv_options.set_skip_rows(skip_rows);
  let reader = csv_options.skip_csv_rows()?;
  let mut lines = reader.lines();
  let headers = lines.next().transpose()?;

  let parent_path = path.as_ref().parent().unwrap().to_str().unwrap();
  let file_stem = path.as_ref().file_stem().unwrap().to_str().unwrap();
  let output_path = format!("{parent_path}/{file_stem}");

  let mut wtr = new_lines_writer(&headers, 0, &output_path)?;
  let mut i = 0;
  let mut cnt = 1;

  for line in lines {
    let line = line?;
    if i > 0 && i % size == 0 {
      wtr.flush()?;
      wtr = new_lines_writer(&headers, cnt, &output_path)?;
      cnt += 1;
    }
    writeln!(wtr, "{}", line)?;
    i += 1;
  }

  Ok(wtr.flush()?)
}

#[tauri::command]
pub async fn split(
  path: String,
  size: u32,
  skip_rows: String,
  mode: String,
) -> Result<String, String> {
  let start_time = Instant::now();

  let skip_rows = skip_rows.parse::<usize>().map_err(|e| e.to_string())?;

  match mode.as_str() {
    "rows" => match split_rows(path, size, skip_rows).await {
      Ok(_) => {
        let end_time = Instant::now();
        let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
        Ok(format!("{elapsed_time:.2}"))
      }
      Err(err) => Err(format!("split failed: {err}")),
    },
    _ => match split_lines(path, size, skip_rows).await {
      Ok(_) => {
        let end_time = Instant::now();
        let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
        Ok(format!("{elapsed_time:.2}"))
      }
      Err(err) => Err(format!("split failed: {err}")),
    },
  }
}
