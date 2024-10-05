use std::{
  collections::{BTreeMap, HashMap},
  error::Error,
  fs::File,
  io::BufWriter,
  path::Path,
  time::Instant,
};

use tauri::Emitter;

fn get_header(path: &str, sep: String) -> Result<Vec<HashMap<String, String>>, Box<dyn Error>> {
  let sep = if sep == "\\t" {
    b'\t'
  } else {
    sep.into_bytes()[0]
  };

  let mut rdr = csv::ReaderBuilder::new()
    .delimiter(sep)
    .has_headers(true)
    .from_reader(File::open(path)?);

  let headers = rdr.headers()?.clone();
  let vec_headers: Vec<String> = headers.iter().map(|h| h.to_string()).collect();

  let hs = vec_headers
    .into_iter()
    .enumerate()
    .map(|(index, name)| {
      let mut map = HashMap::new();
      map.insert("name".to_string(), name);
      map.insert("id".to_string(), index.to_string());
      map
    })
    .collect();

  Ok(hs)
}

fn select_columns(
  path: String,
  sep: String,
  cols: String,
  window: tauri::Window,
) -> Result<(), Box<dyn std::error::Error>> {
  let sep = if sep == "\\t" {
    b'\t'
  } else {
    sep.into_bytes()[0]
  };

  let cols_cleaned: String = cols.replace("\r", "").replace("\n", "");
  let cols_select: Vec<&str> = cols_cleaned.split('|').collect();
  let vec_path: Vec<&str> = path.split('|').collect();

  for f in vec_path.iter() {
    let file_path = Path::new(&f);
    let file_name = match file_path.file_name() {
      Some(name) => match name.to_str() {
        Some(name_str) => name_str.split('.').collect::<Vec<&str>>(),
        None => vec![],
      },
      None => vec![],
    };
    let current_time = chrono::Local::now().format("%Y-%m-%d-%H%M%S");
    let parent_path = file_path
      .parent()
      .map(|parent| parent.to_string_lossy())
      .unwrap_or_else(|| "Default Path".to_string().into());
    let output_path = format!(
      "{}/{}_select_{}.csv",
      parent_path, file_name[0], current_time
    );

    let mut rdr = csv::ReaderBuilder::new()
      .delimiter(sep)
      .has_headers(true)
      .from_reader(File::open(f)?);

    let headers = rdr.headers()?.clone();

    // 遍历header以查找所选列的索引
    let mut col_indices: BTreeMap<&str, usize> = BTreeMap::new();
    let mut idx = 0;
    for header in headers.iter() {
      if cols_select.contains(&header) {
        col_indices.insert(header, idx);
      }
      idx += 1;
    }

    // 创建一个向量来存储按照cols_select顺序排列的索引值
    let mut vec_indices = Vec::new();
    for col_select in cols_select.iter() {
      if let Some(&index) = col_indices.get(col_select) {
        vec_indices.push(index);
      }
    }

    let mut wtr = csv::WriterBuilder::new()
      .delimiter(sep)
      .from_writer(BufWriter::new(File::create(output_path)?));

    wtr.write_record(cols_select.iter())?;
    let mut record = csv::ByteRecord::new();

    while rdr.read_byte_record(&mut record)? {
      match wtr.write_record(vec_indices.iter().map(|&i| &record[i])) {
        Ok(()) => (),
        Err(e) => {
          let wtr_msg = format!("{}.{}|error|{}", file_name[0], file_name[1], e);
          window.emit("wtr_err", wtr_msg)?;
          break;
        }
      }
    }

    wtr.flush()?;
  }

  Ok(())
}

#[tauri::command]
pub async fn get_select_headers(
  path: String,
  sep: String,
  window: tauri::Window,
) -> Vec<HashMap<String, String>> {
  let headers = match (async { get_header(path.as_str(), sep) }).await {
    Ok(result) => result,
    Err(err) => {
      eprintln!("get headers error: {err}");
      window.emit("get_err", &err.to_string()).unwrap();
      return Vec::new();
    }
  };

  headers
}

#[tauri::command]
pub async fn select(path: String, sep: String, cols: String, window: tauri::Window) {
  let start_time = Instant::now();
  let sel_window = window.clone();

  match (async { select_columns(path, sep, cols, sel_window) }).await {
    Ok(result) => result,
    Err(err) => {
      eprintln!("select columns error: {err}");
      window.emit("select_err", &err.to_string()).unwrap();
    }
  }

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  let runtime = format!("{elapsed_time:.2} s");
  window.emit("runtime", runtime).unwrap();
}
