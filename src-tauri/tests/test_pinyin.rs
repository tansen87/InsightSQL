#[tokio::test]
async fn test_pinyin() -> anyhow::Result<()> {
  let temp_dir = tempfile::TempDir::new()?;

  let data = vec![
    "name,age,gender",
    "汤姆,18,男",
    "杰瑞,19,male",
    "Sandy,24,female",
  ];
  let file_path = temp_dir.path().join("input.csv");
  let mut wtr = csv::Writer::from_path(&file_path)?;
  for line in &data {
    wtr.write_record(line.split(',').map(|s| s.as_bytes()))?;
  }
  wtr.flush()?;

  insight::cmd::pinyin::chinese_to_pinyin(
    file_path.to_str().unwrap(),
    "name|gender".to_string(),
    false,
    "upper",
    true,
    insight::utils::MockEmitter::default(),
  )
  .await?;

  let output_path = temp_dir.path().join(format!(
    "{}.pinyin.csv",
    file_path.file_stem().unwrap().to_str().unwrap()
  ));

  let context = std::fs::read_to_string(output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec![
    "name,age,gender",
    "TANGMU,18,NAN",
    "JIERUI,19,male",
    "Sandy,24,female",
  ];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}
