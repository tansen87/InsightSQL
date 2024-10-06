use std::{error::Error, fs::File, io::BufWriter, path::Path, time::Instant};

use tauri::Emitter;

fn add_index(file_path: String, sep: String) -> Result<(), Box<dyn Error>> {
  let sep = if sep == "\\t" {
    b'\t'
  } else {
    sep.into_bytes()[0]
  };

  let current_time = chrono::Local::now().format("%Y-%m-%d-%H%M%S");
  let parent_path = Path::new(&file_path)
    .parent()
    .map(|parent| parent.to_string_lossy())
    .unwrap();
  let output_path = format!("{}/index_{}.csv", parent_path, current_time);

  let mut rdr = csv::ReaderBuilder::new()
    .delimiter(sep)
    .from_path(file_path)?;

  let buf_writer = BufWriter::with_capacity(256_000, File::create(output_path)?);
  let mut wtr = csv::WriterBuilder::new()
    .delimiter(sep)
    .from_writer(buf_writer);

  let headers = rdr.headers()?;
  let mut new_headers = vec![String::from("index_unique")];
  new_headers.extend(headers.into_iter().map(String::from));
  wtr.write_record(&new_headers)?;

  for (i, result) in rdr.records().enumerate() {
    let record = result?;
    let mut new_record = vec![i.to_string()];
    new_record.extend(record.iter().map(|field| field.to_string()));
    wtr.write_record(&new_record)?;
  }

  wtr.flush()?;

  Ok(())
}

#[tauri::command]
pub async fn index(file_path: String, sep: String, window: tauri::Window) {
  let start_time = Instant::now();

  match (async { add_index(file_path, sep) }).await {
    Ok(result) => result,
    Err(err) => {
      eprintln!("index error: {err}");
      window.emit("index_err", &err.to_string()).unwrap();
    }
  }

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  let runtime = format!("{elapsed_time:.2} s");
  window.emit("runtime", runtime).unwrap();
}
