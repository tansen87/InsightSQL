use std::{collections::HashMap, path::Path, time::Instant};

use anyhow::Result;
use tauri::AppHandle;

use crate::{io::csv::options::CsvOptions, utils::EventEmitter};

#[cfg(target_os = "windows")]
pub mod access_to_csv;
pub mod csv_to_csv;
pub mod csv_to_excel;
pub mod dbf_to_csv;
pub mod excel_to_csv;
pub mod json_to_csv;
pub mod jsonl_to_csv;

#[cfg(target_os = "windows")]
#[tauri::command]
pub async fn access2csv(
  path: String,
  wtr_sep: String,
  emitter: AppHandle,
) -> Result<String, String> {
  let start_time = Instant::now();

  let paths: Vec<&str> = path.split('|').collect();
  for file in paths.iter() {
    let opts = CsvOptions::new(file);
    let filename = opts
      .file_name()
      .map_err(|e| format!("opts.file_name failed: {e}"))?;
    emitter
      .emit_info(filename)
      .await
      .map_err(|e| e.to_string())?;
    match access_to_csv::access_to_csv(file, wtr_sep.clone()).await {
      Ok(_) => {
        emitter
          .emit_success(filename)
          .await
          .map_err(|e| e.to_string())?;
      }
      Err(err) => {
        emitter
          .emit_err(&format!("{filename}|{err}"))
          .await
          .map_err(|e| e.to_string())?;
        continue;
      }
    }
  }

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  Ok(format!("{:.2}", elapsed_time))
}

#[tauri::command]
pub async fn csv2csv(
  path: String,
  wtr_sep: String,
  quote: String,
  quote_style: String,
  progress: String,
  emitter: AppHandle,
) -> Result<String, String> {
  let start_time = Instant::now();

  let paths: Vec<&str> = path.split('|').collect();
  for file in paths.iter() {
    let opts = CsvOptions::new(file);
    let filename = opts
      .file_name()
      .map_err(|e| format!("opts.file_name failed: {e}"))?;
    emitter
      .emit_info(filename)
      .await
      .map_err(|e| e.to_string())?;
    match csv_to_csv::csv_to_csv(
      file,
      &wtr_sep,
      &quote,
      &quote_style,
      filename.to_string(),
      &progress,
      emitter.clone(),
    )
    .await
    {
      Ok(_) => {
        emitter
          .emit_success(filename)
          .await
          .map_err(|e| e.to_string())?;
      }
      Err(err) => {
        emitter
          .emit_err(&format!("{filename}|{err}"))
          .await
          .map_err(|e| e.to_string())?;
        continue;
      }
    }
  }

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  Ok(format!("{:.2}", elapsed_time))
}

#[tauri::command]
pub async fn encoding2utf8(path: String, bom: bool) -> Result<String, String> {
  let start_time = Instant::now();
  let paths: Vec<&str> = path.split('|').collect();
  let p = paths.first().unwrap();

  match csv_to_csv::encoding_to_utf8(p, bom).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("{err}")),
  }
}

#[tauri::command]
pub async fn csv2xlsx(
  path: String,
  csv_mode: String,
  chunksize: String,
  emitter: AppHandle,
) -> Result<String, String> {
  let start_time = Instant::now();

  let paths: Vec<&str> = path.split('|').collect();
  let chunksize = chunksize.parse::<usize>().map_err(|e| e.to_string())?;
  let use_polars = csv_mode != "csv";

  for file in paths.iter() {
    let opts = CsvOptions::new(file);
    let filename = opts
      .file_name()
      .map_err(|e| format!("opts.file_name failed: {e}"))?;
    emitter
      .emit_info(filename)
      .await
      .map_err(|e| e.to_string())?;
    match csv_to_excel::csv_to_xlsx(file, use_polars, chunksize).await {
      Ok(_) => {
        emitter
          .emit_success(filename)
          .await
          .map_err(|e| e.to_string())?;
      }
      Err(err) => {
        emitter
          .emit_err(&format!("{filename}|{err}"))
          .await
          .map_err(|e| e.to_string())?;
        continue;
      }
    }
  }

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  Ok(format!("{elapsed_time:.2}"))
}

#[tauri::command]
pub async fn dbf2csv(path: String, wtr_sep: String, emitter: AppHandle) -> Result<String, String> {
  let start_time = Instant::now();

  let paths: Vec<&str> = path.split('|').collect();
  for file in paths.iter() {
    let opts = CsvOptions::new(file);
    let filename = opts
      .file_name()
      .map_err(|e| format!("opts.file_name failed: {e}"))?;
    emitter
      .emit_info(filename)
      .await
      .map_err(|e| e.to_string())?;
    match dbf_to_csv::dbf_to_csv(file, wtr_sep.clone()).await {
      Ok(_) => {
        emitter
          .emit_success(filename)
          .await
          .map_err(|e| e.to_string())?;
      }
      Err(err) => {
        emitter
          .emit_err(&format!("{filename}|{err}"))
          .await
          .map_err(|e| e.to_string())?;
        continue;
      }
    }
  }

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  Ok(format!("{:.2}", elapsed_time))
}

