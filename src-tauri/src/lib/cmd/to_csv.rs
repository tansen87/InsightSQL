use std::{
  collections::{HashMap, HashSet},
  ffi::OsStr,
  path::{Path, PathBuf},
  sync::{Arc, Mutex},
  time::{Duration, Instant},
};

use anyhow::{Result, anyhow};
use calamine::{Data, HeaderRow, Range, Reader, open_workbook_auto};
use csv::{ByteRecord, ReaderBuilder, StringRecord, WriterBuilder};
use rayon::{iter::ParallelIterator, slice::ParallelSlice};
use tauri::{Emitter, Window};
use tokio::sync::oneshot;

use crate::utils::{CsvOptions, num_cpus};

/// convert excel to csv
async fn excel_to_csv<P: AsRef<Path>>(
  path: P,
  skip_rows: u32,
  sheet_name: Option<String>,
  output_path: &PathBuf,
) -> Result<()> {
  let mut wtr = WriterBuilder::new().from_path(output_path)?;

  let mut workbook = open_workbook_auto(&path)?;

  let range = match sheet_name {
    None => {
      match workbook
        .with_header_row(HeaderRow::Row(skip_rows))
        .worksheet_range_at(0)
      {
        Some(Ok(range)) => range,
        Some(Err(_)) | None => Range::empty(),
      }
    }
    Some(ref name) => {
      match workbook
        .with_header_row(HeaderRow::Row(skip_rows))
        .worksheet_range(name)
      {
        Ok(range) => range,
        Err(_) => Range::empty(),
      }
    }
  };

  let (row_count, col_count) = range.get_size();
  // check row count
  if row_count == 0 {
    return Err(anyhow!("file is empty, skipping processing"));
  }

  let mut rows_iter = range.rows();

  // amortize allocations
  let mut record = StringRecord::with_capacity(500, col_count);
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
  let ncpus = num_cpus();
  // set chunk_size to number of rows per core/thread
  // let chunk_size = row_count.div_ceil(ncpus);
  let chunk_size = (row_count + ncpus - 1) / ncpus;

  let processed_rows: Vec<Vec<StringRecord>> = rows
    .par_chunks(chunk_size)
    .map(|chunk| {
      let mut record = StringRecord::with_capacity(500, col_count);
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

  Ok(wtr.flush()?)
}

fn get_sheetname_by_filename(
  records: &Vec<HashMap<String, String>>,
  filename: &str,
) -> Option<String> {
  for record in records.iter() {
    if let Some(file) = record.get("filename") {
      if file == filename {
        return record.get("sheetname").cloned();
      }
    }
  }
  None
}

async fn get_all_sheetnames<P: AsRef<Path>>(path: P) -> HashSet<String> {
  let (sheets, _) = map_excel_sheets(path.as_ref().to_str().unwrap().to_string()).await;
  let mut sheetnames = HashSet::new();

  for (_, sheet_list) in sheets.iter() {
    for sheet_name in sheet_list.iter() {
      sheetnames.insert(sheet_name.clone());
    }
  }

  sheetnames
}

#[tauri::command]
pub async fn map_excel_sheets(
  path: String,
) -> (HashMap<String, Vec<String>>, HashMap<String, String>) {
  let mut map_sheets = HashMap::new();
  let mut errors = HashMap::new();

  let paths: Vec<&str> = path.split('|').filter(|&x| !x.is_empty()).collect();
  if paths.is_empty() {
    return (map_sheets, errors);
  }

  for file in paths.iter() {
    let filename = Path::new(file)
      .file_name()
      .unwrap_or_else(|| OsStr::new(""))
      .to_str()
      .unwrap_or("")
      .to_string();
    match open_workbook_auto(file).map_err(|e| e.to_string()) {
      Ok(workbook) => {
        let sheets = workbook.sheet_names();
        map_sheets.insert(filename, sheets);
      }
      Err(err) => {
        errors.insert(filename, format!("get sheets error|{}", err));
      }
    }
  }

  (map_sheets, errors)
}

#[tauri::command]
pub async fn excel2csv(
  path: String,
  skip_rows: String,
  map_file_sheet: Vec<HashMap<String, String>>,
  all_sheets: bool,
  write_sheetname: bool,
  window: Window,
) -> Result<String, String> {
  let start_time = Instant::now();

  let skip_rows = skip_rows.parse::<u32>().map_err(|e| e.to_string())?;
  let paths: Vec<&str> = path.split('|').collect();

  for file in paths.iter() {
    let filename = Path::new(file).file_name().unwrap().to_str().unwrap();
    window
      .emit("start-to", filename)
      .map_err(|e| e.to_string())?;

    let path = Path::new(file);
    let file_stem = path.file_stem().unwrap().to_str().unwrap();

    if !all_sheets {
      let sheet_name = get_sheetname_by_filename(&map_file_sheet, filename);
      let sheetname = match sheet_name.clone() {
        Some(sheet) => sheet,
        None => "None".to_string(),
      };

      let output_path = match write_sheetname {
        true => path.with_file_name(format!("{file_stem}_{sheetname}.csv")),
        false => Path::new(file).with_extension("csv"),
      };

      match excel_to_csv(file, skip_rows, sheet_name, &output_path).await {
        Ok(_) => {
          window.emit("to_msg", filename).map_err(|e| e.to_string())?;
        }
        Err(err) => {
          window
            .emit("to_err", format!("{filename}|{err}"))
            .map_err(|e| e.to_string())?;
          continue;
        }
      }
    } else {
      let sheet_names = get_all_sheetnames(file).await;
      if sheet_names.is_empty() {
        window
          .emit("to-err", format!("{filename}|It's not an Excel file"))
          .map_err(|e| e.to_string())?;
        continue;
      }
      for (index, sheet) in sheet_names.iter().enumerate() {
        let output_path = path.with_file_name(format!("{}_{}.csv", file_stem, sheet));

        match excel_to_csv(file, skip_rows, Some(sheet.to_string()), &output_path).await {
          Ok(_) => {
            // check if it is the last sheet
            if index == sheet_names.len() - 1 {
              window.emit("to-msg", filename).map_err(|e| e.to_string())?;
            }
          }
          Err(err) => {
            window
              .emit("to-err", format!("{filename}|{sheet}:{err}"))
              .map_err(|e| e.to_string())?;
            continue;
          }
        }
      }
    }
  }

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  Ok(format!("{elapsed_time:.2}"))
}

/// convert csv to csv (only replace the delimiter)
async fn csv_to_csv<P: AsRef<Path> + Send + Sync>(
  path: P,
  write_sep: String,
  filename: String,
  mode: &str,
  window: Window,
) -> Result<()> {
  let csv_options = CsvOptions::new(&path);
  let sep = csv_options.detect_separator()?;
  let write_sep = if write_sep == "\\t" {
    b'\t'
  } else {
    write_sep.into_bytes()[0]
  };
  let total_rows = match mode {
    "idx" => csv_options.idx_csv_rows().await?,
    "std" => csv_options.std_csv_rows()?,
    _ => 0,
  };
  window.emit("total-rows", format!("{filename}|{total_rows}"))?;

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(csv_options.skip_csv_rows()?);

  let file_stem = path.as_ref().file_stem().unwrap().to_str().unwrap();
  let parent_path = path.as_ref().parent().unwrap().to_str().unwrap();
  let output_path = format!("{parent_path}/{file_stem}.sep.csv");

  let mut wtr = WriterBuilder::new()
    .delimiter(write_sep)
    .from_path(output_path)?;
  wtr.write_record(rdr.headers()?)?;

  let rows = Arc::new(Mutex::new(0));
  let rows_clone = Arc::clone(&rows);
  let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
  let (done_tx, mut done_rx) = oneshot::channel::<usize>();

  let timer_task = tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_millis(300));
    loop {
      tokio::select! {
        _ = interval.tick() => {
          let current_rows = *rows_clone.lock().unwrap();
          if let Err(err) = window.emit("update-rows", format!("{filename}|{current_rows}")) {
            let _ = window.emit("to-err", format!("{filename}|{err}"));
          }
        },
        Ok(final_rows) = (&mut done_rx) => {
          if let Err(err) = window.emit("update-rows", format!("{filename}|{final_rows}")) {
            let _ = window.emit("to-err", format!("{filename}|{err}"));
          }
          break;
        },
        _ = (&mut stop_rx) => { break; }
      }
    }
  });

  let counter_task = tokio::task::spawn_blocking(move || {
    let mut record = ByteRecord::new();
    while rdr.read_byte_record(&mut record)? {
      wtr.write_byte_record(&record)?;
      let mut count = rows.lock().unwrap();
      *count += 1;
    }

    let final_rows = *rows.lock().unwrap();
    let _ = done_tx.send(final_rows);

    Ok::<_, anyhow::Error>(wtr.flush()?)
  });

  counter_task.await??;
  let _ = stop_tx.send(());
  timer_task.await?;

  Ok(())
}

#[tauri::command]
pub async fn csv2csv(
  path: String,
  sep: String,
  mode: String,
  window: Window,
) -> Result<String, String> {
  let start_time = Instant::now();

  let paths: Vec<&str> = path.split('|').collect();
  for fp in paths.iter() {
    let filename = Path::new(fp).file_name().unwrap().to_str().unwrap();
    window
      .emit("start-to", filename)
      .map_err(|e| e.to_string())?;
    match csv_to_csv(
      fp,
      sep.clone(),
      filename.to_string(),
      mode.as_str(),
      window.clone(),
    )
    .await
    {
      Ok(_) => {
        window.emit("to-msg", filename).map_err(|e| e.to_string())?;
      }
      Err(err) => {
        window
          .emit("to-err", format!("{filename}|{err}"))
          .map_err(|e| e.to_string())?;
        continue;
      }
    }
  }

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  Ok(format!("{:.2}", elapsed_time))
}
