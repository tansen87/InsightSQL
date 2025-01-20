use std::{fs::File, num::NonZeroUsize, path::Path, time::Instant};

use anyhow::{anyhow, Result};
use indexmap::IndexSet;
use polars::{
  frame::DataFrame,
  lazy::dsl::{functions::concat_lf_diagonal, lit},
  prelude::{
    CsvWriter, CsvWriterOptions, IntoLazy, LazyCsvReader, LazyFileListReader, SerWriter, UnionArgs,
  },
};

use crate::{
  excel_reader::{ExcelReader, ToPolarsDataFrame},
  utils::detect_separator,
  xlsx_writer::XlsxWriter,
};

async fn cat_with_polars(
  path: String,
  output_path: String,
  file_type: String,
  mode: String,
  skip_rows: String,
) -> Result<()> {
  /* concat csv and excel files into a xlsx or csv file */
  let low_memory = match mode.as_str() {
    "memory" => false,
    "stream" => true,
    _ => false,
  };

  let vec_path: Vec<&str> = path.split('|').collect();

  let mut lfs = Vec::new();
  let mut vec_sep = Vec::new();

  for (idx, file) in vec_path.iter().enumerate() {
    let filename = Path::new(file).file_name().unwrap().to_str().unwrap();

    let file_extension = match Path::new(file).extension() {
      Some(ext) => ext.to_string_lossy().to_lowercase(),
      None => return Err(anyhow!("File extension not found")),
    };

    match file_extension.as_str() {
      "xls" | "xlsx" | "xlsm" | "xlsb" | "ods" | "parquet" => {
        vec_sep.push(b'|');
      }
      _ => {
        let sep = match detect_separator(file, skip_rows.parse::<usize>()?) {
          Some(separator) => separator as u8,
          None => b',',
        };
        vec_sep.push(sep);
      }
    }

    let lf = match file_extension.as_str() {
      "xls" | "xlsx" | "xlsm" | "xlsb" | "ods" => {
        let mut excel_reader = ExcelReader::new(file);
        let df: DataFrame = excel_reader
          .worksheet_range_at(0, skip_rows.parse::<u32>()?)?
          .to_df()?;
        df.lazy()
      }
      _ => {
        let csv_reader = LazyCsvReader::new(file)
          .with_has_header(true)
          .with_missing_is_null(true)
          .with_separator(vec_sep[idx])
          .with_infer_schema_length(Some(0))
          .with_skip_rows(skip_rows.parse::<usize>()?)
          .with_low_memory(low_memory)
          .finish()?;

        csv_reader
      }
    };

    let lf = lf.with_column(lit(filename).alias("FileName"));
    lfs.push(lf);
  }

  let cat_lf = concat_lf_diagonal(
    lfs,
    UnionArgs {
      parallel: true,
      rechunk: true,
      to_supertypes: true,
      diagonal: true,
      from_partitioned_ds: false,
    },
  )?;

  if !low_memory {
    let mut cat_df = cat_lf.collect()?;
    let row_len = cat_df.shape().0;
    if row_len < 104_0000 && file_type.to_lowercase() == "xlsx" {
      let mut xlsx_writer = XlsxWriter::new();
      xlsx_writer.write_xlsx(&cat_df, output_path.into())?;
    } else {
      CsvWriter::new(File::create(output_path)?)
        .with_separator(vec_sep[0])
        .finish(&mut cat_df)?;
    }
  } else {
    cat_lf.sink_csv(
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
          separator: vec_sep[0],
          quote_char: b'"',
          null: String::new(),
          line_terminator: "\n".into(),
          quote_style: Default::default(),
        },
      },
      Default::default(),
    )?;
  }

  Ok(())
}

async fn cat_with_csv(path: String, output_path: String) -> Result<()> {
  let mut all_columns: IndexSet<Box<[u8]>> = IndexSet::with_capacity(16);

  let mut vec_sep = Vec::new();

  let paths: Vec<&str> = path.split('|').collect();

  for p in &paths {
    let sep = match detect_separator(&p, 0) {
      Some(separator) => separator as u8,
      None => b',',
    };
    vec_sep.push(sep);

    let mut rdr = csv::ReaderBuilder::new().delimiter(sep).from_path(p)?;

    for field in rdr.byte_headers()? {
      let fi = field.to_vec().into_boxed_slice();
      all_columns.insert(fi);
    }
  }

  let mut wtr = csv::WriterBuilder::new()
    .delimiter(vec_sep[0])
    .from_path(output_path)?;

  for c in &all_columns {
    wtr.write_field(c)?;
  }
  wtr.write_byte_record(&csv::ByteRecord::new())?;

  for (idx, p) in paths.iter().enumerate() {
    let mut rdr = csv::ReaderBuilder::new()
      .delimiter(vec_sep[idx])
      .from_path(&p)?;
    let h = rdr.byte_headers()?;

    let mut columns_of_this_file =
      rustc_hash::FxHashMap::with_capacity_and_hasher(all_columns.len(), Default::default());

    for (n, field) in h.into_iter().enumerate() {
      let fi = field.to_vec().into_boxed_slice();
      if columns_of_this_file.contains_key(&fi) {
        eprintln!(
          "Warning: dulplicate column `{}` name in file `{:?}`.",
          String::from_utf8_lossy(&*fi),
          p,
        );
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
      wtr.write_byte_record(&csv::ByteRecord::new())?;
    }
  }

  Ok(())
}

#[tauri::command]
pub async fn concat(
  file_path: String,
  output_path: String,
  file_type: String,
  mode: String,
  skip_rows: String,
) -> Result<String, String> {
  let start_time = Instant::now();

  match mode.to_lowercase().as_str() {
    "csv" => match cat_with_csv(file_path, output_path).await {
      Ok(()) => {
        let end_time = Instant::now();
        let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
        Ok(format!("{elapsed_time:.2}"))
      }
      Err(err) => Err(format!("cat failed: {err}")),
    },
    _ => match cat_with_polars(file_path, output_path, file_type, mode, skip_rows).await {
      Ok(()) => {
        let end_time = Instant::now();
        let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
        Ok(format!("{elapsed_time:.2}"))
      }
      Err(err) => Err(format!("cat failed: {err}")),
    },
  }
}
