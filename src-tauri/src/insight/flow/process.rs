use std::{collections::HashMap, fs::File, io::BufWriter, path::PathBuf, sync::Arc};

use anyhow::{Result, anyhow};
use csv::{ReaderBuilder, WriterBuilder};

use crate::flow::filter;
use crate::flow::str::str_process;
use crate::flow::utils::{ColumnSource, FilterLogic, Operation, ProcessContext};
use crate::io::csv::options::CsvOptions;

pub async fn process_operations(
  input_path: String,
  operations: &[Operation],
  output_path: PathBuf,
) -> Result<()> {
  let opts = CsvOptions::new(&input_path);
  let sep = opts.detect_separator()?;

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(opts.rdr_skip_rows()?);

  let original_headers: Vec<String> = rdr.headers()?.iter().map(|s| s.to_string()).collect();

  let mut context = ProcessContext::new();

  for op in operations {
    match op.op.as_str() {
      "select" => {
        if let Some(column) = &op.column {
          let columns: Vec<&str> = column.split('|').collect();
          context.add_select(&columns);
        }
      }
      "filter" => {
        if let (Some(col), Some(mode), Some(val), Some(logic)) =
          (&op.column, &op.mode, &op.value, &op.logic)
        {
          let col = Arc::from(col.as_str());
          let val = Arc::from(val.as_str());
          let headers = Arc::new(original_headers.clone());
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
            _ => return Err(anyhow!("Not supported filter mode: {}", mode)),
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
      "rename" => {
        if let (Some(old_name), Some(new_name)) = (&op.column, &op.value) {
          context.add_rename(old_name, new_name);
        } else {
          return Err(anyhow!(
            "rename operation requires 'column' (old name) and 'value' (new name)"
          ));
        }
      }
      _ => return Err(anyhow!("Not supported operation: {}", op.op)),
    }
  }

  let dynamic_col_names: Vec<String> = context
    .str_ops
    .iter()
    .filter(|op| op.produces_new_column())
    .map(|op| match op.mode.as_str() {
      "cat" => format!("cat{}", op.id),
      "calcconv" => format!("calcconv{}", op.id),
      mode => format!("{}_{}{}", op.column, mode, op.id),
    })
    .collect();

  let logical_columns: Vec<String> = original_headers
    .iter()
    .cloned()
    .chain(dynamic_col_names.iter().cloned())
    .collect();

  let mut final_column_names = logical_columns.clone();
  let name_to_index: HashMap<_, _> = logical_columns
    .iter()
    .enumerate()
    .map(|(i, name)| (name, i))
    .collect();

  for (old_name, new_name) in &context.rename_columns {
    if let Some(&idx) = name_to_index.get(old_name) {
      final_column_names[idx] = new_name.clone();
    } else {
      return Err(anyhow!(
        "Cannot rename column '{}': not found in available columns.\nAvailable: {:?}",
        old_name,
        logical_columns
      ));
    }
  }

  let (output_headers, output_column_sources) =
    if let Some(ref select_cols) = context.select_columns {
      let mut sources = Vec::with_capacity(select_cols.len());
      let mut headers_out = Vec::with_capacity(select_cols.len());

      for col_name in select_cols {
        if let Some(idx) = logical_columns.iter().position(|h| h == col_name) {
          let source = if idx < original_headers.len() {
            ColumnSource::Original(idx)
          } else {
            ColumnSource::Dynamic(idx - original_headers.len())
          };
          sources.push(source);
          headers_out.push(final_column_names[idx].clone());
        } else {
          return Err(anyhow!(
            "Column '{}' not found in input headers or generated columns.\n\
                     Available logical columns: {:?}",
            col_name,
            logical_columns
          ));
        }
      }
      (headers_out, Some(sources))
    } else {
      let sources: Vec<ColumnSource> = (0..logical_columns.len())
        .map(|i| {
          if i < original_headers.len() {
            ColumnSource::Original(i)
          } else {
            ColumnSource::Dynamic(i - original_headers.len())
          }
        })
        .collect();
      (final_column_names, Some(sources))
    };

  context.output_column_sources = output_column_sources;

  let output_file = File::create(output_path)?;
  let buf_wtr = BufWriter::with_capacity(256_000, output_file);
  let mut wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_wtr);
  wtr.write_record(&output_headers)?;

  for result in rdr.records() {
    let record = result?;
    let (row_fields, str_results) = str_process(&record, &context, &original_headers)?;

    if context.is_valid(&record) {
      let output_row: Vec<&str> = context
        .output_column_sources
        .as_ref()
        .unwrap()
        .iter()
        .map(|source| match source {
          ColumnSource::Original(idx) => row_fields.get(*idx).map_or("", |s| s.as_str()),
          ColumnSource::Dynamic(idx) => str_results.get(*idx).map_or("", |s| s.as_str()),
        })
        .collect();

      wtr.write_record(&output_row)?;
    }
  }

  Ok(wtr.flush()?)
}
