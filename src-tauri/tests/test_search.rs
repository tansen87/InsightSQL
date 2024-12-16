use anyhow::Result;
use tempfile::TempDir;

use lib::search::{
  public_contains_search, public_equal_search, public_regex_search, public_startswith_search,
};

fn create_temp_csv() -> Result<(TempDir, String)> {
  let temp_dir = TempDir::new()?;
  let data = vec![
    "name,age,gender",
    "Tom,18,male",
    "Jerry,19,male",
    "Patrick,4,male",
    "Sandy,24,female",
  ];
  let file_path = temp_dir.path().join("input.csv");
  let mut wtr = csv::Writer::from_path(&file_path)?;
  for line in &data {
    wtr.write_record(line.split(',').map(|s| s.as_bytes()))?;
  }
  wtr.flush()?;

  Ok((temp_dir, file_path.to_str().unwrap().to_string()))
}

#[tokio::test]
async fn test_equal_search() -> Result<()> {
  let (temp_dir, file_path) = create_temp_csv()?;
  let sep = b',';
  let column = "name".to_string();
  let conditions = vec!["Tom".to_string()];
  let output_path = temp_dir
    .path()
    .join("input.search.csv")
    .to_str()
    .unwrap()
    .to_string();

  let result = public_equal_search(file_path, sep, column, conditions, output_path).await?;

  // Expect 1 row matched, matched ("Tom")
  assert_eq!(result, "1");

  Ok(())
}

#[tokio::test]
async fn test_contains_search() -> Result<()> {
  let (temp_dir, file_path) = create_temp_csv()?;
  let sep = b',';
  let column = "name".to_string();
  let conditions = vec!["a".to_string()];
  let output_path = temp_dir
    .path()
    .join("input.search.csv")
    .to_str()
    .unwrap()
    .to_string();

  let result = public_contains_search(file_path, sep, column, conditions, output_path).await?;

  // Expect 2 rows matched, matched ("Patrick", "Sandy")
  assert_eq!(result, "2");

  Ok(())
}

#[tokio::test]
async fn test_startswith_search() -> Result<()> {
  let (temp_dir, file_path) = create_temp_csv()?;
  let sep = b',';
  let column = "name".to_string();
  let conditions = vec!["J".to_string()];
  let output_path = temp_dir
    .path()
    .join("input.search.csv")
    .to_str()
    .unwrap()
    .to_string();

  let result = public_startswith_search(file_path, sep, column, conditions, output_path).await?;

  // Expect 1 row matched, matched ("Jerry")
  assert_eq!(result, "1");

  Ok(())
}

#[tokio::test]
async fn test_regex_search() -> Result<()> {
  let (temp_dir, file_path) = create_temp_csv()?;
  let sep = b',';
  let column = "name".to_string();
  let regex_char = r"^J.*".to_string(); // Matches any string that starts with 'J'
  let output_path = temp_dir
    .path()
    .join("input.search.csv")
    .to_str()
    .unwrap()
    .to_string();

  let result = public_regex_search(file_path, sep, column, regex_char, output_path).await?;

  // Expect 1 row matched, matched ("Jerry")
  assert_eq!(result, "1");

  Ok(())
}
