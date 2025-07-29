use anyhow::{Result, anyhow};
use csv::StringRecord;
use dynfmt::Format;
use dynfmt::SimpleCurlyFormat;
use pinyin::ToPinyin;
use regex::Regex;

use crate::flow::utils::ProcessContext;

pub fn str_process(
  record: &StringRecord,
  context: &ProcessContext,
  headers: &Vec<String>,
) -> Result<(Vec<String>, Vec<String>)> {
  // initital f_fill cache
  let mut ffill_caches: Vec<Option<String>> = vec![None; context.str_ops.len()];
  let mut row_fields: Vec<String> = record.iter().map(|s| s.to_string()).collect();
  let mut str_results = Vec::new();
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
      for field in record {
        record_vec.push(field.to_string());
      }
      if let Ok(formatted) = SimpleCurlyFormat.format(&dynfmt_template_wrk, record_vec) {
        str_results.push(formatted.to_string());
      } else {
        str_results.push(String::new());
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
        "len" => str_results.push(cell.chars().count().to_string()),
        "copy" => str_results.push(cell.clone()),
        _ => str_results.push(cell),
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
        str_results.push(String::new());
      }
    }
  }

  Ok((row_fields, str_results))
}
