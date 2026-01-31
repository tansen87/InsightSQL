use std::path::Path;

use anyhow::{Result, anyhow};
use csv::ReaderBuilder;

use crate::io::excel::xlsx_writer::XlsxWriter;
use crate::{io::csv::options::CsvOptions, utils::EXCEL_MAX_ROW};

/// convert csv to xlsx
pub async fn csv_to_xlsx<P: AsRef<Path> + Send + Sync>(
  path: P,
  multi: bool,
  chunk_size: usize,
  quoting: bool,
  skiprows: usize,
) -> Result<()> {
  let dest = path.as_ref().with_extension("xlsx");
  let mut opts = CsvOptions::new(&path);
  opts.set_skiprows(skiprows);
  let (sep, reader) = opts.skiprows_and_delimiter()?;

  let rdr = ReaderBuilder::new()
    .delimiter(sep)
    .quoting(quoting)
    .from_reader(reader);

  if multi {
    let row_count = opts.count_lines()?;
    if row_count > EXCEL_MAX_ROW {
      return Err(anyhow!("{row_count} rows exceed the maximum row in Excel"));
    }

    XlsxWriter::new().write_xlsx_split(rdr, chunk_size, dest)?;
  } else {
    XlsxWriter::new().write_xlsx(rdr, chunk_size, dest)?;
  }

  Ok(())
}
