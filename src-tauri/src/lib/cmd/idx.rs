use std::{fs::File, io::BufWriter, path::{Path, PathBuf}, time::Instant};

use anyhow::Result;
use csv::ReaderBuilder;
use csv_index::RandomAccessSimple;

use crate::utils::CsvOptions;

pub async fn create_index<P: AsRef<Path> + Send + Sync>(path: P) -> Result<()> {
  let csv_options = CsvOptions::new(&path);
  let parent_path = path.as_ref().parent().unwrap().to_str().unwrap();
  let file_name = path.as_ref().file_name().unwrap().to_str().unwrap();
  let mut output_path = PathBuf::from(parent_path);
  output_path.push(format!("{file_name}.idx"));

  let mut rdr = ReaderBuilder::new()
    .delimiter(csv_options.detect_separator()?)
    .from_reader(File::open(&path)?);
  let mut wtr = BufWriter::new(File::create(output_path)?);
  RandomAccessSimple::create(&mut rdr, &mut wtr)?;

  Ok(())
}

#[tauri::command]
pub async fn idx(path: String) -> Result<String, String> {
  let start_time = Instant::now();

  match create_index(path).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("{err}")),
  }
}
