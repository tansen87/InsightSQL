use csv::StringRecord;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Operation {
  pub op: String,
  pub mode: Option<String>,
  pub logic: Option<String>,
  pub column: Option<String>,
  pub value: Option<String>,
  pub comparand: Option<String>,
  pub replacement: Option<String>,
}

#[derive(Clone)]
pub struct StrOperation {
  pub column: String,
  pub mode: String,
  pub comparand: Option<String>,
  pub replacement: Option<String>,
}

impl StrOperation {
  pub fn produces_new_column(&self) -> bool {
    match self.mode.as_str() {
      // In-place modifications — do NOT produce new column
      "fill" | "f_fill" | "lower" | "upper" | "trim" | "ltrim" | "rtrim" | "squeeze" | "strip"
      | "replace" | "regex_replace" | "round" | "reverse" | "abs" | "neg" | "normalize" => false,
      // All others produce a new column
      _ => true,
    }
  }
}

pub struct Filter {
  pub filter: Box<dyn Fn(&StringRecord) -> bool + Send + Sync>,
  pub logic: FilterLogic,
}

#[derive(Debug, Deserialize)]
pub enum FilterLogic {
  And,
  Or,
}

impl From<&str> for FilterLogic {
  fn from(s: &str) -> Self {
    match s.to_lowercase().as_str() {
      "and" => FilterLogic::And,
      "or" => FilterLogic::Or,
      _ => FilterLogic::Or,
    }
  }
}

#[derive(Clone)]
pub enum ColumnSource {
  Original(usize),
  Dynamic(usize),
}

pub struct ProcessContext {
  pub select_columns: Option<Vec<String>>,
  pub filters: Vec<Filter>,
  pub str_ops: Vec<StrOperation>,
  pub output_column_sources: Option<Vec<ColumnSource>>,
  pub rename_columns: Vec<(String, String)>,
}

impl ProcessContext {
  pub fn new() -> Self {
    ProcessContext {
      select_columns: None,
      filters: Vec::new(),
      str_ops: Vec::new(),
      output_column_sources: None,
      rename_columns: Vec::new(),
    }
  }

  pub fn add_select(&mut self, columns: &[&str]) {
    self.select_columns = Some(columns.iter().map(|s| s.to_string()).collect());
  }

  pub fn add_filter<F>(&mut self, filter: F, logic: FilterLogic)
  where
    F: Fn(&StringRecord) -> bool + Send + Sync + 'static,
  {
    self.filters.push(Filter {
      filter: Box::new(filter),
      logic,
    });
  }

  pub fn add_str(
    &mut self,
    column: &str,
    mode: &str,
    comparand: Option<&str>,
    replacement: Option<&str>,
  ) {
    self.str_ops.push(StrOperation {
      column: column.to_string(),
      mode: mode.to_string(),
      comparand: comparand.map(|s| s.to_string()),
      replacement: replacement.map(|s| s.to_string()),
    });
  }

  pub fn add_rename(&mut self, column: &str, value: &str) {
    self
      .rename_columns
      .push((column.to_string(), value.to_string()));
  }

  pub fn is_valid(&self, record: &StringRecord) -> bool {
    if self.filters.is_empty() {
      return true;
    }

    // 第一个 filter 的结果作为初始值
    let mut result = (self.filters[0].filter)(record);

    // 从第二个开始,依次应用上一个 filter 的 logic
    for i in 1..self.filters.len() {
      let current_value = (self.filters[i].filter)(record);
      let prev_logic = &self.filters[i - 1].logic;

      match prev_logic {
        FilterLogic::And => result = result && current_value,
        FilterLogic::Or => result = result || current_value,
      }
    }

    result
  }
}
