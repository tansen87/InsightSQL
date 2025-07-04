use std::{
  io::{self, BufRead, Write},
  path::Path,
  time::Instant,
};

use anyhow::{Result, anyhow};
use csv::{ReaderBuilder, WriterBuilder};
use ext_sort::{ExternalSorter, ExternalSorterBuilder, LimitedBufferBuilder};

use crate::utils::{self, CsvOptions, Selection};

const RW_BUFFER_CAPACITY: usize = 1_000_000; // 1 MB
const MEMORY_LIMITED_BUFFER: usize = 100 * 1_000_000; // 100 MB

pub async fn sort_csv(
  path: String,
  sel_column: String,
  reverse: bool,
  tmp_dir: &str,
  sorter: &ExternalSorter<String, io::Error, LimitedBufferBuilder>,
) -> Result<()> {
  let csv_options = CsvOptions::new(&path);
  let sep = csv_options.detect_separator()?;

  let mut idxfile = match csv_options.indexed() {
    Ok(idx) => {
      if idx.is_none() {
        return Err(anyhow!("extsort CSV mode requires an index"));
      }
      idx.unwrap()
    }
    _ => {
      return Err(anyhow!("extsort CSV mode requires an index"));
    }
  };

  let mut input_rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(csv_options.skip_csv_rows()?);

  let linewtr_tfile = tempfile::NamedTempFile::new_in(tmp_dir)?;
  let mut line_wtr = io::BufWriter::with_capacity(RW_BUFFER_CAPACITY, linewtr_tfile.as_file());

  let headers = input_rdr.byte_headers()?.clone();
  let sel = Selection::from_headers(&headers, &[sel_column.as_str()][..])?;

  let mut sort_key = String::with_capacity(20);
  let mut utf8_string = String::with_capacity(20);
  let mut curr_row = csv::ByteRecord::new();

  let rowcount = idxfile.count();
  let width = rowcount.to_string().len();

  // first pass. get the selected columns, and the record position
  // then write them to a temp text file with the selected columns and the position
  // separated by "|". Pad the position with leading zeroes, so it will always be the same width
  for row in input_rdr.byte_records() {
    curr_row.clone_from(&row?);
    sort_key.clear();
    for field in sel.select(&curr_row) {
      if let Ok(s_utf8) = simdutf8::basic::from_utf8(field) {
        sort_key.push_str(s_utf8);
      } else {
        utf8_string.clear();
        utf8_string.push_str(&String::from_utf8_lossy(field));
        sort_key.push_str(&utf8_string);
      }
    }
    let idx_position = curr_row.position().unwrap();

    writeln!(line_wtr, "{sort_key}|{:01$}", idx_position.line(), width)?;
  }
  line_wtr.flush()?;

  let line_rdr = io::BufReader::with_capacity(
    RW_BUFFER_CAPACITY,
    std::fs::File::open(linewtr_tfile.path())?,
  );

  let compare = |a: &String, b: &String| {
    if reverse {
      a.cmp(b).reverse()
    } else {
      a.cmp(b)
    }
  };

  // Now sort the temp text file
  let sorted = match sorter.sort_by(line_rdr.lines(), compare) {
    Ok(sorted) => sorted,
    Err(e) => {
      return Err(anyhow!("cannot do external sort: {e:?}"));
    }
  };

  let sorted_tfile = tempfile::NamedTempFile::new_in(tmp_dir)?;
  let mut sorted_line_wtr =
    io::BufWriter::with_capacity(RW_BUFFER_CAPACITY, sorted_tfile.as_file());

  for item in sorted.map(Result::unwrap) {
    sorted_line_wtr.write_all(format!("{item}\n").as_bytes())?;
  }
  sorted_line_wtr.flush()?;
  // Delete the temporary file containing unsorted lines
  drop(line_wtr);
  linewtr_tfile.close()?;

  // now write the sorted CSV file by reading the sorted_line temp file
  // and extracting the position from each line
  // and then using that to seek the input file to retrieve the record
  // and then write the record to the final sorted CSV
  let sorted_lines = std::fs::File::open(sorted_tfile.path())?;
  let sorted_line_rdr = io::BufReader::with_capacity(RW_BUFFER_CAPACITY, sorted_lines);

  let parent_path = Path::new(&path).parent().unwrap().to_str().unwrap();
  let file_stem = Path::new(&path).file_stem().unwrap().to_str().unwrap();
  let output_path = format!("{parent_path}/{file_stem}.extsort.csv");
  let mut sorted_csv_wtr = WriterBuilder::new().delimiter(sep).from_path(output_path)?;

  sorted_csv_wtr.write_byte_record(&headers)?;

  // amortize allocations
  let mut record_wrk = csv::ByteRecord::new();
  let mut line = String::new();

  for l in sorted_line_rdr.lines() {
    line.clone_from(&l?);
    let Ok(position) = atoi_simd::parse::<u64>(&line.as_bytes()[line.len() - width..]) else {
      return Err(anyhow!("Failed to retrieve position: invalid integer"));
    };

    idxfile.seek(position.saturating_sub(1))?;
    idxfile.read_byte_record(&mut record_wrk)?;
    sorted_csv_wtr.write_byte_record(&record_wrk)?;
  }
  sorted_csv_wtr.flush()?;
  drop(sorted_line_wtr);
  sorted_tfile.close()?;

  Ok(())
}

#[tauri::command]
pub async fn extsort(path: String, select_column: String, reverse: bool) -> Result<String, String> {
  let start_time = Instant::now();
  let tmp_dir = "./".to_string();

  let sorter: ExternalSorter<String, io::Error, LimitedBufferBuilder> =
    match ExternalSorterBuilder::new()
      .with_tmp_dir(Path::new(&tmp_dir))
      .with_buffer(LimitedBufferBuilder::new(MEMORY_LIMITED_BUFFER, true))
      .with_rw_buf_size(RW_BUFFER_CAPACITY)
      .with_threads_number(utils::num_cpus())
      .build()
    {
      Ok(sorter) => sorter,
      Err(e) => {
        return Err(format!("cannot create external sorter: {e}"));
      }
    };

  match sort_csv(path, select_column, reverse, &tmp_dir, &sorter).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("{err}")),
  }
}
