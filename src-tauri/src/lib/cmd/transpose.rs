use std::{
  fs::File,
  io::BufWriter,
  path::{Path, PathBuf},
  time::Instant,
};

use anyhow::Result;
use csv::{ByteRecord, ReaderBuilder, WriterBuilder};

use crate::io::csv::options::CsvOptions;

pub async fn in_memory_transpose<P: AsRef<Path> + Send + Sync>(path: P) -> Result<()> {
  let csv_options = CsvOptions::new(&path);
  let sep = csv_options.detect_separator()?;
  let parent_path = path.as_ref().parent().unwrap().to_str().unwrap();
  let file_stem = path.as_ref().file_stem().unwrap().to_str().unwrap();
  let mut output_path = PathBuf::from(parent_path);
  output_path.push(format!("{file_stem}.transpose.csv"));

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(csv_options.rdr_skip_rows()?);

  let mut wtr = WriterBuilder::new()
    .delimiter(sep)
    .from_writer(BufWriter::new(File::create(output_path)?));

  let nrows = rdr.byte_headers()?.len();

  let all = rdr.byte_records().collect::<Result<Vec<_>, _>>()?;
  for i in 0..nrows {
    let mut record = ByteRecord::new();

    for row in &all {
      record.push_field(&row[i]);
    }
    wtr.write_byte_record(&record)?;
  }

  Ok(wtr.flush()?)
}

pub async fn multipass_transpose<P: AsRef<Path> + Send + Sync>(path: P) -> Result<()> {
  let file_options = CsvOptions::new(&path);
  let sep = file_options.detect_separator()?;
  let parent_path = path.as_ref().parent().unwrap().to_str().unwrap();
  let file_stem = path.as_ref().file_stem().unwrap().to_str().unwrap();
  let mut output_path = PathBuf::from(parent_path);
  output_path.push(format!("{file_stem}.transpose.csv"));

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(File::open(&path)?);

  let mut wtr = WriterBuilder::new()
    .delimiter(sep)
    .from_writer(BufWriter::new(File::create(output_path)?));

  let nrows = rdr.byte_headers()?.len();

  for i in 0..nrows {
    let mut rdr = ReaderBuilder::new()
      .delimiter(sep)
      .from_reader(File::open(&path)?);

    let mut record = ByteRecord::new();
    for row in rdr.byte_records() {
      record.push_field(&row?[i]);
    }
    wtr.write_byte_record(&record)?;
  }

  Ok(wtr.flush()?)
}

#[tauri::command]
pub async fn transpose(path: String, mode: String) -> Result<String, String> {
  let start_time = Instant::now();

  match mode.as_str() {
    "memory" => match in_memory_transpose(path).await {
      Ok(()) => {
        let end_time = Instant::now();
        let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
        Ok(format!("{elapsed_time:.2}"))
      }
      Err(err) => Err(format!("{err}")),
    },
    "multipass" => match multipass_transpose(path).await {
      Ok(()) => {
        let end_time = Instant::now();
        let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
        Ok(format!("{elapsed_time:.2}"))
      }
      Err(err) => Err(format!("{err}")),
    },
    _ => Err(format!("Unknown transpose mode")),
  }
}
