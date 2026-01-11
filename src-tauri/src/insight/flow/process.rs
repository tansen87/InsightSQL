use std::{fs::File, io::BufWriter, path::PathBuf, sync::Arc};

use anyhow::{Result, anyhow};
use csv::{ReaderBuilder, WriterBuilder};

use crate::flow::filter;
use crate::flow::str::str_process;
use crate::flow::utils::{FilterLogic, Operation, ProcessContext};
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
        }
      }
      "filter" => {
        if let (Some(col), Some(mode), Some(val), Some(logic)) =
          (&op.column, &op.mode, &op.value, &op.logic)
        {
          let col = Arc::from(col.as_str());
          let val = Arc::from(val.as_str());
          let headers = Arc::new(headers.to_vec());
          let logic = FilterLogic::from(logic.as_str());
          match mode.as_str() {
            "equal" => context.add_filter(filter::equal(col, val, headers)?, logic),
            "not_equal" => context.add_filter(filter::not_equal(col, val, headers)?, logic),
            "contains" => context.add_filter(filter::contains(col, val, headers)?, logic),
            "not_contains" => context.add_filter(filter::not_contains(col, val, headers)?, logic),
            "starts_with" => context.add_filter(filter::starts_with(col, val, headers)?, logic),
            "not_starts_with" => {
              context.add_filter(filter::not_starts_with(col, val, headers)?, logic)
            }
            "ends_with" => context.add_filter(filter::ends_with(col, val, headers)?, logic),
            "not_ends_with" => context.add_filter(filter::not_ends_with(col, val, headers)?, logic),
            "gt" => context.add_filter(filter::gt(col, val, headers)?, logic),
            "ge" => context.add_filter(filter::ge(col, val, headers)?, logic),
            "lt" => context.add_filter(filter::lt(col, val, headers)?, logic),
            "le" => context.add_filter(filter::le(col, val, headers)?, logic),
            "between" => context.add_filter(filter::between(col, val, headers)?, logic),
            "is_null" => context.add_filter(filter::is_null(col, headers)?, logic),
            "is_not_null" => context.add_filter(filter::is_not_null(col, headers)?, logic),
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
          );
        } else if let Some(mode) = &op.mode {
          if mode == "cat" || mode == "calcconv" {
            context.add_str(
              &op.id,
              "",
              mode,
              op.comparand.as_deref(),
              op.replacement.as_deref(),
            );
          }
        }
      }
      _ => return Err(anyhow!("Not support operation: {}", op.op)),
    }
  }

  let output_file = File::create(output_path)?;
  let buf_wtr = BufWriter::with_capacity(256_000, output_file);
  let mut wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_wtr);

  // 构建输出header
  let output_headers: Vec<String> = if let Some(ref selected) = context.select {
    let mut selected_headers: Vec<String> =
      selected.iter().map(|&idx| headers[idx].clone()).collect();

    for str_op in &context.str_ops {
      match str_op.mode.as_str() {
        "fill" | "f_fill" | "lower" | "upper" | "trim" | "ltrim" | "rtrim" | "squeeze"
        | "strip" | "replace" | "regex_replace" | "round" | "reverse" | "abs" | "neg"
        | "normalize" => continue,
        _ => {}
      }
      let str_name = match str_op.mode.as_str() {
        "cat" => format!("cat{}", str_op.id),
        "calcconv" => format!("calcconv{}", str_op.id),
        mode => format!("{}_{}{}", str_op.column, mode, str_op.id),
      };
      selected_headers.push(str_name);
    }
    selected_headers
  } else {
    let mut all_headers: Vec<String> = headers.iter().map(|s| s.to_string()).collect();
    for str_op in &context.str_ops {
      match str_op.mode.as_str() {
        "fill" | "f_fill" | "lower" | "upper" | "trim" | "ltrim" | "rtrim" | "squeeze"
        | "strip" | "replace" | "regex_replace" | "round" | "reverse" | "abs" | "neg"
        | "normalize" => continue,
        _ => {}
      }
      let str_name = match str_op.mode.as_str() {
        "cat" => format!("cat{}", str_op.id),
        "calcconv" => format!("calcconv{}", str_op.id),
        mode => format!("{}_{}{}", str_op.column, mode, str_op.id),
      };
      all_headers.push(str_name);
    }
    all_headers
  };

  wtr.write_record(&output_headers)?;

  for result in rdr.records() {
    let record = result?;
    let (row_fields, str_results) = str_process(&record, &context, &headers)?;

    if context.is_valid(&record) {
      let output_row: Vec<&str> = if let Some(selected) = &context.select {
        let mut filtered: Vec<&str> = selected
          .iter()
          .map(|&idx| row_fields.get(idx).map_or("", |s| s.as_str()))
          .collect();
        filtered.extend(str_results.iter().map(|s| s.as_str()));
        filtered
      } else {
        let mut all_fields: Vec<&str> = row_fields.iter().map(|s| s.as_str()).collect();
        all_fields.extend(str_results.iter().map(|s| s.as_str()));
        all_fields
      };
      wtr.write_record(&output_row)?;
    }
  }

  Ok(())
}
