use std::{
  collections::{HashMap, HashSet},
  fs::File,
  io::{BufRead, BufReader},
  path::Path,
};

use anyhow::{anyhow, Result};
use csv::{ByteRecord, ReaderBuilder};

pub struct CsvOptions<P: AsRef<Path>> {
  path: P,
  skip_rows: usize,
}

impl<P: AsRef<Path>> CsvOptions<P> {
  pub fn new(path: P) -> CsvOptions<P> {
    CsvOptions { path, skip_rows: 0 }
  }

  /// Count csv rows
  pub fn count_csv_rows(&self) -> Result<usize> {
    let file = File::open(&self.path)?;
    let reader = BufReader::new(file);
    let total_rows = reader.lines().count().saturating_sub(1);

    Ok(total_rows)
  }
}

/// Check the delimiter of CSV
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

  pub fn first_indices(&self) -> Result<usize> {
    self
      .indices
      .get(0)
      .copied()
      .ok_or(anyhow!("The indices vector is empty."))
  }
}

/// Get csv headers
pub async fn get_same_headers<P: AsRef<Path>>(path: P) -> Result<Vec<HashMap<String, String>>> {
  let sep = match detect_separator(&path, 0) {
    Some(separator) => separator as u8,
    None => b',',
  };

  let mut rdr = csv::ReaderBuilder::new()
    .delimiter(sep)
    .has_headers(true)
    .from_reader(File::open(&path)?);

  let headers: Vec<HashMap<String, String>> = rdr
    .headers()?
    .iter()
    .map(|header| {
      let mut map = HashMap::new();
      map.insert("value".to_string(), header.to_string());
      map.insert("label".to_string(), header.to_string());
      map
    })
    .collect();

  Ok(headers)
}

/// Skip the first n lines of csv
pub fn skip_csv_rows<P: AsRef<Path>>(path: P, skip_rows: usize) -> Result<BufReader<File>> {
  let mut reader = BufReader::new(File::open(&path)?);
  let mut line = String::new();

  for _ in 0..skip_rows {
    if reader.read_line(&mut line)? == 0 {
      // reached the end of the file before skipping all lines
      break;
    }
    line.clear();
  }

  Ok(reader)
}

/// The intersection of all headers between many csv files
pub async fn inter_headers<P: AsRef<Path>>(path: P, skip_rows: usize) -> Result<HashSet<String>> {
  let path4split = &path.as_ref().to_string_lossy();
  let ff: Vec<&str> = path4split.split('|').collect();
  let mut header_sets: Vec<HashSet<String>> = vec![];

  for f in ff {
    let skip_rows_reader = skip_csv_rows(f, skip_rows)?;
    let sep = match detect_separator(f, skip_rows) {
      Some(separator) => separator as u8,
      None => b',',
    };
    let mut rdr = ReaderBuilder::new()
      .delimiter(sep)
      .from_reader(skip_rows_reader);
    if let Ok(headers) = rdr.headers() {
      let header_set: HashSet<String> = headers.iter().map(|s| s.to_string()).collect();
      header_sets.push(header_set);
    }
  }

  // start with the assumption that all headers are common
  let mut common_headers: HashSet<String> = if let Some(first_set) = header_sets.first() {
    first_set.clone()
  } else {
    HashSet::new()
  };

  // find intersection of all sets
  for headers in header_sets.iter().skip(1) {
    common_headers = common_headers.intersection(headers).cloned().collect();
  }

  Ok(common_headers)
}
