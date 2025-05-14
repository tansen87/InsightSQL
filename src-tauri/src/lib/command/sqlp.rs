use std::{
  borrow::Cow,
  collections::HashMap,
  fs::File,
  io::{BufWriter, Read, Write},
  num::NonZeroUsize,
  path::{Path, PathBuf},
  time::Instant,
};

use anyhow::{Result, anyhow};
use polars::{
  prelude::{
    CsvWriter, CsvWriterOptions, DataFrame, IntoLazy, JsonFormat, JsonWriter, LazyCsvReader,
    LazyFileListReader, LazyFrame, OptFlags, ParquetWriter, SerWriter, SerializeOptions,
  },
  sql::SQLContext,
};

use crate::excel_reader::{ExcelReader, FastExcelReader, FastToDataFrame, ToPolarsDataFrame};
use crate::{utils::CsvOptions, xlsx_writer::XlsxWriter};

fn execute_query(
  query: &str,
  ctx: &mut SQLContext,
  sep: u8,
  output: Option<String>,
  write: bool,
  write_format: &str,
  low_memory: bool,
) -> Result<String> {
  let mut df = DataFrame::default();

  if low_memory {
    // Low memory path: direct streaming write to CSV
    let lf = ctx.execute(query)?;
    let output_path = format!("{}.csv", output.unwrap());
    write_lazyframe(lf, &output_path, sep)?;
  } else {
    // Normal execution path
    df = ctx.execute(query)?.collect()?;
    if write {
      // Handle different write formats
      write_dataframe(&mut df, output.clone(), write_format, sep)?;
    }
  };

  let result = query_df_to_json(df.head(Some(500)))?;
  Ok(result)
}

/// write LazyFrame directly to CSV (low memory mode)
fn write_lazyframe(lf: LazyFrame, output_path: &str, sep: u8) -> Result<()> {
  Ok(lf.sink_csv(
    output_path,
    CsvWriterOptions {
      include_bom: false,
      include_header: true,
      batch_size: NonZeroUsize::new(1024).unwrap(),
      maintain_order: false,
      serialize_options: SerializeOptions {
        date_format: None,
        time_format: None,
        datetime_format: None,
        float_scientific: None,
        float_precision: None,
        separator: sep,
        quote_char: b'"',
        null: String::new(),
        line_terminator: "\n".into(),
        quote_style: Default::default(),
      },
    },
    Default::default(),
  )?)
}

/// Unified dataframe writing handler
fn write_dataframe(df: &DataFrame, output: Option<String>, format: &str, sep: u8) -> Result<()> {
  match (df.shape().0 < 104_0000, format) {
    (true, "xlsx") => write_xlsx(&df, output),
    (_, "parquet") => write_parquet(df.clone(), output),
    _ => write_csv(df.clone(), output, sep),
  }
}

/// XLSX writing implementation
fn write_xlsx(df: &DataFrame, output: Option<String>) -> Result<()> {
  let output_path = output.map_or_else(
    || PathBuf::from("default_output.xlsx"),
    |s| PathBuf::from(format!("{}.xlsx", s)),
  );
  XlsxWriter::new().write_dataframe(&df, output_path)?;
  Ok(())
}

/// Parquet writing implementation
fn write_parquet(mut df: DataFrame, output: Option<String>) -> Result<()> {
  let output_path = output.map_or_else(
    || PathBuf::from("default_output.parquet"),
    |s| PathBuf::from(format!("{}.parquet", s)),
  );
  let file = File::create(output_path)?;
  let writer = BufWriter::new(file);
  ParquetWriter::new(writer)
    .with_row_group_size(Some(768 ^ 2))
    .finish(&mut df)?;
  Ok(())
}

/// CSV writing implementation
fn write_csv(mut df: DataFrame, output: Option<String>, sep: u8) -> Result<()> {
  let w: Box<dyn Write> = match output {
    Some(path) => Box::new(File::create(format!("{}.csv", path))?),
    None => Box::new(std::io::stdout()),
  };
  let mut w = BufWriter::with_capacity(256_000, w);
  CsvWriter::new(&mut w)
    .with_separator(sep)
    .with_float_precision(Some(2))
    .finish(&mut df)?;
  w.flush()?;
  Ok(())
}

