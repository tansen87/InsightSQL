use std::{collections::HashMap, fs::File, io::BufWriter, path::Path, time::Instant};

use anyhow::Result;
use csv::{ReaderBuilder, WriterBuilder};
use pinyin::ToPinyin;

use crate::detect::{detect_separator, Selection};

async fn get_header<P: AsRef<Path>>(path: P) -> Result<Vec<HashMap<String, String>>> {
  let sep = match detect_separator(&path, 0) {
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
    .map(|(_index, value)| {
      let mut map = HashMap::new();
      map.insert("value".to_string(), value.clone());
      map.insert("label".to_string(), value);
      map
    })
    .collect();

  Ok(hs)
}

async fn chinese_to_pinyin<P: AsRef<Path>>(path: P, columns: String) -> Result<()> {
  let sep = match detect_separator(&path, 0) {
    Some(separator) => separator as u8,
    None => b',',
  };

  let mut rdr = ReaderBuilder::new()
    .has_headers(true)
    .delimiter(sep)
    .from_path(&path)?;

  let cols: Vec<&str> = columns.split('|').collect();
  let sel = Selection::from_headers(rdr.byte_headers()?, &cols[..])?;

  let parent_path = &path
    .as_ref()
    .parent()
    .map(|path| path.to_string_lossy())
    .unwrap();
  let file_name = path.as_ref().file_stem().unwrap().to_str().unwrap();
  let output_path = format!("{}/{}.pinyin.csv", parent_path, file_name);
  let buf_writer = BufWriter::with_capacity(256_000, File::create(output_path)?);
  let mut wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_writer);

  let headers = rdr.headers()?;

  wtr.write_record(headers)?;

  for result in rdr.records() {
    let record = result?;

    let mut new_record = Vec::new();

    for (i, field) in record.iter().enumerate() {
      let mut new_field = String::from(field);

      if sel.get_indices().contains(&i) {
        new_field = new_field
          .chars()
          .map(|c| {
            c.to_pinyin()
              .map_or_else(|| c.into(), |py| py.plain().to_string().to_uppercase())
          })
          .collect::<String>();
      }

      new_record.push(new_field);
    }

    wtr.write_record(&new_record)?;
  }

  Ok(())
}

#[tauri::command]
pub async fn get_pinyin_headers(path: String) -> Result<Vec<HashMap<String, String>>, String> {
  match get_header(path).await {
    Ok(result) => Ok(result),
    Err(err) => Err(format!("get header error: {err}")),
  }
}

#[tauri::command]
pub async fn pinyin(path: String, columns: String) -> Result<String, String> {
  let start_time = Instant::now();

  match chinese_to_pinyin(path, columns).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("pinyin failed: {err}")),
  }
}

/// for integration test
pub async fn public_pinyin(path: String, columns: String) -> Result<()> {
  chinese_to_pinyin(path, columns).await
}
