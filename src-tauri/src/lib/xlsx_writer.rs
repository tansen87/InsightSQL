use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use anyhow::Result;
use csv::{Reader, StringRecord};
use polars::{datatypes::AnyValue, frame::DataFrame, series::Series};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
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
    mut rdr: Reader<BufReader<File>>,
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
    let rechunk_df = df
      .iter()
      .map(|series| Self::rechunk_series(series))
      .collect::<Vec<_>>()
      .into_iter()
      .collect::<DataFrame>();

    // write headers to xlsx
    let headers = rechunk_df.get_column_names();
    for (col, col_name) in headers.iter().enumerate() {
      worksheet.write_string(0, col.try_into()?, col_name.to_string())?;
    }

    let format = Format::new().set_num_format("0.00");

    // write data to xlsx
    for (row, row_data) in rechunk_df.iter().enumerate() {
      for (col, col_data) in row_data.iter().enumerate() {
        match col_data {
          AnyValue::Float64(values) => {
            let decimal_values = Decimal::from_f64(values)
              .unwrap_or(Decimal::new(0, 0))
              .round_dp(2)
              .to_f64()
              .unwrap_or(0.0);
            worksheet.write_number_with_format(
              (col + 1).try_into()?,
              row.try_into()?,
              decimal_values,
              &format,
            )?;
          }
          AnyValue::Float32(values) => {
            let decimal_values = Decimal::from_f32(values)
              .unwrap_or(Decimal::new(0, 0))
              .round_dp(2)
              .to_f32()
              .unwrap_or(0.0);
            worksheet.write_number_with_format(
              (col + 1).try_into()?,
              row.try_into()?,
              decimal_values,
              &format,
            )?;
          }
          AnyValue::String(values) => {
            worksheet.write_string((col + 1).try_into()?, row.try_into()?, values)?;
          }
          AnyValue::Int64(values) => {
            worksheet.write_string((col + 1).try_into()?, row.try_into()?, values.to_string())?;
          }
          AnyValue::Int32(values) => {
            worksheet.write_string((col + 1).try_into()?, row.try_into()?, values.to_string())?;
          }
          AnyValue::Int16(values) => {
            worksheet.write_string((col + 1).try_into()?, row.try_into()?, values.to_string())?;
          }
          AnyValue::Int8(values) => {
            worksheet.write_string((col + 1).try_into()?, row.try_into()?, values.to_string())?;
          }
          AnyValue::UInt32(values) => {
            worksheet.write_string((col + 1).try_into()?, row.try_into()?, values.to_string())?;
          }
          AnyValue::UInt16(values) => {
            worksheet.write_string((col + 1).try_into()?, row.try_into()?, values.to_string())?;
          }
          AnyValue::UInt8(values) => {
            worksheet.write_string((col + 1).try_into()?, row.try_into()?, values.to_string())?;
          }
          _ => {}
        }
      }
    }

    Ok(self.workbook.save(output_path)?)
  }

  fn rechunk_series(series: &Series) -> Series {
    if series.chunks().len() > 1 {
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
}
