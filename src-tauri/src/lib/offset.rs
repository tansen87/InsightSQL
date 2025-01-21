use std::{collections::HashMap, fs::File, path::Path, time::Instant};

use anyhow::{anyhow, Result};
use calamine::Reader;
use polars::{
  frame::DataFrame,
  io::SerWriter,
  prelude::{
    col, concat_lf_diagonal, concat_str, lit, when, CsvWriter, DataType, IntoLazy, JoinArgs,
    JoinType, LazyCsvReader, LazyFileListReader, LazyFrame, SortMultipleOptions, UnionArgs,
  },
};

use crate::{
  utils::detect_separator,
  excel_reader::{ExcelReader, ToPolarsDataFrame},
  xlsx_writer::XlsxWriter,
};

async fn get_header(file_path: String) -> Result<Vec<HashMap<String, String>>> {
  let file_extension = match Path::new(&file_path).extension() {
    Some(ext) => ext.to_string_lossy().to_lowercase(),
    None => return Err(anyhow!("File extension not found")),
  };

  let mut vec_sep = Vec::new();
  match file_extension.as_str() {
    "xls" | "xlsx" | "xlsm" | "xlsb" | "ods" | "parquet" => {
      vec_sep.push(b'|');
    }
    _ => {
      let sep = match detect_separator(file_path.as_str(), 0) {
        Some(separator) => separator as u8,
        None => b',',
      };
      vec_sep.push(sep);
    }
  }

  let mut tmp = Vec::new();
  match file_extension.as_str() {
    "xls" | "xlsx" | "xlsm" | "xlsb" | "ods" => {
      let mut workbook = calamine::open_workbook_auto(file_path)?;
      let range = workbook.worksheet_range_at(0).unwrap()?;
      let vec_headers: Vec<String> = range
        .rows()
        .next()
        .ok_or(anyhow!("No data"))?
        .iter()
        .map(|cell| cell.to_string())
        .collect();
      let hs = vec_headers
        .into_iter()
        .enumerate()
        .map(|(_index, name)| {
          let mut map = std::collections::HashMap::new();
          map.insert("value".to_string(), name.clone());
          map.insert("label".to_string(), name);
          map
        })
        .collect::<Vec<_>>();
      tmp.push(hs);
    }
    _ => {
      let mut rdr = csv::ReaderBuilder::new()
        .delimiter(vec_sep[0])
        .has_headers(true)
        .from_reader(File::open(file_path)?);

      let headers = rdr.headers()?.clone();
      let vec_headers: Vec<String> = headers.into_iter().map(String::from).collect();

      let hs = vec_headers
        .into_iter()
        .enumerate()
        .map(|(_index, name)| {
          let mut map = std::collections::HashMap::new();
          map.insert("value".to_string(), name.clone());
          map.insert("label".to_string(), name);
          map
        })
        .collect::<Vec<_>>();
      tmp.push(hs);
    }
  }

  Ok(tmp[0].clone())
}

