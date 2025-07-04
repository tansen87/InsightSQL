use std::{collections::BTreeMap, fs::File, io::BufWriter, path::Path, time::Instant};

use anyhow::{Result, anyhow};
use csv::{ReaderBuilder, WriterBuilder};

use crate::utils::CsvOptions;

pub async fn select_columns<P: AsRef<Path> + Send + Sync>(path: P, cols: String) -> Result<()> {
  let csv_options = CsvOptions::new(&path);
  let sep = csv_options.detect_separator()?;

  let cols_select: Vec<&str> = cols.split('|').collect();

  let parent_path = path.as_ref().parent().unwrap().to_str().unwrap();
  let file_stem = path.as_ref().file_stem().unwrap().to_str().unwrap();
  let output_path = format!("{parent_path}/{file_stem}.select.csv");

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
pub async fn select(path: String, cols: String) -> Result<String, String> {
  let start_time = Instant::now();

  match select_columns(path, cols).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("{err}")),
  }
}
