use std::{
  borrow::Cow,
  collections::HashMap,
  fs::File,
  io::{BufWriter, Write},
  path::{Path, PathBuf},
  sync::Arc,
  time::Instant,
};

use anyhow::{Result, anyhow};
use polars::{
  prelude::{
    CsvWriter, DataFrame, IntoLazy, JsonFormat, JsonWriter, LazyCsvReader, LazyFileListReader,
    LazyFrame, OptFlags, ParquetWriter, PlPath, SerWriter,
  },
  sql::SQLContext,
};

use crate::io::csv::options::CsvOptions;
use crate::io::excel::excel_reader::{
  ExcelReader, FastExcelReader, FastToDataFrame, ToPolarsDataFrame,
};
use crate::io::excel::xlsx_writer::XlsxWriter;

const EXCEL_MAX_ROW: usize = 104_8575; // no headers
const BUFFER_SIZE: usize = 256_000;

trait FileWriter {
  fn write_xlsx(&self, df: &DataFrame, output_path: impl AsRef<Path>) -> Result<()>;
  fn write_csv(&self, df: DataFrame, output_path: impl AsRef<Path>) -> Result<()>;
  fn write_parquet(&self, df: DataFrame, output_path: impl AsRef<Path>) -> Result<()>;
  fn write_json(&self, df: DataFrame, output_path: impl AsRef<Path>) -> Result<()>;
  fn write_jsonl(&self, df: DataFrame, output_path: impl AsRef<Path>) -> Result<()>;
}

#[derive(Clone)]
enum OutputFormat {
  Xlsx,
  Csv,
  Parquet,
  Json,
  Jsonl,
}

impl From<&str> for OutputFormat {
  fn from(format: &str) -> Self {
    match format {
      "xlsx" => OutputFormat::Xlsx,
      "csv" => OutputFormat::Csv,
      "parquet" => OutputFormat::Parquet,
      "json" => OutputFormat::Json,
      "jsonl" => OutputFormat::Jsonl,
      _ => OutputFormat::Csv,
    }
  }
}

struct DataFrameWriter {
  format: OutputFormat,
  separator: u8,
}

impl DataFrameWriter {
  pub fn new(format: OutputFormat) -> Self {
    Self { format, separator: b',' }
  }

  fn with_separator(mut self, sep: u8) -> Self {
    self.separator = sep;
    self
  }

  pub fn write_dataframe(&self, df: DataFrame, output_path: impl AsRef<Path>) -> Result<()> {
    match (&self.format, df.height() <= EXCEL_MAX_ROW) {
      (OutputFormat::Xlsx, true) => self.write_xlsx(&df, output_path),
      (OutputFormat::Csv, _) => self.write_csv(df, output_path),
      (OutputFormat::Parquet, _) => self.write_parquet(df, output_path),
      (OutputFormat::Json, _) => self.write_json(df, output_path),
      (OutputFormat::Jsonl, _) => self.write_jsonl(df, output_path),
      _ => self.write_csv(df, output_path),
    }
  }
}

impl FileWriter for DataFrameWriter {
  fn write_xlsx(&self, df: &DataFrame, output_path: impl AsRef<Path>) -> Result<()> {
    XlsxWriter::new().write_dataframe(&df, output_path.as_ref().to_path_buf())?;
    Ok(())
  }

  fn write_csv(&self, mut df: DataFrame, output_path: impl AsRef<Path>) -> Result<()> {
    let file = File::create(output_path)?;
    let mut wtr = BufWriter::with_capacity(BUFFER_SIZE, file);
    CsvWriter::new(&mut wtr)
      .with_separator(self.separator)
      .with_float_precision(Some(2))
      .finish(&mut df)?;
    Ok(wtr.flush()?)
  }

  fn write_parquet(&self, mut df: DataFrame, output_path: impl AsRef<Path>) -> Result<()> {
    let file = File::create(output_path)?;
    let mut wtr = BufWriter::with_capacity(BUFFER_SIZE, file);
    ParquetWriter::new(&mut wtr)
      .with_row_group_size(Some(768 * 768))
      .finish(&mut df)?;
    Ok(wtr.flush()?)
  }

  fn write_json(&self, mut df: DataFrame, output_path: impl AsRef<Path>) -> Result<()> {
    let file = File::create(output_path)?;
    let mut wtr = BufWriter::with_capacity(BUFFER_SIZE, file);
    JsonWriter::new(&mut wtr)
      .with_json_format(JsonFormat::Json)
      .finish(&mut df)?;
    Ok(wtr.flush()?)
  }

  fn write_jsonl(&self, mut df: DataFrame, output_path: impl AsRef<Path>) -> Result<()> {
    let file = File::create(output_path)?;
    let mut wtr = BufWriter::with_capacity(BUFFER_SIZE, file);
    JsonWriter::new(&mut wtr)
    .with_json_format(JsonFormat::JsonLines)
        .finish(&mut df)?;
    Ok(wtr.flush()?)
  }
}

