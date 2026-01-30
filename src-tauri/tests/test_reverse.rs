#[tokio::test]
async fn test_reverse() -> anyhow::Result<()> {
  use std::io::Write;

  let temp_dir = tempfile::TempDir::new()?;

  let data = vec![
    "",
    "name,age,gender",
    "Tom,18,male",
    "Jerry,19,male",
    "Patrick,4,male",
  ];
  let file_path = temp_dir.path().join("input.csv");
  let mut file = std::fs::File::create(&file_path)?;
  for line in &data {
    writeln!(file, "{}", line)?;
  }

  let output_path = temp_dir.path().join(format!(
    "{}_reverse.csv",
    file_path.file_stem().unwrap().to_str().unwrap()
  ));

  insight::cmd::reverse::reverse_csv(file_path.to_string_lossy().to_string(), true, 0).await?;
  let context = std::fs::read_to_string(output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec![
    "name,age,gender",
    "Patrick,4,male",
    "Jerry,19,male",
    "Tom,18,male",
  ];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}
