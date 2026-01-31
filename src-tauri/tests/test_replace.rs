#[tokio::test]
async fn test_replace() -> anyhow::Result<()> {
  use std::io::Write;

  let temp_dir = tempfile::TempDir::new()?;

  let data = vec![
    "",
    "name,age,gender",
    "Jerry,19,male",
    "Patrick,4,male",
    "Sandy,24,female",
  ];
  let file_path = temp_dir.path().join("input.csv");
  let mut file = std::fs::File::create(&file_path)?;
  for line in &data {
    writeln!(file, "{}", line)?;
  }

  insight::cmd::replace::regex_replace(
    file_path.to_str().unwrap(),
    "age".to_string(),
    r"^\d+$".to_string(),
    "XX".to_string(),
    true,
    true,
    1,
    false,
    insight::utils::MockEmitter::default(),
  )
  .await?;

  let output_path = temp_dir.path().join(format!(
    "{}_replace.csv",
    file_path.file_stem().unwrap().to_str().unwrap()
  ));

  let context = std::fs::read_to_string(&output_path)?;
  let result = context.lines().collect::<Vec<_>>();
  let expected = vec![
    "name,age,gender",
    "Jerry,XX,male",
    "Patrick,XX,male",
    "Sandy,XX,female",
  ];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}
