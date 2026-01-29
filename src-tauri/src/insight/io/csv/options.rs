use std::{
  collections::{HashMap, HashSet},
  fs::File,
  io::{self, BufRead, BufReader, Cursor, Read},
  path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};
use csv::ReaderBuilder;
use encoding_rs::{Encoding, GBK, UTF_8, UTF_16BE, UTF_16LE};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
  index::Indexed,
  io::excel::excel_reader::{self, FastExcelReader},
  utils::RDR_BUFFER_SIZE,
};

pub struct CsvOptions<P: AsRef<Path> + Send + Sync> {
  path: P,
  skiprows: usize,
  quoting: bool,
  flexible: bool,
}

impl<P: AsRef<Path> + Send + Sync> CsvOptions<P> {
  pub fn new(path: P) -> CsvOptions<P> {
    CsvOptions {
      path,
      skiprows: 0,
      quoting: true,
      flexible: false,
    }
  }

  /// Sets the number of rows to skip
  pub fn set_skiprows(&mut self, skiprows: usize) {
    self.skiprows = skiprows;
  }

  /// Get the numer of rows to skip
  pub fn get_skip_rows(&self) -> usize {
    self.skiprows
  }

  pub fn set_quoting(&mut self, quoting: bool) {
    self.quoting = quoting;
  }

  pub fn set_flexible(&mut self, flexible: bool) {
    self.flexible = flexible;
  }

  /// return parent path
  pub fn parent_path(&self) -> Result<&str> {
    self
      .path
      .as_ref()
      .parent()
      .ok_or(anyhow!("get parent path failed"))?
      .to_str()
      .ok_or(anyhow!("parent path is null"))
  }

  /// return file stem
  pub fn file_stem(&self) -> Result<&str> {
    self
      .path
      .as_ref()
      .file_stem()
      .ok_or(anyhow!("get file stem failed"))?
      .to_str()
      .ok_or(anyhow!("file stem is null"))
  }

  /// return file name
  pub fn file_name(&self) -> Result<&str> {
    self
      .path
      .as_ref()
      .file_name()
      .ok_or(anyhow!("get file name failed"))?
      .to_str()
      .ok_or(anyhow!("file name is null"))
  }

  /// return the output path based on the filename
  pub fn output_path(&self, cmd: Option<&str>, ext: Option<&str>) -> Result<PathBuf> {
    let mut output_path = PathBuf::from(self.parent_path()?);
    let file_stem = self.file_stem()?;
    let cmd = cmd.unwrap_or("cmd");
    let ext = ext.unwrap_or("csv");
    output_path.push(format!("{file_stem}_{cmd}.{ext}"));
    Ok(output_path)
  }

