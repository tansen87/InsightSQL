use std::fs::{self, File};
use std::io::Write;

use anyhow::Result;
use tempfile::TempDir;

use lib::command::enumerate;

#[tokio::test]
async fn test_enumerate() -> Result<()> {
  let data = vec![
    "name,age,gender",
    "Jerry,19,male",
    "Patrick,4,male",
    "Sandy,24,female",
  ];

  let temp_dir = TempDir::new()?;
  let file_path = temp_dir.path().join("data.csv");

  let mut file = File::create(&file_path)?;
  for line in data.iter() {
    writeln!(file, "{}", line)?;
  }

  enumerate::add_index(file_path.to_str().unwrap()).await?;

  let output_path = temp_dir.path().join(format!(
    "{}.enumerate.csv",
    file_path.file_stem().unwrap().to_str().unwrap()
  ));

  let enumerate_data = fs::read_to_string(output_path)?;
  let expected_data = vec![
    "enumerate_idx,name,age,gender",
    "0,Jerry,19,male",
    "1,Patrick,4,male",
    "2,Sandy,24,female",
  ]
  .join("\n")
    + "\n";

  assert_eq!(enumerate_data, expected_data);

  Ok(temp_dir.close()?)
}
