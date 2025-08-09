#[tokio::test]
async fn test_fill() -> anyhow::Result<()> {
  let temp_dir = tempfile::TempDir::new()?;

  let data = vec![
    "name,age,gender",
    "Jerry,19,",
    "Patrick,4,male",
    "Sandy,24,female",
  ];

  let file_path = temp_dir.path().join("input.csv");
  let mut wtr = csv::Writer::from_path(&file_path)?;
  for line in &data {
    wtr.write_record(line.split(',').map(|s| s.as_bytes()))?;
  }
  wtr.flush()?;

  lib::cmd::fill::fill_null(
    file_path.to_str().unwrap(),
    "gender|age".to_string(),
    "unknown".to_string(),
    "fill",
  )
  .await?;

  let output_path = temp_dir.path().join(format!(
    "{}.fill.csv",
    file_path.file_stem().unwrap().to_str().unwrap()
  ));

  let context = std::fs::read_to_string(output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec![
    "name,age,gender",
    "Jerry,19,unknown",
    "Patrick,4,male",
    "Sandy,24,female",
  ];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}
