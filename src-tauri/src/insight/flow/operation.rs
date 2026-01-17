use std::{collections::HashMap, fs::File, io::BufWriter, path::PathBuf, sync::Arc};

use anyhow::{Result, anyhow};
use csv::{ReaderBuilder, StringRecord, WriterBuilder};

use crate::flow::filter;
use crate::flow::str::str_process;
use crate::flow::utils::{Operation, ProcessContext};
use crate::io::csv::options::CsvOptions;
use crate::utils::WTR_BUFFER_SIZE;

pub(crate) fn is_pure_rename(ops: &[Operation]) -> Option<HashMap<String, String>> {
  if ops.iter().all(|op| op.op == "rename") {
    let mut map = HashMap::new();
    for op in ops {
      if let (Some(old), Some(new)) = (&op.column, &op.value) {
        map.insert(old.clone(), new.clone());
      } else {
        return None; // malformed rename
      }
    }
    Some(map)
  } else {
    None
  }
}

pub(crate) fn is_pure_select(ops: &[Operation]) -> Option<Vec<String>> {
  if ops.len() == 1 && ops[0].op == "select" {
    ops[0]
      .column
      .as_ref()
      .map(|col| col.split('|').map(|s| s.to_string()).collect())
  } else {
    None
  }
}

pub(crate) fn is_pure_filter(ops: &[Operation]) -> Option<&Operation> {
  if ops.len() == 1 && ops[0].op == "filter" {
    Some(&ops[0])
  } else {
    None
  }
}

pub(crate) fn is_pure_str(operations: &[Operation]) -> bool {
  operations.iter().all(|op| op.op == "str")
}

pub(crate) fn is_select_and_filter_only(ops: &[Operation]) -> bool {
  ops.len() == 2
    && ((ops[0].op == "select" && ops[1].op == "filter")
      || (ops[0].op == "filter" && ops[1].op == "select"))
}

pub(crate) fn process_rename_only(
  input_path: String,
  rename_map: HashMap<String, String>,
  output_path: PathBuf,
) -> Result<()> {
  let opts = CsvOptions::new(&input_path);
  let sep = opts.detect_separator()?;
  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(opts.rdr_skip_rows()?);

  let original_headers: Vec<String> = rdr.headers()?.iter().map(|s| s.to_string()).collect();

  let output_headers: Vec<String> = original_headers
    .iter()
    .map(|h| rename_map.get(h).cloned().unwrap_or_else(|| h.clone()))
    .collect();

  let output_file = File::create(output_path)?;
  let buf_wtr = BufWriter::with_capacity(WTR_BUFFER_SIZE, output_file);
  let mut wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_wtr);
  wtr.write_record(&output_headers)?;

  for result in rdr.records() {
    wtr.write_record(&result?)?;
  }

  Ok(wtr.flush()?)
}

pub(crate) fn process_select_only(
  input_path: String,
  select_cols: Vec<String>,
  output_path: PathBuf,
) -> Result<()> {
  let opts = CsvOptions::new(&input_path);
  let sep = opts.detect_separator()?;
  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(opts.rdr_skip_rows()?);

  let original_headers: Vec<String> = rdr.headers()?.iter().map(|s| s.to_string()).collect();

  let selected_indices: Vec<usize> = select_cols
    .iter()
    .map(|col| {
      original_headers
        .iter()
        .position(|h| h == col)
        .ok_or_else(|| {
          anyhow!(
            "Column '{}' not found. Available: {:?}",
            col,
            original_headers
          )
        })
    })
    .collect::<Result<Vec<_>>>()?;

  let output_headers: Vec<&String> = selected_indices
    .iter()
    .map(|&i| &original_headers[i])
    .collect();

  let output_file = File::create(output_path)?;
  let buf_wtr = BufWriter::with_capacity(WTR_BUFFER_SIZE, output_file);
  let mut wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_wtr);
  wtr.write_record(&output_headers)?;

  for result in rdr.records() {
    let record = result?;
    let selected_fields: Vec<&str> = selected_indices
      .iter()
      .map(|&i| record.get(i).unwrap_or(""))
      .collect();
    wtr.write_record(&selected_fields)?;
  }

  Ok(wtr.flush()?)
}

