use std::fs::File;
use std::io::Write;

use anyhow::Result;
use tempfile::TempDir;

use lib::count::public_count;

#[tokio::test]
async fn test_count() -> Result<()> {
  let temp_dir = TempDir::new()?;
  let file_path = temp_dir.path().join("data.csv");
  let data = vec![
    "name,age,gender",
    "Patrick,4,male",
    "汤姆,18,男",
    "杰瑞,19,male",
    "Sandy,24,female",
  ];

  let mut file = File::create(&file_path)?;
  for line in data.iter() {
    writeln!(file, "{}", line)?;
  }

  let row_count = public_count(file_path.to_str().unwrap().to_string()).await?;
  assert_eq!(row_count, 5);

  drop(file_path);
  temp_dir.close()?;

  Ok(())
}
