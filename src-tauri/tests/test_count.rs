use std::fs::File;
use std::io::Write;

use anyhow::Result;
use tempfile::TempDir;

use lib::count;

#[tokio::test]
async fn test_count() -> Result<()> {
  let data = vec![
    "name,age,gender",
    "Patrick,4,male",
    "汤姆,18,男",
    "杰瑞,19,male",
    "Sandy,24,female",
  ];

  let temp_dir = TempDir::new()?;
  let file_path = temp_dir.path().join("data.csv");

  let mut file = File::create(&file_path)?;
  for line in data.iter() {
    writeln!(file, "{}", line)?;
  }

  let row_count = count::count_rows(file_path.to_str().unwrap()).await?;

  assert_eq!(row_count, 5);

  Ok(temp_dir.close()?)
}
