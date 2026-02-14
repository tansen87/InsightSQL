use std::{
  fs::File,
  io::BufWriter,
  path::{Path, PathBuf},
  sync::Arc,
  time::Instant,
};

use anyhow::{Result, anyhow};
use csv::{ByteRecord, ReaderBuilder, WriterBuilder};
use indexmap::IndexSet;
use polars::{
  frame::DataFrame,
  lazy::dsl::{functions::concat_lf_diagonal, lit},
  prelude::{
    CsvWriter, IntoLazy, LazyCsvReader, LazyFileListReader, PlRefPath, SerWriter, UnionArgs, col,
  },
};

use crate::{
  io::{
    csv::options::CsvOptions,
    excel::{
      excel_reader::{ExcelReader, ToPolarsDataFrame},
      xlsx_writer::XlsxWriter,
    },
  },
  utils::{EXCEL_MAX_ROW, WTR_BUFFER_SIZE},
};

async fn cat_with_polars(
  path: String,
  output_path: String,
  file_type: String,
  use_cols: String,
  skiprows: usize,
) -> Result<()> {
  /* merge csv and excel files into a xlsx or csv file */
  let paths: Vec<&str> = path.split('|').collect();
  let use_cols: Vec<&str> = use_cols.split('|').collect();
  let use_cols = match use_cols.len() {
    0 | 1 if use_cols.get(0).map_or(true, |&s| s.is_empty()) => vec!["all"],
    _ => use_cols,
  };

  let mut lfs = Vec::new();
  let mut vec_sep = Vec::new();

  for (idx, file) in paths.iter().enumerate() {
    let filename = Path::new(file)
      .file_name()
      .ok_or(anyhow!("path is null"))?
      .to_str()
      .ok_or(anyhow!("path to str is null"))?;

    let file_extension = match Path::new(file).extension() {
      Some(ext) => ext.to_string_lossy().to_lowercase(),
      None => return Err(anyhow!("File extension not found")),
    };

    match file_extension.as_str() {
      "xls" | "xlsx" | "xlsm" | "xlsb" | "ods" => {
        vec_sep.push(b'|');
      }
      _ => {
        let mut opts = CsvOptions::new(file);
        opts.set_skiprows(skiprows);
        vec_sep.push(opts.get_delimiter()?);
      }
    }

    let lf = match file_extension.as_str() {
      "xls" | "xlsx" | "xlsm" | "xlsb" | "ods" => {
        let df: DataFrame = ExcelReader::from_path(file)?
          .worksheet_range_at(0, skiprows as u32)?
          .to_df()?;

        let excel_reader = if use_cols == vec!["all"] {
          df.lazy()
        } else {
          let exprs = use_cols.iter().map(|s| col(*s)).collect::<Vec<_>>();
          df.lazy().select(exprs)
        };

        excel_reader
      }
      _ => {
        let p: Arc<Path> = Arc::from(PathBuf::from(file));
        let csv_reader = LazyCsvReader::new(PlRefPath::try_from_path(&p)?)
          .with_has_header(true)
          .with_missing_is_null(true)
          .with_separator(vec_sep[idx])
          .with_infer_schema_length(Some(0))
          .with_skip_rows(skiprows)
          .finish()?;

        let csv_reader = if use_cols == vec!["all"] {
          csv_reader
        } else {
          let exprs = use_cols.iter().map(|s| col(*s)).collect::<Vec<_>>();
          csv_reader.select(exprs)
        };

        csv_reader
      }
    };

    let lf = lf.with_column(lit(filename).alias("_filename_"));
    lfs.push(lf);
  }

  let cat_lf = concat_lf_diagonal(
    lfs,
    UnionArgs {
      parallel: true,
      rechunk: true,
      to_supertypes: true,
      diagonal: true,
      strict: false,
      from_partitioned_ds: false,
      maintain_order: true,
    },
  )?;
  let mut cat_df =
    tokio::task::spawn_blocking(move || -> Result<_> { Ok(cat_lf.collect()?) }).await??;
  let row_len = cat_df.shape().0;
  if row_len < EXCEL_MAX_ROW && file_type.to_lowercase() == "xlsx" {
    XlsxWriter::new().write_dataframe(&cat_df, output_path.into())?;
  } else {
    CsvWriter::new(File::create(output_path)?)
      .with_separator(vec_sep[0])
      .finish(&mut cat_df)?;
  }

  Ok(())
}

