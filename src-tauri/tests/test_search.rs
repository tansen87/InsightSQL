use anyhow::Result;
use tempfile::TempDir;

use lib::search;

fn create_temp_csv() -> Result<(TempDir, String)> {
  let data = vec![
    "name,age,gender",
    "Tom,18,male",
    "Jerry,19,male",
    "Patrick,4,male",
    "Sandy,24,female",
  ];

  let temp_dir = TempDir::new()?;
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

  let result = search::equal_search(file_path, sep, column, conditions, output_path).await?;

  // Expect 1 row matched, matched ("Tom")
  assert_eq!(result, "1");

  Ok(temp_dir.close()?)
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

  let result = search::contains_search(file_path, sep, column, conditions, output_path).await?;

  // Expect 2 rows matched, matched ("Patrick", "Sandy")
  assert_eq!(result, "2");

  Ok(temp_dir.close()?)
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

  let result = search::startswith_search(file_path, sep, column, conditions, output_path).await?;

  // Expect 1 row matched, matched ("Jerry")
  assert_eq!(result, "1");

  Ok(temp_dir.close()?)
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

  let result = search::regex_search(file_path, sep, column, regex_char, output_path).await?;

  // Expect 1 row matched, matched ("Jerry")
  assert_eq!(result, "1");

  Ok(temp_dir.close()?)
}
