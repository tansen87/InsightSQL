use anyhow::{Result, anyhow};
use cpc::units::Unit;
use csv::StringRecord;
use dynfmt::{Format, SimpleCurlyFormat};
use pinyin::ToPinyin;
use regex::Regex;

use crate::{flow::utils::ProcessContext, regex_oncelock};

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
    if str_op.mode == "cat" {
      // cat操作不依赖特定列，直接处理整个记录
      let template = str_op.comparand.as_deref().unwrap_or("");
      let mut dynfmt_template_wrk = template.to_string();
      let mut dynfmt_fields = Vec::new();

      let formatstr_re: &'static Regex = regex_oncelock!(r"\{(?P<key>\w+)?\}");
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
    } else if str_op.mode == "calcconv" {
      let template = str_op.comparand.as_deref().unwrap_or("");
      let mut dynfmt_template_wrk = template.to_string();
      let mut dynfmt_fields = Vec::new();

      let formatstr_re: &'static Regex = regex_oncelock!(r"\{(?P<key>\w+)?\}");
      for cap in formatstr_re.captures_iter(template) {
        if let Some(key) = cap.name("key") {
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

      let record_strings: Vec<String> = record.iter().map(|field| field.to_string()).collect();

      let formatted = match SimpleCurlyFormat.format(&dynfmt_template_wrk, &record_strings) {
        Ok(s) => s.to_string(),
        Err(_) => {
          str_results.push(String::new());
          continue;
        }
      };

      let (expr_str, append_unit) = if formatted.ends_with("<UNIT>") {
        (formatted.trim_end_matches("<UNIT>").trim(), true)
      } else {
        (formatted.trim(), false)
      };

      match cpc::eval(expr_str, true, Unit::Celsius, false) {
        Ok(answer) => {
          let value_str = if append_unit {
            format!("{} {:?}", answer.value, answer.unit)
          } else {
            answer.value.to_string()
          };
          str_results.push(value_str);
        }
        Err(e) => {
          // str_results.push(formatted);  // fallback initial value
          str_results.push(format!("ERROR: {}", e));
        }
      }
    } else if let Some(idx) = headers.iter().position(|h| h == &str_op.column) {
      let cell = row_fields[idx].clone();
      let cell_opt = headers
        .iter()
        .position(|h| h == &str_op.column)
        .map(|idx| row_fields[idx].clone());
      let length = match str_op.mode.as_str() {
        "left" | "right" | "slice" | "split" | "pad_left" | "pad_right" | "pad_both" => str_op
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
          let re: &'static Regex = regex_oncelock!(r"\s+");
          row_fields[idx] = re.replace_all(&cell, " ").into_owned();
        }
        "strip" => {
          let re: &'static Regex = regex_oncelock!(r"[\r\n]+");
          row_fields[idx] = re.replace_all(&cell, " ").into_owned();
        }
        "normalize" => {
          let re: &'static Regex = regex_oncelock!(r"^(\d+(?:\.\d+)?)([+-])$");
          if let Some(caps) = re.captures(&cell) {
            let number = &caps[1];
            let sign = &caps[2];
            row_fields[idx] = match sign {
              "-" => format!("-{number}"),
              _ => format!("{number}"), // "+"
            };
          }
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
        // add new column
        "pinyin" => {
          let result = if let Some(cell) = cell_opt {
            let py_mode_string = str_op.replacement.clone().unwrap_or_default();
            cell
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
              .collect::<String>()
          } else {
            String::new()
          };
          str_results.push(result);
        }
        "left" => {
          let result = if let Some(cell) = cell_opt {
            cell.chars().take(length).collect()
          } else {
            String::new()
          };
          str_results.push(result);
        }
        "right" => {
          let result = if let Some(cell) = cell_opt {
            cell
              .chars()
              .rev()
              .take(length)
              .collect::<String>()
              .chars()
              .rev()
              .collect()
          } else {
            String::new()
          };
          str_results.push(result);
        }
        "slice" => {
          let result = if let Some(cell) = cell_opt {
            let offset = str_op
              .comparand
              .clone()
              .ok_or(anyhow!("start index is invalid number"))?
              .parse::<isize>()?;
            let start = offset - 1;
            let end = start + length as isize;
            cell
              .chars()
              .skip(start.max(0) as usize)
              .take((end - start) as usize)
              .collect::<String>()
          } else {
            String::new()
          };
          str_results.push(result);
        }
        "split" => {
          let result = if let Some(cell) = cell_opt {
            let sep = &str_op
              .comparand
              .clone()
              .ok_or(anyhow!("delimiter is invalid"))?;
            let split_parts: Vec<&str> = cell.split(sep).collect();
            if split_parts.len() >= length {
              split_parts[length - 1].to_string()
            } else {
              String::new()
            }
          } else {
            String::new()
          };
          str_results.push(result);
        }
        "pad_left" => {
          let result = if let Some(cell) = cell_opt {
            let fill_char = str_op
              .comparand
              .clone()
              .ok_or(anyhow!("fill char is empty"))?;
            if cell.len() >= length {
              cell
            } else {
              let pad_len = length - cell.len();
              let pad = fill_char.repeat(pad_len);
              format!("{}{}", pad, cell)
            }
          } else {
            String::new()
          };
          str_results.push(result);
        }
        "pad_right" => {
          let result = if let Some(cell) = cell_opt {
            let fill_char = str_op
              .comparand
              .clone()
              .ok_or(anyhow!("fill char is empty"))?;
            if cell.len() >= length {
              cell
            } else {
              let pad_len = length - cell.len();
              let pad = fill_char.repeat(pad_len);
              format!("{}{}", cell, pad)
            }
          } else {
            String::new()
          };
          str_results.push(result);
        }
        "pad_both" => {
          let result = if let Some(cell) = cell_opt {
            let fill_char = str_op
              .comparand
              .clone()
              .ok_or(anyhow!("fill char is empty"))?;
            if cell.len() >= length {
              cell
            } else {
              let total_pad = length - cell.len();
              let left_pad = total_pad / 2;
              let right_pad = total_pad - left_pad;
              let left_pad_str = fill_char.to_string().repeat(left_pad);
              let right_pad_str = fill_char.to_string().repeat(right_pad);
              format!("{}{}{}", left_pad_str, cell, right_pad_str)
            }
          } else {
            String::new()
          };
          str_results.push(result);
        }
        "len" => str_results.push(cell.chars().count().to_string()),
        "copy" => str_results.push(cell.clone()),
        _ => str_results.push(cell),
      }
    } else {
      // 字段找不到时,只有新增列的操作才追加空字符串
      match str_op.mode.as_str() {
        "fill" | "f_fill" | "lower" | "upper" | "trim" | "ltrim" | "rtrim" | "squeeze"
        | "strip" | "replace" | "regex_replace" | "round" | "reverse" | "abs" | "neg"
        | "normalize" => {}
        _ => {
          str_results.push(String::new());
        }
      }
    }
  }

  Ok((row_fields, str_results))
}
