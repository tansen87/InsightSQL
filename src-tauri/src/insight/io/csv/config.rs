use std::{
  fs::File,
  io::{BufWriter, Read},
  path::PathBuf,
};

use anyhow::Result;
use csv::{QuoteStyle, ReaderBuilder, WriterBuilder};

use crate::utils::WTR_BUFFER_SIZE;

#[derive(Debug, Clone)]
pub struct CsvConfig {
  // Reader & Writer
  pub delimiter: u8,
  pub flexible: bool,
  pub has_headers: bool,

  // Reader
  pub quoting: bool,

  // Writer
  pub write_delim: u8,
  pub quote: u8,
  pub quote_style: QuoteStyle,
}

impl Default for CsvConfig {
  fn default() -> Self {
    Self {
      delimiter: b',',
      flexible: false,
      has_headers: true,
      quoting: true,
      write_delim: b',',
      quote: b'"',
      quote_style: QuoteStyle::Necessary,
    }
  }
}

impl CsvConfig {
  pub fn build_reader<R: Read>(&self, reader: R) -> csv::Reader<R> {
    ReaderBuilder::new()
      .delimiter(self.delimiter)
      .has_headers(self.has_headers)
      .flexible(self.flexible)
      .quoting(self.quoting)
      .from_reader(reader)
  }

  pub fn build_writer(&self, path: &PathBuf) -> Result<csv::Writer<BufWriter<File>>> {
    let buf_wtr = BufWriter::with_capacity(WTR_BUFFER_SIZE, File::create(path)?);

    let wtr = WriterBuilder::new()
      .delimiter(self.write_delim)
      .quote(self.quote)
      .quote_style(self.quote_style)
      .from_writer(buf_wtr);

    Ok(wtr)
  }
}

#[derive(Debug, Clone)]
pub struct CsvConfigBuilder {
  inner: CsvConfig,
}

impl Default for CsvConfigBuilder {
  fn default() -> Self {
    Self {
      inner: CsvConfig::default(),
    }
  }
}

impl CsvConfigBuilder {
  /// 创建一个新的 builder
  pub fn new() -> Self {
    Self::default()
  }

  /// 从现有 CsvConfig 开始构建
  pub fn from_config(config: CsvConfig) -> Self {
    Self { inner: config }
  }

  /// 设置 reader 和 writer 的分隔符为相同值
  pub fn delimiter(mut self, delim: u8) -> Self {
    self.inner.delimiter = delim;
    self.inner.write_delim = delim;
    self
  }

  /// 仅设置 reader 分隔符
  pub fn read_delimiter(mut self, delim: u8) -> Self {
    self.inner.delimiter = delim;
    self
  }

  /// 仅设置 writer 分隔符
  pub fn write_delimiter(mut self, delim: u8) -> Self {
    self.inner.write_delim = delim;
    self
  }

  /// 设置是否允许行字段数不一致
  pub fn flexible(mut self, flexible: bool) -> Self {
    self.inner.flexible = flexible;
    self
  }

  /// 设置第一行是否为 header
  pub fn has_headers(mut self, has_headers: bool) -> Self {
    self.inner.has_headers = has_headers;
    self
  }

  /// 设置 reader 是否解析引号
  pub fn quoting(mut self, quoting: bool) -> Self {
    self.inner.quoting = quoting;
    self
  }

  /// 设置 writer 使用的引号字符
  pub fn quote(mut self, quote: u8) -> Self {
    self.inner.quote = quote;
    self
  }

  /// 设置 writer 何时加引号
  pub fn quote_style(mut self, quote_style: QuoteStyle) -> Self {
    self.inner.quote_style = quote_style;
    self
  }

  /// 构建最终的 CsvConfig
  pub fn build(self) -> CsvConfig {
    self.inner
  }
}
