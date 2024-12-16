use std::{
  collections::{BTreeMap, HashMap},
  fs::File,
  io::BufWriter,
  path::Path,
  time::Instant,
};

use anyhow::{anyhow, Result};

use crate::detect::detect_separator;

async fn get_header(path: String) -> Result<Vec<HashMap<String, String>>> {
  let sep = match detect_separator(path.as_str()) {
    Some(separator) => separator as u8,
    None => b',',
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

async fn select_columns(path: String, cols: String) -> Result<()> {
  let sep = match detect_separator(path.as_str()) {
    Some(separator) => separator as u8,
    None => b',',
  };

  let cols_cleaned: String = cols.replace("\r", "").replace("\n", "");
  let cols_select: Vec<&str> = cols_cleaned.split('|').collect();

  let file_path = Path::new(&path);
  let file_name = file_path.file_stem().unwrap().to_str().unwrap();
  let current_time = chrono::Local::now().format("%Y-%m-%d-%H%M%S");
  let parent_path = file_path
    .parent()
    .map(|parent| parent.to_string_lossy())
    .unwrap();
  let output_path = format!("{}/{}.select_{}.csv", parent_path, file_name, current_time);

  let mut rdr = csv::ReaderBuilder::new()
    .delimiter(sep)
    .has_headers(true)
    .from_reader(File::open(&path)?);

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
      Err(err) => {
        return Err(anyhow!("write to csv error: {err}"));
      }
    }
  }

  Ok(wtr.flush()?)
}

#[tauri::command]
pub async fn get_select_headers(path: String) -> Result<Vec<HashMap<String, String>>, String> {
  match get_header(path).await {
    Ok(result) => Ok(result),
    Err(err) => Err(format!("get header error: {err}")),
  }
}

#[tauri::command]
pub async fn select(path: String, cols: String) -> Result<String, String> {
  let start_time = Instant::now();

  match select_columns(path, cols).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      let runtime = format!("{elapsed_time:.2}");
      Ok(runtime)
    }
    Err(err) => Err(format!("Select failed: {err}")),
  }
}

/// for integration test
pub async fn public_select(file_path: String, cols: String) -> Result<()> {
  select_columns(file_path, cols).await
}