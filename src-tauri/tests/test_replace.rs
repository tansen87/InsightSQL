use std::fs;

use anyhow::Result;
use tempfile::TempDir;

use lib::replace;

#[tokio::test]
async fn test_replace() -> Result<()> {
  let temp_dir = TempDir::new()?;
  let file_path = temp_dir.path().join("input.csv");

  let data = vec![
    "Tom,18,male",
    "name,age,gender",
    "Jerry,19,male",
    "Patrick,4,male",
    "Sandy,24,female",
  ];

  let mut wtr = csv::Writer::from_path(&file_path)?;
  for line in &data {
    wtr.write_record(line.split(',').map(|s| s.as_bytes()))?;
  }
  wtr.flush()?;

  replace::regex_replace(
    file_path.to_str().unwrap(),
    "age".to_string(),
    r"^\d+$".to_string(),
    "XX".to_string(),
    "1".to_string(),
  )
  .await?;

  let output_file_name = format!(
    "{}.replace.csv",
    file_path.file_stem().unwrap().to_str().unwrap()
  );
  let output_path = temp_dir.path().join(output_file_name);

  let binding = fs::read_to_string(&output_path)?;
  let replace_data = binding.lines().collect::<Vec<_>>();

  let expected_data = vec![
    "name,age,gender",
    "Jerry,XX,male",
    "Patrick,XX,male",
    "Sandy,XX,female",
  ];

  assert_eq!(replace_data, expected_data);

  Ok(temp_dir.close()?)
}
