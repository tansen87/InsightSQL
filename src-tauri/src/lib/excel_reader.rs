use std::{collections::HashMap, fmt::Display, fs::File, io::BufReader, path::Path};

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

    // iterating and writing headers or duplicate headers
    let mut header_counts = HashMap::<String, usize>::new();
    let headers: Vec<String> = match self.rows().next() {
      Some(first_row) => first_row
        .iter()
        .map(|cell| {
          let cell_str = cell.to_string();
          let count = header_counts.entry(cell_str.clone()).or_insert(0);
          let current_count = *count;
          *count += 1;
          if current_count > 0 {
            format!("{}_duplicated_{}", cell_str, current_count - 1)
          } else {
            cell_str
          }
        })
        .collect(),
      None => return Err(anyhow!("No data")),
    };

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
    match self
      .workbook
      .with_header_row(HeaderRow::Row(skip_rows))
      .worksheet_range_at(n)
    {
      Some(Ok(sheet_range)) => Ok(sheet_range),
      Some(Err(e)) => Err(e.into()),
      None => Err(anyhow!("Worksheet index out of bounds")),
    }
  }

  /// Get the worksheet's column names
  /// NOTICE: It will load the entire worksheet into memory
  pub fn get_column_names(&mut self, n: usize, skip_rows: u32) -> Result<Vec<String>> {
    let column_names: Vec<String> = self
      .worksheet_range_at(n, skip_rows)?
      .rows()
      .next()
      .ok_or(anyhow!("No data"))?
      .iter()
      .map(|cell| cell.to_string())
      .collect();

    Ok(column_names)
  }
}
