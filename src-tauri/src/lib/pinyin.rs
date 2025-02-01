use std::{fs::File, io::BufWriter, path::Path, time::Instant};

use anyhow::Result;
use csv::{ReaderBuilder, WriterBuilder};
use pinyin::ToPinyin;

use crate::utils::{CsvOptions, Selection};

pub async fn chinese_to_pinyin<P: AsRef<Path>>(
  path: P,
  columns: String,
  skip_rows: String,
) -> Result<()> {
  let mut csv_options = CsvOptions::new(&path);
  csv_options.set_skip_rows(skip_rows.parse::<usize>()?);

  let sep = match csv_options.detect_separator() {
    Some(separator) => separator as u8,
    None => b',',
  };

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(csv_options.skip_csv_rows()?);

  let cols: Vec<&str> = columns.split('|').collect();
  let sel = Selection::from_headers(rdr.byte_headers()?, &cols[..])?;

  let parent_path = path.as_ref().parent().unwrap().to_str().unwrap();
  let file_name = path.as_ref().file_stem().unwrap().to_str().unwrap();
  let output_path = format!("{parent_path}/{file_name}.pinyin.csv");

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

  Ok(wtr.flush()?)
}

#[tauri::command]
pub async fn pinyin(path: String, columns: String, skip_rows: String) -> Result<String, String> {
  let start_time = Instant::now();

  match chinese_to_pinyin(path, columns, skip_rows).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("pinyin failed: {err}")),
  }
}
