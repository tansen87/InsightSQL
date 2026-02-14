use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use anyhow::Result;
use csv::{Reader, StringRecord};
use polars::prelude::Column;
use polars::{datatypes::AnyValue, frame::DataFrame};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_xlsxwriter::{Format, Workbook};

pub struct XlsxWriter {
  workbook: Workbook,
}

impl XlsxWriter {
  pub fn new() -> Self {
    Self {
      workbook: Workbook::new(),
    }
  }

  /// write csv to xlsx
  pub fn write_xlsx<P: AsRef<Path>>(
    &mut self,
    mut rdr: Reader<BufReader<Box<dyn Read + Send>>>,
    chunk_size: usize,
    output: P,
  ) -> Result<()> {
    let headers = rdr.headers()?.clone();

    let mut chunk: Vec<csv::StringRecord> = Vec::with_capacity(chunk_size);

    for result in rdr.records() {
      let record = result?;
      chunk.push(record);

      if chunk.len() >= chunk_size {
        Self::write_chunk(&mut chunk, &headers, &mut self.workbook)?;

        chunk.clear();
      }
    }

    if !chunk.is_empty() {
      Self::write_chunk(&mut chunk, &headers, &mut self.workbook)?;
    }

    Ok(self.workbook.save(output)?)
  }

  /// write dataframe to xlsx
  pub fn write_dataframe(&mut self, df: &DataFrame, output_path: PathBuf) -> Result<()> {
    let worksheet = self.workbook.add_worksheet();

    // Ensure that each Series in the DataFrame is a single chunk
    let columns: Vec<Column> = df
      .columns()
      .into_iter()
      .map(|series| Self::rechunk_series(series))
      .collect();

    let rechunk_df = DataFrame::new_infer_height(columns)?;

    // write headers to xlsx
    let headers = rechunk_df.get_column_names();
    for (col, col_name) in headers.iter().enumerate() {
      worksheet.write_string(0, col.try_into()?, col_name.to_string())?;
    }

    let format = Format::new().set_num_format("0.00");

    let num_rows = rechunk_df.height();
    let columns = rechunk_df.columns();

    for row in 0..num_rows {
      for (col, series) in columns.iter().enumerate() {
        let col_data = series.get(row)?;

        match col_data {
          AnyValue::Float64(val) => {
            let decimal_values = rust_decimal::Decimal::from_f64(val)
              .unwrap_or(rust_decimal::Decimal::new(0, 0))
              .round_dp(2)
              .to_f64()
              .unwrap_or(0.0);
            worksheet.write_number_with_format(
              (row + 1).try_into()?,
              col.try_into()?,
              decimal_values,
              &format,
            )?;
          }
          AnyValue::Float32(val) => {
            let decimal_values = rust_decimal::Decimal::from_f32(val)
              .unwrap_or(rust_decimal::Decimal::new(0, 0))
              .round_dp(2)
              .to_f32()
              .unwrap_or(0.0);
            worksheet.write_number_with_format(
              (row + 1).try_into()?,
              col.try_into()?,
              decimal_values,
              &format,
            )?;
          }
          AnyValue::String(val) => {
            worksheet.write_string((row + 1).try_into()?, col.try_into()?, val)?;
          }
          AnyValue::Int64(val) => {
            worksheet.write_number((row + 1).try_into()?, col.try_into()?, val as f64)?;
          }
          AnyValue::Int32(val) => {
            worksheet.write_number((row + 1).try_into()?, col.try_into()?, val as f64)?;
          }
          AnyValue::Int16(val) => {
            worksheet.write_number((row + 1).try_into()?, col.try_into()?, val as f64)?;
          }
          AnyValue::Int8(val) => {
            worksheet.write_number((row + 1).try_into()?, col.try_into()?, val as f64)?;
          }
          AnyValue::UInt32(val) => {
            worksheet.write_string((row + 1).try_into()?, col.try_into()?, val.to_string())?;
          }
          AnyValue::UInt16(val) => {
            worksheet.write_string((row + 1).try_into()?, col.try_into()?, val.to_string())?;
          }
          AnyValue::UInt8(val) => {
            worksheet.write_string((row + 1).try_into()?, col.try_into()?, val.to_string())?;
          }
          _ => {
            worksheet.write_blank((row + 1).try_into()?, col.try_into()?, &format)?;
          }
        }
      }
    }

    Ok(self.workbook.save(output_path)?)
  }

  fn rechunk_series(series: &Column) -> Column {
    if series.n_chunks() > 1 {
      series.rechunk()
    } else {
      series.clone()
    }
  }

  fn write_chunk<'a>(
    chunk: &mut Vec<StringRecord>,
    headers: &StringRecord,
    workbook: &mut Workbook,
  ) -> Result<()> {
    let worksheet = workbook.add_worksheet();

    for (col, col_name) in headers.iter().enumerate() {
      worksheet.write_string(0, col.try_into()?, col_name.to_string())?;
    }
    for (row, row_value) in chunk.iter().enumerate() {
      for (col, col_value) in row_value.iter().enumerate() {
        worksheet.write_string(
          (row + 1).try_into()?,
          col.try_into()?,
          col_value.to_string(),
        )?;
      }
    }

    chunk.clear();
    Ok(())
  }

  /// Splits CSV into multiple XLSX files, each with up to `chunk_size` rows.
  pub fn write_xlsx_split<P: AsRef<Path>>(
    &mut self,
    mut rdr: Reader<BufReader<Box<dyn Read + Send>>>,
    chunk_size: usize,
    output: P,
  ) -> Result<()> {
    let headers = rdr.headers()?.clone();
    let output = output.as_ref();

    let mut chunk: Vec<StringRecord> = Vec::with_capacity(chunk_size);
    let mut file_index = 0;

    for result in rdr.records() {
      let record = result?;
      chunk.push(record);

      if chunk.len() >= chunk_size {
        // Build output path like "output_0.xlsx"
        let parent = output.parent().unwrap_or(Path::new("."));
        let stem = output.file_stem().unwrap_or_default();
        let filename = format!("{}_{}.xlsx", stem.to_string_lossy(), file_index);
        let output_path = parent.join(filename);

        Self::write_chunk_to_file(&chunk, &headers, output_path)?;

        chunk.clear();
        file_index += 1;
      }
    }

    if !chunk.is_empty() {
      let parent = output.parent().unwrap_or(Path::new("."));
      let stem = output.file_stem().unwrap_or_default();
      let filename = format!("{}_{}.xlsx", stem.to_string_lossy(), file_index);
      let output_path = parent.join(filename);

      Self::write_chunk_to_file(&chunk, &headers, output_path)?;
    }

    Ok(())
  }

  /// Helper to write one chunk to a single XLSX file (kept minimal)
  fn write_chunk_to_file(
    chunk: &[StringRecord],
    headers: &StringRecord,
    output_path: PathBuf,
  ) -> Result<()> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    for (col, col_name) in headers.iter().enumerate() {
      worksheet.write_string(0, col.try_into()?, col_name.to_string())?;
    }

    for (row, row_value) in chunk.iter().enumerate() {
      for (col, col_value) in row_value.iter().enumerate() {
        worksheet.write_string(
          (row + 1).try_into()?,
          col.try_into()?,
          col_value.to_string(),
        )?;
      }
    }

    Ok(workbook.save(output_path)?)
  }
}
