use std::fs::{self, File};
use std::io::Write;

use anyhow::Result;
use tempfile::TempDir;

use lib::fill::public_fill;

#[tokio::test]
async fn test_fill() -> Result<()> {
  let temp_dir = TempDir::new()?;

  let data = vec![
    "name,age,gender",
    "Tom,,",
    "Jerry,19,",
    "Patrick,4,male",
    "Sandy,24,female",
  ];

  let file_path = temp_dir.path().join("data.csv");

  let mut file = File::create(&file_path)?;
  for line in &data {
    writeln!(file, "{}", line)?;
  }

  let fill_column = "gender|age".to_string();
  let fill_value = "unknown".to_string();
  public_fill(
    file_path.to_str().unwrap().to_string(),
    fill_column,
    fill_value,
  )
  .await?;

  let output_path = temp_dir.path().join(format!(
    "{}.fill.csv",
    file_path.file_stem().unwrap().to_str().unwrap()
  ));

  let content = fs::read_to_string(output_path)?;
  let expected_data = vec![
    "name,age,gender",
    "Tom,unknown,unknown",
    "Jerry,19,unknown",
    "Patrick,4,male",
    "Sandy,24,female",
  ]
  .join("\n")
    + "\n";

  assert_eq!(content, expected_data);

  // 清理临时目录
  drop(file_path);
  temp_dir.close()?;

  Ok(())
}
