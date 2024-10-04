use std::{collections::HashMap, fs::File, path::Path, time::Instant};

use calamine::Reader;
use polars::prelude::*;

use crate::{
  excel::{ExcelReader, ToPolarsDataFrame},
  xlsx_writer,
};

fn get_header(
  file_path: String,
  sep: String,
) -> Result<Vec<HashMap<String, String>>, Box<dyn std::error::Error>> {
  let sep = if sep == "\\t" {
    b'\t'
  } else {
    sep.into_bytes()[0]
  };

  let file_extension = match Path::new(&file_path).extension() {
    Some(ext) => ext.to_string_lossy().to_lowercase(),
    None => return Err(("File extension not found").into()),
  };

  let mut tmp = Vec::new();
  match file_extension.as_str() {
    "xls" | "xlsx" | "xlsm" | "xlsb" | "ods" => {
      let mut workbook = calamine::open_workbook_auto(file_path)?;
      let range = workbook.worksheet_range_at(0).unwrap()?;
      let vec_headers: Vec<String> = range
        .rows()
        .next()
        .ok_or("No data")?
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
        .delimiter(sep)
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

fn offset_no_condition(
  file_path: String,
  amount: String,
  sep: String,
  output_path: String,
) -> Result<(), Box<dyn std::error::Error>> {
  let sep = if sep == "\\t" {
    b'\t'
  } else {
    sep.into_bytes()[0]
  };

  let file_extension = match Path::new(&file_path).extension() {
    Some(ext) => ext.to_string_lossy().to_lowercase(),
    None => return Err(("File extension not found").into()),
  };

  let lf = match file_extension.as_str() {
    "xls" | "xlsx" | "xlsm" | "xlsb" | "ods" => {
      let mut excel_reader = ExcelReader::new(file_path);
      let df: DataFrame = excel_reader.worksheet_range_at(0)?.to_df()?;
      df.lazy()
    }
    _ => {
      let csv_reader = LazyCsvReader::new(file_path)
        .with_has_header(true)
        .with_missing_is_null(true)
        .with_separator(sep)
        .with_infer_schema_length(Some(0))
        .with_low_memory(false)
        .finish()?;

      csv_reader
    }
  };

  let lf = lf.with_column(
    col(&amount)
      .cast(DataType::Float64)
      .abs()
      .alias("abs_amount"),
  );

  let grouped = lf.clone().group_by([col("abs_amount")]).agg([
    col("abs_amount").count().alias("count"),
    col(&amount).cast(DataType::Float64).sum().alias("total"),
  ]);

  let merge = lf.clone().join(
    grouped,
    [col("abs_amount")],
    [col("abs_amount")],
    JoinArgs::new(JoinType::Inner),
  );

  let merge = merge
    .with_columns(vec![
      col("count").cast(DataType::String).alias("group") + col("abs_amount").cast(DataType::String),
      when(col(&amount).cast(DataType::Float64).gt(0.0))
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
    .collect()
    .unwrap();

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
        let n_surplus = q2.clone().filter(col("compare").eq(lit("Y"))).tail(diff);
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

  let cat_row = cat.shape().0;
  let cat_surplus_row = cat_surplus.shape().0;
  if cat_row < 104_0000 {
    let save_path = format!("{output_path}_net.xlsx");
    xlsx_writer::write_xlsx(cat, save_path.into())?;
  } else {
    let save_path = format!("{output_path}_net.csv");
    CsvWriter::new(File::create(save_path)?)
      .with_separator(sep)
      .finish(&mut cat)?;
  }

  if cat_surplus_row < 104_0000 {
    let save_path = format!("{output_path}_surplus.xlsx");
    xlsx_writer::write_xlsx(cat_surplus, save_path.into())?;
  } else {
    let save_path = format!("{output_path}_surplus.csv");
    CsvWriter::new(File::create(save_path)?)
      .with_separator(sep)
      .finish(&mut cat_surplus)?;
  }
  Ok(())
}

fn offset_condition(
  file_path: String,
  amount: String,
  cond: String,
  sep: String,
  output_path: String,
) -> Result<(), Box<dyn std::error::Error>> {
  let sep = if sep == "\\t" {
    b'\t'
  } else {
    sep.into_bytes()[0]
  };
  let vec_cond: Vec<&str> = cond.split('|').collect();

  let file_extension = match Path::new(&file_path).extension() {
    Some(ext) => ext.to_string_lossy().to_lowercase(),
    None => return Err(("File extension not found").into()),
  };

  let lf = match file_extension.as_str() {
    "parquet" => LazyFrame::scan_parquet(file_path, Default::default())?,
    "xls" | "xlsx" | "xlsm" | "xlsb" | "ods" => {
      let mut excel_reader = ExcelReader::new(file_path);
      let df: DataFrame = excel_reader.worksheet_range_at(0)?.to_df()?;
      df.lazy()
    }
    _ => {
      let csv_reader = LazyCsvReader::new(file_path)
        .with_has_header(true)
        .with_missing_is_null(true)
        .with_separator(sep)
        .with_infer_schema_length(Some(0))
        .with_low_memory(false)
        .finish()?;

      csv_reader
    }
  };

  let lf = lf.with_column(
    col(&amount)
      .cast(DataType::Float64)
      .abs()
      .alias("abs_amount"),
  );

  let grouped = lf.clone().group_by([col("abs_amount")]).agg([
    col("abs_amount").count().alias("count"),
    col(&amount).cast(DataType::Float64).sum().alias("total"),
  ]);

  let merge = lf.clone().join(
    grouped,
    [col("abs_amount")],
    [col("abs_amount")],
    JoinArgs::new(JoinType::Inner),
  );

  let merge = merge
    .with_columns(vec![
      col("count").cast(DataType::String).alias("group")
        + col("abs_amount").cast(DataType::String)
        + cols(vec_cond),
      when(col(&amount).cast(DataType::Float64).gt(0.0))
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
    .collect()
    .unwrap();

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
        let n_surplus = q2.clone().filter(col("compare").eq(lit("Y"))).tail(diff);
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

  let cat_row = cat.shape().0;
  let cat_surplus_row = cat_surplus.shape().0;
  if cat_row < 104_0000 {
    let save_path = format!("{output_path}_net.xlsx");
    xlsx_writer::write_xlsx(cat, save_path.into())?;
  } else {
    let save_path = format!("{output_path}_net.csv");
    CsvWriter::new(File::create(save_path)?)
      .with_separator(sep)
      .finish(&mut cat)?;
  }

  if cat_surplus_row < 104_0000 {
    let save_path = format!("{output_path}_surplus.xlsx");
    xlsx_writer::write_xlsx(cat_surplus, save_path.into())?;
  } else {
    let save_path = format!("{output_path}_surplus.csv");
    CsvWriter::new(File::create(save_path)?)
      .with_separator(sep)
      .finish(&mut cat_surplus)?;
  }
  Ok(())
}

#[tauri::command]
pub async fn get_offset_headers(
  file_path: String,
  sep: String,
  window: tauri::Window,
) -> Vec<HashMap<String, String>> {
  let headers = match (async { get_header(file_path, sep) }).await {
    Ok(result) => result,
    Err(err) => {
      eprintln!("get headers error: {err}");
      window.emit("get_err", &err.to_string()).unwrap();
      return Vec::new();
    }
  };

  headers
}

#[tauri::command]
pub async fn offset(
  file_path: String,
  sep: String,
  amount: String,
  cond: String,
  has_cond: bool,
  output_path: String,
  window: tauri::Window,
) {
  let start_time = Instant::now();

  if has_cond {
    match (async { offset_condition(file_path, amount, cond, sep, output_path) }).await {
      Ok(result) => result,
      Err(error) => {
        eprintln!("offset error:: {error}");
        window.emit("offset_err", &error.to_string()).unwrap();
        return ();
      }
    };
  } else {
    match (async { offset_no_condition(file_path, amount, sep, output_path) }).await {
      Ok(result) => result,
      Err(error) => {
        eprintln!("offset error:: {error}");
        window.emit("offset_err", &error.to_string()).unwrap();
        return ();
      }
    };
  }

  let end_time = Instant::now();
  let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
  let runtime = format!("{elapsed_time:.2} s");
  window.emit("runtime", runtime).unwrap();
}
