use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use std::{collections::HashMap, time::Instant};

use anyhow::Result;
use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tokio::sync::oneshot;

use crate::io::csv::{config::CsvConfigBuilder, options::CsvOptions, selection::Selection};
use crate::utils::EventEmitter;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnConfig {
  pub input_format: Option<String>,
  pub output_format: Option<String>,
}

// 日期格式列表(按优先级排序)
const DATE_FORMATS: &[&str] = &[
  // 1.无分隔符(高优先级)
  "%Y%m%d",       // yyyymmdd
  "%Y%m%d%H%M%S", // yyyymmddhhmmss
  "%Y%m%d%H%M",   // yyyymmddhhmm
  "%d%m%Y",       // ddmmyyyy
  "%d%m%Y%H%M%S", // ddmmyyyyhhmmss
  "%m%d%Y",       // mmddyyyy
  "%m%d%Y%H%M%S", // mmddyyyyhhmmss
  // 2.所有 4 位年份的六种排列带分隔符
  // YMD: yyyy-mm-dd
  "%Y-%m-%d",
  "%Y/%m/%d",
  "%Y-%m-%d %H:%M:%S",
  "%Y/%m/%d %H:%M:%S",
  // YDM: yyyy-dd-mm
  "%Y-%d-%m",
  "%Y/%d/%m",
  "%Y-%d-%m %H:%M:%S",
  "%Y/%d/%m %H:%M:%S",
  // MDY: mm-dd-yyyy
  "%m-%d-%Y",
  "%m/%d/%Y",
  "%m-%d-%Y %H:%M:%S",
  "%m/%d/%Y %H:%M:%S",
  // MYD: mm-yy-dddd
  "%m-%Y-%d",
  "%m/%Y/%d",
  "%m-%Y-%d %H:%M:%S",
  "%m/%Y/%d %H:%M:%S",
  // DMY: dd-mm-yyyy
  "%d-%m-%Y",
  "%d/%m/%Y",
  "%d-%m-%Y %H:%M:%S",
  "%d/%m/%Y %H:%M:%S",
  // DYM: dd-yyyy-mm
  "%d-%Y-%m",
  "%d/%Y/%m",
  "%d-%Y-%m %H:%M:%S",
  "%d/%Y/%m %H:%M:%S",
  // 3.其他常见带时间格式
  "%Y-%m-%d %H:%M:%S%.f", // 毫秒
  "%Y-%m-%dT%H:%M:%S",
  "%Y-%m-%dT%H:%M:%S%.f",
  "%Y-%m-%d %H:%M",
  "%Y/%m/%d %H:%M",
  // 4.中文格式
  "%Y年%m月%d日",
  "%Y年%m月%d日 %H时%M分%S秒",
  "%Y年%m月%d日 %H:%M:%S",
  "%d日%m月%Y年",
  "%d日%m月%Y年 %H时%M分%S秒",
  "%d日%m月%Y年 %H:%M:%S",
  "%m月%d日%Y年",
  "%m月%d日%Y年 %H时%M分%S秒",
  "%m月%d日%Y年 %H:%M:%S",
  // 5.时间在前
  "%H:%M:%S %Y-%m-%d",
  "%H:%M %Y-%m-%d",
];

/// 尝试将字符串解析为 NaiveDateTime (支持日期,日期时间)
fn parse_to_naive_datetime(s: &str) -> Option<NaiveDateTime> {
  // 1.parse datetime
  for fmt in DATE_FORMATS {
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, fmt) {
      return Some(dt);
    }
  }

  // 2.parse date
  for fmt in DATE_FORMATS {
    if let Ok(date) = NaiveDate::parse_from_str(s, fmt) {
      return date.and_hms_opt(0, 0, 0);
    }
  }

  None
}

