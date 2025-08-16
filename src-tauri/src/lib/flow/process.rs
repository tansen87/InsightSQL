use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use csv::ReaderBuilder;
use csv::WriterBuilder;

use crate::flow::filter;
use crate::flow::str::str_process;
use crate::flow::utils::Operation;
use crate::flow::utils::ProcessContext;
use crate::io::csv::options::CsvOptions;

pub async fn process_operations(
  input_path: String,
  operations: &[Operation],
  output_path: PathBuf,
) -> Result<()> {
  let opts = CsvOptions::new(input_path);
  let sep = opts.detect_separator()?;

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(opts.rdr_skip_rows()?);
  let headers = rdr
    .headers()?
    .clone()
    .iter()
    .map(|s| s.to_string())
    .collect::<Vec<_>>();

  let mut context = ProcessContext::new();

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
            &op.id,
            col,
            mode,
            op.comparand.as_deref(),
            op.replacement.as_deref(),
            op.alias.clone(),
          );
        } else if let Some(mode) = &op.mode {
          if mode == "cat" || mode == "calcconv" {
            context.add_str(
              &op.id,
              "",
              mode,
              op.comparand.as_deref(),
              op.replacement.as_deref(),
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
    let mut new_field_is_slice = Vec::new(); // true=slice, false=str
    let mut new_field_aliases = Vec::new();

    for str_op in &context.str_ops {
      match str_op.mode.as_str() {
        "fill" | "f_fill" | "lower" | "upper" | "trim" | "ltrim" | "rtrim" | "squeeze"
        | "strip" | "replace" | "regex_replace" | "round" | "reverse" | "abs" | "neg"
        | "pinyin" | "left" | "right" | "slice" | "split" => {
          continue;
        }
        _ => {}
      }
      let str_name = match &str_op.alias {
        Some(alias) => alias.clone(),
        None => match str_op.mode.as_str() {
          "cat" => format!("cat{}", str_op.id),
          "calcconv" => format!("calcconv{}", str_op.id),
          mode => format!("{}_{}{}", str_op.column, mode, str_op.id),
        },
      };
      new_field_names.push(str_name.clone());
      new_field_is_slice.push(false);
      new_field_aliases.push(str_name);
    }

    for str_op in &context.str_ops {
      match str_op.mode.as_str() {
        "fill" | "f_fill" | "lower" | "upper" | "trim" | "ltrim" | "rtrim" | "squeeze"
        | "strip" | "replace" | "regex_replace" | "round" | "reverse" | "abs" | "neg"
        | "pinyin" | "left" | "right" | "slice" | "split" => {
          continue;
        }
        _ => {}
      };
      let str_name = match &str_op.alias {
        Some(alias) => alias.clone(),
        None => match str_op.mode.as_str() {
          "cat" => format!("cat{}", str_op.id),
          "calcconv" => format!("calcconv{}", str_op.id),
          mode => format!("{}_{}{}", str_op.column, mode, str_op.id),
        },
      };
      selected_headers.push(str_name);
    }

    wtr.write_record(&selected_headers)?;
  } else {
    let mut all_headers: Vec<String> = headers.iter().map(|s| s.to_string()).collect();

    for str_op in &context.str_ops {
      match str_op.mode.as_str() {
        "fill" | "f_fill" | "lower" | "upper" | "trim" | "ltrim" | "rtrim" | "squeeze"
        | "strip" | "replace" | "regex_replace" | "round" | "reverse" | "abs" | "neg"
        | "pinyin" | "left" | "right" | "slice" | "split" => {
          continue;
        }
        _ => {}
      };
      let str_name = match &str_op.alias {
        Some(alias) => alias.clone(),
        None => match str_op.mode.as_str() {
          "cat" => format!("cat{}", str_op.id),
          "calcconv" => format!("calcconv{}", str_op.id),
          mode => format!("{}_{}{}", str_op.column, mode, str_op.id),
        },
      };
      all_headers.push(str_name);
    }

    wtr.write_record(all_headers.iter().map(|s| s.as_str()))?;
  }

  for result in rdr.records() {
    let record = result?;

    let (row_fields, str_results) = str_process(&record, &context, &headers)?;

    if context.is_valid(&record) {
      if let Some(selected) = &context.select {
        let mut filtered: Vec<_> = selected
          .iter()
          .map(|&idx| row_fields.get(idx).map(|s| s.as_str()).unwrap_or(""))
          .collect();
        filtered.extend(str_results.iter().map(|s| s.as_str()));
        wtr.write_record(&filtered)?;
      } else {
        let mut all_fields: Vec<_> = row_fields.iter().map(|s| s.as_str()).collect();
        all_fields.extend(str_results.iter().map(|s| s.as_str()));
        wtr.write_record(&all_fields)?;
      }
    }
  }

  Ok(())
}
