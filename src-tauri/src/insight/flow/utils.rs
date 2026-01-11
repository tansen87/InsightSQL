use csv::StringRecord;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Operation {
  pub id: String,
  pub op: String,
  pub mode: Option<String>,
  pub logic: Option<String>,
  pub column: Option<String>,
  pub value: Option<String>,
  pub comparand: Option<String>,
  pub replacement: Option<String>,
}

pub struct StrOperation {
  pub id: String,
  pub column: String,
  pub mode: String,
  pub comparand: Option<String>,
  pub replacement: Option<String>,
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

pub struct ProcessContext {
  pub select: Option<Vec<usize>>,
  pub filters: Vec<Filter>,
  pub filter_logic: FilterLogic,
  pub str_ops: Vec<StrOperation>,
}

impl ProcessContext {
  pub fn new() -> Self {
    ProcessContext {
      select: None,
      filters: Vec::new(),
      filter_logic: FilterLogic::Or,
      str_ops: Vec::new(),
    }
  }

  pub fn add_select(&mut self, columns: &[&str], header: &[String]) {
    let selected_indices: Vec<usize> = columns
      .iter()
      .filter_map(|col| header.iter().position(|h| h == *col))
      .collect();

    self.select = Some(selected_indices);
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
    id: &str,
    column: &str,
    mode: &str,
    comparand: Option<&str>,
    replacement: Option<&str>,
  ) {
    self.str_ops.push(StrOperation {
      id: id.to_string(),
      column: column.to_string(),
      mode: mode.to_string(),
      comparand: comparand.map(|s| s.to_string()),
      replacement: replacement.map(|s| s.to_string()),
    });
  }

  pub fn set_filter_logic(&mut self, logic: FilterLogic) {
    self.filter_logic = logic
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
        FilterLogic::And => {
          result = result && current_value;
        }
        FilterLogic::Or => {
          result = result || current_value;
        }
      }
    }

    result
  }
}
