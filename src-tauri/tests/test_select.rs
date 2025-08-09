#[tokio::test]
async fn test_select() -> anyhow::Result<()> {
  let data = vec![
    "name,age,gender",
    "Tom,18,male",
    "Jerry,19,male",
    "Patrick,4,male",
    "Sandy,24,female",
  ];
  let temp_dir = tempfile::TempDir::new()?;
  let file_path = temp_dir.path().join("input.csv");
  let mut wtr = csv::Writer::from_path(&file_path)?;
  for line in &data {
    wtr.write_record(line.split(',').map(|s| s.as_bytes()))?;
  }
  wtr.flush()?;
  let cols = "name|age".to_string();

  lib::cmd::select::select_columns(
    file_path.to_str().unwrap(),
    cols,
    "nil".into(),
    "include".into(),
    lib::utils::MockEmitter::default(),
  )
  .await?;

  let output_path = temp_dir.path().join(format!(
    "{}.select.csv",
    file_path.file_stem().unwrap().to_str().unwrap()
  ));

  let mut rdr = csv::ReaderBuilder::new().from_path(output_path)?;
  let expected_headers = ["name", "age"];
  for header in expected_headers {
    assert!(
      rdr.headers()?.iter().any(|h| h == header),
      "wrong header: {header}",
    );
  }

  let expected = vec![
    vec!["Tom", "18"],
    vec!["Jerry", "19"],
    vec!["Patrick", "4"],
    vec!["Sandy", "24"],
  ];
  for (row, result) in rdr.byte_records().enumerate() {
    let record = result?;
    let expected_record = &expected[row];
    for (col, field) in record.iter().enumerate() {
      assert_eq!(
        std::str::from_utf8(field)?,
        expected_record[col],
        "Row {}, Column {} do not match",
        row + 1,
        col + 1
      );
    }
  }

  Ok(temp_dir.close()?)
}
