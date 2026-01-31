use std::{collections::HashMap, fs::File, io::BufWriter, path::PathBuf, sync::Arc};

use anyhow::{Result, anyhow};
use csv::{ReaderBuilder, StringRecord, WriterBuilder};

use crate::flow::filter;
use crate::flow::operation;
use crate::flow::str::str_process;
use crate::flow::utils::{ColumnSource, FilterLogic, Operation, ProcessContext};
use crate::io::csv::options::CsvOptions;
use crate::utils::WTR_BUFFER_SIZE;

pub async fn process_operations(
  input_path: String,
  operations: &[Operation],
  output_path: PathBuf,
  quoting: bool,
) -> Result<()> {
  if let Some(rename_map) = operation::is_pure_rename(operations) {
    return operation::process_rename_only(input_path, rename_map, output_path, quoting);
  }
  if let Some(select_cols) = operation::is_pure_select(operations) {
    return operation::process_select_only(input_path, select_cols, output_path, quoting);
  }
  if let Some(filter_op) = operation::is_pure_filter(operations) {
    return operation::process_filter_only(input_path, filter_op, output_path, quoting);
  }
  if operation::is_pure_str(operations) {
    return operation::process_pure_str_fast(input_path, operations, output_path, quoting);
  }
  if operation::is_select_and_filter_only(operations) {
    return operation::process_select_filter(input_path, operations, output_path, quoting);
  }

  let opts = CsvOptions::new(input_path);
  let (sep, reader) = opts.skiprows_and_delimiter()?;

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .quoting(quoting)
    .from_reader(reader);

  let original_headers: Vec<String> = rdr.headers()?.iter().map(|s| s.to_string()).collect();
  let headers_arc = Arc::new(original_headers.clone());
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
        let col = op
          .column
          .as_deref()
          .ok_or_else(|| anyhow!("Missing column in filter"))?;
        let mode = op
          .mode
          .as_deref()
          .ok_or_else(|| anyhow!("Missing mode in filter"))?;
        let val = op
          .value
          .as_deref()
          .ok_or_else(|| anyhow!("Missing value in filter"))?;
        let logic = FilterLogic::from(op.logic.as_deref().unwrap_or("or"));
        let col_arc = Arc::from(col);
        let val_arc = Arc::from(val);

        let filter_fn: Box<dyn Fn(&StringRecord) -> bool + Send + Sync> = match mode {
          "equal" => filter::equal(col_arc, val_arc, headers_arc.clone())?,
          "not_equal" => filter::not_equal(col_arc, val_arc, headers_arc.clone())?,
          "contains" => filter::contains(col_arc, val_arc, headers_arc.clone())?,
          "not_contains" => filter::not_contains(col_arc, val_arc, headers_arc.clone())?,
          "starts_with" => filter::starts_with(col_arc, val_arc, headers_arc.clone())?,
          "not_starts_with" => filter::not_starts_with(col_arc, val_arc, headers_arc.clone())?,
          "ends_with" => filter::ends_with(col_arc, val_arc, headers_arc.clone())?,
          "not_ends_with" => filter::not_ends_with(col_arc, val_arc, headers_arc.clone())?,
          "gt" => filter::gt(col_arc, val_arc, headers_arc.clone())?,
          "ge" => filter::ge(col_arc, val_arc, headers_arc.clone())?,
          "lt" => filter::lt(col_arc, val_arc, headers_arc.clone())?,
          "le" => filter::le(col_arc, val_arc, headers_arc.clone())?,
          "between" => filter::between(col_arc, val_arc, headers_arc.clone())?,
          "is_null" => filter::is_null(col_arc, headers_arc.clone())?,
          "is_not_null" => filter::is_not_null(col_arc, headers_arc.clone())?,
          _ => return Err(anyhow!("Not supported filter mode: {}", mode)),
        };

        context.add_filter(filter_fn, logic);
      }
      "str" => {
        if let (Some(col), Some(mode)) = (&op.column, &op.mode) {
          context.add_str(
            col,
            mode,
            op.comparand.as_deref(),
            op.replacement.as_deref(),
          );
        } else if let Some(mode) = &op.mode {
          if mode == "cat" || mode == "calcconv" {
            context.add_str(
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
      "cat" => format!("concatenated"),
      "calcconv" => format!("calculated"),
      mode => format!("{}_{}", op.column, mode),
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
  let buf_wtr = BufWriter::with_capacity(WTR_BUFFER_SIZE, output_file);
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
