use std::{
  collections::{HashMap, HashSet},
  fs::File,
  io::{BufRead, BufReader, Read},
  path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use csv::{ByteRecord, ReaderBuilder};

use crate::{excel_reader, index::Indexed};

type ByteString = Vec<u8>;

pub struct CsvOptions<P: AsRef<Path>> {
  path: P,
  skip_rows: usize,
}

impl<P: AsRef<Path>> CsvOptions<P> {
  pub fn new(path: P) -> CsvOptions<P> {
    CsvOptions { path, skip_rows: 0 }
  }

  /// Sets the number of rows to skip
  pub fn set_skip_rows(&mut self, skip_rows: usize) {
    self.skip_rows = skip_rows;
  }

  /// Get the numer of rows to skip
  pub fn get_skip_rows(&self) -> usize {
    self.skip_rows
  }

  /// Get the path
  pub fn get_path(&self) -> Option<&str> {
    self.path.as_ref().to_str()
  }

  /// Check the delimiter of CSV
  pub fn detect_separator(&self) -> Result<u8> {
    let file = File::open(&self.path)?;
    let reader = BufReader::new(file);

    let mut lines_iter = reader.lines();

    // Skip the first `skip_rows` lines
    for _ in 0..self.skip_rows {
      if let Some(Ok(_)) = lines_iter.next() {
        // Line skipped
      } else {
        // If there are not enough lines to skip, return Err
        return Err(anyhow!("there are not enough lines to skip"));
      }
    }

    let seg_symbols = [b';', b',', b'\t', b'|', b'^'];
    let mut separators_count: HashMap<u8, usize> = seg_symbols.iter().map(|&c| (c, 0)).collect();
    let mut max_count = 0;
    let mut separator = b',';

    // read next line after skipping
    if let Some(next_line) = lines_iter.next() {
      let line = next_line?;
      for c in line.as_bytes() {
        if let Some(count) = separators_count.get_mut(c) {
          *count += 1;
          if *count > max_count {
            max_count = *count;
            separator = *c;
          }
        }
      }
    }

    Ok(separator)
  }

  /// Count csv rows
  pub fn count_csv_rows(&self) -> Result<usize> {
    let reader = BufReader::new(File::open(&self.path)?);
    let total_rows = reader.lines().count().saturating_sub(1);

    Ok(total_rows)
  }

  /// Skip the first n lines of csv
  pub fn skip_csv_rows(&self) -> Result<BufReader<File>> {
    let mut reader = BufReader::new(File::open(&self.path)?);
    let mut line = String::new();

    if self.skip_rows == 0 {
      return Ok(reader);
    }

    for _ in 0..self.skip_rows {
      if reader.read_line(&mut line)? == 0 {
        // reached the end of the file before skipping all lines
        break;
      }
      line.clear();
    }

    Ok(reader)
  }

  /// The intersection of all headers between many csv or excel files
  pub fn inter_headers(&self) -> Result<HashSet<String>> {
    let path4split = &self.path.as_ref().to_string_lossy();
    let ff: Vec<&str> = path4split.split('|').collect();
    let mut header_sets: Vec<HashSet<String>> = vec![];
    let excel_extension = ["xls", "xlsx", "xlsm", "xlsb", "ods"];

    for f in ff {
      let mut csv_options = CsvOptions::new(f);
      csv_options.set_skip_rows(self.skip_rows);
      if excel_extension.iter().any(|&ext| f.ends_with(ext)) {
        let mut n: usize = 1;
        if self.skip_rows >= n {
          n = n + self.skip_rows
        }
        let n_rows = excel_reader::n_rows(f, n)?;
        if n_rows.is_empty() {
          return Err(anyhow!("worksheet is empty"));
        }
        let column_names = n_rows.get(self.skip_rows).expect("failed to get headers");
        let header_set: HashSet<String> = column_names
          .split('|')
          .map(|s| s.trim_matches('"').to_string())
          .collect();
        header_sets.push(header_set);
      } else {
        let mut rdr = ReaderBuilder::new()
          .delimiter(csv_options.detect_separator()?)
          .from_reader(csv_options.skip_csv_rows()?);
        if let Ok(headers) = rdr.headers() {
          let header_set: HashSet<String> = headers.iter().map(|s| s.to_string()).collect();
          header_sets.push(header_set);
        }
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

  /// Get csv headers {key: label, value: value}
  pub fn map_headers(&self) -> Result<Vec<HashMap<String, String>>> {
    let mut rdr = ReaderBuilder::new()
      .delimiter(self.detect_separator()?)
      .from_reader(self.skip_csv_rows()?);

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

  pub fn from_reader<R: Read>(&self, rdr: R) -> csv::Reader<R> {
    ReaderBuilder::new()
      .delimiter(self.detect_separator().expect("no detect separator"))
      .from_reader(rdr)
  }

  pub fn idx_path(&self) -> PathBuf {
    let mut p = self
      .path
      .as_ref()
      .to_path_buf()
      .into_os_string()
      .into_string()
      .unwrap();
    p.push_str(".idx");
    PathBuf::from(&p)
  }

  pub fn index_files(&self) -> Result<Option<(csv::Reader<File>, File)>> {
    let csv_file_result = File::open(&self.path);
    let idx_file_result = File::open(&self.idx_path());

    match (csv_file_result, idx_file_result) {
      (Ok(csv_file), Ok(idx_file)) => {
        let data_modified = csv_file.metadata()?.modified()?;
        let idx_modified = idx_file.metadata()?.modified()?;
        if data_modified > idx_modified {
          return Err(anyhow!(
            "The CSV file was modified after the index file. Please re-create the index."
          ));
        }
        let csv_rdr = self.from_reader(csv_file);
        Ok(Some((csv_rdr, idx_file)))
      }
      (Err(_), Err(_)) => Ok(None),
      (Ok(_), Err(_)) => Ok(None),
      _ => Ok(None),
    }
  }

  pub fn indexed(&self) -> Result<Option<Indexed<File, File>>> {
    match self.index_files()? {
      None => Ok(None),
      Some((r, i)) => Ok(Some(Indexed::open(r, i)?)),
    }
  }

  pub fn from_headers(&self) -> Result<Vec<String>> {
    let mut rdr = ReaderBuilder::new()
      .delimiter(self.detect_separator()?)
      .from_reader(self.skip_csv_rows()?);

    let headers: Vec<String> = rdr.headers()?.iter().map(|h| h.to_string()).collect();

    Ok(headers)
  }
}

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

#[inline]
pub fn num_cpus() -> usize {
  num_cpus::get()
}

pub fn num_of_chunks(nitems: usize, chunk_size: usize) -> usize {
  if chunk_size == 0 {
    return nitems;
  }
  let mut n = nitems / chunk_size;
  if nitems % chunk_size != 0 {
    n += 1;
  }
  n
}
