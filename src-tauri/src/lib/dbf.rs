use std::{error::Error, path::Path, time::Instant};

use csv::WriterBuilder;
use dbase::Reader;

fn dbf_to_csv(file_path: String, sep: String, window: tauri::Window) -> Result<(), Box<dyn Error>> {
  let mut separator = Vec::new();
  let sep = if sep == "\\t" {
    b'\t'
  } else {
    sep.into_bytes()[0]
  };
  separator.push(sep);

  let vec_path: Vec<&str> = file_path.split('|').collect();
  let parent_path = Path::new(&vec_path[0])
    .parent()
    .map(|parent| parent.to_string_lossy())
    .unwrap_or_else(|| "Default Path".to_string().into());

  let mut count: usize = 0;
  let file_len = vec_path.len();

  for fp in vec_path.iter() {
    let start_convert = format!("{}|start", fp);
    window.emit("start_convert", start_convert)?;

    let mut reader = Reader::from_path(fp)?;

    let headers: Vec<String> = reader
      .fields()
      .iter()
      .map(|field| field.name().to_string())
      .collect();

    let file_name = match Path::new(&fp).file_name() {
      Some(name) => match name.to_str() {
        Some(name_str) => name_str.split('|').collect::<Vec<&str>>(),
        None => vec![],
      },
      None => vec![],
    };
    let output = format!("{}/{}.csv", parent_path, file_name[0]);

    let mut wtr = WriterBuilder::new()
      .delimiter(separator[0])
      .from_path(output)?;

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

    count += 1;
    let progress = ((count as f32) / (file_len as f32)) * 100.0;
    let progress_s = format!("{progress:.0}");
    window.emit("dbf2csv_progress", progress_s)?;

    let dbf2csv_msg = format!("{}", fp);
    window.emit("dbf2csv_msg", dbf2csv_msg)?;
  }

  Ok(())
}

#[tauri::command]
pub async fn dbf(file_path: String, sep: String, window: tauri::Window) {
  let start_time = Instant::now();
  let dbf_window = window.clone();

  match (async { dbf_to_csv(file_path, sep, dbf_window) }).await {
    Ok(result) => result,
    Err(error) => {
      eprintln!("dbf2csv error: {error}");
      window.emit("dbf2csv_err", &error.to_string()).unwrap();
      error.to_string();
    }
  };

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  let runtime = format!("{elapsed_time:.2} s");
  window.emit("runtime", runtime).unwrap();
}
