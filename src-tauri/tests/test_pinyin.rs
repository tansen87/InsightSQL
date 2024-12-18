use std::fs::File;
use std::io::Write;

use anyhow::Result;
use csv::ReaderBuilder;
use tempfile::TempDir;

use lib::pinyin::public_pinyin;

#[tokio::test]
async fn test_pinyin() -> Result<()> {
  let temp_dir = TempDir::new()?;
  let file_path = temp_dir.path().join("data.csv");

  let data = vec![
    "name,age,gender",
    "汤姆,18,male",
    "杰瑞,19,male",
    "Patrick,4,male",
    "Sandy,24,female",
  ];

  let mut file = File::create(&file_path)?;
  for line in data.iter() {
    writeln!(file, "{}", line)?;
  }

  public_pinyin(file_path.to_str().unwrap().to_string(), "name".to_string()).await?;

  let output_path = temp_dir.path().join(format!(
    "{}.pinyin.csv",
    file_path.file_stem().unwrap().to_str().unwrap()
  ));

  let mut rdr = ReaderBuilder::new()
    .has_headers(true)
    .from_path(output_path)?;

  let expected_data = vec![
    vec!["TANGMU", "18", "male"],
    vec!["JIERUI", "19", "male"],
    vec!["Patrick", "4", "male"],
    vec!["Sandy", "24", "female"],
  ];

  for (i, result) in rdr.records().enumerate() {
    let record = result?;
    let fields: Vec<String> = record.iter().map(|s| s.to_string()).collect();
    assert_eq!(fields, expected_data[i], "record {} does not match", i);
  }

  // 清理临时目录
  drop(file_path);
  temp_dir.close()?;

  Ok(())
}
