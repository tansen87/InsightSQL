use std::{
  collections::{BTreeMap, HashMap},
  fs::File,
  io::BufWriter,
  path::Path,
  time::Instant,
};

use anyhow::{anyhow, Result};
use csv::{ReaderBuilder, WriterBuilder};

use crate::utils::CsvOptions;

async fn get_header<P: AsRef<Path>>(
  path: P,
  skip_rows: String,
) -> Result<Vec<HashMap<String, String>>> {
  let mut csv_options = CsvOptions::new(&path);
  csv_options.set_skip_rows(skip_rows.parse::<usize>()?);

  let sep = match csv_options.detect_separator() {
    Some(separator) => separator as u8,
    None => b',',
  };

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .has_headers(true)
    .from_reader(csv_options.skip_csv_rows()?);

  let vec_headers: Vec<String> = rdr.headers()?.iter().map(|h| h.to_string()).collect();

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

pub async fn select_columns<P: AsRef<Path>>(
  path: P,
  cols: String,
  skip_rows: String,
) -> Result<()> {
  let mut csv_options = CsvOptions::new(&path);
  csv_options.set_skip_rows(skip_rows.parse::<usize>()?);

  let sep = match csv_options.detect_separator() {
    Some(separator) => separator as u8,
    None => b',',
  };

  let cols_select: Vec<&str> = cols.split('|').collect();

  let parent_path = path.as_ref().parent().unwrap().to_str().unwrap();
  let file_name = path.as_ref().file_stem().unwrap().to_str().unwrap();
  let output_path = format!("{parent_path}/{file_name}.select.csv");

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(csv_options.skip_csv_rows()?);

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

  let mut wtr = WriterBuilder::new()
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
pub async fn get_select_headers(
  path: String,
  skip_rows: String,
) -> Result<Vec<HashMap<String, String>>, String> {
  match get_header(path, skip_rows).await {
    Ok(result) => Ok(result),
    Err(err) => Err(format!("{err}")),
  }
}

#[tauri::command]
pub async fn select(path: String, cols: String, skip_rows: String) -> Result<String, String> {
  let start_time = Instant::now();

  match select_columns(path, cols, skip_rows).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("{err}")),
  }
}
