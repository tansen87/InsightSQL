use std::path::Path;

use anyhow::{Result, anyhow};
use csv::ReaderBuilder;
use polars::{
  io::SerReader,
  prelude::{CsvParseOptions, CsvReadOptions},
};

use crate::io::csv::options::CsvOptions;
use crate::io::excel::xlsx_writer::XlsxWriter;

/// convert csv to xlsx
pub async fn csv_to_xlsx<P: AsRef<Path> + Send + Sync>(
  path: P,
  use_polars: bool,
  chunk_size: usize,
) -> Result<()> {
  let dest = path.as_ref().with_extension("xlsx");
  let opts = CsvOptions::new(&path);
  let sep = opts.detect_separator()?;

  if use_polars {
    let row_count = opts.std_count_rows()?;
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
      .from_reader(opts.rdr_skip_rows()?);

    XlsxWriter::new().write_xlsx(rdr, chunk_size, dest)?;
  }

  Ok(())
}
