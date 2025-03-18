use std::{collections::HashMap, fs::File, io::BufWriter, path::Path, time::Instant};

use anyhow::{anyhow, Result};
use csv::{ReaderBuilder, WriterBuilder};

use crate::utils::{CsvOptions, Selection};

pub async fn fill_null<P: AsRef<Path> + Send + Sync>(
  path: P,
  fill_column: String,
  fill_value: String,
  mode: &str,
) -> Result<()> {
  let csv_options = CsvOptions::new(&path);
  let sep = csv_options.detect_separator()?;

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(csv_options.skip_csv_rows()?);

  let fill_columns: Vec<&str> = fill_column.split('|').collect();
  let sel = Selection::from_headers(rdr.byte_headers()?, &fill_columns[..])?;

  let parent_path = path.as_ref().parent().unwrap().to_str().unwrap();
  let file_name = path.as_ref().file_stem().unwrap().to_str().unwrap();
  let output_path = format!("{}/{}.fill.csv", parent_path, file_name);

  let mut wtr = WriterBuilder::new()
    .delimiter(sep)
    .from_writer(BufWriter::new(File::create(output_path)?));

  wtr.write_record(rdr.headers()?)?;

  // initialize forward filled cache if needed
  let mut forward_fill_cache: HashMap<usize, String> = HashMap::new();

  for record in rdr.deserialize() {
    let mut row: Vec<String> = record?;
    for &index in sel.get_indices() {
      match mode {
        // Fill null values
        "fill" => {
          if row.get(index).map_or(true, |s| s.is_empty()) {
            row[index] = fill_value.clone();
          }
        }
        // Fill null values by propagating the last valid observation to next valid
        // just like `pandas.Series.ffill`
        "ffill" => {
          if row.get(index).map_or(true, |s| s.is_empty()) {
            if let Some(fill_val) = forward_fill_cache.get(&index) {
              row[index] = fill_val.clone();
            }
          } else {
            forward_fill_cache.insert(index, row[index].clone());
          }
        }
        _ => return Err(anyhow!("Not supported fill mode")),
      }
    }
    wtr.write_record(&row)?;
  }

  Ok(wtr.flush()?)
}

#[tauri::command]
pub async fn fill(
  path: String,
  columns: String,
  values: String,
  mode: String,
) -> Result<String, String> {
  let start_time = Instant::now();

  match fill_null(path, columns, values, mode.as_str()).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("{err}")),
  }
}
