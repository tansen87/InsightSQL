use std::{error::Error, fs::File, path::Path};

use polars::{
  frame::DataFrame,
  lazy::dsl::{functions::concat_lf_diagonal, lit},
  prelude::{
    CsvWriter, IntoLazy, LazyCsvReader, LazyFileListReader, LazyFrame, SerWriter, UnionArgs,
  },
};

use crate::{
  excel::{ExcelReader, ToPolarsDataFrame},
  xlsx_writer::write_xlsx,
};

fn concat_all(path: String, sep: String) -> Result<(), Box<dyn Error>> {
  /* concat csv and excel files into a xlsx or csv file */
  let mut separator = Vec::new();
  let sep_u8 = if sep == "\\t" {
    b'\t'
  } else {
    sep.into_bytes()[0]
  };
  separator.push(sep_u8);

  let vec_path: Vec<&str> = path.split(',').collect();
  let mut lfs = Vec::new();

  for file in vec_path.iter() {
    let fname = match Path::new(&file).file_name() {
      Some(name) => match name.to_str() {
        Some(name_str) => name_str.split('.').collect::<Vec<&str>>(),
        None => vec![],
      },
      None => vec![],
    };

    let file_extension = match Path::new(file).extension() {
      Some(ext) => ext.to_string_lossy().to_lowercase(),
      None => return Err(("File extension not found").into()),
    };

    let lf = match file_extension.as_str() {
      "parquet" => LazyFrame::scan_parquet(file, Default::default())?,
      "xls" | "xlsx" | "xlsm" | "xlsb" | "ods" => {
        let mut excel_reader = ExcelReader::new(file);
        let df: DataFrame = excel_reader.worksheet_range_at(0)?.to_df()?;
        df.lazy()
      }
      _ => {
        let csv_reader = LazyCsvReader::new(file)
          .with_has_header(true)
          .with_missing_is_null(true)
          .with_separator(separator[0])
          .with_infer_schema_length(Some(0))
          .with_low_memory(false)
          .finish()?;

        csv_reader
      }
    };

    let file_name = format!("{}.{}", fname[0], fname[1]);
    let lf = lf.with_column(lit(file_name).alias("FileName"));
    lfs.push(lf);
  }

  // concat dataframe
  let mut union_df = concat_lf_diagonal(
    lfs,
    UnionArgs {
      parallel: true,
      rechunk: true,
      to_supertypes: true,
      diagonal: true,
      from_partitioned_ds: false,
    },
  )?
  .collect()?;

  let file_path = Path::new(&path)
    .parent()
    .map(|parent| parent.to_string_lossy())
    .unwrap_or_else(|| "Default Path".to_string().into());
  let current_time = chrono::Local::now().format("%Y-%m-%d-%H%M%S");

  let row_len = union_df.shape().0;
  if row_len < 104_0000 {
    let save_path = format!("{}/cat {}.xlsx", file_path, current_time);
    write_xlsx(union_df, save_path.into())?;
  } else {
    let save_path = format!("{}/cat {}.csv", file_path, current_time);
    CsvWriter::new(File::create(save_path)?)
    .with_separator(separator[0])
    .finish(&mut union_df)?;
  }

  Ok(())
}

#[tauri::command]
pub async fn concat(file_path: String, sep: String, window: tauri::Window) {
  match (async { concat_all(file_path, sep) }).await {
    Ok(result) => result,
    Err(err) => {
      eprintln!("concat error: {err}");
      window.emit("cat_err", &err.to_string()).unwrap();
      err.to_string();
    }
  };
}
