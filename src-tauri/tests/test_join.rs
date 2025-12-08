fn create_temp_csv() -> anyhow::Result<(tempfile::TempDir, String, String, String)> {
  let temp_dir = tempfile::TempDir::new()?;

  let data1 = vec!["idx,name", "1,Tom", "2,Jerry", "3,Patrick"];
  let file_path1 = temp_dir.path().join("input1.csv");
  let mut wtr1 = csv::Writer::from_path(&file_path1)?;
  for line in &data1 {
    wtr1.write_record(line.split(',').map(|s| s.as_bytes()))?;
  }
  wtr1.flush()?;

  let data2 = vec!["idx,age", "1,18", "3,20", "4,19"];
  let file_path2 = temp_dir.path().join("input2.csv");
  let mut wtr2 = csv::Writer::from_path(&file_path2)?;
  for line in &data2 {
    wtr2.write_record(line.split(',').map(|s| s.as_bytes()))?;
  }
  wtr2.flush()?;

  let p1 = file_path1.to_string_lossy().to_string();
  let p2 = file_path2.to_string_lossy().to_string();
  let output_path = temp_dir
    .path()
    .join(format!(
      "{}.join.csv",
      file_path1.file_stem().unwrap().to_str().unwrap()
    ))
    .to_string_lossy()
    .to_string();

  Ok((temp_dir, p1, p2, output_path))
}

#[tokio::test]
async fn test_left_outer_join() -> anyhow::Result<()> {
  let (temp_dir, path1, path2, output_path) = create_temp_csv()?;
  let sel1 = "idx".to_string();
  let sel2 = "idx".to_string();

  insight::cmd::join::run_join(path1, path2, sel1, sel2, "left", false).await?;
  let context = std::fs::read_to_string(&output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec![
    "idx,name,idx,age",
    "1,Tom,1,18",
    "2,Jerry,,",
    "3,Patrick,3,20",
  ];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_right_outer_join() -> anyhow::Result<()> {
  let (temp_dir, path1, path2, output_path) = create_temp_csv()?;
  let sel1 = "idx".to_string();
  let sel2 = "idx".to_string();

  insight::cmd::join::run_join(path1, path2, sel1, sel2, "right", false).await?;
  let context = std::fs::read_to_string(&output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec!["idx,name,idx,age", "1,Tom,1,18", "3,Patrick,3,20", ",,4,19"];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_full_outer_join() -> anyhow::Result<()> {
  let (temp_dir, path1, path2, output_path) = create_temp_csv()?;
  let sel1 = "idx".to_string();
  let sel2 = "idx".to_string();

  insight::cmd::join::run_join(path1, path2, sel1, sel2, "full", false).await?;
  let context = std::fs::read_to_string(&output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec![
    "idx,name,idx,age",
    "1,Tom,1,18",
    "2,Jerry,,",
    "3,Patrick,3,20",
    ",,4,19",
  ];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_cross_join() -> anyhow::Result<()> {
  let (temp_dir, path1, path2, output_path) = create_temp_csv()?;
  let sel1 = "idx".to_string();
  let sel2 = "idx".to_string();

  insight::cmd::join::run_join(path1, path2, sel1, sel2, "cross", false).await?;
  let context = std::fs::read_to_string(&output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec![
    "idx,name,idx,age",
    "1,Tom,1,18",
    "1,Tom,3,20",
    "1,Tom,4,19",
    "2,Jerry,1,18",
    "2,Jerry,3,20",
    "2,Jerry,4,19",
    "3,Patrick,1,18",
    "3,Patrick,3,20",
    "3,Patrick,4,19",
  ];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_inner_join() -> anyhow::Result<()> {
  let (temp_dir, path1, path2, output_path) = create_temp_csv()?;
  let sel1 = "idx".to_string();
  let sel2 = "idx".to_string();

  insight::cmd::join::run_join(path1, path2, sel1, sel2, "inner", false).await?;
  let context = std::fs::read_to_string(&output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec!["idx,name,idx,age", "1,Tom,1,18", "3,Patrick,3,20"];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_left_semi_join() -> anyhow::Result<()> {
  let (temp_dir, path1, path2, output_path) = create_temp_csv()?;
  let sel1 = "idx".to_string();
  let sel2 = "idx".to_string();

  insight::cmd::join::run_join(path1, path2, sel1, sel2, "left_semi", false).await?;
  let context = std::fs::read_to_string(&output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec!["idx,name", "1,Tom", "3,Patrick"];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_left_anti_join() -> anyhow::Result<()> {
  let (temp_dir, path1, path2, output_path) = create_temp_csv()?;
  let sel1 = "idx".to_string();
  let sel2 = "idx".to_string();

  insight::cmd::join::run_join(path1, path2, sel1, sel2, "left_anti", false).await?;
  let context = std::fs::read_to_string(&output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec!["idx,name", "2,Jerry"];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_right_semi_join() -> anyhow::Result<()> {
  let (temp_dir, path1, path2, output_path) = create_temp_csv()?;
  let sel1 = "idx".to_string();
  let sel2 = "idx".to_string();

  insight::cmd::join::run_join(path1, path2, sel1, sel2, "right_semi", false).await?;
  let context = std::fs::read_to_string(&output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec!["idx,age", "1,18", "3,20"];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_right_anti_join() -> anyhow::Result<()> {
  let (temp_dir, path1, path2, output_path) = create_temp_csv()?;
  let sel1 = "idx".to_string();
  let sel2 = "idx".to_string();

  insight::cmd::join::run_join(path1, path2, sel1, sel2, "right_anti", false).await?;
  let context = std::fs::read_to_string(&output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec!["idx,age", "4,19"];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}