#[tauri::command]
pub async fn excel2csv(
  path: String,
  skip_rows: String,
  map_file_sheet: Vec<HashMap<String, String>>,
  all_sheets: bool,
  write_sheetname: bool,
  emitter: AppHandle,
) -> Result<String, String> {
  let start_time = Instant::now();

  let skip_rows = skip_rows.parse::<u32>().map_err(|e| e.to_string())?;
  let paths: Vec<&str> = path.split('|').collect();

  for file in paths.iter() {
    let opts = CsvOptions::new(file);
    let filename = opts
      .file_name()
      .map_err(|e| format!("opts.file_name failed: {e}"))?;
    emitter
      .emit_info(filename)
      .await
      .map_err(|e| e.to_string())?;

    let path = Path::new(file);
    let file_stem = opts.file_stem().map_err(|e| e.to_string())?;

    if !all_sheets {
      let sheet_name = excel_to_csv::get_sheetname_by_filename(&map_file_sheet, filename);
      let sheetname = match sheet_name.clone() {
        Some(sheet) => sheet,
        None => "None".to_string(),
      };

      let output_path = match write_sheetname {
        true => path.with_file_name(format!("{file_stem}_{sheetname}.csv")),
        false => Path::new(file).with_extension("csv"),
      };

      match excel_to_csv::excel_to_csv(file, skip_rows, sheet_name, &output_path).await {
        Ok(_) => {
          emitter
            .emit_success(filename)
            .await
            .map_err(|e| e.to_string())?;
        }
        Err(err) => {
          emitter
            .emit_err(&format!("{filename}|{err}"))
            .await
            .map_err(|e| e.to_string())?;
          continue;
        }
      }
    } else {
      let sheet_names = excel_to_csv::get_all_sheetnames(file).await;
      if sheet_names.is_empty() {
        emitter
          .emit_err(&format!("{filename}||Not an Excel file"))
          .await
          .map_err(|e| e.to_string())?;
        continue;
      }
      for (index, sheet) in sheet_names.iter().enumerate() {
        let output_path = path.with_file_name(format!("{}_{}.csv", file_stem, sheet));

        match excel_to_csv::excel_to_csv(file, skip_rows, Some(sheet.to_string()), &output_path)
          .await
        {
          Ok(_) => {
            // check if it is the last sheet
            if index == sheet_names.len() - 1 {
              emitter
                .emit_success(filename)
                .await
                .map_err(|e| e.to_string())?;
            }
          }
          Err(err) => {
            emitter
              .emit_err(&format!("{filename}|{sheet}:{err}"))
              .await
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

#[tauri::command]
pub async fn json2csv(path: String, wtr_sep: String, emitter: AppHandle) -> Result<String, String> {
  let start_time = Instant::now();

  let paths: Vec<&str> = path.split('|').collect();
  for file in paths.iter() {
    let opts = CsvOptions::new(file);
    let filename = opts
      .file_name()
      .map_err(|e| format!("opts.file_name failed: {e}"))?;
    emitter
      .emit_info(filename)
      .await
      .map_err(|e| e.to_string())?;
    match json_to_csv::json_to_csv(file, wtr_sep.clone()).await {
      Ok(_) => {
        emitter
          .emit_success(filename)
          .await
          .map_err(|e| e.to_string())?;
      }
      Err(err) => {
        emitter
          .emit_err(&format!("{filename}|{err}"))
          .await
          .map_err(|e| e.to_string())?;
        continue;
      }
    }
  }

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  Ok(format!("{:.2}", elapsed_time))
}

#[tauri::command]
pub async fn jsonl2csv(
  path: String,
  wtr_sep: String,
  ignore_err: bool,
  emitter: AppHandle,
) -> Result<String, String> {
  let start_time = Instant::now();

  let paths: Vec<&str> = path.split('|').collect();
  for file in paths.iter() {
    let opts = CsvOptions::new(file);
    let filename = opts
      .file_name()
      .map_err(|e| format!("opts.file_name failed: {e}"))?;
    emitter
      .emit_info(filename)
      .await
      .map_err(|e| e.to_string())?;
    match jsonl_to_csv::jsonl_to_csv(file, &wtr_sep, ignore_err).await {
      Ok(_) => {
        emitter
          .emit_success(filename)
          .await
          .map_err(|e| e.to_string())?;
      }
      Err(err) => {
        emitter
          .emit_err(&format!("{filename}|{err}"))
          .await
          .map_err(|e| e.to_string())?;
        continue;
      }
    }
  }

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  Ok(format!("{:.2}", elapsed_time))
}
