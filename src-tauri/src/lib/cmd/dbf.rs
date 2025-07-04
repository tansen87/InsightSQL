use std::{path::Path, time::Instant};

use anyhow::Result;
use dbase::{Reader, FieldValue};
use tauri::Emitter;

async fn dbf_to_csv(path: String, sep: String, window: tauri::Window) -> Result<()> {
  let sep = if sep == "\\t" {
    b'\t'
  } else {
    sep.into_bytes()[0]
  };

  let vec_path: Vec<&str> = path.split('|').collect();
  let parent_path = Path::new(&vec_path[0]).parent().unwrap().to_str().unwrap();

  for fp in vec_path.iter() {
    window.emit("start_convert", fp)?;

    let mut rdr = Reader::from_path(fp)?;

    let headers: Vec<String> = rdr
      .fields()
      .iter()
      .map(|field| field.name().to_string())
      .collect();

    let file_stem = Path::new(&fp).file_stem().unwrap().to_str().unwrap();
    let output = format!("{parent_path}/{file_stem}.dbf.csv");

    let mut wtr = csv::WriterBuilder::new().delimiter(sep).from_path(output)?;

    wtr.write_record(&headers)?;

    for record_result in rdr.iter_records() {
      let record = record_result?;
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

    window.emit("dbf2csv_msg", fp)?;
  }

  Ok(())
}

#[tauri::command]
pub async fn dbf(file_path: String, sep: String, window: tauri::Window) -> Result<String, String> {
  let start_time = Instant::now();

  match dbf_to_csv(file_path, sep, window).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("{err}")),
  }
}
