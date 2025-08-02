use std::io::Write;

#[tokio::test]
async fn test_rename() -> anyhow::Result<()> {
  let temp_dir = tempfile::TempDir::new()?;
  let file_path = temp_dir.path().join("data.csv");

  let data = vec![
    "name,age,gender",
    "Jerry,19,male",
    "Patrick,4,male",
    "Sandy,24,female",
  ];

  let mut file = std::fs::File::create(&file_path)?;
  for line in &data {
    writeln!(file, "{}", line)?;
  }

  let new_header = "first_name,years_old,sex";

  lib::cmd::rename::rename_headers(
    file_path.to_str().unwrap(),
    new_header.to_string(),
    "idx",
    lib::utils::MockEmitter::default(),
  )
  .await?;

  let output_path = temp_dir.path().join(format!(
    "{}.rename.csv",
    file_path.file_stem().unwrap().to_str().unwrap()
  ));

  let content = std::fs::read_to_string(output_path)?;
  let expected_content = vec![
    "first_name,years_old,sex",
    "Jerry,19,male",
    "Patrick,4,male",
    "Sandy,24,female",
  ]
  .join("\n")
    + "\n";

  assert_eq!(content, expected_content);

  Ok(temp_dir.close()?)
}
