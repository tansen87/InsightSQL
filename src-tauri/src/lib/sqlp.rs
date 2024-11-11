use std::{
  borrow::Cow,
  collections::HashMap,
  error::Error,
  fs::File,
  io::{BufWriter, Read, Write},
  num::NonZeroUsize,
  path::{Path, PathBuf},
  time::Instant,
};

// use chrono::TimeZone;
use indexmap::IndexMap;
use polars::{
  datatypes::AnyValue,
  error::PolarsError,
  prelude::{
    CsvWriter, CsvWriterOptions, DataFrame, IntoLazy, LazyCsvReader, LazyFileListReader, LazyFrame,
    OptFlags, ParquetWriter, SerWriter,
  },
  sql::SQLContext,
};
use tauri::Emitter;

use crate::detect::detect_separator;
use crate::excel::{ExcelReader, ToPolarsDataFrame};
use crate::xlsx_writer::write_xlsx;

fn execute_query(
  query: &str,
  ctx: &mut SQLContext,
  sep: u8,
  output: Option<String>,
  write: bool,
  write_format: &str,
  low_memory: bool,
) -> Result<String, Box<PolarsError>> {
  let mut df = DataFrame::default();

  let execute_inner = || -> Result<(), PolarsError> {
    if low_memory {
      let lf = ctx.execute(query)?;
      let output_path = format!("{}.csv", output.unwrap());
      lf.sink_csv(
        output_path,
        CsvWriterOptions {
          include_bom: false,
          include_header: true,
          batch_size: NonZeroUsize::new(1024).unwrap(),
          maintain_order: false,
          serialize_options: polars::prelude::SerializeOptions {
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
      )?;
      let re = regex::Regex::new(r"(?m)limit.*")?;
      let cleaned_sql = re.replace_all(query, "");
      let q = format!("{cleaned_sql} limit 100");
      df = ctx.execute(&q).and_then(LazyFrame::collect)?;
      Ok(())
    } else {
      df = ctx.execute(query).and_then(LazyFrame::collect)?;
      match (write, df.shape().0 < 104_0000, &write_format) {
        (false, _, _) => Ok(()),
        (true, true, &"xlsx") => {
          // rows less than 104w and write_format is xlsx
          let output_path = output.map_or_else(
            || PathBuf::from("default_output.xlsx"),
            |s| PathBuf::from(format!("{}.xlsx", s)),
          );
          write_xlsx(df.clone(), output_path).expect("Writing to xlsx failed");
          Ok(())
        }
        (true, true, &"parquet") => {
          // write to parquet
          let output_path = Some(format!("{}.parquet", output.unwrap()));
          let w = match output_path {
            Some(path) => Box::new(std::fs::File::create(path)?) as Box<dyn std::io::Write>,
            None => Box::new(std::io::stdout()) as Box<dyn std::io::Write>,
          };
          let mut w = BufWriter::with_capacity(256_000, w);
          let out_result = {
            ParquetWriter::new(&mut w)
              .with_row_group_size(Some(768 ^ 2))
              .finish(&mut df)
              .map(|_| ())
          };
          w.flush()?;
          out_result
        }
        (true, _, _) => {
          // others
          let output_path = Some(format!("{}.csv", output.unwrap()));
          let w = match output_path {
            Some(path) => Box::new(std::fs::File::create(path)?) as Box<dyn std::io::Write>,
            None => Box::new(std::io::stdout()) as Box<dyn std::io::Write>,
          };
          let mut w = BufWriter::with_capacity(256_000, w);
          let out_result = {
            CsvWriter::new(&mut w)
              .with_separator(sep)
              .n_threads(4)
              .finish(&mut df)
          };
          w.flush()?;
          out_result
        }
      }
    }
  };

  match execute_inner() {
    Ok(()) => Ok(query_df_to_json(df.head(Some(500)))?),
    Err(e) => {
      eprintln!("Failed to execute query: {query}\n{e}");
      let errmsg = format!("Error: {e}");
      return Ok(errmsg);
    }
  }
}

async fn prepare_query(
  file_path: Vec<&str>,
  sql_query: &str,
  write: bool,
  write_format: &str,
  low_memory: bool,
) -> Result<Vec<String>, Box<dyn Error>> {
  let mut ctx = SQLContext::new();

  let mut output: Vec<Option<String>> = Vec::new();
  let current_time = chrono::Local::now().format("%Y-%m-%d-%H%M%S");

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

  let optimization_state = if low_memory {
    let mut opts = OptFlags::default();
    opts.set(OptFlags::FILE_CACHING, false);
    opts.set(OptFlags::STREAMING, true);
    opts
  } else {
    OptFlags::default()
  };

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
      None => return Err(("").into()),
    };

    match file_extension.as_str() {
      "xls" | "xlsx" | "xlsm" | "xlsb" | "ods" | "parquet" => {
        vec_sep.push(b'|');
      }
      _ => {
        let sep = match detect_separator(table) {
          Some(separator) => {
            let separator_u8: u8 = separator as u8;
            separator_u8
          }
          None => b',',
        };
        vec_sep.push(sep);
      }
    }

    let lf = match file_extension.as_str() {
      "parquet" => LazyFrame::scan_parquet(table, Default::default())?,
      "xls" | "xlsx" | "xlsm" | "xlsb" | "ods" => {
        let mut excel_reader: ExcelReader = ExcelReader::new(table);
        let df: DataFrame = excel_reader.worksheet_range_at(0, 0)?.to_df()?;
        df.lazy()
      }
      _ => {
        let csv_reader = LazyCsvReader::new(table)
          .with_has_header(true)
          .with_missing_is_null(true)
          .with_separator(vec_sep[idx])
          .with_infer_schema_length(Some(0))
          .with_low_memory(low_memory)
          .with_truncate_ragged_lines(true)
          .finish()?;

        csv_reader
      }
    };

    ctx.register(table_name, lf.with_optimizations(optimization_state));
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

fn query_df_to_json(df: DataFrame) -> Result<String, PolarsError> {
  if df.is_empty() {
    let column_names = df.get_column_names();
    let empty_row = column_names
      .iter()
      .map(|column_name| (column_name.to_string(), serde_json::Value::Null))
      .collect::<IndexMap<_, _>>();
    return serde_json::to_string(&empty_row)
      .map_err(|e| PolarsError::ComputeError(e.to_string().into()));
  }

  let column_names = df.get_column_names();
  let max_height = if df.height() > 500 { 500 } else { df.height() };

  let rows = (0..max_height)
    .map(|i| {
      let row = df.get_row(i)?;
      let object = column_names
        .iter()
        .zip(row.0.iter())
        .map(|(column_name, data)| {
          let formatted_value = match data {
            AnyValue::Float64(f) => format!("{:.2}", f),
            AnyValue::Float32(f) => format!("{:.2}", f),
            AnyValue::Int64(i) => i.to_string().trim_matches('"').to_string(),
            AnyValue::Int32(i) => i.to_string().trim_matches('"').to_string(),
            AnyValue::Int16(i) => i.to_string().trim_matches('"').to_string(),
            AnyValue::UInt64(u) => u.to_string().trim_matches('"').to_string(),
            AnyValue::UInt32(u) => u.to_string().trim_matches('"').to_string(),
            AnyValue::Boolean(b) => b.to_string().trim_matches('"').to_string(),
            _ => data.to_string().trim_matches('"').to_string(),
          };
          (column_name.to_string(), formatted_value)
        })
        .collect::<IndexMap<_, _>>();
      Ok(object)
    })
    .collect::<Result<Vec<_>, PolarsError>>()?;

  let json_rows = if max_height > 1 || rows.len() > 1 {
    serde_json::to_string(&rows).map_err(|e| PolarsError::ComputeError(e.to_string().into()))
  } else if let Some(single_row) = rows.into_iter().next() {
    serde_json::to_string(&single_row).map_err(|e| PolarsError::ComputeError(e.to_string().into()))
  } else {
    unreachable!(
      "This branch should not be reached because the empty DataFrame case is handled earlier."
    )
  }?;

  Ok(json_rows)
}

#[tauri::command]
pub async fn query(
  path: String,
  sql_query: String,
  write: bool,
  write_format: String,
  low_memory: bool,
  window: tauri::Window,
) -> Result<Vec<String>, String> {
  let start_time = Instant::now();

  let file_path: Vec<&str> = path.split('|').collect();

  match prepare_query(file_path, &sql_query, write, &write_format, low_memory).await {
    Ok(result) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      let runtime = format!("{elapsed_time:.2} s");
      window.emit("runtime", runtime).unwrap();
      Ok(result)
    }
    Err(e) => Err(format!("{e}")),
  }
}
