#[tokio::test]
async fn test_count() -> anyhow::Result<()> {
  use std::io::Write;

  let temp_dir = tempfile::TempDir::new()?;

  let data = vec![
    "",
    "name,age,gender",
    "Patrick,4,male",
    "汤姆,18,男",
    "杰瑞,19,male",
    "Sandy,24,female",
  ];
  let file_path = temp_dir.path().join("input.csv");
  let mut file = std::fs::File::create(&file_path)?;
  for line in &data {
    writeln!(file, "{}", line)?;
  }

  let row_count = insight::cmd::count::count_rows(file_path.to_str().unwrap(), 1).await?;
  assert_eq!(row_count, 4);

  Ok(temp_dir.close()?)
}
