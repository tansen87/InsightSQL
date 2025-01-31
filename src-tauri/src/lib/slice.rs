use std::{fs::File, io::BufWriter, path::Path, time::Instant};

use anyhow::Result;
use csv::{ReaderBuilder, WriterBuilder};

use crate::utils::{CsvOptions, Selection};

pub async fn slice_column_with_n_char<P: AsRef<Path>>(
  path: P,
  skip_rows: String,
  select_column: String,
  n: usize,
  mode: &str,
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

  let parent_path = &path.as_ref().parent().unwrap().to_str().unwrap();
  let file_name = &path.as_ref().file_stem().unwrap().to_str().unwrap();
  let output_path = format!("{parent_path}/{file_name}.slice.csv");

  let buf_writer = BufWriter::with_capacity(256_000, File::create(output_path)?);
  let mut wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_writer);

  let headers = rdr.headers()?.clone();

  let sel = Selection::from_headers(rdr.byte_headers()?, &[select_column.as_str()][..])?;

  let mut new_headers = headers.clone();
  let new_column_name = format!("{}_slice-n-char", select_column);
  new_headers.push_field(&new_column_name);

  wtr.write_record(&new_headers)?;

  for result in rdr.records() {
    let record = result?;

    if let Some(value) = record.get(sel.first_indices()?) {
      let slice_n = if mode == "left" {
        if value.len() >= n {
          &value[..n]
        } else {
          value
        }
      } else {
        if value.len() >= n {
          &value[value.len() - n..]
        } else {
          value
        }
      };

      let mut new_record = record.clone();
      new_record.push_field(slice_n);

      wtr.write_record(&new_record)?;
    }
  }

  Ok(wtr.flush()?)
}

#[tauri::command]
pub async fn slice(
  path: String,
  skip_rows: String,
  select_column: String,
  n: String,
  mode: String,
) -> Result<String, String> {
  let start_time = Instant::now();

  match slice_column_with_n_char(
    path,
    skip_rows,
    select_column,
    n.parse::<usize>().map_err(|e| e.to_string())?,
    mode.as_str(),
  )
  .await
  {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("slice failed: {err}")),
  }
}
