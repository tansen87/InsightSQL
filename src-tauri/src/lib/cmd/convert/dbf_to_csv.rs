use std::path::{Path, PathBuf};

use anyhow::Result;
use csv::WriterBuilder;
use dbase::FieldValue;

/// convert dbf to csv
pub async fn dbf_to_csv(path: &str, sep: String) -> Result<()> {
  let sep = if sep == "\\t" {
    b'\t'
  } else {
    sep.into_bytes()[0]
  };
  let parent_path = Path::new(path).parent().unwrap().to_str().unwrap();
  let file_stem = Path::new(path).file_stem().unwrap().to_str().unwrap();
  let mut output_path = PathBuf::from(parent_path);
  output_path.push(format!("{file_stem}.dbf.csv"));

  let mut rdr = dbase::Reader::from_path(path)?;

  let headers: Vec<String> = rdr
    .fields()
    .iter()
    .map(|field| field.name().to_string())
    .collect();

  let mut wtr = WriterBuilder::new().delimiter(sep).from_path(output_path)?;
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
  wtr.flush()?;

  Ok(())
}
