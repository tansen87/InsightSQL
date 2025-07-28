use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use csv::ReaderBuilder;
use csv::WriterBuilder;
use dynfmt::Format;
use dynfmt::SimpleCurlyFormat;
use pinyin::ToPinyin;
use regex::Regex;

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
      "str" => {
        if let (Some(col), Some(mode)) = (&op.column, &op.mode) {
          context.add_str(
            col,
            mode,
            op.comparand.clone(),
            op.replacement.clone(),
            op.alias.clone(),
          );
        } else if let Some(mode) = &op.mode {
          if mode == "dynfmt" {
            context.add_str(
              "",
              mode,
              op.comparand.clone(),
              op.replacement.clone(),
              op.alias.clone(),
            )
          }
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

    for str_op in &context.str_ops {
      if str_op.mode == "fill"
        || str_op.mode == "f_fill"
        || str_op.mode == "lower"
        || str_op.mode == "upper"
        || str_op.mode == "trim"
        || str_op.mode == "ltrim"
        || str_op.mode == "rtrim"
        || str_op.mode == "squeeze"
        || str_op.mode == "strip"
        || str_op.mode == "replace"
        || str_op.mode == "regex_replace"
        || str_op.mode == "round"
        || str_op.mode == "reverse"
        || str_op.mode == "abs"
        || str_op.mode == "neg"
        || str_op.mode == "pinyin"
        || str_op.mode == "left"
        || str_op.mode == "right"
        || str_op.mode == "slice"
        || str_op.mode == "split"
      {
        continue;
      }
      let str_name = if let Some(ref alias) = str_op.alias {
        alias.clone()
      } else if str_op.mode == "dynfmt" {
        "_dynfmt".to_string()
      } else {
        format!("{}_{}", str_op.column, str_op.mode)
      };
      new_field_names.push(str_name.clone());
      new_field_is_slice.push(false);
      new_field_aliases.push(str_name);
    }

    for str_op in &context.str_ops {
      if str_op.mode == "fill"
        || str_op.mode == "f_fill"
        || str_op.mode == "lower"
        || str_op.mode == "upper"
        || str_op.mode == "trim"
        || str_op.mode == "ltrim"
        || str_op.mode == "rtrim"
        || str_op.mode == "squeeze"
        || str_op.mode == "strip"
        || str_op.mode == "replace"
        || str_op.mode == "regex_replace"
        || str_op.mode == "round"
        || str_op.mode == "reverse"
        || str_op.mode == "abs"
        || str_op.mode == "neg"
        || str_op.mode == "pinyin"
        || str_op.mode == "left"
        || str_op.mode == "right"
        || str_op.mode == "slice"
        || str_op.mode == "split"
      {
        continue;
      }
      let str_name = if let Some(ref alias) = str_op.alias {
        alias.clone()
      } else if str_op.mode == "dynfmt" {
        "_dynfmt".to_string()
      } else {
        format!("{}_{}", str_op.column, str_op.mode)
      };
      selected_headers.push(str_name);
    }

    wtr.write_record(&selected_headers)?;
  } else {
    let mut all_headers: Vec<String> = headers.iter().map(|s| s.to_string()).collect();

    for str_op in &context.str_ops {
      if str_op.mode == "fill"
        || str_op.mode == "f_fill"
        || str_op.mode == "lower"
        || str_op.mode == "upper"
        || str_op.mode == "trim"
        || str_op.mode == "ltrim"
        || str_op.mode == "rtrim"
        || str_op.mode == "squeeze"
        || str_op.mode == "strip"
        || str_op.mode == "replace"
        || str_op.mode == "regex_replace"
        || str_op.mode == "round"
        || str_op.mode == "reverse"
        || str_op.mode == "abs"
        || str_op.mode == "neg"
        || str_op.mode == "pinyin"
        || str_op.mode == "left"
        || str_op.mode == "right"
        || str_op.mode == "slice"
        || str_op.mode == "split"
      {
        continue;
      }
      let str_name = if let Some(ref alias) = str_op.alias {
        alias.clone()
      } else if str_op.mode == "dynfmt" {
        "_dynfmt".to_string()
      } else {
        format!("{}_{}", str_op.column, str_op.mode)
      };
      all_headers.push(str_name);
    }

    wtr.write_record(all_headers.iter().map(|s| s.as_str()))?;
  }

  // initital f_fill cache
  let mut ffill_caches: Vec<Option<String>> = vec![None; context.str_ops.len()];

  for result in rdr.records() {
    let record = result?;

    let mut row_fields: Vec<String> = record.iter().map(|s| s.to_string()).collect();
    let mut string_results = Vec::new();
    for (i, str_op) in context.str_ops.iter().enumerate() {
      if str_op.mode == "dynfmt" {
        // dynfmt操作不依赖特定列，直接处理整个记录
        let template = str_op.comparand.as_deref().unwrap_or("");
        let mut dynfmt_template_wrk = template.to_string();
        let mut dynfmt_fields = Vec::new();

        let formatstr_re: &'static Regex = crate::regex_oncelock!(r"\{(?P<key>\w+)?\}");
        for format_fields in formatstr_re.captures_iter(template) {
          // safety: we already checked that the regex match is valid
          if let Some(key) = format_fields.name("key") {
            dynfmt_fields.push(key.as_str());
          }
        }
        dynfmt_fields.sort_unstable();

        for (i, field) in headers.iter().enumerate() {
          if dynfmt_fields.binary_search(&field.as_str()).is_ok() {
            let field_with_curly = format!("{{{field}}}");
            let field_index = format!("{{{i}}}");
            dynfmt_template_wrk = dynfmt_template_wrk.replace(&field_with_curly, &field_index);
          }
        }

        let mut record_vec: Vec<String> = Vec::with_capacity(record.len());
        for field in &record {
          record_vec.push(field.to_string());
        }
        if let Ok(formatted) = SimpleCurlyFormat.format(&dynfmt_template_wrk, record_vec) {
          string_results.push(formatted.to_string());
        } else {
          string_results.push(String::new());
        }
      } else if let Some(idx) = headers.iter().position(|h| h == &str_op.column) {
        let cell = row_fields[idx].clone();
        let length = match str_op.mode.as_str() {
          "left" | "right" | "slice" | "split" => str_op
            .replacement
            .clone()
            .ok_or(anyhow!("length is invalid number"))?
            .parse::<usize>()?,
          _ => 0,
        };
        match str_op.mode.as_str() {
          // do not add new column
          "fill" => {
            if cell.is_empty() {
              row_fields[idx] = str_op.replacement.clone().unwrap_or_default();
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
            let comparand = str_op.comparand.as_deref().unwrap_or("");
            let replacement = str_op.replacement.as_deref().unwrap_or("");
            row_fields[idx] = cell.replace(comparand, replacement);
          }
          "regex_replace" => {
            let comparand = str_op.comparand.as_deref().unwrap_or("");
            let replacement = str_op.replacement.as_deref().unwrap_or("");
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
            let py_mode_string = str_op.replacement.clone().unwrap_or("".to_owned());
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
          "left" => row_fields[idx] = cell.chars().take(length).collect(),
          "right" => {
            row_fields[idx] = cell
              .chars()
              .rev()
              .take(length)
              .collect::<String>()
              .chars()
              .rev()
              .collect()
          }
          "slice" => {
            let offset = str_op
              .comparand
              .clone()
              .ok_or(anyhow!("start index is invalid number"))?
              .parse::<isize>()?;
            let start = offset - 1;
            let end = start + length as isize;
            row_fields[idx] = cell
              .chars()
              .skip(start.max(0) as usize)
              .take((end - start) as usize)
              .collect::<String>();
          }
          "split" => {
            let sep = &str_op
              .comparand
              .clone()
              .ok_or(anyhow!("delimiter is invalid"))?;
            let split_parts: Vec<&str> = cell.split(sep).collect();
            if split_parts.len() >= length {
              row_fields[idx] = split_parts[length - 1].to_string();
            } else {
              row_fields[idx] = "".to_string();
            }
          }
          // add new column
          "len" => string_results.push(cell.chars().count().to_string()),
          "copy" => string_results.push(cell.clone()),
          _ => string_results.push(cell),
        }
      } else {
        // 字段找不到时,只有新增列的操作才追加空字符串
        if str_op.mode != "fill"
          && str_op.mode != "f_fill"
          && str_op.mode != "lower"
          && str_op.mode != "upper"
          && str_op.mode != "trim"
          && str_op.mode != "ltrim"
          && str_op.mode != "rtrim"
          && str_op.mode != "squeeze"
          && str_op.mode != "strip"
          && str_op.mode != "replace"
          && str_op.mode != "regex_replace"
          && str_op.mode != "round"
          && str_op.mode != "reverse"
          && str_op.mode != "abs"
          && str_op.mode != "neg"
          && str_op.mode != "pinyin"
          && str_op.mode == "left"
          && str_op.mode == "right"
          && str_op.mode == "slice"
          && str_op.mode == "split"
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
        filtered.extend(string_results.iter().map(|s| s.as_str()));
        wtr.write_record(&filtered)?;
      } else {
        let mut all_fields: Vec<_> = row_fields.iter().map(|s| s.as_str()).collect();
        all_fields.extend(string_results.iter().map(|s| s.as_str()));
        wtr.write_record(&all_fields)?;
      }
    }
  }

  Ok(())
}
