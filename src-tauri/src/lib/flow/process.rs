use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use csv::ReaderBuilder;
use csv::WriterBuilder;
use pinyin::ToPinyin;

use crate::flow::filter;
use crate::flow::utils::Operation;
use crate::flow::utils::ProcessingContext;
use crate::io::csv::options::CsvOptions;

pub async fn process_operations(
  input_path: String,
  operations: &[Operation],
  output_path: PathBuf,
) -> Result<()> {
  let csv_options = CsvOptions::new(input_path);
  let sep = csv_options.detect_separator()?;

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(csv_options.rdr_skip_rows()?);
  let headers = rdr
    .headers()?
    .clone()
    .iter()
    .map(|s| s.to_string())
    .collect::<Vec<_>>();

  let mut context = ProcessingContext::new();

  for op in operations {
    match op.op.as_str() {
      "select" => {
        if let Some(column) = &op.column {
          let columns: Vec<&str> = column.split('|').collect();
          context.add_select(&columns, &headers);
          let aliases = if let Some(alias) = &op.alias {
            alias
              .split(|c| c == '|')
              .map(|s| Some(s.to_string()))
              .collect()
          } else {
            vec![None; columns.len()]
          };
          context.alias = Some(aliases);
        }
      }
      "filter" => {
        if let (Some(col), Some(mode), Some(val)) = (&op.column, &op.mode, &op.value) {
          let col = Arc::from(col.as_str());
          let val = Arc::from(val.as_str());
          let headers = Arc::new(headers.to_vec());
          match mode.as_str() {
            "equal" => context.add_filter(filter::equal(col, val, headers)?),
            "not_equal" => context.add_filter(filter::not_equal(col, val, headers)?),
            "contains" => context.add_filter(filter::contains(col, val, headers)?),
            "not_contains" => context.add_filter(filter::not_contains(col, val, headers)?),
            "starts_with" => context.add_filter(filter::starts_with(col, val, headers)?),
            "not_starts_with" => context.add_filter(filter::not_starts_with(col, val, headers)?),
            "ends_with" => context.add_filter(filter::ends_with(col, val, headers)?),
            "not_ends_with" => context.add_filter(filter::not_ends_with(col, val, headers)?),
            "gt" => context.add_filter(filter::gt(col, val, headers)?),
            "ge" => context.add_filter(filter::ge(col, val, headers)?),
            "lt" => context.add_filter(filter::lt(col, val, headers)?),
            "le" => context.add_filter(filter::le(col, val, headers)?),
            "between" => context.add_filter(filter::between(col, val, headers)?),
            "is_null" => context.add_filter(filter::is_null(col, headers)?),
            "is_not_null" => context.add_filter(filter::is_not_null(col, headers)?),
            _ => return Err(anyhow!("Not support filter mode: {}", mode)),
          }
        }
      }
      "slice" => {
        if let (Some(col), Some(mode)) = (&op.column, &op.mode) {
          let offset = op.offset.clone().ok_or(anyhow!("offset is null"))?;
          let length = op.length.clone().ok_or(anyhow!("length is null"))?;
          context.add_slice(col, mode, offset, length, op.alias.clone());
        }
      }
      "str" => {
        if let (Some(col), Some(mode)) = (&op.column, &op.mode) {
          context.add_string(
            col,
            mode,
            op.comparand.clone(),
            op.replacement.clone(),
            op.alias.clone(),
          );
        }
      }
      _ => return Err(anyhow!("Not support operation: {}", op.op)),
    }
  }

  let output_file = File::create(output_path)?;
  let buf_wtr = BufWriter::with_capacity(256_000, output_file);
  let mut wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_wtr);

  if let Some(ref selected) = context.select {
    let mut selected_headers: Vec<String> = Vec::new();
    if let Some(ref aliases) = context.alias {
      for (i, &idx) in selected.iter().enumerate() {
        if let Some(Some(alias)) = aliases.get(i) {
          selected_headers.push(alias.clone());
        } else {
          selected_headers.push(headers[idx].clone());
        }
      }
    } else {
      for &idx in selected {
        selected_headers.push(headers[idx].clone());
      }
    }

    let mut new_field_names = Vec::new();
    let mut new_field_is_slice = Vec::new(); // true=slice, false=string
    let mut new_field_aliases = Vec::new();
    for slice_op in &context.slice_ops {
      let slice_name = if let Some(ref alias) = slice_op.alias {
        alias.clone()
      } else {
        format!("{}_{}", slice_op.column, slice_op.mode)
      };
      new_field_names.push(slice_name.clone());
      new_field_is_slice.push(true);
      new_field_aliases.push(slice_name);
    }
    for string_op in &context.string_ops {
      if string_op.mode == "fill"
        || string_op.mode == "f_fill"
        || string_op.mode == "lower"
        || string_op.mode == "upper"
        || string_op.mode == "trim"
        || string_op.mode == "ltrim"
        || string_op.mode == "rtrim"
        || string_op.mode == "squeeze"
        || string_op.mode == "strip"
        || string_op.mode == "replace"
        || string_op.mode == "regex_replace"
        || string_op.mode == "round"
        || string_op.mode == "reverse"
        || string_op.mode == "abs"
        || string_op.mode == "neg"
        || string_op.mode == "pinyin"
      {
        continue;
      }
      let string_name = if let Some(ref alias) = string_op.alias {
        alias.clone()
      } else {
        format!("{}_{}", string_op.column, string_op.mode)
      };
      new_field_names.push(string_name.clone());
      new_field_is_slice.push(false);
      new_field_aliases.push(string_name);
    }

    for slice_op in &context.slice_ops {
      let slice_name = if let Some(ref alias) = slice_op.alias {
        alias.clone()
      } else {
        format!("{}_{}", slice_op.column, slice_op.mode)
      };
      selected_headers.push(slice_name);
    }

    for string_op in &context.string_ops {
      if string_op.mode == "fill"
        || string_op.mode == "f_fill"
        || string_op.mode == "lower"
        || string_op.mode == "upper"
        || string_op.mode == "trim"
        || string_op.mode == "ltrim"
        || string_op.mode == "rtrim"
        || string_op.mode == "squeeze"
        || string_op.mode == "strip"
        || string_op.mode == "replace"
        || string_op.mode == "regex_replace"
        || string_op.mode == "round"
        || string_op.mode == "reverse"
        || string_op.mode == "abs"
        || string_op.mode == "neg"
        || string_op.mode == "pinyin"
      {
        continue;
      }
      let string_name = if let Some(ref alias) = string_op.alias {
        alias.clone()
      } else {
        format!("{}_{}", string_op.column, string_op.mode)
      };
      selected_headers.push(string_name);
    }

    wtr.write_record(&selected_headers)?;
  } else {
    let mut all_headers: Vec<String> = headers.iter().map(|s| s.to_string()).collect();
    for slice_op in &context.slice_ops {
      let slice_name = if let Some(ref alias) = slice_op.alias {
        alias.clone()
      } else {
        format!("{}_{}", slice_op.column, slice_op.mode)
      };
      all_headers.push(slice_name);
    }

    for string_op in &context.string_ops {
      if string_op.mode == "fill"
        || string_op.mode == "f_fill"
        || string_op.mode == "lower"
        || string_op.mode == "upper"
        || string_op.mode == "trim"
        || string_op.mode == "ltrim"
        || string_op.mode == "rtrim"
        || string_op.mode == "squeeze"
        || string_op.mode == "strip"
        || string_op.mode == "replace"
        || string_op.mode == "regex_replace"
        || string_op.mode == "round"
        || string_op.mode == "reverse"
        || string_op.mode == "abs"
        || string_op.mode == "neg"
        || string_op.mode == "pinyin"
      {
        continue;
      }
      let string_name = if let Some(ref alias) = string_op.alias {
        alias.clone()
      } else {
        format!("{}_{}", string_op.column, string_op.mode)
      };
      all_headers.push(string_name);
    }

    wtr.write_record(all_headers.iter().map(|s| s.as_str()))?;
  }

  // initital f_fill cache
  let mut ffill_caches: Vec<Option<String>> = vec![None; context.string_ops.len()];

  for result in rdr.records() {
    let record = result?;

    // collect all slice results first
    let mut slice_results = Vec::new();
    for slice_op in &context.slice_ops {
      let idx = headers.iter().position(|h| h == &slice_op.column);
      let length = slice_op.length.parse::<usize>()?;
      if let Some(idx) = idx {
        if let Some(val) = record.get(idx) {
          let new_val = match slice_op.mode.as_str() {
            "left" => {
              val.chars().take(length).collect()
            }
            "right" => {
              val
              .chars()
              .rev()
              .take(length)
              .collect::<String>()
              .chars()
              .rev()
              .collect()
            }
            "slice" => {
              let offset = slice_op.offset.parse::<isize>()?;
              let start = offset - 1;
              let end = start + length as isize;
              val
                .chars()
                .skip(start.max(0) as usize)
                .take((end - start) as usize)
                .collect()
            }
            _ => val.to_string(),
          };
          slice_results.push(new_val);
        } else {
          slice_results.push(String::new());
        }
      } else {
        slice_results.push(String::new());
      }
    }

    let mut row_fields: Vec<String> = record.iter().map(|s| s.to_string()).collect();
    let mut string_results = Vec::new();
    for (i, string_op) in context.string_ops.iter().enumerate() {
      if let Some(idx) = headers.iter().position(|h| h == &string_op.column) {
        let cell = row_fields[idx].clone();
        match string_op.mode.as_str() {
          // do not add new column
          "fill" => {
            if cell.is_empty() {
              row_fields[idx] = string_op.replacement.clone().unwrap_or_default();
            }
          }
          "f_fill" => {
            if cell.is_empty() {
              if let Some(ref cache_val) = ffill_caches[i] {
                row_fields[idx] = cache_val.clone();
              }
            } else {
              ffill_caches[i] = Some(cell.clone());
            }
          }
          "lower" => row_fields[idx] = cell.to_lowercase(),
          "upper" => row_fields[idx] = cell.to_uppercase(),
          "trim" => row_fields[idx] = cell.trim().to_string(),
          "ltrim" => row_fields[idx] = cell.trim_start().to_string(),
          "rtrim" => row_fields[idx] = cell.trim_end().to_string(),
          "squeeze" => {
            let re = regex::Regex::new(r"\s+").unwrap();
            row_fields[idx] = re.replace_all(&cell, " ").into_owned();
          }
          "strip" => {
            let re = regex::Regex::new(r"[\r\n]+").unwrap();
            row_fields[idx] = re.replace_all(&cell, " ").into_owned();
          }
          "replace" => {
            let comparand = string_op.comparand.as_deref().unwrap_or("");
            let replacement = string_op.replacement.as_deref().unwrap_or("");
            row_fields[idx] = cell.replace(comparand, replacement);
          }
          "regex_replace" => {
            let comparand = string_op.comparand.as_deref().unwrap_or("");
            let replacement = string_op.replacement.as_deref().unwrap_or("");
            let pattern = regex::RegexBuilder::new(comparand).build()?;
            row_fields[idx] = pattern.replace_all(&cell, replacement).to_string();
          }
          "round" => {
            if let Ok(num) = cell.parse::<f64>() {
              row_fields[idx] = format!("{:.2}", num);
            } else {
              row_fields[idx] = cell;
            }
          }
          "reverse" => row_fields[idx] = cell.chars().rev().collect(),
          "abs" => {
            if let Ok(num) = cell.parse::<f64>() {
              row_fields[idx] = num.abs().to_string();
            } else {
              row_fields[idx] = cell;
            }
          }
          "neg" => {
            if let Ok(num) = cell.parse::<f64>() {
              row_fields[idx] = (-num).to_string();
            } else {
              row_fields[idx] = cell;
            }
          }
          "pinyin" => {
            let py_mode_string = string_op.replacement.clone().unwrap_or("".to_owned());
            row_fields[idx] = cell
              .chars()
              .map(|c| {
                c.to_pinyin().map_or_else(
                  || c.to_string(),
                  |py| match py_mode_string.as_str() {
                    "upper" => py.plain().to_uppercase(),
                    "lower" => py.plain().to_lowercase(),
                    _ => py.plain().to_string(),
                  },
                )
              })
              .collect();
          }
          // add new column
          "len" => string_results.push(cell.chars().count().to_string()),
          "copy" => string_results.push(cell.clone()),
          _ => string_results.push(cell),
        }
      } else {
        // 字段找不到时,只有新增列的操作才追加空字符串
        if string_op.mode != "fill"
          && string_op.mode != "f_fill"
          && string_op.mode != "lower"
          && string_op.mode != "upper"
          && string_op.mode != "trim"
          && string_op.mode != "ltrim"
          && string_op.mode != "rtrim"
          && string_op.mode != "squeeze"
          && string_op.mode != "strip"
          && string_op.mode != "replace"
          && string_op.mode != "regex_replace"
          && string_op.mode != "round"
          && string_op.mode != "reverse"
          && string_op.mode != "abs"
          && string_op.mode != "neg"
          && string_op.mode != "pinyin"
        {
          string_results.push(String::new());
        }
      }
    }

    if context.is_valid(&record) {
      if let Some(selected) = &context.select {
        let mut filtered: Vec<_> = selected
          .iter()
          .map(|&idx| row_fields.get(idx).map(|s| s.as_str()).unwrap_or(""))
          .collect();
        filtered.extend(slice_results.iter().map(|s| s.as_str()));
        filtered.extend(string_results.iter().map(|s| s.as_str()));
        wtr.write_record(&filtered)?;
      } else {
        let mut all_fields: Vec<_> = row_fields.iter().map(|s| s.as_str()).collect();
        all_fields.extend(slice_results.iter().map(|s| s.as_str()));
        all_fields.extend(string_results.iter().map(|s| s.as_str()));
        wtr.write_record(&all_fields)?;
      }
    }
  }

  Ok(())
}
