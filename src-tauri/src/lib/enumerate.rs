use std::{fs::File, io::BufWriter, path::Path, time::Instant};

use anyhow::Result;

use crate::detect::detect_separator;

async fn add_index(file_path: String) -> Result<()> {
  let sep = match detect_separator(file_path.as_str(), 0) {
    Some(separator) => separator as u8,
    None => b',',
  };

  let parent_path = Path::new(&file_path)
    .parent()
    .map(|parent| parent.to_string_lossy())
    .unwrap();
  let file_name = Path::new(&file_path).file_stem().unwrap().to_str().unwrap();
  let output_path = format!("{}/{}.enumerate.csv", parent_path, file_name);

  let mut rdr = csv::ReaderBuilder::new()
    .delimiter(sep)
    .from_path(file_path)?;

  let buf_writer = BufWriter::with_capacity(256_000, File::create(output_path)?);
  let mut wtr = csv::WriterBuilder::new()
    .delimiter(sep)
    .from_writer(buf_writer);

  let headers = rdr.headers()?;
  let mut new_headers = vec![String::from("unique_index")];
  new_headers.extend(headers.into_iter().map(String::from));
  wtr.write_record(&new_headers)?;

  for (i, result) in rdr.records().enumerate() {
    let record = result?;
    let mut new_record = vec![i.to_string()];
    new_record.extend(record.iter().map(|field| field.to_string()));
    wtr.write_record(&new_record)?;
  }

  Ok(wtr.flush()?)
}

#[tauri::command]
pub async fn enumer(file_path: String) -> Result<String, String> {
  let start_time = Instant::now();

  match add_index(file_path).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("enumerate failed: {err}")),
  }
}
