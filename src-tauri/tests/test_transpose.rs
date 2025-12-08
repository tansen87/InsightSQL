fn create_temp_csv() -> anyhow::Result<(tempfile::TempDir, String, String)> {
  let temp_dir = tempfile::TempDir::new()?;

  let data = vec![
    "name,age,gender",
    "Tom,18,female",
    "Jerry,19,male",
    "Patrick,4,male",
  ];
  let file_path = temp_dir.path().join("input.csv");
  let mut wtr = csv::Writer::from_path(&file_path)?;
  for line in &data {
    wtr.write_record(line.split(',').map(|s| s.as_bytes()))?;
  }
  wtr.flush()?;

  let path = file_path.to_string_lossy().to_string();
  let output_path = temp_dir
    .path()
    .join(format!(
      "{}.transpose.csv",
      file_path.file_stem().unwrap().to_str().unwrap()
    ))
    .to_string_lossy()
    .to_string();

  Ok((temp_dir, path, output_path))
}

#[tokio::test]
async fn test_memory() -> anyhow::Result<()> {
  let (temp_dir, path, output_path) = create_temp_csv()?;

  insight::cmd::transpose::in_memory_transpose(path).await?;
  let context = std::fs::read_to_string(output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec!["Tom,Jerry,Patrick", "18,19,4", "female,male,male"];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_multipass() -> anyhow::Result<()> {
  let (temp_dir, path, output_path) = create_temp_csv()?;

  insight::cmd::transpose::multipass_transpose(path).await?;
  let context = std::fs::read_to_string(output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec!["Tom,Jerry,Patrick", "18,19,4", "female,male,male"];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}
