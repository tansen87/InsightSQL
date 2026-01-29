#[tokio::test]
async fn test_cat_csv() -> anyhow::Result<()> {
  let temp_dir = tempfile::TempDir::new()?;

  let data1 = vec!["Tom,1", "name,age", "Tom,18", "Jerry,19"];
  let file_path1 = temp_dir.path().join("input1.csv");
  let mut wtr1 = csv::Writer::from_path(&file_path1)?;
  for line in &data1 {
    wtr1.write_record(line.split(',').map(|s| s.as_bytes()))?;
  }
  wtr1.flush()?;

  let data2 = vec!["Tom,1", "idx,name", "1,Sandy", "2,Patrick"];
  let file_path2 = temp_dir.path().join("input2.csv");
  let mut wtr2 = csv::Writer::from_path(&file_path2)?;
  for line in &data2 {
    wtr2.write_record(line.split(',').map(|s| s.as_bytes()))?;
  }
  wtr2.flush()?;

  let output_path = temp_dir.path().join(format!(
    "{}.cat.csv",
    file_path1.file_stem().unwrap().to_str().unwrap()
  ));
  let p1 = file_path1.to_string_lossy().to_string();
  let p2 = file_path2.to_string_lossy().to_string();
  insight::cmd::cat::cat_with_csv(
    format!("{p1}|{p2}"),
    output_path.to_string_lossy().to_string(),
    true,
    1
  )
  .await?;
  let binding = std::fs::read_to_string(&output_path)?;
  let result = binding.trim().split('\n').collect::<Vec<_>>();
  let expected = vec![
    "name,age,idx",
    "Tom,18,",
    "Jerry,19,",
    "Sandy,,1",
    "Patrick,,2",
  ];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}
