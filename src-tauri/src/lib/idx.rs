use std::fs;
use std::fs::File;
use std::io;
use std::path::Path;

use anyhow::Result;
use csv::ReaderBuilder;
use csv_index::RandomAccessSimple;

use crate::utils::CsvOptions;

pub async fn create_index<P: AsRef<Path>>(path: P) -> Result<()> {
  let csv_options = CsvOptions::new(&path);

  let sep = match csv_options.detect_separator() {
    Some(separator) => separator as u8,
    None => b',',
  };

  let file_name = path.as_ref().file_name().unwrap().to_str().unwrap();
  let parent_path = path.as_ref().parent().unwrap().to_str().unwrap();
  let output_path = format!("{parent_path}/{file_name}.idx");

  let mut rdr = ReaderBuilder::new()
    .delimiter(sep)
    .from_reader(File::open(&path)?);
  let mut wtr = io::BufWriter::new(fs::File::create(&output_path)?);
  RandomAccessSimple::create(&mut rdr, &mut wtr)?;

  Ok(())
}