async fn prepare_query(
  file_path: Vec<&str>,
  sql_query: &str,
  write: bool,
  write_format: &str,
  low_memory: bool,
  skip_rows: String,
  schema_length: &str,
) -> Result<Vec<String>> {
  let infer_schema_length = match schema_length {
    schema_legth => Some(schema_legth.parse::<usize>()?),
  };

  let mut ctx = SQLContext::new();

  let mut output: Vec<Option<String>> = Vec::new();
  let current_time = chrono::Local::now().format("%H%M%S");

  let output_suffix = format!("sqlp_{}", current_time);

  for path in file_path.clone() {
    let mut output_path = PathBuf::from(path);
    output_path.set_extension(&output_suffix);
    let output_str = if let Some(output_path_str) = output_path.to_str() {
      Some(output_path_str.to_string())
    } else {
      None
    };
    output.push(output_str);
  }

  let mut opt_state = OptFlags::from_bits_truncate(0);
  if low_memory {
    opt_state |= OptFlags::PROJECTION_PUSHDOWN
      | OptFlags::PREDICATE_PUSHDOWN
      | OptFlags::CLUSTER_WITH_COLUMNS
      | OptFlags::SIMPLIFY_EXPR
      | OptFlags::FILE_CACHING
      | OptFlags::SLICE_PUSHDOWN
      | OptFlags::COMM_SUBPLAN_ELIM
      | OptFlags::COMM_SUBEXPR_ELIM
      | OptFlags::ROW_ESTIMATE
      | OptFlags::FAST_PROJECTION;
  } else {
    opt_state |= OptFlags::default();
  };
  opt_state.set(OptFlags::STREAMING, low_memory);

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
        let mut csv_options = CsvOptions::new(table);
        csv_options.set_skip_rows(skip_rows.parse::<usize>()?);
        vec_sep.push(csv_options.detect_separator()?);
      }
    }

    let lf = match file_extension.as_str() {
      "parquet" => LazyFrame::scan_parquet(table, Default::default())?,
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
        let csv_reader = LazyCsvReader::new(table)
          .with_has_header(true)
          .with_missing_is_null(true)
          .with_separator(vec_sep[idx])
          .with_infer_schema_length(infer_schema_length)
          .with_low_memory(low_memory)
          .with_skip_rows(skip_rows.parse::<usize>()?)
          .finish()?;

        csv_reader
      }
    };

    ctx.register(table_name, lf.with_optimizations(opt_state));
  }

  let mut vec_result = Vec::new();

  // check if the query is a SQL script
  let queries = if Path::new(&sql_query)
    .extension()
    .map_or(false, |ext| ext.eq_ignore_ascii_case("sql"))
  {
    let mut file = File::open(&sql_query)?;
    let mut sql_script = String::new();
    file.read_to_string(&mut sql_script)?;
    sql_script
      .split(';')
      .map(std::string::ToString::to_string)
      .filter(|s| !s.trim().is_empty())
      .collect()
  } else {
    // its not a sql script, just a single query
    vec![sql_query.to_string().clone()]
  };

  let mut current_query = String::new();

  for (idx, query) in queries.iter().enumerate() {
    // replace aliases in query
    current_query.clone_from(query);
    for (table_name, table_alias) in &table_aliases {
      // we quote the table name to avoid issues with reserved keywords and
      // other characters that are not allowed in identifiers
      current_query = current_query.replace(table_alias, &format!(r#""{table_name}""#));
    }

    let output_path = Some(format!("{}_{idx}", output[0].clone().unwrap()));
    let res = execute_query(
      &current_query,
      &mut ctx,
      vec_sep[0],
      output_path,
      write,
      write_format,
      low_memory,
    )?;
    vec_result.push(res);
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
  low_memory: bool,
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
    low_memory,
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
