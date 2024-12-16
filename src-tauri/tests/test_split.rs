use std::{
  fs::{self, File},
  io::{BufRead, BufReader},
};

use anyhow::Result;
use tempfile::TempDir;

use lib::split::public_split;

#[tokio::test]
async fn test_split() -> Result<()> {
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

  let size: usize = 2;

  public_split(file_path.to_str().unwrap().to_string(), size.try_into()?).await?;

  // 验证结果
  let output_files: Vec<_> = fs::read_dir(temp_dir.path())?
    .filter_map(Result::ok)
    .filter(|entry| entry.path().is_file())
    .filter(|entry| {
      entry
        .file_name()
        .to_string_lossy()
        .starts_with("input.split")
    })
    .collect();

  // 检查文件数量是否正确
  assert_eq!(output_files.len(), 2);

  // 检查每个文件的内容
  for (i, entry) in output_files.iter().enumerate() {
    let file = File::open(entry.path())?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;

    // 检查header是否正确
    let expected_headers = "name,age,gender\n";
    assert_eq!(lines[0], expected_headers.trim_end());

    // 从第二行开始检查数据行
    for (j, line) in lines.iter().enumerate().skip(1) {
      let data_index = i * size + j - 1;
      if data_index < data.len() - 1 {
        assert_eq!(*line, data[data_index + 1]);
      }
    }
  }

  // 清理临时目录
  drop(file_path);
  temp_dir.close()?;

  Ok(())
}
