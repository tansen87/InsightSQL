use std::fs;

use anyhow::Result;
use tempfile::TempDir;

use lib::replace::public_replace;

#[tokio::test]
async fn test_replace() -> Result<()> {
  let temp_dir = TempDir::new()?;
  let file_path = temp_dir.path().join("input.csv");

  let data = vec![
    "name,age,gender",
    "Tom,18,male",
    "Jerry,19,male",
    "Patrick,4,male",
    "Sandy,24,female",
  ];

  let mut wtr = csv::Writer::from_path(&file_path)?;
  for line in &data {
    wtr.write_record(line.split(',').map(|s| s.as_bytes()))?;
  }
  wtr.flush()?;

  public_replace(
    file_path.to_str().unwrap().to_string(),
    "age".to_string(),
    r"^\d+$".to_string(),
    "XX".to_string(),
  )
  .await?;

  let output_file_name = format!(
    "{}.replace.csv",
    file_path.file_stem().unwrap().to_str().unwrap()
  );
  let output_path = temp_dir.path().join(output_file_name);

  // 读取输出文件内容
  let binding = fs::read_to_string(&output_path)?;
  let output_content = binding.lines().collect::<Vec<_>>();

  // 验证输出文件内容是否正确
  let expected_data = vec![
    "name,age,gender",
    "Tom,XX,male",
    "Jerry,XX,male",
    "Patrick,XX,male",
    "Sandy,XX,female",
  ];

  assert_eq!(output_content, expected_data);

  // 清理临时目录
  drop(file_path);
  temp_dir.close()?;

  Ok(())
}
