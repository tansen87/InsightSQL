use std::{fmt::Display, fs::File, io::BufReader, path::Path};

use anyhow::{anyhow, Result};
use calamine::{CellType, Data, DataType, HeaderRow, Range, Reader};
use polars::{frame::DataFrame, prelude::Column};

pub struct ExcelReader {
  workbook: calamine::Sheets<BufReader<File>>,
}

pub trait ToPolarsDataFrame {
  fn to_df(&mut self) -> Result<DataFrame>;
}

impl<T> ToPolarsDataFrame for Range<T>
where
  T: DataType + CellType + Display,
{
  fn to_df(&mut self) -> Result<DataFrame> {
    let mut columns = Vec::new();

    // Headers
    let headers: Vec<String> = self
      .rows()
      .next()
      .ok_or(anyhow!("No data"))?
      .iter()
      .map(|cell| cell.to_string())
      .collect();

    // Vec<String> for each column
    for _ in 0..headers.len() {
      columns.push(Vec::<String>::new());
    }

    // iterating through all rows
    for row in self.rows().skip(1) {
      for (col_idx, cell) in row.iter().enumerate() {
        columns[col_idx].push(cell.to_string());
      }
    }

    // list of `Series`s
    let series: Vec<Column> = columns
      .into_iter()
      .zip(headers)
      .map(|(col, name)| Column::new((&name).into(), col))
      .collect();

    // constructing DataFrame
    let df = DataFrame::new(series)?;

    Ok(df)
  }
}

impl ExcelReader {
  /// Opens a workbook and define the file type at runtime.
  pub fn open_workbook_auto<P: AsRef<Path>>(path: P) -> calamine::Sheets<BufReader<File>> {
    let workbook = calamine::open_workbook_auto(path).expect("Could not open workbook");

    workbook
  }

  pub fn new<P: AsRef<Path>>(path: P) -> Self {
    Self {
      workbook: ExcelReader::open_workbook_auto(path),
    }
  }

  /// Get the nth worksheet. Shortcut for getting the nth
  /// sheet_name, then the corresponding worksheet.
  pub fn worksheet_range_at(&mut self, n: usize, skip_rows: u32) -> Result<Range<Data>> {
    match self.workbook.with_header_row(HeaderRow::Row(skip_rows)).worksheet_range_at(n) {
      Some(Ok(sheet_range)) => Ok(sheet_range),
      Some(Err(e)) => Err(e.into()),
      None => Err(anyhow!("Worksheet index out of bounds")),
    }
  }
}
