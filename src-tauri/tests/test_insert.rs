use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

/// 创建一个标准临时 CSV 文件用于测试 insert_columns。
///
/// 内容固定为：
/// - 空第一行（skiprows=1 时会被跳过）
/// - header: name,age,gender
/// - records: Tom, Jerry, Patrick, Sandy
///
/// Returns: (temp_dir, input_file_path)
fn create_temp_csv() -> anyhow::Result<(TempDir, PathBuf)> {
  let data = vec![
    "", // empty line to be skipped
    "name,age,gender",
    "Tom,18,male",
    "Jerry,19,male",
    "Patrick,4,male",
    "Sandy,24,female",
  ];

  let temp_dir = TempDir::new()?;
  let file_path = temp_dir.path().join("input.csv");
  let mut file = std::fs::File::create(&file_path)?;
  for line in &data {
    writeln!(file, "{}", line)?;
  }

  Ok((temp_dir, file_path))
}

#[tokio::test]
async fn test_insert_before_and_left() -> anyhow::Result<()> {
  let (temp_dir, file_path) = create_temp_csv()?;

  insight::cmd::insert::insert_columns(
    file_path.to_str().unwrap(),
    "age".to_string(),
    "before|left".to_string(),
    "X|Y".to_string(),
    1,
    false,
    false,
    false,
    insight::utils::MockEmitter::default(),
  )
  .await?;

  let output_path = temp_dir.path().join("input_insert.csv");
  let content = std::fs::read_to_string(output_path)?;
  let lines: Vec<&str> = content.trim().split('\n').collect();

  // Header: name,X,Y,age,gender
  // Data: Tom,X,Y,18,male
  assert_eq!(lines[0], "name,Y,X,age,gender");
  assert_eq!(lines[1], "Tom,Y,X,18,male");

  Ok(())
}

#[tokio::test]
async fn test_insert_after_and_right() -> anyhow::Result<()> {
  let (temp_dir, file_path) = create_temp_csv()?;

  insight::cmd::insert::insert_columns(
    file_path.to_str().unwrap(),
    "age".to_string(),
    "after|right".to_string(),
    "A|B".to_string(),
    1,
    false,
    false,
    false,
    insight::utils::MockEmitter::default(),
  )
  .await?;

  let output_path = temp_dir.path().join("input_insert.csv");
  let content = std::fs::read_to_string(output_path)?;
  let lines: Vec<&str> = content.trim().split('\n').collect();

  assert_eq!(lines[0], "name,age,B,A,gender");
  assert_eq!(lines[1], "Tom,18,B,A,male");

  Ok(())
}

#[tokio::test]
async fn test_insert_1_based_indices() -> anyhow::Result<()> {
  let (temp_dir, file_path) = create_temp_csv()?;

  // Insert at position 1 (before 'name'), 2 (between name/age), 4 (end)
  insight::cmd::insert::insert_columns(
    file_path.to_str().unwrap(),
    "".to_string(),
    "1|2|4".to_string(),
    "P1|P2|P4".to_string(),
    1,
    false,
    false,
    false,
    insight::utils::MockEmitter::default(),
  )
  .await?;

  let output_path = temp_dir.path().join("input_insert.csv");
  let content = std::fs::read_to_string(output_path)?;
  let lines: Vec<&str> = content.trim().split('\n').collect();

  // Original after skip: [name, age, gender] → n=3
  // 1 → idx=0, 2 → idx=1, 4 → idx=3 (end)
  // Result: P1, name, P2, age, gender, P4
  assert_eq!(lines[0], "P1,name,P2,age,gender,P4");
  assert_eq!(lines[1], "P1,Tom,P2,18,male,P4");

  Ok(())
}

#[tokio::test]
async fn test_insert_negative_indices() -> anyhow::Result<()> {
  let (temp_dir, file_path) = create_temp_csv()?;

  insight::cmd::insert::insert_columns(
    file_path.to_str().unwrap(),
    "".to_string(),
    "-1|-2".to_string(),
    "END|MID".to_string(),
    1,
    false,
    false,
    false,
    insight::utils::MockEmitter::default(),
  )
  .await?;

  let output_path = temp_dir.path().join("input_insert.csv");
  let content = std::fs::read_to_string(output_path)?;
  let lines: Vec<&str> = content.trim().split('\n').collect();

  // n=3 → -1 → 3 (end), -2 → 2 (before 'gender')
  // Result: name, age, MID, gender, END
  assert_eq!(lines[0], "name,age,MID,gender,END");
  assert_eq!(lines[1], "Tom,18,MID,male,END");

  Ok(())
}

#[tokio::test]
async fn test_insert_empty_values() -> anyhow::Result<()> {
  let (temp_dir, file_path) = create_temp_csv()?;

  insight::cmd::insert::insert_columns(
    file_path.to_str().unwrap(),
    "gender".to_string(),
    "before|after".to_string(),
    "|EMPTY".to_string(),
    1,
    false,
    false,
    false,
    insight::utils::MockEmitter::default(),
  )
  .await?;

  let output_path = temp_dir.path().join("input_insert.csv");
  let content = std::fs::read_to_string(output_path)?;
  let lines: Vec<&str> = content.trim().split('\n').collect();

  assert_eq!(lines[0], "name,age,,gender,EMPTY");
  assert_eq!(lines[1], "Tom,18,,male,EMPTY");

  Ok(())
}

#[tokio::test]
async fn test_insert_numeric_without_sel() -> anyhow::Result<()> {
  let (temp_dir, file_path) = create_temp_csv()?;

  insight::cmd::insert::insert_columns(
    file_path.to_str().unwrap(),
    "".to_string(),
    "3".to_string(), // before 'gender'
    "TAG".to_string(),
    1,
    false,
    false,
    false,
    insight::utils::MockEmitter::default(),
  )
  .await?;

  let output_path = temp_dir.path().join("input_insert.csv");
  let content = std::fs::read_to_string(output_path)?;
  let lines: Vec<&str> = content.trim().split('\n').collect();

  assert_eq!(lines[0], "name,age,TAG,gender");
  assert_eq!(lines[1], "Tom,18,TAG,male");

  Ok(())
}

#[tokio::test]
async fn test_insert_relative_without_sel_should_fail() -> anyhow::Result<()> {
  let (temp_dir, file_path) = create_temp_csv()?;

  let result = insight::cmd::insert::insert_columns(
    file_path.to_str().unwrap(),
    "".to_string(),
    "before".to_string(),
    "X".to_string(),
    1,
    false,
    false,
    false,
    insight::utils::MockEmitter::default(),
  )
  .await;

  assert!(result.is_err());
  assert!(
    result
      .unwrap_err()
      .to_string()
      .contains("'column' must be specified")
  );

  drop(temp_dir); // explicitly keep alive until here
  Ok(())
}