pub(crate) fn process_filter_only(
  input_path: String,
  filter_op: &Operation,
  output_path: PathBuf,
) -> Result<()> {
  let opts = CsvOptions::new(&input_path);
  let sep = opts.detect_separator()?;
  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(opts.rdr_skip_rows()?);

  let original_headers: Vec<String> = rdr.headers()?.iter().map(|s| s.to_string()).collect();
  let headers_arc = Arc::new(original_headers.clone());

  let (col, mode, val) = (
    filter_op
      .column
      .as_deref()
      .ok_or_else(|| anyhow!("Missing column in filter"))?,
    filter_op
      .mode
      .as_deref()
      .ok_or_else(|| anyhow!("Missing mode in filter"))?,
    filter_op
      .value
      .as_deref()
      .ok_or_else(|| anyhow!("Missing value in filter"))?,
  );

  let filter_fn: Box<dyn Fn(&StringRecord) -> bool + Send + Sync> = match mode {
    "equal" => filter::equal(Arc::from(col), Arc::from(val), headers_arc)?,
    "not_equal" => filter::not_equal(Arc::from(col), Arc::from(val), headers_arc)?,
    "contains" => filter::contains(Arc::from(col), Arc::from(val), headers_arc)?,
    "not_contains" => filter::not_contains(Arc::from(col), Arc::from(val), headers_arc)?,
    "starts_with" => filter::starts_with(Arc::from(col), Arc::from(val), headers_arc)?,
    "not_starts_with" => filter::not_starts_with(Arc::from(col), Arc::from(val), headers_arc)?,
    "ends_with" => filter::ends_with(Arc::from(col), Arc::from(val), headers_arc)?,
    "not_ends_with" => filter::not_ends_with(Arc::from(col), Arc::from(val), headers_arc)?,
    "gt" => filter::gt(Arc::from(col), Arc::from(val), headers_arc)?,
    "ge" => filter::ge(Arc::from(col), Arc::from(val), headers_arc)?,
    "lt" => filter::lt(Arc::from(col), Arc::from(val), headers_arc)?,
    "le" => filter::le(Arc::from(col), Arc::from(val), headers_arc)?,
    "between" => filter::between(Arc::from(col), Arc::from(val), headers_arc)?,
    "is_null" => filter::is_null(Arc::from(col), headers_arc)?,
    "is_not_null" => filter::is_not_null(Arc::from(col), headers_arc)?,
    _ => return Err(anyhow!("Unsupported filter mode: {}", mode)),
  };

  let output_file = File::create(output_path)?;
  let buf_wtr = BufWriter::with_capacity(WTR_BUFFER_SIZE, output_file);
  let mut wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_wtr);
  wtr.write_record(&original_headers)?;

  for result in rdr.records() {
    let record = result?;
    if filter_fn(&record) {
      wtr.write_record(&record)?;
    }
  }

  Ok(wtr.flush()?)
}

pub(crate) fn process_pure_str_fast(
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

  let mut context = ProcessContext {
    str_ops: Vec::new(),
    select_columns: None,
    filters: Vec::new(),
    rename_columns: Vec::new(),
    output_column_sources: None,
  };

  for op in operations {
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

  let output_headers: Vec<String> = original_headers
    .iter()
    .cloned()
    .chain(dynamic_col_names.into_iter())
    .collect();

  let output_file = File::create(output_path)?;
  let buf_wtr = BufWriter::with_capacity(WTR_BUFFER_SIZE, output_file);
  let mut wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_wtr);
  wtr.write_record(&output_headers)?;

  for result in rdr.records() {
    let record = result?;
    let (row_fields, str_results) = str_process(&record, &context, &original_headers)?;
    let mut output_row: Vec<&str> = row_fields.iter().map(|s| s.as_str()).collect();
    output_row.extend(str_results.iter().map(|s| s.as_str()));
    wtr.write_record(&output_row)?;
  }

  Ok(wtr.flush()?)
}

