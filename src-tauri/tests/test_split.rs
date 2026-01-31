use std::io::{BufRead, Write};

#[tokio::test]
async fn test_split_rows() -> anyhow::Result<()> {
  let temp_dir = tempfile::TempDir::new()?;

  let data = vec![
    "name,age,gender",
    "Tom,18,male",
    "Jerry,19,male",
    "Patrick,4,male",
    "Sandy,24,female",
  ];
  let file_path = temp_dir.path().join("input.csv");
  let mut wtr = csv::Writer::from_path(&file_path)?;
  for line in &data {
    wtr.write_record(line.split(',').map(|s| s.as_bytes()))?;
  }
  wtr.flush()?;

  let size: usize = 2;

  let csv_options = insight::io::csv::options::CsvOptions::new(file_path.to_str().unwrap());
  let parent_path = file_path.parent().unwrap().to_str().unwrap();
  let file_stem = file_path.file_stem().unwrap().to_str().unwrap();
  let output_path = format!("{parent_path}/{file_stem}");

  insight::cmd::split::sequential_split_rows(csv_options, size.try_into()?, &output_path).await?;

  let output_files: Vec<_> = std::fs::read_dir(temp_dir.path())?
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
    let file = std::fs::File::open(entry.path())?;
    let reader = std::io::BufReader::new(file);
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
async fn test_split_lines() -> anyhow::Result<()> {
  let temp_dir = tempfile::TempDir::new()?;

  let data = vec![
    "name,age,gender",
    "Tom,18",
    "Jerry,19,male",
    "Patrick,4,male",
    "Sandy,24,female",
  ];
  let file_path = temp_dir.path().join("input.csv");

  let mut wtr = std::io::BufWriter::new(std::fs::File::create(&file_path)?);
  for line in data.iter() {
    writeln!(wtr, "{}", line.to_string())?;
  }
  wtr.flush()?;

  let parent_path = file_path.parent().unwrap().to_str().unwrap();
  let file_stem = file_path.file_stem().unwrap().to_str().unwrap();
  let output_path = format!("{parent_path}/{file_stem}");

  insight::cmd::split::split_lines(file_path.to_string_lossy().to_string(), 2, &output_path)
    .await?;

  let output_files: Vec<_> = std::fs::read_dir(temp_dir.path())?
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
