use std::{collections::HashSet, fs::File, io::BufReader};

use anyhow::{Result, anyhow};
use csv::WriterBuilder;
use json_objects_to_csv::{Json2Csv, flatten_json_object::Flattener};

use crate::{io::csv::options::CsvOptions, utils::RDR_BUFFER_SIZE};

pub async fn json_to_csv(path: &str, wtr_sep: String) -> Result<()> {
  let sep = if wtr_sep == "\\t" {
    b'\t'
  } else {
    wtr_sep.into_bytes()[0]
  };
  let file = File::open(path)?;
  let reader = BufReader::with_capacity(RDR_BUFFER_SIZE, file);

  let value: serde_json::Value = serde_json::from_reader(reader)?;
  if value.is_null() {
    return Err(anyhow!("No JSON data found."));
  }

  let first_dict = if value.is_array() {
    value
      .as_array()
      .and_then(|arr| arr.first())
      .and_then(|v| v.as_object())
      .ok_or_else(|| anyhow!("Expected an array of objects in JSON"))?
  } else {
    value
      .as_object()
      .ok_or_else(|| anyhow!("Expected a JSON object"))?
  };

  if first_dict.is_empty() {
    return Err(anyhow!("Expected a non-empty JSON object"));
  }

  let mut seen_keys = HashSet::new();
  for key in first_dict.keys() {
    if key.is_empty() {
      return Err(anyhow!("Empty JSON key found"));
    }
    if !seen_keys.insert(key) {
      return Err(anyhow!("Duplicate key: {key}"));
    }
  }

  let empty_values = vec![serde_json::Value::Null; 1];
  let values = if value.is_array() {
    value.as_array().unwrap_or(&empty_values)
  } else {
    &vec![value.clone()]
  };

  let opts = CsvOptions::new(path);
  let output_path = opts.output_path(Some("json"), None)?;

  let wtr = WriterBuilder::new()
    .has_headers(true)
    .delimiter(sep)
    .from_path(output_path)?;

  let flattener = Flattener::new();
  Json2Csv::new(flattener)
    .preserve_key_order(true)
    .convert_from_array(&values, wtr)?;

  Ok(())
}
