use std::{fs::File, io::BufWriter};

use anyhow::Result;
use csv::WriterBuilder;
use dbase::FieldValue;

use crate::io::csv::options::CsvOptions;

/// convert dbf to csv
pub async fn dbf_to_csv(path: &str, wtr_sep: String) -> Result<()> {
  let sep = if wtr_sep == "\\t" {
    b'\t'
  } else {
    wtr_sep.into_bytes()[0]
  };
  let opts = CsvOptions::new(path);
  let output_path = opts.output_path(Some("dbf"), None)?;

  let mut rdr = dbase::Reader::from_path(path)?;

  let headers: Vec<String> = rdr
    .fields()
    .iter()
    .map(|field| field.name().to_string())
    .collect();

  let buf_writer = BufWriter::with_capacity(256_000, File::create(output_path)?);
  let mut wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_writer);
  wtr.write_record(&headers)?;

  for result in rdr.iter_records() {
    let record = result?;
    let mut row: Vec<String> = Vec::new();

    for field_name in &headers {
      let value = match record.get(field_name.as_str()) {
        Some(FieldValue::Character(Some(value))) => value.trim().to_string(),
        Some(FieldValue::Character(None)) => "".to_string(),
        Some(FieldValue::Date(Some(value))) => value.to_string(),
        Some(FieldValue::Date(None)) => "".to_string(),
        Some(FieldValue::Float(Some(value))) => value.to_string(),
        Some(FieldValue::Float(None)) => "".to_string(),
        Some(FieldValue::Logical(Some(value))) => value.to_string(),
        Some(FieldValue::Logical(None)) => "".to_string(),
        Some(FieldValue::Numeric(Some(value))) => value.to_string(),
        Some(FieldValue::Numeric(None)) => "".to_string(),
        Some(FieldValue::Memo(value)) => value.to_string(),
        Some(FieldValue::Integer(value)) => value.to_string(),
        _ => "".to_string(),
      };
      row.push(value);
    }
    wtr.write_record(&row)?;
  }

  Ok(wtr.flush()?)
}
