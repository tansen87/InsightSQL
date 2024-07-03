use std::{
  borrow::Cow,
  collections::HashMap,
  error::Error,
  fs::File,
  io::{ BufWriter, Read, Write },
  path::{ Path, PathBuf },
  time::Instant,
};

use polars::{
  io::{ csv::read::{ CsvParseOptions, CsvReadOptions }, SerReader },
  prelude::{ CsvWriter, DataFrame, LazyCsvReader, LazyFileListReader, SerWriter },
  sql::SQLContext,
};
use chrono::TimeZone;
use indexmap::IndexMap;

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
    sep: String,
    output: Option<String>,
    show: bool,
    window: tauri::Window
  ) -> Result<(usize, usize), Box<dyn Error>> {
    let mut df = DataFrame::default();
    let mut separator = Vec::new();
    let sep_u8 = if sep == "\\t" { b'\t' } else { sep.clone().into_bytes()[0] };
    separator.push(sep_u8);
    let execute_inner = || {
      df = ctx.execute(query).and_then(polars::prelude::LazyFrame::collect)?;
      if show {
        let display_df = df.head(Some(100));
        let res = query_df_to_json(display_df)?;
        window.emit("show", res).unwrap();

        // we don't want to write anything if the output mode is None
        if matches!(self, OutputMode::None) {
          return Ok(());
        }

        let w = match output {
          Some(path) => { Box::new(File::create(path)?) as Box<dyn Write> }
          None => Box::new(std::io::stdout()) as Box<dyn Write>,
        };
        let mut w = BufWriter::with_capacity(256_000, w);
        let out_result = match self {
          OutputMode::Csv =>
            CsvWriter::new(&mut w).with_separator(separator[0]).n_threads(4).finish(&mut df),
          OutputMode::None => Ok(()),
        };

        w.flush()?;
        out_result
      } else {
        let display_df = df.head(Some(100));
        let res = query_df_to_json(display_df)?;
        window.emit("show", res).unwrap();
        Ok(())
      }
    };

    match execute_inner() {
      Ok(()) => Ok(df.shape()),
      Err(e) => {
        eprintln!("Failed to execute query: {query}: {e}");
        let errmsg = format!("Failed to execute query: {query}: {e}");
        window.emit("exec_err", errmsg)?;
        return Ok((0, 0));
      }
    }
  }
}

fn prepare_query(
  filepath: Vec<&str>,
  sqlsrc: &str,
  sep: String,
  show: bool,
  window: tauri::Window
) -> Result<(), Box<dyn Error>> {
  let mut ctx = SQLContext::new();
  let mut separator = Vec::new();
  let sep_u8 = if sep == "\\t" { b'\t' } else { sep.clone().into_bytes()[0] };
  separator.push(sep_u8);
  let mut output: Vec<Option<String>> = Vec::new();
  let current_time = chrono::Local::now().format("%Y-%m-%d-%H%M%S");
  let output_suffix = format!("sqlp {}.csv", current_time);
  for path in filepath.clone() {
    let mut output_path = PathBuf::from(path);

    // check file size
    let metadata = std::fs::metadata(path)?;
    let file_size = metadata.len();
    let kb = file_size / 1024;
    if kb > 7_340_032 {
      let size_msg = format!("{path} - {kb}, the data is too large.");
      window.emit("size_msg", size_msg)?;
      return Ok(());
    }

    output_path.set_extension(output_suffix.clone());
    let output_str = if let Some(output_path_str) = output_path.to_str() {
      Some(output_path_str.to_string())
    } else {
      None
    };

    output.push(output_str);
  }

  let optimization_state = polars::lazy::frame::OptState {
    file_caching: true,
    ..Default::default()
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

    // let tmp_df = match
    //   CsvReadOptions::default()
    //     .with_parse_options(CsvParseOptions::default().with_separator(separator[0]))
    //     .with_infer_schema_length(Some(0))
    //     .with_n_rows(Some(1))
    //     .with_n_threads(Some(1))
    //     .try_into_reader_with_file_path(Some(table.into()))?
    //     .finish()
    // {
    //   Ok(df) => df,
    //   Err(err) => {
    //     let err_msg = format!("error: {} | {}", table, err);
    //     eprintln!("{}", err_msg);
    //     return Ok(());
    //   }
    // };

    let lf = LazyCsvReader::new(table)
      .with_has_header(true)
      .with_missing_is_null(true)
      .with_separator(separator[0])
      .with_infer_schema_length(Some(0))
      .with_low_memory(false)
      .finish()?;

    ctx.register(table_name, lf.with_optimizations(optimization_state));
  }

  let output_mode: OutputMode = OutputMode::Csv;
  let no_output: OutputMode = OutputMode::None;

  // check if the query is a SQL script
  let queries = if
    Path::new(&sqlsrc)
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
      output_mode
        .execute_query(&current_query, &mut ctx, sep.clone(), output[0].clone(), show, window.clone())
        .unwrap();
    } else {
      // this is not the last query, we only execute the query, but don't write the output
      no_output
        .execute_query(&current_query, &mut ctx, sep.clone(), output[0].clone(), show, window.clone())
        .unwrap();
    }
  }

  Ok(())
}

