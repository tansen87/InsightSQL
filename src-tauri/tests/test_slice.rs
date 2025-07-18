use std::{
  fs::{File, read_to_string},
  io::BufWriter,
};

use anyhow::Result;
use csv::{ReaderBuilder, WriterBuilder};
use tempfile::TempDir;

use lib::cmd::string::{slice, split};
use lib::io::csv::options::CsvOptions;

#[tokio::test]
async fn test_slice_left() -> Result<()> {
  let data = vec![
    "Patrick,4,male",
    "name,age,gender",
    "汤姆,18,男",
    "杰瑞,19,male",
    "Sandy,24,female",
  ];

  let temp_dir = TempDir::new()?;
  let file_path = temp_dir.path().join("input.csv");

  let mut wtr = WriterBuilder::new().from_path(&file_path)?;
  for line in &data {
    wtr.write_record(line.split(','))?;
  }
  wtr.flush()?;

  let output_file_name = format!(
    "{}.slice.csv",
    file_path.file_stem().unwrap().to_str().unwrap()
  );
  let output_path = temp_dir.path().join(output_file_name);

  let mut csv_options = CsvOptions::new(file_path);
  csv_options.set_skip_rows(1);
  let rdr = ReaderBuilder::new().from_reader(csv_options.rdr_skip_rows()?);

  let output_file = File::create(&output_path)?;
  let buf_writer = BufWriter::with_capacity(256_000, output_file);
  let wtr = WriterBuilder::new().from_writer(buf_writer);

  slice::slice_nchar(rdr, wtr, "name", 1, false, "left").await?;

  let binding = read_to_string(&output_path)?;
  let slice_data = binding.trim().split('\n').collect::<Vec<_>>();

  let expected_data = vec![
    "name,age,gender,name_nchar",
    "汤姆,18,男,汤",
    "杰瑞,19,male,杰",
    "Sandy,24,female,S",
  ];

  assert_eq!(slice_data, expected_data);

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_slice_right() -> Result<()> {
  let data = vec![
    "Patrick,4,male",
    "name,age,gender",
    "汤姆,18,男",
    "杰瑞,19,male",
    "Sandy,24,female",
  ];

  let temp_dir = TempDir::new()?;
  let file_path = temp_dir.path().join("input.csv");

  let mut wtr = WriterBuilder::new().from_path(&file_path)?;
  for line in &data {
    wtr.write_record(line.split(','))?;
  }
  wtr.flush()?;

  let output_file_name = format!(
    "{}.slice.csv",
    file_path.file_stem().unwrap().to_str().unwrap()
  );
  let output_path = temp_dir.path().join(output_file_name);

  let mut csv_options = CsvOptions::new(file_path);
  csv_options.set_skip_rows(1);
  let rdr = ReaderBuilder::new().from_reader(csv_options.rdr_skip_rows()?);

  let output_file = File::create(&output_path)?;
  let buf_writer = BufWriter::with_capacity(256_000, output_file);
  let wtr = WriterBuilder::new().from_writer(buf_writer);

  slice::slice_nchar(rdr, wtr, "name", 1, false, "right").await?;

  let binding = read_to_string(&output_path)?;
  let slice_data = binding.trim().split('\n').collect::<Vec<_>>();

  let expected_data = vec![
    "name,age,gender,name_nchar",
    "汤姆,18,男,姆",
    "杰瑞,19,male,瑞",
    "Sandy,24,female,y",
  ];

  assert_eq!(slice_data, expected_data);

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_split_nth() -> Result<()> {
  let data = vec![
    "Patrick,4,male",
    "name,age,gender",
    "汤-姆-1,18,男",
    "杰-瑞-2,19,male",
    "Sa-n-dy,24,female",
  ];

  let temp_dir = TempDir::new()?;
  let file_path = temp_dir.path().join("input.csv");

  let mut wtr = WriterBuilder::new().from_path(&file_path)?;
  for line in &data {
    wtr.write_record(line.split(','))?;
  }
  wtr.flush()?;

  let output_file_name = format!(
    "{}.slice.csv",
    file_path.file_stem().unwrap().to_str().unwrap()
  );
  let output_path = temp_dir.path().join(output_file_name);

  let mut csv_options = CsvOptions::new(file_path);
  csv_options.set_skip_rows(1);
  let rdr = ReaderBuilder::new().from_reader(csv_options.rdr_skip_rows()?);

  let output_file = File::create(&output_path)?;
  let buf_writer = BufWriter::with_capacity(256_000, output_file);
  let wtr = WriterBuilder::new().from_writer(buf_writer);

  split::split_nth(rdr, wtr, "name", 2, "-").await?;

  let binding = read_to_string(&output_path)?;
  let slice_data = binding.trim().split('\n').collect::<Vec<_>>();

  let expected_data = vec![
    "name,age,gender,name_nth",
    "汤-姆-1,18,男,姆",
    "杰-瑞-2,19,male,瑞",
    "Sa-n-dy,24,female,n",
  ];

  assert_eq!(slice_data, expected_data);

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_split_nmax() -> Result<()> {
  let data = vec![
    "Patrick,4,male",
    "name,age,gender",
    "汤-姆-1,18,男",
    "杰-瑞-2,19,male",
    "Sa-n-dy,24,female",
  ];

  let temp_dir = TempDir::new()?;
  let file_path = temp_dir.path().join("input.csv");

  let mut wtr = WriterBuilder::new().from_path(&file_path)?;
  for line in &data {
    wtr.write_record(line.split(','))?;
  }
  wtr.flush()?;

  let output_file_name = format!(
    "{}.slice.csv",
    file_path.file_stem().unwrap().to_str().unwrap()
  );
  let output_path = temp_dir.path().join(output_file_name);

  let mut csv_options = CsvOptions::new(file_path);
  csv_options.set_skip_rows(1);
  let rdr = ReaderBuilder::new().from_reader(csv_options.rdr_skip_rows()?);

  let output_file = File::create(&output_path)?;
  let buf_writer = BufWriter::with_capacity(256_000, output_file);
  let wtr = WriterBuilder::new().from_writer(buf_writer);

  split::split_nmax(rdr, wtr, "name", 2, "-").await?;

  let binding = read_to_string(&output_path)?;
  let slice_data = binding.trim().split('\n').collect::<Vec<_>>();

  let expected_data = vec![
    "name,age,gender,name_nmax1,name_nmax2",
    "汤-姆-1,18,男,汤,姆",
    "杰-瑞-2,19,male,杰,瑞",
    "Sa-n-dy,24,female,Sa,n",
  ];

  assert_eq!(slice_data, expected_data);

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_slice_start_length() -> Result<()> {
  let data = vec![
    "Patrick,4,male",
    "name,age,gender",
    "汤-姆-1,18,男",
    "杰-瑞-2,19,male",
    "Sa-n-dy,24,female",
  ];

  let temp_dir = TempDir::new()?;
  let file_path = temp_dir.path().join("input.csv");

  let mut wtr = WriterBuilder::new().from_path(&file_path)?;
  for line in &data {
    wtr.write_record(line.split(','))?;
  }
  wtr.flush()?;

  let output_file_name = format!(
    "{}.slice.csv",
    file_path.file_stem().unwrap().to_str().unwrap()
  );
  let output_path = temp_dir.path().join(output_file_name);

  let mut csv_options = CsvOptions::new(file_path);
  csv_options.set_skip_rows(1);
  let rdr = ReaderBuilder::new().from_reader(csv_options.rdr_skip_rows()?);

  let output_file = File::create(&output_path)?;
  let buf_writer = BufWriter::with_capacity(256_000, output_file);
  let wtr = WriterBuilder::new().from_writer(buf_writer);

  slice::slice_start_length(rdr, wtr, "name", 2, 3, false).await?;

  let binding = read_to_string(&output_path)?;
  let slice_data = binding.trim().split('\n').collect::<Vec<_>>();

  let expected_data = vec![
    "name,age,gender,name_ss",
    "汤-姆-1,18,男,-姆",
    "杰-瑞-2,19,male,-瑞",
    "Sa-n-dy,24,female,a-",
  ];

  assert_eq!(slice_data, expected_data);

  Ok(temp_dir.close()?)
}
