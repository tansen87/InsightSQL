use std::fs;

use anyhow::Result;
use tempfile::TempDir;

use lib::slice;

#[tokio::test]
async fn test_slice_column_left_mode() -> Result<()> {
  let data = vec![
    "Patrick,4,male",
    "name,age,gender",
    "汤姆,18,男",
    "杰瑞,19,male",
    "Sandy,24,female",
  ];

  let temp_dir = TempDir::new()?;
  let file_path = temp_dir.path().join("input.csv");

  let mut wtr = csv::Writer::from_path(&file_path)?;
  for line in &data {
    wtr.write_record(line.split(',').map(|s| s.as_bytes()))?;
  }
  wtr.flush()?;

  slice::slice_column_with_n_char(
    file_path.to_str().unwrap().to_string(),
    "1".to_string(),
    "name".to_string(),
    1,
    "left",
  )
  .await?;

  let output_file_name = format!(
    "{}.slice.csv",
    file_path.file_stem().unwrap().to_str().unwrap()
  );
  let output_path = temp_dir.path().join(output_file_name);

  let binding = fs::read_to_string(&output_path)?;
  let slice_data = binding.lines().collect::<Vec<_>>();

  let expected_data = vec![
    "name,age,gender,name_slice-n-char",
    "汤姆,18,男,汤",
    "杰瑞,19,male,杰",
    "Sandy,24,female,S",
  ];

  assert_eq!(slice_data, expected_data);

  drop(file_path);
  temp_dir.close()?;

  Ok(())
}

#[tokio::test]
async fn test_slice_column_right_mode() -> Result<()> {
  let data = vec![
    "Patrick,4,male",
    "name,age,gender",
    "汤姆,18,男",
    "杰瑞,19,male",
    "Sandy,24,female",
  ];

  let temp_dir = TempDir::new()?;
  let file_path = temp_dir.path().join("input.csv");

  let mut wtr = csv::Writer::from_path(&file_path)?;
  for line in &data {
    wtr.write_record(line.split(',').map(|s| s.as_bytes()))?;
  }
  wtr.flush()?;

  slice::slice_column_with_n_char(
    file_path.to_str().unwrap().to_string(),
    "1".to_string(),
    "name".to_string(),
    1,
    "right",
  )
  .await?;

  let output_file_name = format!(
    "{}.slice.csv",
    file_path.file_stem().unwrap().to_str().unwrap()
  );
  let output_path = temp_dir.path().join(output_file_name);

  let binding = fs::read_to_string(&output_path)?;
  let slice_data = binding.lines().collect::<Vec<_>>();

  let expected_data = vec![
    "name,age,gender,name_slice-n-char",
    "汤姆,18,男,姆",
    "杰瑞,19,male,瑞",
    "Sandy,24,female,y",
  ];

  assert_eq!(slice_data, expected_data);

  drop(file_path);
  temp_dir.close()?;

  Ok(())
}
