use calamine::{Data, Range, Reader};
use polars::{
  io::SerReader,
  prelude::{CsvParseOptions, CsvReadOptions},
};
use rayon::prelude::*;
use std::{
  error::Error,
  path::{Path, PathBuf},
  time::Instant,
};
use tauri::Emitter;

use crate::{detect::detect_separator, xlsx_writer::write_xlsx};

fn excel_to_csv(path: String, window: tauri::Window) -> Result<(), Box<dyn Error>> {
  /* convert excel to csv */
  let vec_path: Vec<&str> = path.split('|').collect();
  let mut count: usize = 0;
  let file_len = vec_path.len();

  for file in vec_path.iter() {
    window.emit("start_convert", file)?;

    let sce = PathBuf::from(file);
    let dest = sce.with_extension("csv");
    let mut wtr = csv::WriterBuilder::new().delimiter(b'|').from_path(dest)?;

    let mut workbook = calamine::open_workbook_auto(&sce)?;
    let range = if let Some(result) = workbook.worksheet_range_at(0) {
      result?
    } else {
      Range::empty()
    };

    let (row_count, col_count) = range.get_size();
    // check row count
    if row_count == 0 {
      let warning_msg = format!("{file}| is empty, skipping processing.");
      window.emit("row_count_err", warning_msg)?;
      count += 1;
      continue;
    }

    let mut rows_iter = range.rows();

    // amortize allocations
    let mut record = csv::StringRecord::with_capacity(500, col_count);
    let mut col_name: String;

    // get the first row as header
    let first_row = match rows_iter.next() {
      Some(first_row) => first_row,
      None => &[Data::Empty],
    };

    for cell in first_row {
      col_name = match *cell {
        Data::String(ref s) => s.to_string(),
        Data::Empty => String::new(),
        Data::Error(ref _e) => String::new(),
        Data::Int(ref i) => i.to_string(),
        Data::Float(ref f) => f.to_string(),
        Data::DateTime(ref edt) => edt.to_string(),
        Data::Bool(ref b) => b.to_string(),
        Data::DateTimeIso(ref dt) => dt.to_string(),
        Data::DurationIso(ref d) => d.to_string(),
        // Data::Duration(ref d) => d.to_string(),
      };
      record.push_field(&col_name);
    }

    wtr.write_record(&record)?;

    let mut rows = Vec::with_capacity(row_count);
    // process rest of the rows
    for row in rows_iter {
      rows.push(row);
    }

    // set RAYON_NUM_THREADS
    let ncpus = 4;
    // set chunk_size to number of rows per core/thread
    // let chunk_size = row_count.div_ceil(ncpus);
    let chunk_size = (row_count + ncpus - 1) / ncpus;

    let processed_rows: Vec<Vec<csv::StringRecord>> = rows
      .par_chunks(chunk_size)
      .map(|chunk| {
        let mut record = csv::StringRecord::with_capacity(500, col_count);
        let mut float_val;
        let mut work_date = String::new();
        let mut ryu_buffer = ryu::Buffer::new();
        let mut itoa_buffer = itoa::Buffer::new();
        let mut formatted_date = String::new();
        let mut processed_chunk = Vec::with_capacity(chunk_size);

        for row in chunk {
          for cell in *row {
            match *cell {
              Data::Empty => record.push_field(""),
              Data::String(ref s) => record.push_field(s),
              Data::Int(ref i) => record.push_field(itoa_buffer.format(*i)),
              Data::Float(ref f) => {
                float_val = *f;

                #[allow(clippy::cast_precision_loss)]
                if float_val.fract().abs() > f64::EPSILON
                  || float_val > (i64::MAX as f64)
                  || float_val < (i64::MIN as f64)
                {
                  record.push_field(ryu_buffer.format_finite(float_val));
                } else {
                  record.push_field(itoa_buffer.format(float_val as i64));
                }
              }
              Data::DateTime(ref edt) => {
                if edt.is_datetime() {
                  work_date.clear();
                  if let Some(dt) = edt.as_datetime() {
                    formatted_date.clear();
                    work_date = dt.to_string();
                  }
                } else {
                  work_date = edt.as_duration().unwrap().to_string();
                }

                record.push_field(&work_date);
              }
              Data::Error(ref e) => record.push_field(&format!("{e:?}")),
              Data::Bool(ref b) => {
                record.push_field(if *b { "true" } else { "false" });
              }
              Data::DateTimeIso(ref dt) => record.push_field(dt),
              Data::DurationIso(ref d) => record.push_field(d),
            };
          }

          processed_chunk.push(record.clone());
          record.clear();
        }
        processed_chunk
      })
      .collect();

    for processed_chunk in processed_rows {
      for processed_row in processed_chunk {
        wtr.write_record(&processed_row)?;
      }
    }

    wtr.flush()?;

    window.emit("e2c_msg", file)?;

    count += 1;
    let progress = ((count as f32) / (file_len as f32)) * 100.0;
    let progress_s = format!("{progress:.0}");
    window.emit("e2c_progress", progress_s)?;
  }

  Ok(())
}

