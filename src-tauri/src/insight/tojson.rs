use anyhow::Result;
use csv::ReaderBuilder;
use serde_json::{Value, json};

use crate::io::csv::options::CsvOptions;

pub fn csv_to_json(path: String, skiprows: usize) -> Result<String> {
  let n_rows = 20 + skiprows;
  let mut opts = CsvOptions::new(&path);
  opts.set_skiprows(skiprows);
  let (sep, reader) = opts.skiprows_and_delimiter()?;

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(reader);

  let headers = rdr.headers()?.clone();

  let mut json_records: Vec<Value> = Vec::with_capacity(n_rows);

  for result in rdr.records().take(n_rows) {
    let record = result?;

    let mut json_obj = serde_json::map::Map::new();

    for (header, value) in headers.iter().zip(record.iter()) {
      json_obj.insert(header.to_string(), json!(value));
    }

    json_records.push(json_obj.into());
  }

  let json_output = serde_json::to_string_pretty(&json_records)?;

  Ok(json_output)
}
