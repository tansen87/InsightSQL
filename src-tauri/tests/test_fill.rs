use std::fs::{self, File};
use std::io::Write;

use anyhow::Result;
use tempfile::TempDir;

use lib::fill;

#[tokio::test]
async fn test_fill() -> Result<()> {
  let data = vec![
    "Tom,,",
    "name,age,gender",
    "Jerry,19,",
    "Patrick,4,male",
    "Sandy,24,female",
  ];

  let temp_dir = TempDir::new()?;
  let file_path = temp_dir.path().join("data.csv");

  let mut file = File::create(&file_path)?;
  for line in &data {
    writeln!(file, "{}", line)?;
  }

  let fill_column = "gender|age".to_string();
  let fill_value = "unknown".to_string();
  fill::fill_values(
    file_path.to_str().unwrap(),
    fill_column,
    fill_value,
    "1".to_string(),
  )
  .await?;

  let output_path = temp_dir.path().join(format!(
    "{}.fill.csv",
    file_path.file_stem().unwrap().to_str().unwrap()
  ));

  let fill_data = fs::read_to_string(output_path)?;
  let expected_data = vec![
    "name,age,gender",
    "Jerry,19,unknown",
    "Patrick,4,male",
    "Sandy,24,female",
  ]
  .join("\n")
    + "\n";

  assert_eq!(fill_data, expected_data);

  Ok(temp_dir.close()?)
}
