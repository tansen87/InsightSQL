use std::fs::{self, File};
use std::io::Write;

use anyhow::Result;
use tempfile::TempDir;

use lib::rename::public_rename;

#[tokio::test]
async fn test_rename() -> Result<()> {
  let temp_dir = TempDir::new()?;
  let file_path = temp_dir.path().join("data.csv");

  let data = vec![
    "name,age,gender",
    "Tom,18,male",
    "Jerry,19,male",
    "Patrick,4,male",
    "Sandy,24,female",
  ];

  let mut file = File::create(&file_path)?;
  for line in &data {
    writeln!(file, "{}", line)?;
  }

  // 定义新表头
  let new_header = "first_name,years_old,sex";

  public_rename(file_path.to_str().unwrap(), new_header.to_string()).await?;

  let output_path = temp_dir.path().join(format!(
    "{}.rename.csv",
    file_path.file_stem().unwrap().to_str().unwrap()
  ));

  let content = fs::read_to_string(output_path)?;
  let expected_content = vec![
    "first_name,years_old,sex",
    "Tom,18,male",
    "Jerry,19,male",
    "Patrick,4,male",
    "Sandy,24,female",
  ]
  .join("\n")
    + "\n";

  assert_eq!(content, expected_content);

  // 清理临时目录
  drop(file_path);
  temp_dir.close()?;

  Ok(())
}