pub async fn cat_with_csv(
  path: String,
  output_path: String,
  quoting: bool,
  skiprows: usize,
) -> Result<()> {
  let mut all_columns: IndexSet<Box<[u8]>> = IndexSet::with_capacity(16);
  let mut first_sep = None;
  let paths: Vec<&str> = path.split('|').collect();

  for (i, p) in paths.iter().enumerate() {
    let mut opts = CsvOptions::new(p);
    opts.set_skiprows(skiprows);
    let (sep, reader) = opts.skiprows_and_delimiter()?;

    if i == 0 {
      first_sep = Some(sep);
    }

    let mut rdr = ReaderBuilder::new()
      .delimiter(sep)
      .quoting(quoting)
      .from_reader(reader);

    for field in rdr.byte_headers()? {
      let fi = field.to_vec().into_boxed_slice();
      all_columns.insert(fi);
    }
  }

  let buf_wtr = BufWriter::with_capacity(WTR_BUFFER_SIZE, File::create(output_path)?);
  let mut wtr = WriterBuilder::new()
    .delimiter(first_sep.unwrap_or(b'|'))
    .from_writer(buf_wtr);

  for c in &all_columns {
    wtr.write_field(c)?;
  }
  wtr.write_byte_record(&ByteRecord::new())?;

  for p in paths.iter() {
    let mut opts = CsvOptions::new(p);
    opts.set_skiprows(skiprows);
    let (sep, reader) = opts.skiprows_and_delimiter()?;
    let mut rdr = ReaderBuilder::new()
      .delimiter(sep)
      .quoting(quoting)
      .from_reader(reader);

    let h = rdr.byte_headers()?;

    let mut columns_of_this_file =
      rustc_hash::FxHashMap::with_capacity_and_hasher(all_columns.len(), Default::default());

    for (n, field) in h.into_iter().enumerate() {
      let fi = field.to_vec().into_boxed_slice();
      if columns_of_this_file.contains_key(&fi) {
        return Err(anyhow!(
          "dulplicate column `{}` in file `{:?}`.",
          String::from_utf8_lossy(&*fi),
          p,
        ));
      }
      columns_of_this_file.insert(fi, n);
    }

    for row in rdr.byte_records() {
      let row = row?;
      for c in &all_columns {
        if let Some(idx) = columns_of_this_file.get(c) {
          if let Some(d) = row.get(*idx) {
            wtr.write_field(d)?;
          } else {
            wtr.write_field(b"")?;
          }
        } else {
          wtr.write_field(b"")?;
        }
      }
      wtr.write_byte_record(&ByteRecord::new())?;
    }
  }

  Ok(wtr.flush()?)
}

#[tauri::command]
pub async fn concat(
  path: String,
  output_path: String,
  file_type: String,
  mode: String,
  use_cols: String,
  quoting: bool,
  skiprows: usize,
) -> Result<String, String> {
  let start_time = Instant::now();

  match mode.as_str() {
    "csv" => match cat_with_csv(path, output_path, quoting, skiprows).await {
      Ok(()) => {
        let end_time = Instant::now();
        let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
        Ok(format!("{elapsed_time:.2}"))
      }
      Err(err) => Err(format!("{err}")),
    },
    _ => match cat_with_polars(path, output_path, file_type, use_cols, skiprows).await {
      Ok(()) => {
        let end_time = Instant::now();
        let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
        Ok(format!("{elapsed_time:.2}"))
      }
      Err(err) => Err(format!("{err}")),
    },
  }
}
