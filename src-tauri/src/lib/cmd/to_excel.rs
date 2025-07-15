use std::{path::Path, time::Instant};

use anyhow::{Result, anyhow};
use csv::ReaderBuilder;
use polars::{
  io::SerReader,
  prelude::{CsvParseOptions, CsvReadOptions},
};
use tauri::{Emitter, Window};

use crate::{io::xlsx_writer::XlsxWriter, utils::CsvOptions};

/// convert csv to xlsx
async fn csv_to_xlsx<P: AsRef<Path> + Send + Sync>(
  path: P,
  use_polars: bool,
  chunk_size: usize,
) -> Result<()> {
  let dest = path.as_ref().with_extension("xlsx");
  let csv_options = CsvOptions::new(&path);
  let sep = csv_options.detect_separator()?;

  if use_polars {
    let row_count = csv_options.std_csv_rows()?;
    if row_count > 104_8575 {
      return Err(anyhow!("{row_count} rows exceed the maximum row in Excel"));
    }

    let df = CsvReadOptions::default()
      .with_parse_options(CsvParseOptions::default().with_separator(sep))
      .with_infer_schema_length(Some(0))
      .try_into_reader_with_file_path(Some((&path.as_ref()).to_path_buf()))?
      .finish()?;

    XlsxWriter::new().write_dataframe(&df, dest)?;
  } else {
    let rdr = ReaderBuilder::new()
      .delimiter(sep)
      .from_reader(csv_options.skip_csv_rows()?);

    XlsxWriter::new().write_xlsx(rdr, chunk_size, dest)?;
  }

  Ok(())
}

#[tauri::command]
pub async fn csv2xlsx(
  path: String,
  mode: String,
  chunk_size: String,
  window: Window,
) -> Result<String, String> {
  let start_time = Instant::now();

  let paths: Vec<&str> = path.split('|').collect();
  let chunk_size = chunk_size.parse::<usize>().map_err(|e| e.to_string())?;
  let use_polars = mode != "csv";

  for file in paths.iter() {
    let filename = Path::new(file).file_name().unwrap().to_str().unwrap();
    window
      .emit("start-to", filename)
      .map_err(|e| e.to_string())?;

    match csv_to_xlsx(file, use_polars, chunk_size).await {
      Ok(_) => {
        window
          .emit("c2x-msg", filename)
          .map_err(|e| e.to_string())?;
      }
      Err(err) => {
        window
          .emit("rows-err", format!("{filename}|{err}"))
          .map_err(|e| e.to_string())?;
        continue;
      }
    }
  }

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  Ok(format!("{elapsed_time:.2}"))
}
