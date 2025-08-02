use std::{
  collections::hash_map::{Entry, HashMap},
  fmt,
  fs::File,
  io::{Cursor, Read, Seek, Write},
  iter::repeat,
  mem::swap,
  path::{Path, PathBuf},
  time::Instant,
};

use anyhow::Result;
use byteorder::{BigEndian, WriteBytesExt};
use csv::{ReaderBuilder, WriterBuilder};

use crate::index::Indexed;
use crate::io::csv::{options::CsvOptions, selection::Selection};

type ByteString = Vec<u8>;

struct IoState<R, W: Write> {
  wtr: csv::Writer<W>,
  rdr1: csv::Reader<R>,
  sel1: Selection,
  rdr2: csv::Reader<R>,
  sel2: Selection,
  nulls: bool,
}

impl<R: Read + Seek, W: Write> IoState<R, W> {
  fn write_headers(&mut self, extend: bool) -> Result<()> {
    if extend {
      let mut headers = self.rdr1.byte_headers()?.clone();
      headers.extend(self.rdr2.byte_headers()?.iter());
      self.wtr.write_record(&headers)?;
    } else {
      let headers = self.rdr1.byte_headers()?;
      self.wtr.write_record(headers)?;
    }

    Ok(())
  }

  fn inner_join(mut self) -> Result<()> {
    let mut scratch = csv::ByteRecord::new();
    let mut validx = ValueIndex::new(self.rdr2, self.sel2, self.nulls)?;
    for row in self.rdr1.byte_records() {
      let row = row?;
      let key = (&self.sel1).get_row_key(&row);
      match validx.values.get(&key) {
        None => continue,
        Some(rows) => {
          for &rowi in rows.iter() {
            validx.idx.seek(rowi as u64)?;

            validx.idx.read_byte_record(&mut scratch)?;
            let combined = row.iter().chain(scratch.iter());
            self.wtr.write_record(combined)?;
          }
        }
      }
    }
    Ok(())
  }

  fn outer_join(mut self, right: bool) -> Result<()> {
    if right {
      swap(&mut self.rdr1, &mut self.rdr2);
      swap(&mut self.sel1, &mut self.sel2);
    }

    let mut scratch = csv::ByteRecord::new();
    let (_, pad2) = self.get_padding()?;
    let mut validx = ValueIndex::new(self.rdr2, self.sel2, self.nulls)?;
    for row in self.rdr1.byte_records() {
      let row = row?;
      let key = (&self.sel1).get_row_key(&row);
      match validx.values.get(&key) {
        None => {
          if right {
            self.wtr.write_record(pad2.iter().chain(&row))?;
          } else {
            self.wtr.write_record(row.iter().chain(&pad2))?;
          }
        }
        Some(rows) => {
          for &rowi in rows.iter() {
            validx.idx.seek(rowi as u64)?;
            let row1 = row.iter();
            validx.idx.read_byte_record(&mut scratch)?;
            if right {
              self.wtr.write_record(scratch.iter().chain(row1))?;
            } else {
              self.wtr.write_record(row1.chain(&scratch))?;
            }
          }
        }
      }
    }
    Ok(())
  }

  fn left_join(mut self, anti: bool) -> Result<()> {
    let validx = ValueIndex::new(self.rdr2, self.sel2, self.nulls)?;
    let mut row = csv::ByteRecord::new();
    let mut key;

    while self.rdr1.read_byte_record(&mut row)? {
      key = (&self.sel1).get_row_key(&row);
      if !validx.values.contains_key(&key) {
        if anti {
          self.wtr.write_record(&row)?;
        }
      } else if !anti {
        self.wtr.write_record(&row)?;
      }
    }
    self.wtr.flush()?;

    Ok(())
  }

