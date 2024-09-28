use std::{error::Error, path::Path, time::Instant};

fn drop_headers(
  file_path: String,
  sep: String,
  window: tauri::Window,
) -> Result<(), Box<dyn Error>> {
  let sep = if sep == "\\t" {
    b'\t'
  } else {
    sep.into_bytes()[0]
  };

  let vec_path: Vec<&str> = file_path.split('|').collect();
  let parent_path = Path::new(&vec_path[0])
    .parent()
    .map(|parent| parent.to_string_lossy())
    .unwrap();

  let mut count: usize = 0;
  let file_len = vec_path.len();

  for fp in vec_path.iter() {
    window.emit("start_convert", fp)?;

    let filename = Path::new(fp).file_name().unwrap().to_str().unwrap();
    let output_path = format!("{}/{}_behead.csv", parent_path, filename);

    let mut rdr = csv::ReaderBuilder::new()
      .delimiter(sep)
      .has_headers(true)
      .from_path(fp)?;

    let mut wtr = csv::WriterBuilder::new()
      .delimiter(sep)
      .has_headers(false)
      .flexible(true)
      .from_path(output_path)?;

    let mut record = csv::ByteRecord::new();

    while rdr.read_byte_record(&mut record)? {
      wtr.write_byte_record(&record)?;
    }
    wtr.flush()?;

    window.emit("drop_msg", fp)?;

    count += 1;
    let progress = ((count as f32) / (file_len as f32)) * 100.0;
    let drop_progress = format!("{progress:.0}");
    window.emit("drop_progress", drop_progress)?;
  }

  Ok(())
}

#[tauri::command]
pub async fn behead(file_path: String, sep: String, window: tauri::Window) {
  let start_time = Instant::now();
  let drop_window = window.clone();

  match (async { drop_headers(file_path, sep, drop_window) }).await {
    Ok(result) => result,
    Err(err) => {
      eprintln!("behead error: {err}");
      window.emit("behead_err", &err.to_string()).unwrap();
      err.to_string();
    }
  };

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  let runtime = format!("{elapsed_time:.2} s");
  window.emit("runtime", runtime).unwrap();
}
