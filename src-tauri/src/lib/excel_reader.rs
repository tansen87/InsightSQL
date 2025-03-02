use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

use anyhow::{anyhow, Result};
use calamine::{Data, HeaderRow, Range, Reader};
use polars::{frame::DataFrame, prelude::Column};

pub struct ExcelReader {
  workbook: calamine::Sheets<BufReader<File>>,
}

pub trait ToPolarsDataFrame {
  fn to_df(&mut self) -> Result<DataFrame>;
}

impl ToPolarsDataFrame for Range<Data> {
  fn to_df(&mut self) -> Result<DataFrame> {
    // iterating headers or duplicate headers
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

    let mut columns = vec![Vec::new(); headers.len()];

    for row in self.rows().skip(1) {
      for (col_idx, cell) in row.iter().enumerate() {
        let value = match cell {
          Data::DateTime(dt) => dt
            .as_datetime()
            .map(|d| d.to_string())
            .unwrap_or_else(|| String::from("<Invalid DateTime>")),
          Data::Float(f) => f.to_string(),
          Data::Int(i) => i.to_string(),
          Data::Bool(b) => b.to_string(),
          Data::String(s) => s.to_string(),
          Data::DateTimeIso(dt) => dt.to_string(),
          Data::DurationIso(dur) => dur.to_string(),
          Data::Empty => String::new(),
          Data::Error(e) => format!("Error({:?})", e),
        };

        if col_idx < columns.len() {
          columns[col_idx].push(value);
        }
      }
    }

    // list of `Series`s
    let series: Vec<Column> = headers
      .into_iter()
      .zip(columns)
      .map(|(col, data)| Column::new(col.into(), data))
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

/// Get the first n rows of xlsx
/// It's very fast
pub fn n_rows(path: &str, n: usize) -> Result<Vec<String>> {
  let mut workbook = match xl::Workbook::new(path) {
    Ok(wb) => wb,
    Err(e) => {
      return Err(anyhow!("failed to open xlsx: {e}"));
    }
  };
  let worksheets = workbook.sheets();
  let first_sheet_name = match worksheets.by_name().get(0) {
    Some(sheet) => *sheet,
    None => "Sheet1",
  };
  let sheet = worksheets
    .get(first_sheet_name)
    .expect("worksheet is empty");
  let nrows: Vec<String> = sheet
    .rows(&mut workbook)
    .take(n)
    .map(|row| row.to_string().replace(",", "|"))
    .collect();
  Ok(nrows)
}
