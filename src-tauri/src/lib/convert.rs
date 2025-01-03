use std::{
  fs::File,
  io::{BufRead, BufReader},
  path::PathBuf,
  time::Instant,
};

use anyhow::Result;
use calamine::{Data, HeaderRow, Range, Reader};
use polars::{
  io::SerReader,
  prelude::{CsvParseOptions, CsvReadOptions},
};
use rayon::{iter::ParallelIterator, slice::ParallelSlice};
use tauri::Emitter;

use crate::{detect::detect_separator, xlsx_writer::XlsxWriter};

async fn excel_to_csv(path: String, skip_rows: String, window: tauri::Window) -> Result<()> {
  /* convert excel to csv */
  let vec_path: Vec<&str> = path.split('|').collect();
  let mut count: usize = 0;
  let file_len = vec_path.len();

  for file in vec_path.iter() {
    window.emit("start_convert", file)?;

    let sce = PathBuf::from(file);
    let dest = sce.with_extension("csv");
    let mut wtr = csv::WriterBuilder::new().delimiter(b'|').from_path(dest)?;

    match calamine::open_workbook_auto(&sce) {
      Ok(mut workbook) => {
        let range = if let Some(result) = workbook
          .with_header_row(HeaderRow::Row(skip_rows.parse::<u32>()?))
          .worksheet_range_at(0)
        {
          result?
        } else {
          Range::empty()
        };

        let (row_count, col_count) = range.get_size();
        // check row count
        if row_count == 0 {
          window.emit(
            "row_count_err",
            format!("{file}|file is empty, skipping processing"),
          )?;
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
      }
      Err(err) => {
        window.emit("row_count_err", format!("{file}|{err}"))?;
        continue;
      }
    }

    window.emit("e2c_msg", file)?;

    count += 1;
    let progress = ((count as f32) / (file_len as f32)) * 100.0;
    window.emit("e2c_progress", format!("{progress:.0}"))?;
  }

  Ok(())
}

fn count_csv_rows(file_path: &str) -> Result<usize> {
  let file = File::open(file_path)?;
  let reader = BufReader::new(file);

  Ok(reader.lines().count())
}

async fn csv_to_xlsx(path: String, skip_rows: String, window: tauri::Window) -> Result<()> {
  /* csv to xlsx */
  let vec_path: Vec<&str> = path.split('|').collect();

  let mut count: usize = 0;
  let file_len = vec_path.len();

  for file in vec_path.iter() {
    window.emit("start_convert", file)?;

    let sep = match detect_separator(file, 0) {
      Some(separator) => separator as u8,
      None => b',',
    };

    let sce = PathBuf::from(file);
    let dest = sce.with_extension("xlsx");
    let row_count = count_csv_rows(file)?;

    if row_count < 104_0000 {
      let dfs = CsvReadOptions::default()
        .with_parse_options(
          CsvParseOptions::default()
            .with_separator(sep)
            .with_missing_is_null(false),
        )
        .with_skip_rows(skip_rows.parse::<usize>()?)
        .with_infer_schema_length(Some(0))
        .try_into_reader_with_file_path(Some(file.into()))
        .and_then(|reader| reader.finish());

      match dfs {
        Ok(df) => {
          if let Err(err) = XlsxWriter::new().write_xlsx(&df, dest) {
            window.emit("rows_err", format!("{file}|{err}"))?;
            return Ok(());
          }
          window.emit("c2x_msg", file)?;
        }
        Err(err) => {
          window.emit("rows_err", format!("{file}|{err}"))?;
        }
      }
    } else {
      window.emit(
        "rows_err",
        format!("{file}|{row_count} rows exceed the maximum row in Excel"),
      )?;
    }

    count += 1;
    let progress = ((count as f32) / (file_len as f32)) * 100.0;
    window.emit("c2x_progress", format!("{progress:.0}"))?;
  }

  Ok(())
}

#[tauri::command]
pub async fn switch_csv(
  path: String,
  skip_rows: String,
  window: tauri::Window,
) -> Result<String, String> {
  let start_time = Instant::now();
  let switch_csv_window = window.clone();

  match csv_to_xlsx(path, skip_rows, switch_csv_window).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      let runtime = format!("{elapsed_time:.2}");
      Ok(runtime)
    }
    Err(err) => Err(format!("csv to xlsx failed: {err}")),
  }
}

#[tauri::command]
pub async fn switch_excel(
  path: String,
  skip_rows: String,
  window: tauri::Window,
) -> Result<String, String> {
  let start_time = Instant::now();
  let switch_excel_window = window.clone();

  match excel_to_csv(path, skip_rows, switch_excel_window).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      let runtime = format!("{elapsed_time:.2}");
      Ok(runtime)
    }
    Err(err) => Err(format!("excel to csv failed: {err}")),
  }
}