  /// Check the delimiter of CSV
  pub fn detect_separator(&self) -> Result<u8> {
    let file = File::open(&self.path)?;
    let reader = BufReader::new(file);

    let mut lines_iter = reader.lines();

    // Skip the first `skiprows` lines
    for _ in 0..self.skiprows {
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
  pub fn std_count_rows(&self) -> Result<usize> {
    let reader = BufReader::new(File::open(&self.path)?);
    let total_rows = reader.lines().count().saturating_sub(1);

    Ok(total_rows)
  }

  /// Count csv rows (applicable to all csv files)
  pub async fn idx_count_rows(&self) -> Result<usize> {
    let total_rows = crate::cmd::count::count_rows(&self.path, self.skiprows)
      .await
      .map_err(|e| anyhow!("Failed to count the number of lines: {e}"))?
      as usize;

    Ok(total_rows)
  }

  /// Count the lines of file
  pub fn count_lines(&self) -> Result<usize> {
    let reader = BufReader::new(File::open(&self.path)?);
    let line_count = reader.lines().count();

    Ok(line_count)
  }

  /// Skip the first n lines of csv
  pub fn rdr_skip_rows(&self) -> Result<BufReader<File>> {
    let file = File::open(&self.path)?;
    let mut reader = BufReader::with_capacity(RDR_BUFFER_SIZE, file);
    let mut line = String::new();

    if self.skiprows > 0 {
      for _ in 0..self.skiprows {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
          break;
        }
      }
    }

    Ok(reader)
  }

  /// 跳过CSV文件开头的指定行数,并自动检测分隔符,返回分隔符与可继续读取剩余内容的reader
  ///
  /// # 返回值
  /// - `Ok((delimiter, reader))`: 成功时返回delimiter和 reader
  /// - `Err(...)`: 文件不存在,I/O错误或跳行过程中文件提前结束
  pub fn skiprows_and_delimiter(&self) -> Result<(u8, BufReader<Box<dyn Read + Send>>)> {
    let file = File::open(&self.path)?;
    let mut reader = BufReader::with_capacity(RDR_BUFFER_SIZE, file);

    // 跳过前skiprows行
    for i in 0..self.skiprows {
      let mut line = String::new();
      let n = reader.read_line(&mut line)?;
      if n == 0 {
        return Err(anyhow!(
          "File ended at line {} while skipping {} rows",
          i,
          self.skiprows
        ));
      }
      log::debug!("Skipped line {}: {:?}", i, line.trim_end());
    }

    let mut header_line = String::new();
    let n = reader.read_line(&mut header_line)?;
    if n == 0 {
      // 文件在跳行后无数据: 返回默认分隔符+空reader
      let empty: Box<dyn Read + Send> = Box::new(io::empty());
      return Ok((b',', BufReader::new(empty)));
    }

    let candidates = [b';', b',', b'\t', b'|', b'^'];
    let mut counts: HashMap<u8, usize> = candidates.iter().map(|&c| (c, 0)).collect();
    let mut max_count = 0;
    let mut best_sep = b',';

    for &byte in header_line.as_bytes() {
      if let Some(cnt) = counts.get_mut(&byte) {
        *cnt += 1;
        if *cnt > max_count {
          max_count = *cnt;
          best_sep = byte;
        }
      }
    }

    // 把header_line放回reader前面
    let chained: Box<dyn Read + Send> =
      Box::new(Cursor::new(header_line.into_bytes()).chain(reader));
    let final_reader = BufReader::new(chained);

    Ok((best_sep, final_reader))
  }

  pub fn skiprows_reader(
    &self,
  ) -> Result<csv::Reader<std::io::BufReader<Box<dyn std::io::Read + Send>>>> {
    let (delimiter, reader) = self.skiprows_and_delimiter()?;
    let rdr = ReaderBuilder::new()
      .delimiter(delimiter)
      .quoting(self.quoting)
      .flexible(self.flexible)
      .from_reader(reader);
    Ok(rdr)
  }

  pub fn get_delimiter(&self) -> Result<u8> {
    let (delimiter, _) = self.skiprows_and_delimiter()?;
    Ok(delimiter)
  }

  /// Detect the text encoding of csv
  pub fn detect_encoding(&self, bom: bool) -> Result<&'static Encoding> {
    let mut sample = Vec::new();
    File::open(&self.path)?
      .take(256 * 1024)
      .read_to_end(&mut sample)?;

    if sample.is_empty() {
      return Ok(UTF_8);
    }

    if bom && sample.len() >= 2 {
      match (&sample[0..2], sample.len().checked_sub(2)) {
        ([0xFF, 0xFE], _) => return Ok(UTF_16LE), // UTF-16LE BOM
        ([0xFE, 0xFF], _) => return Ok(UTF_16BE), // UTF-16BE BOM
        ([0xEF, 0xBB], Some(rest)) if rest > 0 && sample[2] == 0xBF => {
          return Ok(UTF_8); // UTF-8 BOM
        }
        _ => {}
      }
    }

    if std::str::from_utf8(&sample).is_ok() {
      return Ok(UTF_8);
    }

    let is_odd_len = sample.len() % 2 == 1;
    if !is_odd_len && sample.len() >= 2 {
      let total_pairs = sample.len() / 2;
      let nulls_le = sample
        .iter()
        .skip(1)
        .step_by(2)
        .filter(|&&b| b == 0)
        .count();
      let nulls_be = sample.iter().step_by(2).filter(|&&b| b == 0).count();

      if nulls_le > total_pairs / 3 {
        return Ok(UTF_16LE);
      }
      if nulls_be > total_pairs / 3 {
        return Ok(UTF_16BE);
      }
    }

    Ok(GBK)
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
          if self.skiprows >= n {
            n = self.skiprows + 1;
          }
          let column_names = if file_extension == "xlsx" {
            // use `xl` to get the headers
            let n_rows = FastExcelReader::from_path(f).ok()?.n_rows(n).ok()?;
            // let n_rows = excel_reader::n_rows(f, n).unwrap_or_else(|_| vec![]);
            if n_rows.is_empty() {
              return None;
            }

            n_rows.get(self.skiprows)?.to_string()
          } else {
            // use `calamine` to get the headers
            let columns: Vec<String> = excel_reader::ExcelReader::from_path(f)
              .ok()?
              .get_column_names(0, self.skiprows as u32)
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
          let mut opts = CsvOptions::new(f);
          opts.set_skiprows(self.skiprows);
          let (sep, reader) = opts.skiprows_and_delimiter().ok()?;
          let mut rdr = ReaderBuilder::new()
            .delimiter(sep)
            // .has_headers(false)
            .from_reader(reader);

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
    let (sep, reader) = self.skiprows_and_delimiter()?;
    let mut rdr = ReaderBuilder::new()
      .delimiter(sep)
      // .has_headers(false)
      .from_reader(reader);

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

    let (sep, reader) = self.skiprows_and_delimiter()?;
    let mut rdr = ReaderBuilder::new()
      .delimiter(sep)
      .from_reader(reader);

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
    let mut rdr = self.skiprows_reader()?;
    let headers: Vec<String> = rdr.headers()?.iter().map(|h| h.to_string()).collect();

    Ok(headers)
  }
}