  fn full_outer_join(mut self) -> Result<()> {
    let mut scratch = csv::ByteRecord::new();
    let (pad1, pad2) = self.get_padding()?;
    let mut validx = ValueIndex::new(self.rdr2, self.sel2, self.nulls)?;

    // Keep track of which rows we've written from rdr2.
    let mut rdr2_written: Vec<_> = repeat(false).take(validx.num_rows).collect();
    for row1 in self.rdr1.byte_records() {
      let row1 = row1?;
      let key = (&self.sel1).get_row_key(&row1);
      match validx.values.get(&key) {
        None => {
          self.wtr.write_record(row1.iter().chain(&pad2))?;
        }
        Some(rows) => {
          for &rowi in rows.iter() {
            rdr2_written[rowi] = true;

            validx.idx.seek(rowi as u64)?;
            validx.idx.read_byte_record(&mut scratch)?;
            self.wtr.write_record(row1.iter().chain(&scratch))?;
          }
        }
      }
    }

    // OK, now write any row from rdr2 that didn't get joined with a row
    // from rdr1.
    for (i, &written) in rdr2_written.iter().enumerate() {
      if !written {
        validx.idx.seek(i as u64)?;
        validx.idx.read_byte_record(&mut scratch)?;
        self.wtr.write_record(pad1.iter().chain(&scratch))?;
      }
    }
    Ok(())
  }

  fn cross_join(mut self) -> Result<()> {
    let mut pos = csv::Position::new();
    pos.set_byte(0);
    let mut row2 = csv::ByteRecord::new();
    for row1 in self.rdr1.byte_records() {
      let row1 = row1?;
      self.rdr2.seek(pos.clone())?;
      if self.rdr2.has_headers() {
        // Read and skip the header row, since CSV readers disable
        // the header skipping logic after being seeked.
        self.rdr2.read_byte_record(&mut row2)?;
      }
      while self.rdr2.read_byte_record(&mut row2)? {
        self.wtr.write_record(row1.iter().chain(&row2))?;
      }
    }
    Ok(())
  }

  fn get_padding(&mut self) -> Result<(csv::ByteRecord, csv::ByteRecord)> {
    let len1 = self.rdr1.byte_headers()?.len();
    let len2 = self.rdr2.byte_headers()?.len();
    Ok((
      repeat(b"").take(len1).collect(),
      repeat(b"").take(len2).collect(),
    ))
  }
}

fn new_io_state<P: AsRef<Path> + Send + Sync>(
  path1: P,
  path2: P,
  sel1: String,
  sel2: String,
  nulls: bool,
) -> Result<IoState<File, Box<dyn Write + 'static>>> {
  let csv_options1 = CsvOptions::new(&path1);
  let sep1 = csv_options1.detect_separator()?;
  let csv_options2 = CsvOptions::new(&path2);
  let sep2 = csv_options2.detect_separator()?;
  let file_stem = csv_options1.file_stem()?;
  let mut output_path = PathBuf::from(csv_options1.parent_path()?);
  output_path.push(format!("{file_stem}.join.csv"));

  let mut rdr1 = ReaderBuilder::new()
    .delimiter(sep1)
    .from_reader(File::open(&path1)?);
  let mut rdr2 = ReaderBuilder::new()
    .delimiter(sep2)
    .from_reader(File::open(&path2)?);

  let boxed_writer: Box<dyn Write> = Box::new(File::create(output_path)?);

  let wtr = WriterBuilder::new()
    .delimiter(sep1)
    .from_writer(boxed_writer);

  let sel1 = Selection::from_headers(rdr1.byte_headers()?, &[sel1.as_str()][..])?;
  let sel2 = Selection::from_headers(rdr2.byte_headers()?, &[sel2.as_str()][..])?;

  Ok(IoState {
    wtr,
    rdr1: rdr1,
    sel1: sel1,
    rdr2: rdr2,
    sel2: sel2,
    nulls: nulls,
  })
}

struct ValueIndex<R> {
  // This maps tuples of values to corresponding rows.
  values: HashMap<Vec<ByteString>, Vec<usize>>,
  idx: Indexed<R, Cursor<Vec<u8>>>,
  num_rows: usize,
}

