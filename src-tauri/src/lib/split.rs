use std::{
  error::Error,
  fs::File,
  io::BufWriter,
  path::{Path, PathBuf}, time::Instant,
};

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

fn split_csv(input_csv: String, sep: String, size: i32) -> Result<(), Box<dyn Error>> {
  let sep = if sep == "\\t" {
    b'\t'
  } else {
    sep.into_bytes()[0]
  };

  let binding = PathBuf::from(input_csv.clone());
  let path_parent = binding.parent().unwrap();

  let mut rdr = csv::ReaderBuilder::new()
    .delimiter(sep)
    .has_headers(true)
    .from_reader(File::open(input_csv)?);

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
pub async fn split(file_path: String, sep: String, size: i32, window: tauri::Window) {
  let start_time = Instant::now();

  match (async { split_csv(file_path, sep, size) }).await {
    Ok(result) => result,
    Err(err) => {
      eprintln!("split error: {err}");
      window.emit("split_err", &err.to_string()).unwrap();
      err.to_string();
    }
  };

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  let runtime = format!("{elapsed_time:.2} s");
  window.emit("runtime", runtime).unwrap();
}
