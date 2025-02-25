use std::{
  fs::File,
  io::{BufRead, BufWriter, Write},
  path::Path,
  time::Instant,
};

use anyhow::Result;
use csv::{ByteRecord, ReaderBuilder, Writer, WriterBuilder};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
  index::Indexed,
  utils::{num_of_chunks, CsvOptions},
};

fn new_writer(
  headers: &ByteRecord,
  index: usize,
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

pub async fn sequential_split_rows(
  csv_options: CsvOptions<&str>,
  size: u32,
  output_path: &str,
) -> Result<()> {
  let sep = csv_options.detect_separator()?;

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

pub async fn parallel_split_rows(
  idx: &Indexed<File, File>,
  csv_options: CsvOptions<&str>,
  chunk_size: usize,
  output_path: &str,
) -> Result<()> {
  let nchunks = num_of_chunks(idx.count() as usize, chunk_size);
  if nchunks == 1 {
    // there's only one chunk, we can just do a sequential split
    // which has less overhead and better error handling
    return sequential_split_rows(csv_options, chunk_size.try_into()?, output_path).await;
  }

  let sep = csv_options.detect_separator()?;

  // safety: we cannot use ? here because we're in a closure
  (0..nchunks).into_par_iter().for_each(|i| {
    // safety: safe to unwrap because we know the file is indexed
    let mut idx = csv_options.indexed().unwrap().unwrap();
    // safety: the only way this can fail is if the file first row of the chunk
    // is not a valid CSV record, which is impossible because we're reading
    // from a file with a valid index
    let headers = idx.byte_headers().unwrap();

    // safety: the only way this can fail is if we cannot create a file
    let mut wtr = new_writer(headers, i * chunk_size, &output_path, sep).unwrap();

    // safety: we know that there is more than one chunk, so we can safely
    // seek to the start of the chunk
    idx.seek((i * chunk_size) as u64).unwrap();
    let mut write_row;
    for row in idx.byte_records().take(chunk_size) {
      write_row = row.unwrap();
      wtr.write_byte_record(&write_row).unwrap();
    }
    // safety: safe to unwrap because we know the writer is a file
    // the only way this can fail is if we cannot write to the file
    wtr.flush().unwrap();
  });

  Ok(())
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

pub async fn split_lines(
  csv_options: CsvOptions<&str>,
  size: u32,
  output_path: &str,
) -> Result<()> {
  let reader = csv_options.skip_csv_rows()?;
  let mut lines = reader.lines();
  let headers = lines.next().transpose()?;

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
pub async fn split(path: String, size: u32, mode: String) -> Result<String, String> {
  let start_time = Instant::now();

  let csv_options = CsvOptions::new(path.as_str());
  let parent_path = Path::new(path.as_str()).parent().unwrap().to_str().unwrap();
  let file_stem = Path::new(path.as_str())
    .file_stem()
    .unwrap()
    .to_str()
    .unwrap();
  let output_path = format!("{parent_path}/{file_stem}");

  match mode.as_str() {
    "rows" => {
      match csv_options.indexed().map_err(|e| e.to_string())? {
        Some(idx) => parallel_split_rows(
          &idx,
          csv_options,
          size.try_into().expect("not valid size"),
          &output_path,
        )
        .await
        .map_err(|e| e.to_string())?,
        None => sequential_split_rows(csv_options, size, &output_path)
          .await
          .map_err(|e| e.to_string())?,
      };
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    "index" => match crate::idx::create_index(path).await {
      Ok(_) => {
        let end_time = Instant::now();
        let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
        Ok(format!("{elapsed_time:.2}"))
      }
      Err(err) => Err(format!("{err}")),
    },
    _ => match split_lines(csv_options, size, &output_path).await {
      Ok(_) => {
        let end_time = Instant::now();
        let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
        Ok(format!("{elapsed_time:.2}"))
      }
      Err(err) => Err(format!("{err}")),
    },
  }
}