async fn prepare_query(
  file_path: Vec<&str>,
  sql_query: &str,
  write: bool,
  write_format: &str,
  skip_rows: String,
  schema_length: &str,
) -> Result<Vec<String>> {
  let infer_schema_length = match schema_length {
    schema_legth => Some(schema_legth.parse::<usize>()?),
  };

  let mut ctx = SQLContext::new();
  let opts = CsvOptions::new(file_path.get(0).ok_or(anyhow!("No file choice"))?);
  let mut output_path = PathBuf::from(opts.parent_path()?);
  let file_stem = opts.file_stem()?;
  match write_format {
    "xlsx" => output_path.push(format!("{file_stem}.sql.xlsx")),
    "parquet" => output_path.push(format!("{file_stem}.sql.parquet")),
    "json" | "jsonl" => output_path.push(format!("{file_stem}.sql.json")),
    _ => output_path.push(format!("{file_stem}.sql.csv")),
  };

  let mut opt_state = OptFlags::from_bits_truncate(0);
  opt_state |= OptFlags::default();

  let mut table_aliases = HashMap::with_capacity(file_path.len());
  let mut lossy_table_name = Cow::default();
  let mut table_name;
  let mut vec_sep = Vec::new();

  for (idx, table) in file_path.iter().enumerate() {
    // as we are using the table name as alias, we need to make sure that the table name is a
    // valid identifier. if its not utf8, we use the lossy version
    table_name = Path::new(table)
      .file_stem()
      .and_then(std::ffi::OsStr::to_str)
      .unwrap_or_else(|| {
        lossy_table_name = Path::new(table).to_string_lossy();
        &lossy_table_name
      });
    table_aliases.insert(table_name.to_string(), format!("_t_{}", idx + 1));

    let file_extension = match Path::new(table).extension() {
      Some(ext) => ext.to_string_lossy().to_lowercase(),
      None => return Err(anyhow!("File extension not found")),
    };

    match file_extension.as_str() {
      "xls" | "xlsx" | "xlsm" | "xlsb" | "ods" | "parquet" => {
        vec_sep.push(b'|');
      }
      _ => {
        let mut opts = CsvOptions::new(table);
        opts.set_skip_rows(skip_rows.parse::<usize>()?);
        vec_sep.push(opts.detect_separator()?);
      }
    }

    let lf = match file_extension.as_str() {
      "parquet" => {
        let p: Arc<Path> = Arc::from(PathBuf::from(table));
        LazyFrame::scan_parquet(PlPath::Local(p), Default::default())?
      }
      "xls" | "xlsm" | "xlsb" | "ods" => {
        let df: DataFrame = ExcelReader::from_path(table)?
          .worksheet_range_at(0, skip_rows.parse::<u32>()?)?
          .to_df()?;
        df.lazy()
      }
      "xlsx" => {
        let re = regex::Regex::new(r"limit\s+(\d+)")?;
        let n = re
          .captures(&sql_query.to_lowercase())
          .and_then(|cap| cap.get(1))
          .and_then(|num_match| num_match.as_str().parse::<usize>().ok());

        let df: DataFrame = match n {
          Some(n) => {
            FastExcelReader::from_path(table)?.fast_to_df(n, skip_rows.parse::<usize>()?)?
          }
          None => ExcelReader::from_path(table)?
            .worksheet_range_at(0, skip_rows.parse::<u32>()?)?
            .to_df()?,
        };
        df.lazy()
      }
      _ => {
        let p: Arc<Path> = Arc::from(PathBuf::from(table));
        let csv_reader = LazyCsvReader::new(PlPath::Local(p))
          .with_has_header(true)
          .with_missing_is_null(true)
          .with_separator(vec_sep[idx])
          .with_infer_schema_length(infer_schema_length)
          .with_skip_rows(skip_rows.parse::<usize>()?)
          .finish()?;

        csv_reader
      }
    };

    ctx.register(table_name, lf.with_optimizations(opt_state));
  }

  let mut vec_result = Vec::new();

  let mut current_query = String::new();

  // replace aliases in query
  current_query.clone_from(&sql_query.to_string());
  for (table_name, table_alias) in &table_aliases {
    // we quote the table name to avoid issues with reserved keywords and
    // other characters that are not allowed in identifiers
    current_query = current_query.replace(table_alias, &format!(r#""{table_name}""#));
  }

  let mut ctx = ctx.clone();

  let df = tokio::task::spawn_blocking(move || -> Result<_> {
    Ok(ctx.execute(&current_query)?.collect()?)
  })
  .await??;

  vec_result.push(query_df_to_json(df.head(Some(500)))?);

  if write {
    DataFrameWriter::new(write_format.into())
      .with_separator(vec_sep[0])
      .write_dataframe(df, output_path)?;
  }

  Ok(vec_result)
}

fn query_df_to_json(mut df: DataFrame) -> Result<String> {
  if df.is_empty() {
    let empty_json = serde_json::json!({});
    return Ok(empty_json.to_string());
  }

  let mut buffer = Vec::new();
  JsonWriter::new(&mut buffer)
    .with_json_format(JsonFormat::Json)
    .finish(&mut df)?;

  Ok(String::from_utf8(buffer).unwrap())
}

#[tauri::command]
pub async fn query(
  path: String,
  sql_query: String,
  write: bool,
  write_format: String,
  skip_rows: String,
  schema_length: String,
) -> Result<(Vec<String>, String), String> {
  let start_time = Instant::now();

  let file_path: Vec<&str> = path.split('|').collect();

  match prepare_query(
    file_path,
    &sql_query,
    write,
    &write_format,
    skip_rows,
    schema_length.as_str(),
  )
  .await
  {
    Ok(result) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      let runtime = format!("{elapsed_time:.2}");
      Ok((result, runtime))
    }
    Err(err) => Err(format!("Query failed: {err}")),
  }
}
