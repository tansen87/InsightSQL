use csv::StringRecord;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Operation {
  pub op: String,
  pub mode: Option<String>,
  pub column: Option<String>,
  pub value: Option<String>,
  pub offset: Option<String>,
  pub length: Option<String>,
  pub comparand: Option<String>,
  pub replacement: Option<String>,
  pub alias: Option<String>,
}

pub struct SliceOperation {
  pub column: String,
  pub mode: String,
  pub offset: String,
  pub length: String,
  pub alias: Option<String>,
}

pub struct StringOperation {
  pub column: String,
  pub mode: String,
  pub comparand: Option<String>,
  pub replacement: Option<String>,
  pub alias: Option<String>,
}

pub struct ProcessingContext {
  pub select: Option<Vec<usize>>,
  pub alias: Option<Vec<Option<String>>>,
  pub filters: Vec<Box<dyn Fn(&StringRecord) -> bool + Send + Sync>>,
  pub slice_ops: Vec<SliceOperation>,
  pub string_ops: Vec<StringOperation>,
}

impl ProcessingContext {
  pub fn new() -> Self {
    ProcessingContext {
      select: None,
      alias: None,
      filters: Vec::new(),
      slice_ops: Vec::new(),
      string_ops: Vec::new(),
    }
  }

  pub fn add_select(&mut self, columns: &[&str], header: &[String]) {
    let selected_indices: Vec<usize> = columns
      .iter()
      .filter_map(|col| header.iter().position(|h| h == *col))
      .collect();

    self.select = Some(selected_indices);
  }

  pub fn add_filter<F>(&mut self, filter: F)
  where
    F: Fn(&StringRecord) -> bool + Send + Sync + 'static,
  {
    self.filters.push(Box::new(filter));
  }

  pub fn add_slice(
    &mut self,
    column: &str,
    mode: &str,
    offset: String,
    length: String,
    alias: Option<String>,
  ) {
    self.slice_ops.push(SliceOperation {
      column: column.to_string(),
      mode: mode.to_string(),
      offset,
      length,
      alias,
    });
  }

  pub fn add_string(
    &mut self,
    column: &str,
    mode: &str,
    comparand: Option<String>,
    replacement: Option<String>,
    alias: Option<String>,
  ) {
    self.string_ops.push(StringOperation {
      column: column.to_string(),
      mode: mode.to_string(),
      comparand,
      replacement,
      alias,
    });
  }

  // And
  // fn is_valid(&self, record: &StringRecord) -> bool {
  //     self.filters.iter().all(|f| f(record))
  // }

  // Or
  pub fn is_valid(&self, record: &StringRecord) -> bool {
    self.filters.is_empty() || self.filters.iter().any(|f| f(record))
  }
}