async fn offset_no_condition(file_path: &str, amount: String) -> Result<()> {
  let file_extension = match Path::new(&file_path).extension() {
    Some(ext) => ext.to_string_lossy().to_lowercase(),
    None => return Err(anyhow!("File extension not found")),
  };

  let mut vec_sep = Vec::new();
  match file_extension.as_str() {
    "xls" | "xlsx" | "xlsm" | "xlsb" | "ods" | "parquet" => {
      vec_sep.push(b'|');
    }
    _ => {
      let sep = match detect_separator(file_path, 0) {
        Some(separator) => separator as u8,
        None => b',',
      };
      vec_sep.push(sep);
    }
  }

  let lf = match file_extension.as_str() {
    "xls" | "xlsx" | "xlsm" | "xlsb" | "ods" => {
      let mut excel_reader = ExcelReader::new(file_path);
      let df: DataFrame = excel_reader.worksheet_range_at(0, 0)?.to_df()?;
      df.lazy()
    }
    _ => {
      let csv_reader = LazyCsvReader::new(file_path)
        .with_has_header(true)
        .with_missing_is_null(true)
        .with_separator(vec_sep[0])
        .with_infer_schema_length(Some(0))
        .with_low_memory(false)
        .finish()?;

      csv_reader
    }
  };

  let lf = lf.with_column(
    col(&amount)
      .cast(DataType::Float64)
      .fill_null(0.0)
      .abs()
      .alias("abs_amount"),
  );

  let grouped = lf.clone().group_by([col("abs_amount")]).agg([
    col("abs_amount").count().alias("count"),
    col(&amount)
      .cast(DataType::Float64)
      .fill_null(0.0)
      .sum()
      .alias("total"),
  ]);

  let merge = lf.clone().join(
    grouped,
    [col("abs_amount")],
    [col("abs_amount")],
    JoinArgs::new(JoinType::Inner),
  );

  let merge = merge
    .with_columns(vec![
      (col("count").cast(DataType::String)
        + lit("_")
        + col("abs_amount").cast(DataType::Decimal(Some(18), Some(2))))
      .alias("group"),
      when(col(&amount).cast(DataType::Float64).fill_null(0.0).gt(0.0))
        .then(lit("Y"))
        .otherwise(lit("N"))
        .alias("compare"),
    ])
    .sort(
      [&amount],
      SortMultipleOptions::new().with_order_descending_multi([true]),
    );

  let y_count_expr = when(col("compare").eq(lit("Y")))
    .then(lit(1))
    .otherwise(lit(0))
    .sum();
  let n_count_expr = when(col("compare").eq(lit("N")))
    .then(lit(1))
    .otherwise(lit(0))
    .sum();
  let merge = merge.with_columns(vec![
    y_count_expr.over([col("group")]).alias("Y_count"),
    n_count_expr.over([col("group")]).alias("N_count"),
  ]);

  let unique = merge
    .clone()
    .select([col("group").unique().alias("unique")])
    .collect()?;

  let mut unique_string = Vec::new();
  for xx in unique.iter() {
    for yy in xx.iter() {
      unique_string.push(yy.to_string());
    }
  }

  let mut lfs = Vec::new();
  let mut lfs_surplus = Vec::new();

  for x in unique_string.iter() {
    let subset = merge
      .clone()
      .filter(col("group").eq(lit(x.as_str().replace("\"", ""))));
    let q2 = subset.clone();
    let total = subset
      .clone()
      .select([col("total").sum()])
      .collect()?
      .get(0)
      .unwrap()[0]
      .to_string();
    let count = subset
      .clone()
      .select([col("count")])
      .collect()?
      .get(0)
      .unwrap()[0]
      .to_string();
    let y_count = subset
      .clone()
      .select([col("Y_count")])
      .collect()?
      .get(0)
      .unwrap()[0]
      .to_string();
    let n_count = subset
      .clone()
      .select([col("N_count")])
      .collect()?
      .get(0)
      .unwrap()[0]
      .to_string();

    if (total.parse::<f64>()? == 0.0)
      && (y_count.parse::<i32>()? > 0)
      && (n_count.parse::<i32>()? > 0)
    {
      lfs.push(subset.clone());
    }
    if (y_count.parse::<i32>()? > 0)
      && (n_count.parse::<i32>()? > 0)
      && (y_count.parse::<i32>()? == n_count.parse::<i32>()?)
    {
      lfs.push(subset.clone());
    }
    if (y_count.parse::<i32>()? != n_count.parse::<i32>()?)
      && (count.parse::<i32>()? > 1)
      && (y_count.parse::<i32>()? > 0)
      && (n_count.parse::<i32>()? > 0)
    {
      let y_num: u32 = y_count.parse()?;
      let n_num: u32 = n_count.parse()?;
      let min_count = if y_num < n_num { y_num } else { n_num };
      let max_count = if y_num > n_num { y_num } else { n_num };
      let diff = max_count - min_count;

      let y_subset = q2
        .clone()
        .filter(col("compare").eq(lit("Y")))
        .limit(min_count);
      let n_subset = q2
        .clone()
        .filter(col("compare").eq(lit("N")))
        .limit(min_count);

      lfs.push(y_subset);
      lfs.push(n_subset);

      if y_num > n_num {
        let y_surplus = q2.clone().filter(col("compare").eq(lit("Y"))).tail(diff);
        lfs_surplus.push(y_surplus);
      }
      if y_num < n_num {
        let n_surplus = q2.clone().filter(col("compare").eq(lit("N"))).tail(diff);
        lfs_surplus.push(n_surplus);
      }
    }
    if !(total.parse::<f64>()? == 0.0 && y_count.parse::<i32>()? > 0 && n_count.parse::<i32>()? > 0)
      && !(y_count.parse::<i32>()? != n_count.parse::<i32>()?
        && count.parse::<i32>()? > 1
        && y_count.parse::<i32>()? > 0
        && n_count.parse::<i32>()? > 0)
    {
      lfs_surplus.push(subset.clone());
    }
  }

  let parent_path = Path::new(&file_path)
    .parent()
    .map(|path| path.to_string_lossy())
    .unwrap();
  let file_name = Path::new(&file_path).file_stem().unwrap().to_str().unwrap();

  if !lfs.is_empty() {
    let mut cat = concat_lf_diagonal(
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

    let cat_row = cat.shape().0;
    if cat_row < 104_0000 {
      let save_path = format!("{parent_path}/{file_name}.net.xlsx");
      let mut xlsx_writer = XlsxWriter::new();
      xlsx_writer.write_dataframe(&cat, save_path.into())?;
    } else {
      let save_path = format!("{parent_path}/{file_name}.net.csv");
      CsvWriter::new(File::create(save_path)?)
        .with_separator(vec_sep[0])
        .finish(&mut cat)?;
    }
  }

  if !lfs_surplus.is_empty() {
    let mut cat_surplus = concat_lf_diagonal(
      lfs_surplus,
      UnionArgs {
        parallel: true,
        rechunk: true,
        to_supertypes: true,
        diagonal: true,
        from_partitioned_ds: false,
      },
    )?
    .collect()?;

    let cat_surplus_row = cat_surplus.shape().0;
    if cat_surplus_row < 104_0000 {
      let save_path = format!("{parent_path}/{file_name}.surplus.xlsx");
      let mut xlsx_writer = XlsxWriter::new();
      xlsx_writer.write_dataframe(&cat_surplus, save_path.into())?;
    } else {
      let save_path = format!("{parent_path}/{file_name}.surplus.csv");
      CsvWriter::new(File::create(save_path)?)
        .with_separator(vec_sep[0])
        .finish(&mut cat_surplus)?;
    }
  }

  Ok(())
}

async fn offset_condition(file_path: &str, amount: String, cond: String) -> Result<()> {
  let vec_cond: Vec<&str> = cond.split('|').collect();

  let file_extension = match Path::new(&file_path).extension() {
    Some(ext) => ext.to_string_lossy().to_lowercase(),
    None => return Err(anyhow!("File extension not found")),
  };

  let mut vec_sep = Vec::new();
  match file_extension.as_str() {
    "xls" | "xlsx" | "xlsm" | "xlsb" | "ods" | "parquet" => {
      vec_sep.push(b'|');
    }
    _ => {
      let sep = match detect_separator(file_path, 0) {
        Some(separator) => separator as u8,
        None => b',',
      };
      vec_sep.push(sep);
    }
  }

  let lf = match file_extension.as_str() {
    "parquet" => LazyFrame::scan_parquet(file_path, Default::default())?,
    "xls" | "xlsx" | "xlsm" | "xlsb" | "ods" => {
      let mut excel_reader = ExcelReader::new(file_path);
      let df: DataFrame = excel_reader.worksheet_range_at(0, 0)?.to_df()?;
      df.lazy()
    }
    _ => {
      let csv_reader = LazyCsvReader::new(file_path)
        .with_has_header(true)
        .with_missing_is_null(true)
        .with_separator(vec_sep[0])
        .with_infer_schema_length(Some(0))
        .with_low_memory(false)
        .finish()?;

      csv_reader
    }
  };

  let lf = lf.with_column(
    col(&amount)
      .cast(DataType::Float64)
      .fill_null(0.0)
      .abs()
      .alias("abs_amount"),
  );

  let grouped = lf.clone().group_by([col("abs_amount")]).agg([
    col("abs_amount").count().alias("count"),
    col(&amount)
      .cast(DataType::Float64)
      .fill_null(0.0)
      .sum()
      .alias("total"),
  ]);

  let merge = lf.clone().join(
    grouped,
    [col("abs_amount")],
    [col("abs_amount")],
    JoinArgs::new(JoinType::Inner),
  );

  let merge = merge
    .with_columns(vec![
      (col("count").cast(DataType::String)
        + lit("_")
        + col("abs_amount").cast(DataType::Decimal(Some(18), Some(2)))
        + lit("_")
        + concat_str(
          vec_cond.iter().map(|c| col(*c)).collect::<Vec<_>>(),
          "_",
          true,
        ))
      .alias("group"),
      when(col(&amount).cast(DataType::Float64).fill_null(0.0).gt(0.0))
        .then(lit("Y"))
        .otherwise(lit("N"))
        .alias("compare"),
    ])
    .sort(
      [&amount],
      SortMultipleOptions::new().with_order_descending_multi([true]),
    );

  let y_count_expr = when(col("compare").eq(lit("Y")))
    .then(lit(1))
    .otherwise(lit(0))
    .sum();
  let n_count_expr = when(col("compare").eq(lit("N")))
    .then(lit(1))
    .otherwise(lit(0))
    .sum();
  let merge = merge.with_columns(vec![
    y_count_expr.over([col("group")]).alias("Y_count"),
    n_count_expr.over([col("group")]).alias("N_count"),
  ]);

  let unique = merge
    .clone()
    .select([col("group").unique().alias("unique")])
    .collect()?;

  let mut unique_string = Vec::new();
  for xx in unique.iter() {
    for yy in xx.iter() {
      unique_string.push(yy.to_string());
    }
  }

  let mut lfs = Vec::new();
  let mut lfs_surplus = Vec::new();

  for x in unique_string.iter() {
    let subset = merge
      .clone()
      .filter(col("group").eq(lit(x.as_str().replace("\"", ""))));
    let q2 = subset.clone();
    let total = subset
      .clone()
      .select([col("total").sum()])
      .collect()?
      .get(0)
      .unwrap()[0]
      .to_string();
    let count = subset
      .clone()
      .select([col("count")])
      .collect()?
      .get(0)
      .unwrap()[0]
      .to_string();
    let y_count = subset
      .clone()
      .select([col("Y_count")])
      .collect()?
      .get(0)
      .unwrap()[0]
      .to_string();
    let n_count = subset
      .clone()
      .select([col("N_count")])
      .collect()?
      .get(0)
      .unwrap()[0]
      .to_string();

    if (total.parse::<f64>()? == 0.0)
      && (y_count.parse::<i32>()? > 0)
      && (n_count.parse::<i32>()? > 0)
    {
      lfs.push(subset.clone());
    }
    if (y_count.parse::<i32>()? > 0)
      && (n_count.parse::<i32>()? > 0)
      && (y_count.parse::<i32>()? == n_count.parse::<i32>()?)
    {
      lfs.push(subset.clone());
    }
    if (y_count.parse::<i32>()? != n_count.parse::<i32>()?)
      && (count.parse::<i32>()? > 1)
      && (y_count.parse::<i32>()? > 0)
      && (n_count.parse::<i32>()? > 0)
    {
      let y_num: u32 = y_count.parse()?;
      let n_num: u32 = n_count.parse()?;
      let min_count = if y_num < n_num { y_num } else { n_num };
      let max_count = if y_num > n_num { y_num } else { n_num };
      let diff = max_count - min_count;

      let y_subset = q2
        .clone()
        .filter(col("compare").eq(lit("Y")))
        .limit(min_count);
      let n_subset = q2
        .clone()
        .filter(col("compare").eq(lit("N")))
        .limit(min_count);

      lfs.push(y_subset);
      lfs.push(n_subset);

      if y_num > n_num {
        let y_surplus = q2.clone().filter(col("compare").eq(lit("Y"))).tail(diff);
        lfs_surplus.push(y_surplus);
      }
      if y_num < n_num {
        let n_surplus = q2.clone().filter(col("compare").eq(lit("N"))).tail(diff);
        lfs_surplus.push(n_surplus);
      }
    }
    if !(total.parse::<f64>()? == 0.0 && y_count.parse::<i32>()? > 0 && n_count.parse::<i32>()? > 0)
      && !(y_count.parse::<i32>()? != n_count.parse::<i32>()?
        && count.parse::<i32>()? > 1
        && y_count.parse::<i32>()? > 0
        && n_count.parse::<i32>()? > 0)
    {
      lfs_surplus.push(subset.clone());
    }
  }

  let parent_path = Path::new(&file_path)
    .parent()
    .map(|path| path.to_string_lossy())
    .unwrap();
  let file_name = Path::new(&file_path).file_stem().unwrap().to_str().unwrap();

  if !lfs.is_empty() {
    let mut cat = concat_lf_diagonal(
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

    let cat_row = cat.shape().0;
    if cat_row < 104_0000 {
      let save_path = format!("{parent_path}/{file_name}.net.xlsx");
      let mut xlsx_writer = XlsxWriter::new();
      xlsx_writer.write_dataframe(&cat, save_path.into())?;
    } else {
      let save_path = format!("{parent_path}/{file_name}.net.csv");
      CsvWriter::new(File::create(save_path)?)
        .with_separator(vec_sep[0])
        .finish(&mut cat)?;
    }
  }

  if !lfs_surplus.is_empty() {
    let mut cat_surplus = concat_lf_diagonal(
      lfs_surplus,
      UnionArgs {
        parallel: true,
        rechunk: true,
        to_supertypes: true,
        diagonal: true,
        from_partitioned_ds: false,
      },
    )?
    .collect()?;

    let cat_surplus_row = cat_surplus.shape().0;
    if cat_surplus_row < 104_0000 {
      let save_path = format!("{parent_path}/{file_name}.surplus.xlsx");
      let mut xlsx_writer = XlsxWriter::new();
      xlsx_writer.write_dataframe(&cat_surplus, save_path.into())?;
    } else {
      let save_path = format!("{parent_path}/{file_name}.surplus.csv");
      CsvWriter::new(File::create(save_path)?)
        .with_separator(vec_sep[0])
        .finish(&mut cat_surplus)?;
    }
  }

  Ok(())
}

#[tauri::command]
pub async fn get_offset_headers(file_path: String) -> Result<Vec<HashMap<String, String>>, String> {
  match get_header(file_path).await {
    Ok(result) => Ok(result),
    Err(err) => Err(format!("get header error: {err}")),
  }
}

#[tauri::command]
pub async fn offset(
  file_path: String,
  amount: String,
  cond: String,
  has_cond: bool,
) -> Result<String, String> {
  let start_time = Instant::now();

  if has_cond {
    match offset_condition(file_path.as_str(), amount, cond).await {
      Ok(_) => {
        let end_time = Instant::now();
        let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
        Ok(format!("{elapsed_time:.2}"))
      }
      Err(err) => Err(format!("offset failed: {err}")),
    }
  } else {
    match offset_no_condition(file_path.as_str(), amount).await {
      Ok(_) => {
        let end_time = Instant::now();
        let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
        Ok(format!("{elapsed_time:.2}"))
      }
      Err(err) => Err(format!("offset failed: {err}")),
    }
  }
}
