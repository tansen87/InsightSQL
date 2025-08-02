#[tokio::test]
async fn test_skip() -> anyhow::Result<()> {
  let data = vec![
    "汤姆,18,男",
    "Patrick,4,male",
    "name,age,gender",
    "杰瑞,19,male",
    "Sandy,24,female",
  ];

  let temp_dir = tempfile::TempDir::new()?;
  let file_path = temp_dir.path().join("input.csv");

  let mut wtr = csv::WriterBuilder::new().from_path(&file_path)?;
  for line in &data {
    wtr.write_record(line.split(','))?;
  }
  wtr.flush()?;

  let file_stem = file_path.file_stem().unwrap().to_string_lossy().to_string();
  let file_name = file_path.file_name().unwrap().to_string_lossy().to_string();
  let parent_path = file_path.parent().unwrap().to_str().unwrap();
  let output_path = temp_dir
    .path()
    .join(format!("{parent_path}/{file_stem}.skiprows.csv"));

  lib::cmd::skip::skip_csv(
    file_path.to_str().unwrap(),
    file_name,
    2,
    "nil",
    lib::utils::MockEmitter::default(),
  )
  .await?;

  let binding = std::fs::read_to_string(&output_path)?;
  let skip_data = binding.lines().collect::<Vec<_>>();

  let expected_data = vec!["name,age,gender", "杰瑞,19,male", "Sandy,24,female"];

  assert_eq!(skip_data, expected_data);

  Ok(temp_dir.close()?)
}
