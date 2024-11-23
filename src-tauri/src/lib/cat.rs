use std::{error::Error, fs::File, num::NonZeroUsize, path::Path, time::Instant};

use polars::{
  frame::DataFrame,
  lazy::dsl::{functions::concat_lf_diagonal, lit},
  prelude::{
    CsvWriter, CsvWriterOptions, IntoLazy, LazyCsvReader, LazyFileListReader, LazyFrame, SerWriter,
    UnionArgs,
  },
};

use crate::{
  detect::detect_separator,
  excel::{ExcelReader, ToPolarsDataFrame},
  xlsx_writer::write_xlsx,
};

async fn concat_all(
  path: String,
  output_path: String,
  file_type: String,
  memory: bool,
  skip_rows: String,
) -> Result<(), Box<dyn Error>> {
  /* concat csv and excel files into a xlsx or csv file */
  let vec_path: Vec<&str> = path.split('|').collect();

  let mut lfs = Vec::new();
  let mut vec_sep = Vec::new();

  for (idx, file) in vec_path.iter().enumerate() {
    let filename = Path::new(file).file_name().unwrap().to_str().unwrap();

    let file_extension = match Path::new(file).extension() {
      Some(ext) => ext.to_string_lossy().to_lowercase(),
      None => return Err(("File extension not found").into()),
    };

    match file_extension.as_str() {
      "xls" | "xlsx" | "xlsm" | "xlsb" | "ods" | "parquet" => {
        vec_sep.push(b'|');
      }
      _ => {
        let sep = match detect_separator(file) {
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
      "parquet" => LazyFrame::scan_parquet(file, Default::default())?,
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
          .with_low_memory(!memory)
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

  if memory {
    let mut cat_df = cat_lf.collect()?;
    let row_len = cat_df.shape().0;
    if row_len < 104_0000 && file_type.to_lowercase() == "xlsx" {
      write_xlsx(cat_df, output_path.into())?;
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
    )?;
  }

  Ok(())
}

#[tauri::command]
pub async fn concat(
  file_path: String,
  output_path: String,
  file_type: String,
  memory: bool,
  skip_rows: String,
) -> Result<String, String> {
  let start_time = Instant::now();

  match concat_all(file_path, output_path, file_type, memory, skip_rows).await {
    Ok(()) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      let runtime = format!("{elapsed_time:.2} s");
      Ok(runtime)
    }
    Err(e) => Err(format!("concat_all => {e}")),
  }
}
