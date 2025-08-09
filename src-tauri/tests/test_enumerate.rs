#[tokio::test]
async fn test_enumerate() -> anyhow::Result<()> {
  let temp_dir = tempfile::TempDir::new()?;

  let data = vec![
    "name,age,gender",
    "Jerry,19,male",
    "Patrick,4,male",
    "Sandy,24,female",
  ];
  let file_path = temp_dir.path().join("data.csv");
  let mut wtr = csv::Writer::from_path(&file_path)?;
  for line in &data {
    wtr.write_record(line.split(',').map(|s| s.as_bytes()))?;
  }
  wtr.flush()?;

  lib::cmd::enumerate::enumerate_index(
    file_path.to_str().unwrap(),
    "nil",
    lib::utils::MockEmitter::default(),
  )
  .await?;

  let output_path = temp_dir.path().join(format!(
    "{}.enumer.csv",
    file_path.file_stem().unwrap().to_str().unwrap()
  ));

  let context = std::fs::read_to_string(output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec![
    "enumerate_idx,name,age,gender",
    "1,Jerry,19,male",
    "2,Patrick,4,male",
    "3,Sandy,24,female",
  ];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}
