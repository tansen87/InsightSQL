use std::{cmp, fs::File, io::BufWriter, path::Path, time::Instant};

use anyhow::Result;
use csv::{ReaderBuilder, WriterBuilder};

use self::Number::{Float, Int};
use crate::{
  io::csv::{options::CsvOptions, selection::Selection},
  utils::WTR_BUFFER_SIZE,
};

pub async fn sort_csv<P: AsRef<Path> + Send + Sync>(
  path: P,
  column: String,
  numeric: bool,
  reverse: bool,
  quoting: bool,
  skiprows: usize,
) -> Result<()> {
  let mut opts = CsvOptions::new(&path);
  opts.set_skiprows(skiprows);
  let (sep, reader) = opts.skiprows_and_delimiter()?;
  let output_path = opts.output_path(Some("sort"), None)?;

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .quoting(quoting)
    .from_reader(reader);
  let headers = rdr.byte_headers()?.clone();
  let sel = Selection::from_headers(&headers, &[column.as_str()][..])?;

  let mut all = rdr.byte_records().collect::<Result<Vec<_>, _>>()?;
  match (numeric, reverse) {
    (false, false) => all.sort_by(|r1, r2| {
      let a = sel.get_row_key(r1);
      let b = sel.get_row_key(r2);
      iter_cmp(a.iter(), b.iter())
    }),
    (true, false) => all.sort_by(|r1, r2| {
      let a = sel.get_row_key(r1);
      let b = sel.get_row_key(r2);
      iter_cmp_num(
        a.iter().map(|x| x.as_slice()),
        b.iter().map(|x| x.as_slice()),
      )
    }),
    (false, true) => all.sort_by(|r1, r2| {
      let a = sel.get_row_key(r1);
      let b = sel.get_row_key(r2);
      iter_cmp(b.iter(), a.iter())
    }),
    (true, true) => all.sort_by(|r1, r2| {
      let a = sel.get_row_key(r1);
      let b = sel.get_row_key(r2);
      iter_cmp_num(
        b.iter().map(|x| x.as_slice()),
        a.iter().map(|x| x.as_slice()),
      )
    }),
  }

  let output_file = File::create(output_path)?;
  let buf_wtr = BufWriter::with_capacity(WTR_BUFFER_SIZE, output_file);
  let mut wtr = WriterBuilder::new().delimiter(sep).from_writer(buf_wtr);

  wtr.write_record(&headers)?;

  for r in all.into_iter() {
    wtr.write_byte_record(&r)?;
  }

  Ok(wtr.flush()?)
}

/// Order `a` and `b` lexicographically using `Ord`
pub fn iter_cmp<A, L, R>(mut a: L, mut b: R) -> cmp::Ordering
where
  A: Ord,
  L: Iterator<Item = A>,
  R: Iterator<Item = A>,
{
  loop {
    match (a.next(), b.next()) {
      (None, None) => return cmp::Ordering::Equal,
      (None, _) => return cmp::Ordering::Less,
      (_, None) => return cmp::Ordering::Greater,
      (Some(x), Some(y)) => match x.cmp(&y) {
        cmp::Ordering::Equal => (),
        non_eq => return non_eq,
      },
    }
  }
}

/// Try parsing `a` and `b` as numbers when ordering
pub fn iter_cmp_num<'a, L, R>(mut a: L, mut b: R) -> cmp::Ordering
where
  L: Iterator<Item = &'a [u8]>,
  R: Iterator<Item = &'a [u8]>,
{
  loop {
    match (next_num(&mut a), next_num(&mut b)) {
      (None, None) => return cmp::Ordering::Equal,
      (None, _) => return cmp::Ordering::Less,
      (_, None) => return cmp::Ordering::Greater,
      (Some(x), Some(y)) => match compare_num(x, y) {
        cmp::Ordering::Equal => (),
        non_eq => return non_eq,
      },
    }
  }
}

#[derive(Clone, Copy, PartialEq)]
enum Number {
  Int(i64),
  Float(f64),
}

fn compare_num(n1: Number, n2: Number) -> cmp::Ordering {
  match (n1, n2) {
    (Int(i1), Int(i2)) => i1.cmp(&i2),
    (Int(i1), Float(f2)) => compare_float(i1 as f64, f2),
    (Float(f1), Int(i2)) => compare_float(f1, i2 as f64),
    (Float(f1), Float(f2)) => compare_float(f1, f2),
  }
}

fn compare_float(f1: f64, f2: f64) -> cmp::Ordering {
  f1.partial_cmp(&f2).unwrap_or(cmp::Ordering::Equal)
}

fn next_num<'a, X>(xs: &mut X) -> Option<Number>
where
  X: Iterator<Item = &'a [u8]>,
{
  xs.next()
    .and_then(|bytes| std::str::from_utf8(bytes).ok())
    .and_then(|s| {
      if let Ok(i) = s.parse::<i64>() {
        Some(Number::Int(i))
      } else if let Ok(f) = s.parse::<f64>() {
        Some(Number::Float(f))
      } else {
        None
      }
    })
}

#[tauri::command]
pub async fn sort(
  path: String,
  column: String,
  numeric: bool,
  reverse: bool,
  quoting: bool,
  skiprows: usize,
) -> Result<String, String> {
  let start_time = Instant::now();

  match sort_csv(path, column, numeric, reverse, quoting, skiprows).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("{err}")),
  }
}
