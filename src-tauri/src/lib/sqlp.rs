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

use chrono::TimeZone;
use indexmap::IndexMap;
use polars::{
  datatypes::AnyValue,
  error::PolarsError,
  prelude::{
    CsvWriter, CsvWriterOptions, DataFrame, IntoLazy, LazyCsvReader, LazyFileListReader, LazyFrame,
    OptFlags, SerWriter,
  },
  sql::SQLContext,
};

use crate::excel::{ExcelReader, ToPolarsDataFrame};
use crate::xlsx_writer::write_xlsx;

#[derive(Default, Clone, PartialEq)]
enum OutputMode {
  #[default]
  Csv,
  None,
}

impl OutputMode {
  fn execute_query(
    &self,
    query: &str,
    ctx: &mut SQLContext,
    sep: u8,
    output: Option<String>,
    write: bool,
    write_format: &str,
    low_memory: bool,
    window: tauri::Window,
  ) -> Result<String, Box<PolarsError>> {
    let mut df = DataFrame::default();

    let execute_inner = || -> Result<(), PolarsError> {
      if low_memory {
        let lf = ctx.execute(query)?;
        lf.sink_csv(
          output.unwrap(),
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
        if write {
          // we don't want to write anything if the output mode is None
          if matches!(self, OutputMode::None) {
            return Ok(());
          }

          if (df.shape().0 < 104_0000) && (write_format == "xlsx") {
            let output: Option<PathBuf> = output.map(|s| PathBuf::from(s));
            let output_path = match output {
              Some(path) => path,
              None => PathBuf::new(),
            };
            write_xlsx(df.clone(), output_path).expect("Writing to xlsx failed.");
            Ok(())
          } else {
            let w = match output {
              Some(path) => Box::new(File::create(path)?) as Box<dyn Write>,
              None => Box::new(std::io::stdout()) as Box<dyn Write>,
            };
            let mut w = BufWriter::with_capacity(256_000, w);
            let out_result = match self {
              OutputMode::Csv => CsvWriter::new(&mut w)
                .with_separator(sep)
                .n_threads(4)
                .finish(&mut df),
              OutputMode::None => Ok(()),
            };
            w.flush()?;
            out_result
          }
        } else {
          Ok(())
        }
      }
    };

    match execute_inner() {
      Ok(()) => Ok(query_df_to_json(df.head(Some(100)))?),
      Err(e) => {
        eprintln!("Failed to execute query: {query}\n{e}");
        let errmsg = format!("{e}");
        window.emit("exec_err", &errmsg).unwrap();
        return Ok(errmsg);
      }
    }
  }
}

fn prepare_query(
  filepath: Vec<&str>,
  sqlsrc: &str,
  sep: String,
  write: bool,
  write_format: &str,
  low_memory: bool,
  window: tauri::Window,
) -> Result<(), Box<dyn Error>> {
  let mut ctx = SQLContext::new();
  let mut separator = Vec::new();
  let sep = if sep == "\\t" {
    b'\t'
  } else {
    sep.clone().into_bytes()[0]
  };
  separator.push(sep);

  let mut output: Vec<Option<String>> = Vec::new();
  let current_time = chrono::Local::now().format("%Y-%m-%d-%H%M%S");

  let output_suffix = match write_format {
    "xlsx" => format!("sqlp {}.xlsx", current_time),
    _ => format!("sqlp {}.csv", current_time),
  };

  for path in filepath.clone() {
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

  let mut table_aliases = HashMap::with_capacity(filepath.len());
  let mut lossy_table_name = Cow::default();
  let mut table_name;

  for (idx, table) in filepath.iter().enumerate() {
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
      None => return Err(("File extension not found").into()),
    };

    let lf = match file_extension.as_str() {
      "parquet" => LazyFrame::scan_parquet(table, Default::default())?,
      "xls" | "xlsx" | "xlsm" | "xlsb" | "ods" => {
        let mut excel_reader: ExcelReader = ExcelReader::new(table);
        let df: DataFrame = excel_reader.worksheet_range_at(0)?.to_df()?;
        df.lazy()
      }
      _ => {
        let csv_reader = LazyCsvReader::new(table)
          .with_has_header(true)
          .with_missing_is_null(true)
          .with_separator(separator[0])
          .with_infer_schema_length(Some(0))
          .with_low_memory(low_memory)
          .finish()?;

        csv_reader
      }
    };

    ctx.register(table_name, lf.with_optimizations(optimization_state));
  }

  let output_mode: OutputMode = OutputMode::Csv;
  let no_output: OutputMode = OutputMode::None;

  // check if the query is a SQL script
  let queries = if Path::new(&sqlsrc)
    .extension()
    .map_or(false, |ext| ext.eq_ignore_ascii_case("sql"))
  {
    let mut file = File::open(&sqlsrc)?;
    let mut sql_script = String::new();
    file.read_to_string(&mut sql_script)?;
    sql_script
      .split(';')
      .map(std::string::ToString::to_string)
      .filter(|s| !s.trim().is_empty())
      .collect()
  } else {
    // its not a sql script, just a single query
    vec![sqlsrc.to_string().clone()]
  };

  let num_queries = queries.len();
  let last_query: usize = num_queries.saturating_sub(1);
  let mut is_last_query;
  let mut current_query = String::new();

  for (idx, query) in queries.iter().enumerate() {
    // check if this is the last query in the script
    is_last_query = idx == last_query;

    // replace aliases in query
    current_query.clone_from(query);
    for (table_name, table_alias) in &table_aliases {
      // we quote the table name to avoid issues with reserved keywords and
      // other characters that are not allowed in identifiers
      current_query = current_query.replace(table_alias, &format!(r#""{table_name}""#));
    }

    if is_last_query {
      // if this is the last query, we use the output mode specified by the user
      let res = output_mode.execute_query(
        &current_query,
        &mut ctx,
        sep.clone(),
        output[0].clone(),
        write,
        write_format,
        low_memory,
        window.clone(),
      )?;
      window.emit("show", res)?;
    } else {
      // this is not the last query, we only execute the query, but don't write the output
      no_output.execute_query(
        &current_query,
        &mut ctx,
        sep.clone(),
        output[0].clone(),
        write,
        write_format,
        low_memory,
        window.clone(),
      )?;
    }
  }

  Ok(())
}

fn query_df_to_json(df: DataFrame) -> Result<String, polars::prelude::PolarsError> {
  let column_names = df.get_column_names();
  let mut height = Vec::new();
  if df.height() <= 100 {
    height.push(df.height());
  } else {
    height.push(5);
  }

  let buffer = (0..height[0])
    .into_iter()
    .map(|i| {
      let row = df
        .get_row(i)
        .expect(&*format!(
          "Could not access row {}, please try again.",
          i + 2
        ))
        .0;

      let object = column_names
        .iter()
        .zip(row.iter())
        .map(|(column_name, data)| {
          let formatted_value = match data {
            AnyValue::Float64(f) => format!("{:.2}", f),
            AnyValue::Float32(f) => format!("{:.2}", f),
            AnyValue::Int64(i) => i.to_string(),
            AnyValue::Int32(i) => i.to_string(),
            AnyValue::Int16(i) => i.to_string(),
            AnyValue::UInt64(u) => u.to_string(),
            AnyValue::UInt32(u) => u.to_string(),
            AnyValue::Boolean(b) => b.to_string(),
            _ => data.to_string(),
          };
          (column_name.to_string(), formatted_value)
        })
        .collect::<IndexMap<String, String>>();
      serde_json::to_string(&object).expect("Unable to serialize the result.")
    })
    .collect::<Vec<String>>();

  let result = if height[0] > 1 {
    format!("[{}]", buffer.join(","))
  } else {
    buffer
      .get(0)
      .expect("Unable to get value from buffer.")
      .clone()
  };

  Ok(result)
}

pub fn expired() -> bool {
  let current_date = chrono::Local::now().time();
  let expiration_date = chrono::Local
    .with_ymd_and_hms(2024, 8, 11, 23, 59, 0)
    .unwrap()
    .time();

  current_date > expiration_date
}

#[tauri::command]
pub async fn get(window: tauri::Window) {
  if !expired() {
    "hi there".to_string();
  } else {
    let expired_msg = "Your application has expired. Please renew your subscription.".to_string();
    window.emit("expired", expired_msg).unwrap();
  }
}

#[tauri::command]
pub async fn query(
  path: String,
  sqlsrc: String,
  sep: String,
  write: bool,
  write_format: String,
  low_memory: bool,
  window: tauri::Window,
) {
  let start_time = Instant::now();

  let filepath: Vec<&str> = path.split('|').collect();

  let prep_window = window.clone();
  match (async {
    prepare_query(
      filepath,
      sqlsrc.as_str(),
      sep,
      write,
      write_format.as_str(),
      low_memory,
      prep_window,
    )
  })
  .await
  {
    Ok(result) => result,
    Err(err) => {
      eprintln!("sql query error: {err}");
      // window.emit("query_err", &error.to_string()).unwrap();
      return ();
    }
  };

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  let runtime = format!("{elapsed_time:.2} s");
  window.emit("runtime", runtime).unwrap();
}
