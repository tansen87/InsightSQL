use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

use anyhow::{Result, anyhow};
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
    let headers: Vec<String> = self
      .rows()
      .next()
      .ok_or(anyhow!("No data"))?
      .iter()
      .map(|cell| {
        let count = header_counts.entry(cell.to_string()).or_insert(0);
        let name = if *count > 0 {
          format!("{}_duplicated_{}", cell, count)
        } else {
          cell.to_string()
        };
        *count += 1;
        name
      })
      .collect();

    let mut columns = vec![Vec::new(); headers.len()];

    for row in self.rows().skip(1) {
      for (col_idx, cell) in row.iter().enumerate() {
        let value = match cell {
          Data::DateTime(dt) => dt
            .as_datetime()
            .map(|d| d.to_string())
            .unwrap_or_else(|| String::new()),
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

    // list of `Column`s
    let series: Vec<Column> = headers
      .into_iter()
      .zip(columns)
      .map(|(col, data)| Column::new(col.into(), data))
      .collect();

    // constructing DataFrame
    let df = DataFrame::new_infer_height(series)?;

    Ok(df)
  }
}

impl ExcelReader {
  /// Opens a workbook and define the file type at runtime.
  pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
    let workbook = calamine::open_workbook_auto(path)?;
    Ok(ExcelReader { workbook })
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

pub struct FastExcelReader {
  fast_workbook: xl::Workbook,
}

pub trait FastToDataFrame {
  fn fast_to_df(&mut self, n: usize, skip_rows: usize) -> Result<DataFrame>;
}

impl FastExcelReader {
  pub fn from_path(path: &str) -> Result<Self> {
    let fast_workbook = match xl::Workbook::new(path) {
      Ok(wb) => wb,
      Err(e) => {
        return Err(anyhow!("failed to open xlsx: {e}"));
      }
    };
    Ok(FastExcelReader { fast_workbook })
  }

  /// Get the first n rows of xlsx
  /// It's very fast
  pub fn n_rows(&mut self, n: usize) -> Result<Vec<String>> {
    let worksheets = self.fast_workbook.sheets();
    let first_sheet_name = match worksheets.by_name().get(0) {
      Some(sheet) => *sheet,
      None => "Sheet1",
    };
    let sheet = if let Some(s) = worksheets.get(first_sheet_name) {
      s
    } else {
      return Err(anyhow!("worksheet is empty"));
    };
    let nrows: Vec<String> = sheet
      .rows(&mut self.fast_workbook)
      .take(n + 1)
      .map(|row| row.to_string().replace(",", "|").replace("\"", ""))
      .collect();

    Ok(nrows)
  }
}

impl FastToDataFrame for FastExcelReader {
  fn fast_to_df(&mut self, n: usize, skip_rows: usize) -> Result<DataFrame> {
    let data = self.n_rows(n + skip_rows)?;

    if data.len() <= skip_rows {
      return Err(anyhow::anyhow!("Not enough rows in the file"));
    }

    let headers = data
      .get(skip_rows)
      .ok_or_else(|| anyhow::anyhow!("Header not found"))?
      .split('|')
      .map(String::from)
      .collect::<Vec<String>>();

    let num_cols = headers.len();

    let rows = data
      .into_iter()
      .skip(skip_rows + 1)
      .filter(|row| !row.trim().is_empty())
      .collect::<Vec<_>>();

    let mut columns: Vec<Vec<String>> = vec![vec![]; num_cols];

    for row in &rows {
      let cells = row.split('|').map(|s| s.to_string()).collect::<Vec<_>>();
      if cells.len() != num_cols {
        continue;
      }
      for (i, cell) in cells.into_iter().enumerate() {
        columns[i].push(cell);
      }
    }

    let series = headers
      .into_iter()
      .zip(columns)
      .map(|(col, data)| Column::new(col.into(), data))
      .collect();

    Ok(DataFrame::new_infer_height(series)?)
  }
}
