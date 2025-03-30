use std::{fs::File, io::BufWriter, path::Path, time::Instant};

use anyhow::Result;
use csv::{ReaderBuilder, WriterBuilder};

use crate::utils::CsvOptions;

pub async fn add_index<P: AsRef<Path> + Send + Sync>(path: P) -> Result<()> {
  let csv_options = CsvOptions::new(&path);
  let sep = csv_options.detect_separator()?;

  let parent_path = path.as_ref().parent().unwrap().to_str().unwrap();
  let file_stem = path.as_ref().file_stem().unwrap().to_str().unwrap();
  let output_path = format!("{parent_path}/{file_stem}.enumerate.csv");

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(csv_options.skip_csv_rows()?);

  let buf_writer = BufWriter::with_capacity(256_000, File::create(output_path)?);
  let mut wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_writer);

  let headers = rdr.headers()?;
  let mut new_headers = vec![String::from("enumerate_idx")];
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
pub async fn enumer(path: String) -> Result<String, String> {
  let start_time = Instant::now();

  match add_index(path).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("{err}")),
  }
}