impl<R: Read + Seek> ValueIndex<R> {
  fn new(mut rdr: csv::Reader<R>, sel: Selection, nulls: bool) -> Result<ValueIndex<R>> {
    let mut val_idx = HashMap::with_capacity(10000);
    let mut row_idx = Cursor::new(Vec::with_capacity(8 * 10000));
    let (mut rowi, mut count) = (0usize, 0usize);

    // This logic is kind of tricky. Basically, we want to include
    // the header row in the line index (because that's what csv::index
    // does), but we don't want to include header values in the ValueIndex.
    if !rdr.has_headers() {
      // ... so if there are no headers, we seek to the beginning and
      // index everything.
      let mut pos = csv::Position::new();
      pos.set_byte(0);
      rdr.seek(pos)?;
    } else {
      // ... and if there are headers, we make sure that we've parsed
      // them, and write the offset of the header row to the index.
      rdr.byte_headers()?;
      row_idx.write_u64::<BigEndian>(0)?;
      count += 1;
    }

    let mut row = csv::ByteRecord::new();
    while rdr.read_byte_record(&mut row)? {
      // This is a bit hokey. We're doing this manually instead of using
      // the `csv-index` crate directly so that we can create both
      // indexes in one pass.
      row_idx.write_u64::<BigEndian>(row.position().unwrap().byte())?;

      let fields: Vec<_> = sel.get_row_key(&row);
      if nulls || !fields.iter().any(|f| f.is_empty()) {
        match val_idx.entry(fields) {
          Entry::Vacant(v) => {
            let mut rows = Vec::with_capacity(4);
            rows.push(rowi);
            v.insert(rows);
          }
          Entry::Occupied(mut v) => {
            v.get_mut().push(rowi);
          }
        }
      }
      rowi += 1;
      count += 1;
    }

    row_idx.write_u64::<BigEndian>(count as u64)?;
    let idx = Indexed::open(rdr, Cursor::new(row_idx.into_inner()))?;
    Ok(ValueIndex {
      values: val_idx,
      idx: idx,
      num_rows: rowi,
    })
  }
}

impl<R> fmt::Debug for ValueIndex<R> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    // Sort the values by order of first appearance.
    let mut kvs = self.values.iter().collect::<Vec<_>>();
    kvs.sort_by(|&(_, v1), &(_, v2)| v1[0].cmp(&v2[0]));
    for (keys, rows) in kvs.into_iter() {
      // This is just for debugging, so assume Unicode for now.
      let keys = keys
        .iter()
        .map(|k| String::from_utf8(k.to_vec()).unwrap())
        .collect::<Vec<_>>();
      writeln!(f, "({}) => {:?}", keys.join(", "), rows)?
    }
    Ok(())
  }
}

pub async fn run_join<P: AsRef<Path> + Send + Sync>(
  path1: P,
  path2: P,
  sel1: String,
  sel2: String,
  join_type: &str,
  nulls: bool,
) -> Result<()> {
  let mut state = new_io_state(path1, path2, sel1, sel2, nulls)?;
  match join_type {
    "left" => {
      state.write_headers(true)?;
      state.outer_join(false)
    }
    "right" => {
      state.write_headers(true)?;
      state.outer_join(true)
    }
    "full" => {
      state.write_headers(true)?;
      state.full_outer_join()
    }
    "cross" => {
      state.write_headers(true)?;
      state.cross_join()
    }
    "inner" => {
      state.write_headers(true)?;
      state.inner_join()
    }
    "left_semi" => {
      state.write_headers(false)?;
      state.left_join(false)
    }
    "left_anti" => {
      state.write_headers(false)?;
      state.left_join(true)
    }
    "right_semi" => {
      let mut swapped_join = state;
      swap(&mut swapped_join.rdr1, &mut swapped_join.rdr2);
      swap(&mut swapped_join.sel1, &mut swapped_join.sel2);
      swapped_join.write_headers(false)?;
      swapped_join.left_join(false)
    }
    "right_anti" => {
      let mut swapped_join = state;
      swap(&mut swapped_join.rdr1, &mut swapped_join.rdr2);
      swap(&mut swapped_join.sel1, &mut swapped_join.sel2);
      swapped_join.write_headers(false)?;
      swapped_join.left_join(true)
    }
    _ => {
      state.write_headers(true)?;
      state.inner_join()
    }
  }
}

#[tauri::command]
pub async fn join(
  path1: String,
  path2: String,
  sel1: String,
  sel2: String,
  join_type: String,
  nulls: bool,
) -> Result<String, String> {
  let start_time = Instant::now();

  match run_join(path1, path2, sel1, sel2, join_type.as_str(), nulls).await {
    Ok(()) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("{err}")),
  }
}
