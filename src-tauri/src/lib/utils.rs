use std::{
  collections::{HashMap, HashSet},
  fs::File,
  io::{BufRead, BufReader, Read},
  iter,
  path::{Path, PathBuf},
  slice,
};

use anyhow::{Result, anyhow};
use csv::{ByteRecord, ReaderBuilder};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
  excel_reader::{self, FastExcelReader},
  index::Indexed,
};

type ByteString = Vec<u8>;

pub struct CsvOptions<P: AsRef<Path> + Send + Sync> {
  path: P,
  skip_rows: usize,
}

impl<P: AsRef<Path> + Send + Sync> CsvOptions<P> {
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

  /// Count csv rows (only applicable to standard csv files)
  pub fn count_csv_rows(&self) -> Result<usize> {
    let reader = BufReader::new(File::open(&self.path)?);
    let total_rows = reader.lines().count().saturating_sub(1);

    Ok(total_rows)
  }

  /// Count csv rows (applicable to all csv files)
  pub async fn parse_csv_rows(&self) -> Result<usize> {
    let cnt = crate::command::count::count_rows(&self.path).await? as usize;

    Ok(cnt)
  }

  /// Skip the first n lines of csv
  pub fn skip_csv_rows(&self) -> Result<BufReader<File>> {
    let mut reader = BufReader::new(File::open(&self.path)?);
    let mut line = String::new();

    if self.skip_rows > 0 {
      for _ in 0..self.skip_rows {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
          break;
        }
      }
    }

    Ok(reader)
  }

  /// The intersection of all headers between many csv or excel files
  pub fn inter_headers(&self) -> Result<HashSet<String>> {
    let excel_extensions: HashSet<&str> = ["xls", "xlsx", "xlsm", "xlsb", "ods"]
      .iter()
      .cloned()
      .collect();
    let path4split = self.path.as_ref().to_string_lossy().to_lowercase();
    let file_paths: Vec<&str> = path4split.split('|').collect();

    let header_sets: Vec<HashSet<String>> = file_paths
      .par_iter()
      .filter_map(|f| {
        if f.is_empty() {
          return None;
        }
        let file_extension = if let Some(ext) = f.split('.').last() {
          if ext.is_empty() {
            return None;
          }
          ext.to_lowercase()
        } else {
          return None;
        };

        if excel_extensions.contains(file_extension.as_str()) {
          let mut n: usize = 1;
          if self.skip_rows >= n {
            n = self.skip_rows + 1;
          }
          let column_names = if file_extension == "xlsx" {
            // use `xl` to get the headers
            let n_rows = FastExcelReader::from_path(f).ok()?.n_rows(n).ok()?;
            // let n_rows = excel_reader::n_rows(f, n).unwrap_or_else(|_| vec![]);
            if n_rows.is_empty() {
              return None;
            }

            n_rows.get(self.skip_rows)?.to_string()
          } else {
            // use `calamine` to get the headers
            let columns: Vec<String> = excel_reader::ExcelReader::from_path(f)
              .ok()?
              .get_column_names(0, self.skip_rows as u32)
              .ok()?;

            columns.join("|")
          };
          Some(
            column_names
              .split('|')
              .map(|s| s.trim_matches('"').to_string())
              .collect::<HashSet<String>>(),
          )
        } else {
          // use `csv` to get the headers
          let mut csv_options = CsvOptions::new(f);
          csv_options.set_skip_rows(self.skip_rows);
          let mut rdr = ReaderBuilder::new()
            .delimiter(csv_options.detect_separator().ok()?)
            .has_headers(false)
            .from_reader(csv_options.skip_csv_rows().ok()?);

          rdr
            .headers()
            .ok()
            .map(|headers| headers.iter().map(|s| s.to_string()).collect())
        }
      })
      .collect::<Vec<_>>();

    // start with the assumption that all headers are common
    let mut common_headers: HashSet<String> = match header_sets.first() {
      Some(first_set) => first_set.clone(),
      None => return Ok(HashSet::new()),
    };
    for headers in header_sets.iter().skip(1) {
      common_headers = common_headers.intersection(headers).cloned().collect();
    }

    Ok(common_headers)
  }

  /// Get csv headers {key: label, value: value}
  pub fn map_headers(&self) -> Result<Vec<HashMap<String, String>>> {
    let mut rdr = ReaderBuilder::new()
      .delimiter(self.detect_separator()?)
      .has_headers(false)
      .from_reader(self.skip_csv_rows()?);

    let headers: Vec<HashMap<String, String>> = rdr
      .headers()?
      .iter()
      .map(|header| {
        let mut map = HashMap::new();
        map.insert("label".to_string(), header.to_string());
        map.insert("value".to_string(), header.to_string());
        map
      })
      .collect();

    Ok(headers)
  }

  /// find the same headers and different headers in csv
  pub fn dupli_headers(&self) -> Result<(HashSet<String>, HashSet<String>)> {
    let mut headers_counter: HashMap<String, usize> = HashMap::new();
    let mut duplicate_headers: HashSet<String> = HashSet::new();
    let mut unique_headers: HashSet<String> = HashSet::new();

    let mut rdr = ReaderBuilder::new()
      .delimiter(self.detect_separator()?)
      .from_reader(self.skip_csv_rows()?);

    match rdr.headers() {
      Ok(headers) => {
        for header in headers.iter() {
          let count = headers_counter.entry(header.to_string()).or_insert(0);
          *count += 1;
        }
      }
      Err(e) => return Err(anyhow!("{e}")),
    }

    // classify headers into duplicate_headers and unique_ceaders
    for (header, count) in headers_counter {
      if count > 1 {
        duplicate_headers.insert(header);
      } else {
        unique_headers.insert(header);
      }
    }

    Ok((duplicate_headers, unique_headers))
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

type _GetField = for<'c> fn(&mut &'c ByteRecord, &usize) -> Option<&'c [u8]>;

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

  #[inline]
  /// Returns an iterator that yields selected fields from a CSV record.
  ///
  /// This method takes a CSV record and returns an iterator that yields only the fields
  /// specified by this Selection. The fields are returned in the order they were selected.
  ///
  /// # Arguments
  ///
  /// * `row` - The CSV record to select fields from
  ///
  /// # Returns
  ///
  /// An iterator that yields references to the selected fields as byte slices
  pub fn select<'a, 'b>(
    &'a self,
    row: &'b ByteRecord,
  ) -> iter::Scan<slice::Iter<'a, usize>, &'b ByteRecord, _GetField> {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn get_field<'c>(row: &mut &'c ByteRecord, idx: &usize) -> Option<&'c [u8]> {
      Some(&row[*idx])
    }

    let get_field: _GetField = get_field;
    self.get_indices().into_iter().scan(row, get_field)
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
