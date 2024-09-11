use std::{error::Error, path::PathBuf};

use polars::{datatypes::AnyValue, frame::DataFrame};

pub fn write_xlsx(df: DataFrame, output_path: PathBuf) -> Result<(), Box<dyn Error>> {
  /* write dataframe to xlsx */
  let mut workbook = rust_xlsxwriter::Workbook::new();
  let worksheet = workbook.add_worksheet();

  // write headers to xlsx
  let headers = df.get_column_names();
  for (col, col_name) in headers.iter().enumerate() {
    worksheet.write_string(0, col.try_into()?, col_name.to_string())?;
  }

  // write data to xlsx
  for (row, row_data) in df.iter().enumerate() {
    for (col, col_data) in row_data.iter().enumerate() {
      match col_data {
        AnyValue::Float64(values) => {
          worksheet.write_number((col + 1).try_into()?, row.try_into()?, values)?;
        }
        AnyValue::Float32(values) => {
          worksheet.write_string((col + 1).try_into()?, row.try_into()?, values.to_string())?;
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
  workbook.save(output_path)?;

  Ok(())
}
