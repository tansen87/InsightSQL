use std::fs::{self};

use anyhow::Result;
use tempfile::TempDir;

use lib::select::public_select;

#[tokio::test]
async fn test_select() -> Result<()> {
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

  let cols = "name|age".to_string();

  public_select(
    file_path.to_str().unwrap().to_string(),
    cols,
    "0".to_string(),
  )
  .await?;

  // 检查输出文件是否正确生成
  let output_files: Vec<_> = fs::read_dir(temp_dir.path())?
    .filter_map(Result::ok)
    .filter(|entry| entry.path().is_file())
    .filter(|entry| {
      entry
        .file_name()
        .to_string_lossy()
        .starts_with("input.select")
    })
    .collect();

  // 检查文件数量是否正确
  assert_eq!(output_files.len(), 1);

  // 找到输出文件
  let output_file_path = output_files
    .into_iter()
    .map(|entry| entry.path())
    .find(|path| {
      path
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .starts_with("input.select")
    })
    .unwrap();

  let mut rdr = csv::ReaderBuilder::new()
    .delimiter(b',')
    .has_headers(true)
    .from_path(output_file_path)?;

  // 验证header信息
  let headers = rdr.headers()?;
  assert_eq!(headers.len(), 2);
  assert!(headers.iter().any(|h| h == "name"));
  assert!(headers.iter().any(|h| h == "age"));

  // 验证行数据
  let expected_records = vec![
    vec!["Tom", "18"],
    vec!["Jerry", "19"],
    vec!["Patrick", "4"],
    vec!["Sandy", "24"],
  ];

  for (i, result) in rdr.byte_records().enumerate() {
    let record = result?;
    let expected_record = &expected_records[i];
    for (j, field) in record.iter().enumerate() {
      assert_eq!(
        std::str::from_utf8(field)?,
        expected_record[j],
        "第 {} 行, 第 {} 列不匹配",
        i + 1,
        j + 1
      );
    }
  }

  // 清理临时目录
  drop(file_path);
  temp_dir.close()?;

  Ok(())
}
