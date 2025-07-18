use std::{collections::HashMap, path::Path, time::Instant};

use tauri::{Emitter, Window};

#[cfg(target_os = "windows")]
pub mod access_to_csv;
pub mod csv_to_csv;
pub mod csv_to_excel;
pub mod dbf_to_csv;
pub mod excel_to_csv;

#[tauri::command]
pub async fn access2csv(path: String, sep: String, window: Window) -> Result<String, String> {
  let start_time = Instant::now();

  let paths: Vec<&str> = path.split('|').collect();
  for fp in paths.iter() {
    let filename = Path::new(fp).file_name().unwrap().to_str().unwrap();
    window
      .emit("start-to", filename)
      .map_err(|e| e.to_string())?;
    match access_to_csv::access_to_csv(fp, sep.clone()).await {
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
    match csv_to_csv::csv_to_csv(
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

#[tauri::command]
pub async fn csv2xlsx(
  path: String,
  mode: String,
  chunk_size: String,
  window: Window,
) -> Result<String, String> {
  let start_time = Instant::now();

  let paths: Vec<&str> = path.split('|').collect();
  let chunk_size = chunk_size.parse::<usize>().map_err(|e| e.to_string())?;
  let use_polars = mode != "csv";

  for file in paths.iter() {
    let filename = Path::new(file).file_name().unwrap().to_str().unwrap();
    window
      .emit("start-to", filename)
      .map_err(|e| e.to_string())?;

    match csv_to_excel::csv_to_xlsx(file, use_polars, chunk_size).await {
      Ok(_) => {
        window
          .emit("c2x-msg", filename)
          .map_err(|e| e.to_string())?;
      }
      Err(err) => {
        window
          .emit("rows-err", format!("{filename}|{err}"))
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
pub async fn dbf2csv(path: String, sep: String, window: tauri::Window) -> Result<String, String> {
  let start_time = Instant::now();

  let paths: Vec<&str> = path.split('|').collect();
  for fp in paths.iter() {
    let filename = Path::new(fp).file_name().unwrap().to_str().unwrap();
    window
      .emit("start-to", filename)
      .map_err(|e| e.to_string())?;
    match dbf_to_csv::dbf_to_csv(fp, sep.clone()).await {
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
          window.emit("to-msg", filename).map_err(|e| e.to_string())?;
        }
        Err(err) => {
          window
            .emit("to-err", format!("{filename}|{err}"))
            .map_err(|e| e.to_string())?;
          continue;
        }
      }
    } else {
      let sheet_names = excel_to_csv::get_all_sheetnames(file).await;
      if sheet_names.is_empty() {
        window
          .emit("to-err", format!("{filename}|It's not an Excel file"))
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
