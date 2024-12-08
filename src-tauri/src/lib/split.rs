use std::{
  error::Error,
  fs::File,
  io::BufWriter,
  path::{Path, PathBuf},
  time::Instant,
};

use crate::detect::detect_separator;

fn new_writer(
  headers: &csv::ByteRecord,
  start: i32,
  output: &Path,
  sep: u8,
) -> Result<csv::Writer<BufWriter<File>>, Box<dyn Error>> {
  let dir = Path::new(output);
  let path = dir.join(&format!("split_{}.csv", start));
  let spath = path.display().to_string();

  let mut wtr = csv::WriterBuilder::new()
    .delimiter(sep)
    .from_writer(BufWriter::new(File::create(spath)?));
  wtr.write_record(headers)?;

  Ok(wtr)
}

async fn split_csv(file_path: String, size: u32) -> Result<(), Box<dyn Error>> {
  let sep = match detect_separator(&file_path.as_str()) {
    Some(separator) => separator as u8,
    None => b',',
  };

  let binding = PathBuf::from(&file_path);
  let path_parent = binding.parent().unwrap();

  let mut rdr = csv::ReaderBuilder::new()
    .delimiter(sep)
    .has_headers(true)
    .from_reader(File::open(&file_path)?);

  let headers = rdr.byte_headers()?.clone();

  let mut wtr = new_writer(&headers, 0, path_parent, sep)?;
  let mut i = 0;
  let mut cnt = 1;
  let mut row = csv::ByteRecord::new();
  while rdr.read_byte_record(&mut row)? {
    if i > 0 && i % size == 0 {
      wtr.flush()?;
      wtr = new_writer(&headers, cnt, path_parent, sep)?;
      cnt += 1;
    }
    wtr.write_byte_record(&row)?;
    i += 1;
  }
  wtr.flush()?;

  Ok(())
}

#[tauri::command]
pub async fn split(file_path: String, size: u32) -> Result<String, String> {
  let start_time = Instant::now();

  match split_csv(file_path, size).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      let runtime = format!("{elapsed_time:.2}");
      Ok(runtime)
    }
    Err(err) => Err(format!("split failed: {err}")),
  }
}

/// for integration test
pub async fn public_split_csv(file_path: String, size: u32) -> Result<(), Box<dyn Error>> {
  split_csv(file_path, size).await
}
