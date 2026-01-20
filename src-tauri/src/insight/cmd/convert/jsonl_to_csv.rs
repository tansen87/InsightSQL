use std::{
  fs::File,
  io::{BufRead, BufReader},
};

use anyhow::{Result, anyhow};
use csv::StringRecord;
use rayon::{
  iter::{IndexedParallelIterator, ParallelIterator},
  prelude::IntoParallelRefIterator,
};
use serde_json::Value;

use crate::{io::csv::options::CsvOptions, utils};

fn recurse_to_infer_headers(value: &Value, headers: &mut Vec<Vec<String>>, path: &[String]) {
  match value {
    Value::Object(map) => {
      for (key, value) in map {
        match value {
          Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) | Value::Array(_) => {
            let mut full_path = path.to_owned();
            full_path.push(key.to_string());

            headers.push(full_path);
          }
          Value::Object(_) => {
            let mut new_path = path.to_owned();
            new_path.push(key.to_string());

            recurse_to_infer_headers(value, headers, &new_path);
          }
          #[allow(unreachable_patterns)]
          _ => {}
        }
      }
    }
    _ => {
      headers.push(vec![String::from("value")]);
    }
  }
}

fn infer_headers(value: &Value) -> Vec<Vec<String>> {
  let mut headers: Vec<Vec<String>> = Vec::new();

  recurse_to_infer_headers(value, &mut headers, &Vec::new());

  headers
}

fn get_value_at_path(value: &Value, path: &[String]) -> Option<Value> {
  let mut current = value;

  for key in path {
    match current.get(key) {
      Some(new_value) => {
        current = new_value;
      }
      None => {
        return None;
      }
    }
  }

  Some(current.clone())
}

#[inline]
fn json_line_to_csv_record(value: &Value, headers: &[Vec<String>]) -> StringRecord {
  let mut record = StringRecord::new();

  for path in headers {
    let value = get_value_at_path(value, path);

    match value {
      Some(value) => {
        record.push_field(&match value {
          Value::Bool(v) => {
            if v {
              String::from("true")
            } else {
              String::from("false")
            }
          }
          Value::Number(v) => v.to_string(),
          Value::String(v) => v,
          Value::Array(v) => v
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(","),
          _ => String::new(),
        });
      }
      _ => {
        record.push_field("");
      }
    }
  }

  record
}

pub async fn jsonl_to_csv(path: &str, wtr_sep: &str, ignore_err: bool) -> Result<()> {
  let sep = if wtr_sep == "\\t" {
    b'\t'
  } else {
    wtr_sep.as_bytes().get(0).copied().unwrap_or(b',')
  };
  let opts = CsvOptions::new(path);
  let output_path = opts.output_path(Some("jsonl"), None)?;
  let mut wtr = csv::WriterBuilder::new()
    .delimiter(sep)
    .from_path(output_path)?;

  let mut rdr = BufReader::with_capacity(utils::RDR_BUFFER_SIZE, File::open(path)?);

  let mut headers: Vec<Vec<String>> = Vec::new();
  let mut headers_emitted: bool = false;

  // amortize memory allocation by reusing record
  let mut batch_line = String::new();

  // reuse batch buffers
  let batchsize: usize = opts.count_lines()?;
  let mut batch = Vec::with_capacity(batchsize);
  let mut batch_results = Vec::with_capacity(batchsize);

  utils::num_cpus();

  let mut result_idx = 0_u64;

  'batch_loop: loop {
    for _ in 0..batchsize {
      batch_line.clear();
      match rdr.read_line(&mut batch_line) {
        Ok(0) => {
          // EOF
          break;
        }
        Ok(_) => {
          batch.push(batch_line.clone());
        }
        Err(e) => {
          if ignore_err {
            continue;
          }
          return Err(anyhow!(
            "Could not read input line: {e}. Set `ignore errors` true to skip malformed input lines."
          ));
        }
      }
    }

    if batch.is_empty() {
      break 'batch_loop; // EOF
    }

    if !headers_emitted {
      let value: Value = match serde_json::from_str(&batch[0]) {
        Ok(v) => v,
        Err(e) => {
          return Err(anyhow!(
            "Could not parse first input line as JSON to infer headers: {e}"
          ));
        }
      };
      headers = infer_headers(&value);

      let headers_formatted = headers.iter().map(|v| v.join(".")).collect::<Vec<String>>();
      let headers_record = csv::StringRecord::from(headers_formatted);
      wtr.write_record(&headers_record)?;

      headers_emitted = true;
    }

    // do actual work via rayon
    batch
      .par_iter()
      .map(|json_line| match serde_json::from_str(json_line) {
        Ok(v) => Some(json_line_to_csv_record(&v, &headers)),
        Err(e) => {
          if !ignore_err {
            log::error!("serde_json::from_str error: {e:#?}");
          }
          None
        }
      })
      .collect_into_vec(&mut batch_results);

    // rayon collect() guarantees original order, so we can just append results of each batch
    for result_record in &batch_results {
      result_idx += 1;
      if let Some(record) = result_record {
        wtr.write_record(record)?;
      } else if !ignore_err {
        // there was an error parsing a json line
        return Err(anyhow!(
          "Could not parse input line {result_idx} as JSON. Set `ignore errors` true to skip malformed input lines.",
        ));
      }
    }

    batch.clear();
  } // end batch loop

  Ok(wtr.flush()?)
}
