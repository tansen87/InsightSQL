use std::path::Path;

use anyhow::{Result, anyhow};
use csv::ReaderBuilder;
use polars::{
  io::SerReader,
  prelude::{CsvParseOptions, CsvReadOptions},
};

use crate::io::excel::xlsx_writer::XlsxWriter;
use crate::{io::csv::options::CsvOptions, utils::EXCEL_MAX_ROW};

/// convert csv to xlsx
pub async fn csv_to_xlsx<P: AsRef<Path> + Send + Sync>(
  path: P,
  use_polars: bool,
  chunk_size: usize,
  quoting: bool,
  skiprows: usize,
) -> Result<()> {
  let dest = path.as_ref().with_extension("xlsx");
  let mut opts = CsvOptions::new(&path);
  opts.set_skiprows(skiprows);
  let (sep, reader) = opts.skiprows_and_delimiter()?;

  if use_polars {
    let row_count = opts.std_count_rows()?;
    if row_count > EXCEL_MAX_ROW {
      return Err(anyhow!("{row_count} rows exceed the maximum row in Excel"));
    }

    let df = CsvReadOptions::default()
      .with_parse_options(CsvParseOptions::default().with_separator(sep))
      .with_infer_schema_length(Some(0))
      .with_skip_rows(skiprows)
      .try_into_reader_with_file_path(Some((&path.as_ref()).to_path_buf()))?
      .finish()?;

    XlsxWriter::new().write_dataframe(&df, dest)?;
  } else {
    let rdr = ReaderBuilder::new()
      .delimiter(sep)
      .quoting(quoting)
      .from_reader(reader);

    XlsxWriter::new().write_xlsx(rdr, chunk_size, dest)?;
  }

  Ok(())
}