pub(crate) fn process_select_filter(
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
  let headers_arc = Arc::new(original_headers.clone());

  // 分离 select 和 filter
  let (select_op, filter_op) = if operations[0].op == "select" {
    (&operations[0], &operations[1])
  } else {
    (&operations[1], &operations[0])
  };

  // Parse select
  let select_cols = select_op
    .column
    .as_ref()
    .ok_or_else(|| anyhow!("Missing column in select"))?;
  let selected_names: Vec<&str> = select_cols.split('|').collect();
  let selected_indices: Vec<usize> = selected_names
    .iter()
    .map(|&col| {
      original_headers
        .iter()
        .position(|h| h == col)
        .ok_or_else(|| anyhow!("Column '{}' not found", col))
    })
    .collect::<Result<Vec<_>>>()?;
  let output_headers: Vec<&String> = selected_indices
    .iter()
    .map(|&i| &original_headers[i])
    .collect();

  // Parse filter
  let (col, mode, val, _logic) = (
    filter_op
      .column
      .as_deref()
      .ok_or_else(|| anyhow!("Missing column in filter"))?,
    filter_op
      .mode
      .as_deref()
      .ok_or_else(|| anyhow!("Missing mode in filter"))?,
    filter_op
      .value
      .as_deref()
      .ok_or_else(|| anyhow!("Missing value in filter"))?,
    filter_op.logic.as_deref().unwrap_or("and"),
  );

  let col_arc = Arc::from(col);
  let val_arc = Arc::from(val);

  let filter_fn: Box<dyn Fn(&StringRecord) -> bool + Send + Sync> = match mode {
    "equal" => filter::equal(col_arc, val_arc, headers_arc)?,
    "not_equal" => filter::not_equal(col_arc, val_arc, headers_arc)?,
    "contains" => filter::contains(col_arc, val_arc, headers_arc)?,
    "not_contains" => filter::not_contains(col_arc, val_arc, headers_arc)?,
    "starts_with" => filter::starts_with(col_arc, val_arc, headers_arc)?,
    "not_starts_with" => filter::not_starts_with(col_arc, val_arc, headers_arc)?,
    "ends_with" => filter::ends_with(col_arc, val_arc, headers_arc)?,
    "not_ends_with" => filter::not_ends_with(col_arc, val_arc, headers_arc)?,
    "gt" => filter::gt(col_arc, val_arc, headers_arc)?,
    "ge" => filter::ge(col_arc, val_arc, headers_arc)?,
    "lt" => filter::lt(col_arc, val_arc, headers_arc)?,
    "le" => filter::le(col_arc, val_arc, headers_arc)?,
    "between" => filter::between(col_arc, val_arc, headers_arc)?,
    "is_null" => filter::is_null(col_arc, headers_arc)?,
    "is_not_null" => filter::is_not_null(col_arc, headers_arc)?,
    _ => return Err(anyhow!("Unsupported filter mode: {}", mode)),
  };

  let output_file = File::create(output_path)?;
  let buf_wtr = BufWriter::with_capacity(WTR_BUFFER_SIZE, output_file);
  let mut wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_wtr);
  wtr.write_record(&output_headers)?;

  for result in rdr.records() {
    let record = result?;
    if filter_fn(&record) {
      let selected_fields: Vec<&str> = selected_indices
        .iter()
        .map(|&i| record.get(i).unwrap_or(""))
        .collect();
      wtr.write_record(&selected_fields)?;
    }
  }

  Ok(wtr.flush()?)
}
