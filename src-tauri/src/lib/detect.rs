use std::{
  collections::HashMap,
  fs::File,
  io::{BufRead, BufReader},
  path::Path,
};

use anyhow::{anyhow, Result};
use csv::ByteRecord;

pub fn detect_separator<P>(path: P, skip_rows: usize) -> Option<char>
where
  P: AsRef<Path>,
{
  let file = File::open(path).expect("Failed to open file");
  let reader = BufReader::new(file);

  let mut lines_iter = reader.lines();

  // Skip the first `skip_rows` lines
  for _ in 0..skip_rows {
    if let Some(Ok(_)) = lines_iter.next() {
      // Line skipped
    } else {
      // If there are not enough lines to skip, return None or handle as you see fit
      return None;
    }
  }

  let mut line = String::new();
  let mut separators_count = HashMap::new();
  let mut max_count = 0;
  let mut separator = None;

  // read next line after skipping
  if let Some(Ok(next_line)) = lines_iter.next() {
    line.push_str(&next_line);

    // count all possible occurrences of segmentation symbols
    for c in [';', ',', '\t', '|', '^'] {
      let count = line.matches(c).count();
      if count > max_count {
        max_count = count;
        separator = Some(c);
      }
      separators_count.insert(c, count);
    }
  }

  separator
}

type ByteString = Vec<u8>;

pub struct Selection {
  indices: Vec<usize>,
}

impl Selection {
  pub fn from_headers(headers: &ByteRecord, field_names: &[&str]) -> Result<Self> {
    let header_map: HashMap<_, _> = headers
      .iter()
      .enumerate()
      .map(|(idx, name)| (String::from_utf8_lossy(name).into_owned(), idx))
      .collect();
    let mut indices = Vec::new();
    for &field_name in field_names {
      match header_map.get(field_name) {
        Some(&index) => indices.push(index),
        None => return Err(anyhow!("Field '{}' not found in headers.", field_name).into()),
      }
    }

    Ok(Selection { indices })
  }

  pub fn get_row_key(&self, row: &ByteRecord) -> Vec<ByteString> {
    self
      .indices
      .iter()
      .filter_map(|&idx| row.get(idx).map(ByteString::from))
      .collect()
  }

  pub fn get_indices(&self) -> &Vec<usize> {
    &self.indices
  }
}
