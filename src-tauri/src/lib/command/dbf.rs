use std::{path::Path, time::Instant};

use anyhow::Result;
use tauri::Emitter;

async fn dbf_to_csv(file_path: String, sep: String, window: tauri::Window) -> Result<()> {
  let sep = if sep == "\\t" {
    b'\t'
  } else {
    sep.into_bytes()[0]
  };

  let vec_path: Vec<&str> = file_path.split('|').collect();
  let parent_path = Path::new(&vec_path[0])
    .parent()
    .map(|path| path.to_string_lossy())
    .unwrap();

  for fp in vec_path.iter() {
    window.emit("start_convert", fp)?;

    let mut reader = dbase::Reader::from_path(fp)?;

    let headers: Vec<String> = reader
      .fields()
      .iter()
      .map(|field| field.name().to_string())
      .collect();

    let file_name = Path::new(&fp).file_stem().unwrap().to_str().unwrap();
    let output = format!("{parent_path}/{file_name}.dbf.csv");

    let mut wtr = csv::WriterBuilder::new().delimiter(sep).from_path(output)?;

    wtr.write_record(&headers)?;

    for record_result in reader.iter_records() {
      let record = record_result?;
      let mut row: Vec<String> = Vec::new();

      for field_name in &headers {
        let value = match record.get(field_name.as_str()) {
          Some(dbase::FieldValue::Character(Some(value))) => value.trim().to_string(),
          Some(dbase::FieldValue::Character(None)) => "".to_string(),
          Some(dbase::FieldValue::Date(Some(value))) => value.to_string(),
          Some(dbase::FieldValue::Date(None)) => "".to_string(),
          Some(dbase::FieldValue::Float(Some(value))) => value.to_string(),
          Some(dbase::FieldValue::Float(None)) => "".to_string(),
          Some(dbase::FieldValue::Logical(Some(value))) => value.to_string(),
          Some(dbase::FieldValue::Logical(None)) => "".to_string(),
          Some(dbase::FieldValue::Numeric(Some(value))) => value.to_string(),
          Some(dbase::FieldValue::Numeric(None)) => "".to_string(),
          Some(dbase::FieldValue::Memo(value)) => value.to_string(),
          Some(dbase::FieldValue::Integer(value)) => value.to_string(),
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
    Err(err) => Err(format!("dbf failed: {err}")),
  }
}
