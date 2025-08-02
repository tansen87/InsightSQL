fn create_temp_csv() -> anyhow::Result<(tempfile::TempDir, String, String)> {
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

  let output_path = temp_dir
    .path()
    .join("input.search.csv")
    .to_str()
    .unwrap()
    .to_string();

  Ok((
    temp_dir,
    file_path.to_str().unwrap().to_string(),
    output_path,
  ))
}

#[tokio::test]
async fn test_equal() -> anyhow::Result<()> {
  let (temp_dir, file_path, output_path) = create_temp_csv()?;
  let sep = b',';
  let column = "name".to_string();
  let conditions = vec!["Tom".to_string()];

  let match_rows = lib::cmd::search::contains(
    file_path,
    sep,
    column,
    conditions,
    output_path.clone().into(),
    lib::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 1);

  let mut rdr = csv::ReaderBuilder::new().from_path(output_path)?;
  let expected_headers = ["name", "age", "gender"];
  for header in expected_headers {
    assert!(
      rdr.headers()?.iter().any(|h| h == header),
      "wrong header: {header}",
    );
  }

  let expected = vec![vec!["Tom", "18", "male"]];
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

#[tokio::test]
async fn test_not_equal() -> anyhow::Result<()> {
  let (temp_dir, file_path, output_path) = create_temp_csv()?;
  let sep = b',';
  let column = "name".to_string();
  let conditions = vec!["Tom".to_string()];

  let match_rows = lib::cmd::search::not_equal(
    file_path,
    sep,
    column,
    conditions,
    output_path.clone().into(),
    lib::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 3);

  let mut rdr = csv::ReaderBuilder::new().from_path(output_path)?;
  let expected_headers = ["name", "age", "gender"];
  for header in expected_headers {
    assert!(
      rdr.headers()?.iter().any(|h| h == header),
      "wrong header: {header}",
    );
  }

  let expected = vec![
    vec!["Jerry", "19", "male"],
    vec!["Patrick", "4", "male"],
    vec!["Sandy", "24", "female"],
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

#[tokio::test]
async fn test_contains() -> anyhow::Result<()> {
  let (temp_dir, file_path, output_path) = create_temp_csv()?;
  let sep = b',';
  let column = "name".to_string();
  let conditions = vec!["at".to_string()];

  let match_rows = lib::cmd::search::contains(
    file_path,
    sep,
    column,
    conditions,
    output_path.clone().into(),
    lib::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 1);

  let mut rdr = csv::ReaderBuilder::new().from_path(output_path)?;
  let expected_headers = ["name", "age", "gender"];
  for header in expected_headers {
    assert!(
      rdr.headers()?.iter().any(|h| h == header),
      "wrong header: {header}",
    );
  }

  let expected = vec![vec!["Patrick", "4", "male"]];
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

#[tokio::test]
async fn test_not_contains() -> anyhow::Result<()> {
  let (temp_dir, file_path, output_path) = create_temp_csv()?;
  let sep = b',';
  let column = "name".to_string();
  let conditions = vec!["at".to_string()];

  let match_rows = lib::cmd::search::not_contains(
    file_path,
    sep,
    column,
    conditions,
    output_path.clone().into(),
    lib::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 3);

  let mut rdr = csv::ReaderBuilder::new().from_path(output_path)?;
  let expected_headers = ["name", "age", "gender"];
  for header in expected_headers {
    assert!(
      rdr.headers()?.iter().any(|h| h == header),
      "wrong header: {header}",
    );
  }

  let expected = vec![
    vec!["Tom", "18", "male"],
    vec!["Jerry", "19", "male"],
    vec!["Sandy", "24", "female"],
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

#[tokio::test]
async fn test_starts_with() -> anyhow::Result<()> {
  let (temp_dir, file_path, output_path) = create_temp_csv()?;
  let sep = b',';
  let column = "name".to_string();
  let conditions = vec!["Pa".to_string()];

  let match_rows = lib::cmd::search::starts_with(
    file_path,
    sep,
    column,
    conditions,
    output_path.clone().into(),
    lib::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 1);

  let mut rdr = csv::ReaderBuilder::new().from_path(output_path)?;
  let expected_headers = ["name", "age", "gender"];
  for header in expected_headers {
    assert!(
      rdr.headers()?.iter().any(|h| h == header),
      "wrong header: {header}",
    );
  }

  let expected = vec![vec!["Patrick", "4", "male"]];
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

#[tokio::test]
async fn test_not_starts_with() -> anyhow::Result<()> {
  let (temp_dir, file_path, output_path) = create_temp_csv()?;
  let sep = b',';
  let column = "name".to_string();
  let conditions = vec!["Pa".to_string()];

  let match_rows = lib::cmd::search::not_starts_with(
    file_path,
    sep,
    column,
    conditions,
    output_path.clone().into(),
    lib::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 3);

  let mut rdr = csv::ReaderBuilder::new().from_path(output_path)?;
  let expected_headers = ["name", "age", "gender"];
  for header in expected_headers {
    assert!(
      rdr.headers()?.iter().any(|h| h == header),
      "wrong header: {header}",
    );
  }

  let expected = vec![
    vec!["Tom", "18", "male"],
    vec!["Jerry", "19", "male"],
    vec!["Sandy", "24", "female"],
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

#[tokio::test]
async fn test_ends_with() -> anyhow::Result<()> {
  let (temp_dir, file_path, output_path) = create_temp_csv()?;
  let sep = b',';
  let column = "name".to_string();
  let conditions = vec!["ick".to_string()];

  let match_rows = lib::cmd::search::ends_with(
    file_path,
    sep,
    column,
    conditions,
    output_path.clone().into(),
    lib::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 1);

  let mut rdr = csv::ReaderBuilder::new().from_path(output_path)?;
  let expected_headers = ["name", "age", "gender"];
  for header in expected_headers {
    assert!(
      rdr.headers()?.iter().any(|h| h == header),
      "wrong header: {header}",
    );
  }

  let expected = vec![vec!["Patrick", "4", "male"]];
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

#[tokio::test]
async fn test_not_ends_with() -> anyhow::Result<()> {
  let (temp_dir, file_path, output_path) = create_temp_csv()?;
  let sep = b',';
  let column = "name".to_string();
  let conditions = vec!["ick".to_string()];

  let match_rows = lib::cmd::search::not_ends_with(
    file_path,
    sep,
    column,
    conditions,
    output_path.clone().into(),
    lib::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 3);

  let mut rdr = csv::ReaderBuilder::new().from_path(output_path)?;
  let expected_headers = ["name", "age", "gender"];
  for header in expected_headers {
    assert!(
      rdr.headers()?.iter().any(|h| h == header),
      "wrong header: {header}",
    );
  }

  let expected = vec![
    vec!["Tom", "18", "male"],
    vec!["Jerry", "19", "male"],
    vec!["Sandy", "24", "female"],
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

#[tokio::test]
async fn test_regex() -> anyhow::Result<()> {
  let (temp_dir, file_path, output_path) = create_temp_csv()?;
  let sep = b',';
  let column = "name".to_string();
  let regex_char = r"^J.*".to_string(); // Matches any string that starts with 'J'

  let match_rows = lib::cmd::search::regex_search(
    file_path,
    sep,
    column,
    regex_char,
    output_path.clone().into(),
    lib::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 1);

  let mut rdr = csv::ReaderBuilder::new().from_path(output_path)?;
  let expected_headers = ["name", "age", "gender"];
  for header in expected_headers {
    assert!(
      rdr.headers()?.iter().any(|h| h == header),
      "wrong header: {header}",
    );
  }

  let expected = vec![vec!["Jerry", "19", "male"]];
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

#[tokio::test]
async fn test_is_null() -> anyhow::Result<()> {
  let (temp_dir, file_path, output_path) = create_temp_csv()?;
  let sep = b',';
  let column = "name".to_string();
  let conditions = vec!["".to_string()];

  let match_rows = lib::cmd::search::is_null(
    file_path,
    sep,
    column,
    conditions,
    output_path.clone().into(),
    lib::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 0);

  let mut rdr = csv::ReaderBuilder::new().from_path(output_path)?;
  let expected_headers = ["name", "age", "gender"];
  for header in expected_headers {
    assert!(
      rdr.headers()?.iter().any(|h| h == header),
      "wrong header: {header}",
    );
  }

  let expected = vec![vec![""]];
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

#[tokio::test]
async fn test_is_not_null() -> anyhow::Result<()> {
  let (temp_dir, file_path, output_path) = create_temp_csv()?;
  let sep = b',';
  let column = "name".to_string();
  let conditions = vec!["".to_string()];

  let match_rows = lib::cmd::search::is_not_null(
    file_path,
    sep,
    column,
    conditions,
    output_path.clone().into(),
    lib::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 4);

  let mut rdr = csv::ReaderBuilder::new().from_path(output_path)?;
  let expected_headers = ["name", "age", "gender"];
  for header in expected_headers {
    assert!(
      rdr.headers()?.iter().any(|h| h == header),
      "wrong header: {header}",
    );
  }

  let expected = vec![
    vec!["Tom", "18", "male"],
    vec!["Jerry", "19", "male"],
    vec!["Patrick", "4", "male"],
    vec!["Sandy", "24", "female"],
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

#[tokio::test]
async fn test_gt() -> anyhow::Result<()> {
  let (temp_dir, file_path, output_path) = create_temp_csv()?;
  let sep = b',';
  let column = "age".to_string();
  let conditions = "18".to_string();

  let match_rows = lib::cmd::search::greater_than(
    file_path,
    sep,
    column,
    conditions,
    output_path.clone().into(),
    lib::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 2);

  let mut rdr = csv::ReaderBuilder::new().from_path(output_path)?;
  let expected_headers = ["name", "age", "gender"];
  for header in expected_headers {
    assert!(
      rdr.headers()?.iter().any(|h| h == header),
      "wrong header: {header}",
    );
  }

  let expected = vec![vec!["Jerry", "19", "male"], vec!["Sandy", "24", "female"]];
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

#[tokio::test]
async fn test_ge() -> anyhow::Result<()> {
  let (temp_dir, file_path, output_path) = create_temp_csv()?;
  let sep = b',';
  let column = "age".to_string();
  let conditions = "18".to_string();

  let match_rows = lib::cmd::search::greater_than_or_equal(
    file_path,
    sep,
    column,
    conditions,
    output_path.clone().into(),
    lib::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 3);

  let mut rdr = csv::ReaderBuilder::new().from_path(output_path)?;
  let expected_headers = ["name", "age", "gender"];
  for header in expected_headers {
    assert!(
      rdr.headers()?.iter().any(|h| h == header),
      "wrong header: {header}",
    );
  }

  let expected = vec![
    vec!["Tom", "18", "male"],
    vec!["Jerry", "19", "male"],
    vec!["Sandy", "24", "female"],
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

#[tokio::test]
async fn test_lt() -> anyhow::Result<()> {
  let (temp_dir, file_path, output_path) = create_temp_csv()?;
  let sep = b',';
  let column = "age".to_string();
  let conditions = "18".to_string();

  let match_rows = lib::cmd::search::less_than(
    file_path,
    sep,
    column,
    conditions,
    output_path.clone().into(),
    lib::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 1);

  let mut rdr = csv::ReaderBuilder::new().from_path(output_path)?;
  let expected_headers = ["name", "age", "gender"];
  for header in expected_headers {
    assert!(
      rdr.headers()?.iter().any(|h| h == header),
      "wrong header: {header}",
    );
  }

  let expected = vec![vec!["Patrick", "4", "male"]];
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

#[tokio::test]
async fn test_le() -> anyhow::Result<()> {
  let (temp_dir, file_path, output_path) = create_temp_csv()?;
  let sep = b',';
  let column = "age".to_string();
  let conditions = "18".to_string();

  let match_rows = lib::cmd::search::less_than_or_equal(
    file_path,
    sep,
    column,
    conditions,
    output_path.clone().into(),
    lib::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 2);

  let mut rdr = csv::ReaderBuilder::new().from_path(output_path)?;
  let expected_headers = ["name", "age", "gender"];
  for header in expected_headers {
    assert!(
      rdr.headers()?.iter().any(|h| h == header),
      "wrong header: {header}",
    );
  }

  let expected = vec![vec!["Tom", "18", "male"], vec!["Patrick", "4", "male"]];
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

#[tokio::test]
async fn test_between() -> anyhow::Result<()> {
  let (temp_dir, file_path, output_path) = create_temp_csv()?;
  let sep = b',';
  let column = "age".to_string();
  let conditions = vec!["18".to_string(), "19".to_string()];

  let match_rows = lib::cmd::search::between(
    file_path,
    sep,
    column,
    conditions,
    output_path.clone().into(),
    lib::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 2);

  let mut rdr = csv::ReaderBuilder::new().from_path(output_path)?;
  let expected_headers = ["name", "age", "gender"];
  for header in expected_headers {
    assert!(
      rdr.headers()?.iter().any(|h| h == header),
      "wrong header: {header}",
    );
  }

  let expected = vec![vec!["Tom", "18", "male"], vec!["Jerry", "19", "male"]];
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

fn assert_headers_exist<R: std::io::Read>(rdr: &mut csv::Reader<R>, expected: &[&str]) {
  let headers = rdr.headers().expect("Failed to read headers");
  for &header in expected {
    assert!(
      headers.iter().any(|h| h == header),
      "missing expected header: '{}'",
      header
    );
  }
}

#[tokio::test]
async fn test_equal_multi() -> anyhow::Result<()> {
  let (temp_dir, file_path, _output_path) = create_temp_csv()?;
  let sep = b',';
  let column = "name".to_string();
  let conditions = vec!["Tom".to_string(), "Jerry".to_string()];

  let match_rows = lib::cmd::search::equal_multi(
    file_path,
    sep,
    column,
    conditions.clone(),
    lib::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 2);

  let output_paths: Vec<String> = conditions
    .iter()
    .map(|name| {
      temp_dir
        .path()
        .join(format!("input_{}.csv", name))
        .to_str()
        .unwrap()
        .to_string()
    })
    .collect();

  let mut readers: Vec<csv::Reader<std::fs::File>> = output_paths
    .iter()
    .map(|p| csv::ReaderBuilder::new().from_path(p))
    .collect::<Result<Vec<_>, _>>()?;
  let expected_headers = ["name", "age", "gender"];
  for rdr in &mut readers {
    assert_headers_exist(rdr, &expected_headers);
  }

  let expectations = vec![
    ("Tom", vec![vec!["Tom", "18", "male"]]),
    ("Jerry", vec![vec!["Jerry", "19", "male"]]),
  ];
  for ((condition, expected_rows), reader) in expectations.iter().zip(readers.iter_mut()) {
    for (row_idx, result) in reader.records().enumerate() {
      let record = result.expect(&format!(
        "Failed to read record {} in {}",
        row_idx + 1,
        condition
      ));
      let expected_row = expected_rows
        .get(row_idx)
        .unwrap_or_else(|| panic!("No expected data for row {}", row_idx + 1));
      for (col_idx, field) in record.iter().enumerate() {
        assert_eq!(
          field,
          expected_row[col_idx],
          "Condition: '{}', Row {}, Column {}: expected '{}', found '{}'",
          condition,
          row_idx + 1,
          col_idx + 1,
          expected_row[col_idx],
          field
        );
      }
    }
  }

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_contains_multi() -> anyhow::Result<()> {
  let (temp_dir, file_path, _output_path) = create_temp_csv()?;
  let sep = b',';
  let column = "name".to_string();
  let conditions = vec!["To".to_string(), "Jer".to_string()];

  let match_rows = lib::cmd::search::contains_multi(
    file_path,
    sep,
    column,
    conditions.clone(),
    lib::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 2);

  let output_paths: Vec<String> = conditions
    .iter()
    .map(|name| {
      temp_dir
        .path()
        .join(format!("input_{}.csv", name))
        .to_str()
        .unwrap()
        .to_string()
    })
    .collect();

  let mut readers: Vec<csv::Reader<std::fs::File>> = output_paths
    .iter()
    .map(|p| csv::ReaderBuilder::new().from_path(p))
    .collect::<Result<Vec<_>, _>>()?;
  let expected_headers = ["name", "age", "gender"];
  for rdr in &mut readers {
    assert_headers_exist(rdr, &expected_headers);
  }

  let expectations = vec![
    ("To", vec![vec!["Tom", "18", "male"]]),
    ("Jer", vec![vec!["Jerry", "19", "male"]]),
  ];
  for ((condition, expected_rows), reader) in expectations.iter().zip(readers.iter_mut()) {
    for (row_idx, result) in reader.records().enumerate() {
      let record = result.expect(&format!(
        "Failed to read record {} in {}",
        row_idx + 1,
        condition
      ));
      let expected_row = expected_rows
        .get(row_idx)
        .unwrap_or_else(|| panic!("No expected data for row {}", row_idx + 1));
      for (col_idx, field) in record.iter().enumerate() {
        assert_eq!(
          field,
          expected_row[col_idx],
          "Condition: '{}', Row {}, Column {}: expected '{}', found '{}'",
          condition,
          row_idx + 1,
          col_idx + 1,
          expected_row[col_idx],
          field
        );
      }
    }
  }

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_starts_with_multi() -> anyhow::Result<()> {
  let (temp_dir, file_path, _output_path) = create_temp_csv()?;
  let sep = b',';
  let column = "name".to_string();
  let conditions = vec!["Pa".to_string(), "San".to_string()];

  let match_rows = lib::cmd::search::starts_with_multi(
    file_path,
    sep,
    column,
    conditions.clone(),
    lib::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 2);

  let output_paths: Vec<String> = conditions
    .iter()
    .map(|name| {
      temp_dir
        .path()
        .join(format!("input_{}.csv", name))
        .to_str()
        .unwrap()
        .to_string()
    })
    .collect();

  let mut readers: Vec<csv::Reader<std::fs::File>> = output_paths
    .iter()
    .map(|p| csv::ReaderBuilder::new().from_path(p))
    .collect::<Result<Vec<_>, _>>()?;
  let expected_headers = ["name", "age", "gender"];
  for rdr in &mut readers {
    assert_headers_exist(rdr, &expected_headers);
  }

  let expectations = vec![
    ("Pa", vec![vec!["Patrick", "4", "male"]]),
    ("San", vec![vec!["Sandy", "24", "female"]]),
  ];
  for ((condition, expected_rows), reader) in expectations.iter().zip(readers.iter_mut()) {
    for (row_idx, result) in reader.records().enumerate() {
      let record = result.expect(&format!(
        "Failed to read record {} in {}",
        row_idx + 1,
        condition
      ));
      let expected_row = expected_rows
        .get(row_idx)
        .unwrap_or_else(|| panic!("No expected data for row {}", row_idx + 1));
      for (col_idx, field) in record.iter().enumerate() {
        assert_eq!(
          field,
          expected_row[col_idx],
          "Condition: '{}', Row {}, Column {}: expected '{}', found '{}'",
          condition,
          row_idx + 1,
          col_idx + 1,
          expected_row[col_idx],
          field
        );
      }
    }
  }

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_ends_with_multi() -> anyhow::Result<()> {
  let (temp_dir, file_path, _output_path) = create_temp_csv()?;
  let sep = b',';
  let column = "name".to_string();
  let conditions = vec!["ick".to_string(), "dy".to_string()];

  let match_rows = lib::cmd::search::ends_with_multi(
    file_path,
    sep,
    column,
    conditions.clone(),
    lib::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 2);

  let output_paths: Vec<String> = conditions
    .iter()
    .map(|name| {
      temp_dir
        .path()
        .join(format!("input_{}.csv", name))
        .to_str()
        .unwrap()
        .to_string()
    })
    .collect();

  let mut readers: Vec<csv::Reader<std::fs::File>> = output_paths
    .iter()
    .map(|p| csv::ReaderBuilder::new().from_path(p))
    .collect::<Result<Vec<_>, _>>()?;
  let expected_headers = ["name", "age", "gender"];
  for rdr in &mut readers {
    assert_headers_exist(rdr, &expected_headers);
  }

  let expectations = vec![
    ("ick", vec![vec!["Patrick", "4", "male"]]),
    ("dy", vec![vec!["Sandy", "24", "female"]]),
  ];
  for ((condition, expected_rows), reader) in expectations.iter().zip(readers.iter_mut()) {
    for (row_idx, result) in reader.records().enumerate() {
      let record = result.expect(&format!(
        "Failed to read record {} in {}",
        row_idx + 1,
        condition
      ));
      let expected_row = expected_rows
        .get(row_idx)
        .unwrap_or_else(|| panic!("No expected data for row {}", row_idx + 1));
      for (col_idx, field) in record.iter().enumerate() {
        assert_eq!(
          field,
          expected_row[col_idx],
          "Condition: '{}', Row {}, Column {}: expected '{}', found '{}'",
          condition,
          row_idx + 1,
          col_idx + 1,
          expected_row[col_idx],
          field
        );
      }
    }
  }

  Ok(temp_dir.close()?)
}