fn csv_to_xlsx(path: String, window: tauri::Window) -> Result<(), Box<dyn Error>> {
  /* csv to xlsx */
  let vec_path: Vec<&str> = path.split('|').collect();

  let mut count: usize = 0;
  let file_len = vec_path.len();

  for file in vec_path.iter() {
    window.emit("start_convert", file)?;

    let sep = match detect_separator(file) {
      Some(separator) => {
        let separator_u8: u8 = separator as u8;
        separator_u8
      }
      None => b',',
    };

    let file_name = match Path::new(&file).file_name() {
      Some(name) => match name.to_str() {
        Some(name_str) => name_str.split('|').collect::<Vec<&str>>(),
        None => vec![],
      },
      None => vec![],
    };

    let sce = PathBuf::from(file);
    let dest = sce.with_extension("xlsx");
    let df = CsvReadOptions::default()
      .with_parse_options(
        CsvParseOptions::default()
          .with_separator(sep)
          .with_missing_is_null(false),
      )
      .with_infer_schema_length(Some(0))
      .try_into_reader_with_file_path(Some(file.into()))?
      .finish()?;
    let rows = df.shape().0;
    if rows < 104_0000 {
      write_xlsx(df, dest)?;
      let c2x_msg = format!("{}", file);
      window.emit("c2x_msg", c2x_msg)?;
    } else {
      let rows_msg = format!(
        "{}.{}|rows:{}, cannot converted.",
        file_name[0], file_name[1], rows
      );
      window.emit("rows_err", rows_msg)?;
    }

    count += 1;
    let progress = ((count as f32) / (file_len as f32)) * 100.0;
    let progress_s = format!("{progress:.0}");
    window.emit("c2x_progress", progress_s)?;
  }

  Ok(())
}

#[tauri::command]
pub async fn switch_csv(path: String, window: tauri::Window) {
  let start_time = Instant::now();
  let copy_window = window.clone();

  match (async { csv_to_xlsx(path, copy_window) }).await {
    Ok(result) => result,
    Err(error) => {
      eprintln!("write_range error: {error}");
      window.emit("c2x_err", &error.to_string()).unwrap();
      error.to_string();
    }
  };

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  let runtime = format!("{elapsed_time:.2} s");
  window.emit("runtime", runtime).unwrap();
}

#[tauri::command]
pub async fn switch_excel(path: String, window: tauri::Window) {
  let start_time = Instant::now();
  let file_window = window.clone();

  match (async { excel_to_csv(path, file_window) }).await {
    Ok(result) => result,
    Err(error) => {
      eprintln!("write_range error: {error}");
      window.emit("e2c_err", &error.to_string()).unwrap();
      error.to_string();
    }
  };

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  let runtime = format!("{elapsed_time:.2} s");
  window.emit("runtime", runtime).unwrap();
}
