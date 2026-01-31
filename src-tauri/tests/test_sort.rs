#[tokio::test]
async fn test_sort() -> anyhow::Result<()> {
  use std::io::Write;

  let data = vec!["", "index,age,name", "1,18,AC", "3,19,AD", "2,24,AA"];

  let temp_dir = tempfile::TempDir::new()?;
  let file_path = temp_dir.path().join("input.csv");
  let mut file = std::fs::File::create(&file_path)?;
  for line in &data {
    writeln!(file, "{}", line)?;
  }
  let parent_path = file_path.parent().unwrap().to_str().unwrap();
  let file_name = file_path.file_name().unwrap().to_str().unwrap();
  let file_stem = file_path.file_stem().unwrap().to_str().unwrap();
  let path = format!("{parent_path}/{file_name}");
  let path1 = format!("{parent_path}/{file_stem}");

  let test_cases = vec![
    (false, false, "name", vec!["2,24,AA", "1,18,AC", "3,19,AD"]), // 非数字升序
    (true, false, "age", vec!["1,18,AC", "3,19,AD", "2,24,AA"]),   // 数字升序
    (false, true, "name", vec!["3,19,AD", "1,18,AC", "2,24,AA"]),  // 非数字降序
    (true, true, "age", vec!["2,24,AA", "3,19,AD", "1,18,AC"]),    // 数字降序
  ];

  for (numeric, reverse, select_column, expected) in test_cases {
    insight::cmd::sort::sort_csv(
      &path,
      select_column.to_string(),
      numeric,
      reverse,
      true,
      false,
      1,
    )
    .await?;

    let output_path = format!("{}_sort.csv", path1);
    let mut rdr = csv::ReaderBuilder::new().from_path(output_path)?;

    let result: Vec<String> = rdr
      .byte_records()
      .map(|r| {
        r.unwrap()
          .iter()
          .map(|s| String::from_utf8(s.to_vec()).unwrap())
          .collect::<csv::StringRecord>()
          .iter()
          .collect::<Vec<_>>()
          .join(",")
      })
      .collect();

    assert_eq!(result, expected);
  }

  Ok(temp_dir.close()?)
}
