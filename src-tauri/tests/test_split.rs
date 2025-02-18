use std::{
  fs::{self, File},
  io::{BufRead, BufReader, BufWriter, Write},
};

use anyhow::Result;
use tempfile::TempDir;

use lib::split;

#[tokio::test]
async fn test_split_rows() -> Result<()> {
  let data = vec![
    "name,age,gender",
    "Tom,18,male",
    "Jerry,19,male",
    "Patrick,4,male",
    "Sandy,24,female",
  ];

  let temp_dir = TempDir::new()?;
  let file_path = temp_dir.path().join("input.csv");

  let mut wtr = csv::Writer::from_path(&file_path)?;
  for line in &data {
    wtr.write_record(line.split(',').map(|s| s.as_bytes()))?;
  }
  wtr.flush()?;

  let size: usize = 2;

  split::split_rows(file_path.to_str().unwrap(), size.try_into()?, 0).await?;

  let output_files: Vec<_> = fs::read_dir(temp_dir.path())?
    .filter_map(Result::ok)
    .filter(|entry| entry.path().is_file())
    .filter(|entry| {
      entry
        .file_name()
        .to_string_lossy()
        .starts_with("input.split")
    })
    .collect();

  assert_eq!(output_files.len(), 2);

  for (i, entry) in output_files.iter().enumerate() {
    let file = File::open(entry.path())?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;

    let expected_headers = "name,age,gender\n";
    assert_eq!(lines[0], expected_headers.trim_end());

    for (j, line) in lines.iter().enumerate().skip(1) {
      let data_index = i * size + j - 1;
      if data_index < data.len() - 1 {
        assert_eq!(*line, data[data_index + 1]);
      }
    }
  }

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_split_lines() -> Result<()> {
  let data = vec![
    "name,age,gender",
    "Tom,18",
    "Jerry,19,male",
    "Patrick,4,male",
    "Sandy,24,female",
  ];

  let temp_dir = TempDir::new()?;
  let file_path = temp_dir.path().join("input.csv");

  let mut wtr = BufWriter::new(File::create(&file_path)?);
  for line in data.iter() {
    writeln!(wtr, "{}", line.to_string())?;
  }
  wtr.flush()?;

  split::split_lines(file_path.to_str().unwrap(), 2, 0).await?;

  let output_files: Vec<_> = fs::read_dir(temp_dir.path())?
    .filter_map(Result::ok)
    .filter(|entry| entry.path().is_file())
    .filter(|entry| {
      entry
        .file_name()
        .to_string_lossy()
        .starts_with("input.split")
    })
    .collect();

  assert_eq!(output_files.len(), 2);

  Ok(temp_dir.close()?)
}