/// 转换 CSV 中指定列的日期格式
///
/// - `path`: 输入CSV路径
/// - `column_configs`: column,input_format,output_format
/// - `flexible`: 是否启用列数检查
/// - `quoting`: 读取时是否忽略双引号
/// - `skiprows`: 跳过的行
pub async fn convert_csv_dates<E, P>(
  path: P,
  column_configs: HashMap<String, ColumnConfig>,
  flexible: bool,
  quoting: bool,
  skiprows: usize,
  progress: bool,
  emitter: E,
) -> Result<()>
where
  E: EventEmitter + Send + Sync + 'static,
  P: AsRef<Path> + Send + Sync,
{
  let mut opts = CsvOptions::new(path);
  opts.set_skiprows(skiprows);
  let (sep, reader) = opts.skiprows_and_delimiter()?;
  let output_path = opts.output_path(Some("date"), None)?;
  let error_output_path = opts.output_path(Some("date_errors"), None)?;

  let total_rows = match progress {
    true => opts.idx_count_rows().await?,
    false => 0,
  };
  emitter.emit_total_rows(total_rows).await?;

  let config = CsvConfigBuilder::new()
    .flexible(flexible)
    .delimiter(sep)
    .quoting(quoting)
    .build();

  let mut rdr = config.build_reader(reader);
  let mut wtr = config.build_writer(&output_path)?;
  let mut error_wtr = config.build_writer(&error_output_path)?;

  let columns: Vec<&str> = column_configs.keys().map(|s| s.as_str()).collect();
  let sel = Selection::from_headers(rdr.byte_headers()?, &columns[..])?;

  wtr.write_record(rdr.headers()?)?;
  error_wtr.write_record(rdr.headers()?)?;

  let rows = Arc::new(AtomicUsize::new(0));
  let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
  let (done_tx, mut done_rx) = oneshot::channel::<usize>();

  let timer_task = if progress {
    let rows_clone = Arc::clone(&rows);

    Some(tokio::spawn(async move {
      let mut interval = tokio::time::interval(Duration::from_millis(500));
      loop {
        tokio::select! {
          _ = interval.tick() => {
            let current_rows = rows_clone.load(Ordering::Relaxed);
            if let Err(err) = emitter.emit_update_rows(current_rows).await {
              let _ = emitter.emit_err(&format!("failed to emit current rows: {err}")).await;
            }
          },
          Ok(final_rows) = (&mut done_rx) => {
            if let Err(err) = emitter.emit_update_rows(final_rows).await {
              let _ = emitter.emit_err(&format!("failed to emit final rows: {err}")).await;
            }
            break;
          },
          _ = (&mut stop_rx) => { break; }
        }
      }
    }))
  } else {
    None
  };

  // 构建手动格式映射(列索引 -> 格式字符串)
  let mut input_formats: HashMap<usize, String> = HashMap::new();
  let mut output_formats: HashMap<usize, String> = HashMap::new();
  for (i, &col_idx) in sel.get_indices().iter().enumerate() {
    let col_name = columns[i];

    if let Some(config) = column_configs.get(col_name) {
      if let Some(fmt) = &config.input_format {
        input_formats.insert(col_idx, fmt.clone());
      }

      output_formats.insert(
        col_idx,
        config
          .output_format
          .clone()
          .unwrap_or("%Y-%m-%d".to_owned()),
      );
    }
  }

  let counter_task = tokio::task::spawn_blocking(move || {
    for result in rdr.records() {
      let record = result?;
      let mut fields: Vec<String> = record.iter().map(|s| s.to_string()).collect();
      let mut has_error = false;

      for &idx in sel.get_indices() {
        let cell = record.get(idx).unwrap_or("");
        if cell.is_empty() {
          continue;
        }

        let parsed = if let Some(fmt) = input_formats.get(&idx) {
          // 手动格式
          if let Ok(ndt) = NaiveDateTime::parse_from_str(cell, fmt) {
            Some(ndt)
          } else if let Ok(nd) = NaiveDate::parse_from_str(cell, fmt) {
            nd.and_hms_opt(0, 0, 0)
          } else {
            has_error = true; // 标记该行有错误
            None
          }
        } else {
          // 自动检测
          parse_to_naive_datetime(cell)
        };

        if let Some(ndt) = parsed {
          if let Some(out_fmt) = output_formats.get(&idx) {
            fields[idx] = ndt.format(out_fmt).to_string();
          }
        } else {
          has_error = true;
        }
      }

      if has_error {
        // 写入错误文件(原始 record，未修改)
        error_wtr.write_record(&record)?;
      } else {
        // 写入主输出文件
        wtr.write_record(&fields)?;
      }
      rows.fetch_add(1, Ordering::Relaxed);
    }

    let final_rows = rows.load(Ordering::Relaxed);
    let _ = done_tx.send(final_rows);
    wtr.flush()?;
    error_wtr.flush()?;
    Ok::<_, anyhow::Error>(())
  });

  counter_task.await??;
  let _ = stop_tx.send(());
  if let Some(task) = timer_task {
    task.await?;
  }

  Ok(())
}

#[tauri::command]
pub async fn datefmt(
  path: String,
  column_configs: HashMap<String, ColumnConfig>,
  flexible: bool,
  quoting: bool,
  skiprows: usize,
  progress: bool,
  app_handle: AppHandle,
) -> Result<String, String> {
  let start_time = Instant::now();

  match convert_csv_dates(
    path,
    column_configs,
    flexible,
    quoting,
    skiprows,
    progress,
    app_handle,
  )
  .await
  {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.0}"))
    }
    Err(err) => Err(format!("{err}")),
  }
}