fn csv_to_json(file: String, sep: String) -> Result<String, Box<dyn Error>> {
  let mut separator = Vec::new();
  let sep = if sep == "\\t" { b'\t' } else { sep.clone().into_bytes()[0] };
  separator.push(sep);

  let df = CsvReadOptions::default()
    .with_parse_options(
      CsvParseOptions::default().with_separator(separator[0]).with_missing_is_null(false)
    )
    .with_infer_schema_length(Some(0))
    .with_n_threads(Some(4))
    .with_n_rows(Some(20))
    .try_into_reader_with_file_path(Some(file.into()))?
    .finish()?;

  let column_names = df.get_column_names();
  let mut height = Vec::new();
  if df.height() <= 20 {
    height.push(df.height());
  } else {
    height.push(5);
  }

  let buffer = (0..height[0])
    .into_iter()
    .map(|i| {
      let row = df
        .get_row(i)
        .expect(&*format!("Could not access row {}, please try again.", i + 2)).0;

      let object = column_names
        .iter()
        .zip(row.iter())
        .map(|(column, data)| (column.to_string(), data.get_str().unwrap_or("").to_owned()))
        .collect::<IndexMap<String, String>>();
      serde_json::to_string(&object).expect("Unable to serialize the result.")
    })
    .collect::<Vec<String>>();
  let result = if height[0] > 1 {
    format!("[{}]", buffer.join(","))
  } else {
    buffer.get(0).expect("Unable to get value from buffer.").clone()
  };

  Ok(result)
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
        .expect(&*format!("Could not access row {}, please try again.", i + 2)).0;

      let object = column_names
        .iter()
        .zip(row.iter())
        .map(|(column, data)| (column.to_string(), data.get_str().unwrap_or("").to_owned()))
        .collect::<IndexMap<String, String>>();
      serde_json::to_string(&object).expect("Unable to serialize the result.")
    })
    .collect::<Vec<String>>();
  let result = if height[0] > 1 {
    format!("[{}]", buffer.join(","))
  } else {
    buffer.get(0).expect("Unable to get value from buffer.").clone()
  };

  Ok(result)
}

pub fn expired() -> bool {
  let current_date = chrono::Local::now().time();
  let expiration_date = chrono::Local.with_ymd_and_hms(2024, 7, 11, 23, 59, 0).unwrap().time();

  current_date > expiration_date
}

#[tauri::command]
pub async fn get(path: String, sep: String, window: tauri::Window) -> String {
  let mut vec_results = Vec::new();
  let vec_path: Vec<&str> = path.split(',').collect();
  let file = vec_path[0].to_string();
  if !expired() {
    let results = match (async { csv_to_json(file, sep) }).await {
      Ok(result) => result,
      Err(err) => {
        eprintln!("get headers error: {err}");
        window.emit("get_err", &err.to_string()).unwrap();
        err.to_string()
      }
    };
    vec_results.push(results);
  } else {
    let expired_msg = "Your application has expired. Please renew your subscription.".to_string();
    window.emit("expired", expired_msg).unwrap();
  }

  vec_results[0].clone()
}

#[tauri::command]
pub async fn query(path: String, sqlsrc: String, sep: String, show: bool, window: tauri::Window) {
  let start = Instant::now();
  let filepath: Vec<&str> = path.split(',').collect();
  let prep_window = window.clone();
  match (async { prepare_query(filepath, &sqlsrc.as_str(), sep, show, prep_window) }).await {
    Ok(result) => result,
    Err(error) => {
      eprintln!("sql query error: {error}");
      window.emit("query_err", &error.to_string()).unwrap();
      return ();
    }
  }
  let end = Instant::now();
  let elapsed = end.duration_since(start);
  let elapsed_seconds = elapsed.as_secs_f64();
  let run_time = format!("{elapsed_seconds:.2} s");
  window.emit("run_time", run_time).unwrap();
}
