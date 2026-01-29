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

  let path = file_path.to_str().unwrap().to_string();
  let output_path = temp_dir
    .path()
    .join(format!(
      "{}.search.csv",
      file_path.file_stem().unwrap().to_str().unwrap()
    ))
    .to_string_lossy()
    .to_string();

  Ok((temp_dir, path, output_path))
}

#[tokio::test]
async fn test_equal() -> anyhow::Result<()> {
  let (temp_dir, path, output_path) = create_temp_csv()?;
  let column = "name".to_string();
  let conditions = vec!["Tom".to_string()];

  let match_rows = insight::cmd::search::contains(
    path,
    column,
    conditions,
    0,
    true,
    true,
    insight::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 1);

  let context = std::fs::read_to_string(output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec!["name,age,gender", "Tom,18,male"];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_not_equal() -> anyhow::Result<()> {
  let (temp_dir, path, output_path) = create_temp_csv()?;

  let column = "name".to_string();
  let conditions = vec!["Tom".to_string()];

  let match_rows = insight::cmd::search::not_equal(
    path,
    column,
    conditions,
    0,
    true,
    true,
    insight::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 3);

  let context = std::fs::read_to_string(output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec![
    "name,age,gender",
    "Jerry,19,male",
    "Patrick,4,male",
    "Sandy,24,female",
  ];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_contains() -> anyhow::Result<()> {
  let (temp_dir, path, output_path) = create_temp_csv()?;

  let column = "name".to_string();
  let conditions = vec!["at".to_string()];

  let match_rows = insight::cmd::search::contains(
    path,
    column,
    conditions,
    0,
    true,
    true,
    insight::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 1);

  let context = std::fs::read_to_string(output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec!["name,age,gender", "Patrick,4,male"];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_not_contains() -> anyhow::Result<()> {
  let (temp_dir, path, output_path) = create_temp_csv()?;

  let column = "name".to_string();
  let conditions = vec!["at".to_string()];

  let match_rows = insight::cmd::search::not_contains(
    path,
    column,
    conditions,
    0,
    true,
    true,
    insight::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 3);

  let context = std::fs::read_to_string(output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec![
    "name,age,gender",
    "Tom,18,male",
    "Jerry,19,male",
    "Sandy,24,female",
  ];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_starts_with() -> anyhow::Result<()> {
  let (temp_dir, path, output_path) = create_temp_csv()?;

  let column = "name".to_string();
  let conditions = vec!["Pa".to_string()];

  let match_rows = insight::cmd::search::starts_with(
    path,
    column,
    conditions,
    0,
    true,
    true,
    insight::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 1);

  let context = std::fs::read_to_string(output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec!["name,age,gender", "Patrick,4,male"];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_not_starts_with() -> anyhow::Result<()> {
  let (temp_dir, path, output_path) = create_temp_csv()?;

  let column = "name".to_string();
  let conditions = vec!["Pa".to_string()];

  let match_rows = insight::cmd::search::not_starts_with(
    path,
    column,
    conditions,
    0,
    true,
    true,
    insight::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 3);

  let context = std::fs::read_to_string(output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec![
    "name,age,gender",
    "Tom,18,male",
    "Jerry,19,male",
    "Sandy,24,female",
  ];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_ends_with() -> anyhow::Result<()> {
  let (temp_dir, path, output_path) = create_temp_csv()?;

  let column = "name".to_string();
  let conditions = vec!["ick".to_string()];

  let match_rows = insight::cmd::search::ends_with(
    path,
    column,
    conditions,
    0,
    true,
    true,
    insight::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 1);

  let context = std::fs::read_to_string(output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec!["name,age,gender", "Patrick,4,male"];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_not_ends_with() -> anyhow::Result<()> {
  let (temp_dir, path, output_path) = create_temp_csv()?;

  let column = "name".to_string();
  let conditions = vec!["ick".to_string()];

  let match_rows = insight::cmd::search::not_ends_with(
    path,
    column,
    conditions,
    0,
    true,
    true,
    insight::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 3);

  let context = std::fs::read_to_string(output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec![
    "name,age,gender",
    "Tom,18,male",
    "Jerry,19,male",
    "Sandy,24,female",
  ];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_regex() -> anyhow::Result<()> {
  let (temp_dir, path, output_path) = create_temp_csv()?;

  let column = "name".to_string();
  let regex_char = r"^J.*".to_string(); // Matches any string that starts with 'J'

  let match_rows = insight::cmd::search::regex_search(
    path,
    column,
    regex_char,
    0,
    true,
    true,
    insight::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 1);

  let context = std::fs::read_to_string(output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec!["name,age,gender", "Jerry,19,male"];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_is_null() -> anyhow::Result<()> {
  let (temp_dir, path, output_path) = create_temp_csv()?;

  let column = "name".to_string();
  let conditions = vec!["".to_string()];

  let match_rows = insight::cmd::search::is_null(
    path,
    column,
    conditions,
    0,
    true,
    true,
    insight::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 0);

  let context = std::fs::read_to_string(output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec!["name,age,gender"];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_is_not_null() -> anyhow::Result<()> {
  let (temp_dir, path, output_path) = create_temp_csv()?;

  let column = "name".to_string();
  let conditions = vec!["".to_string()];

  let match_rows = insight::cmd::search::is_not_null(
    path,
    column,
    conditions,
    0,
    true,
    true,
    insight::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 4);

  let context = std::fs::read_to_string(output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec![
    "name,age,gender",
    "Tom,18,male",
    "Jerry,19,male",
    "Patrick,4,male",
    "Sandy,24,female",
  ];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_gt() -> anyhow::Result<()> {
  let (temp_dir, path, output_path) = create_temp_csv()?;

  let column = "age".to_string();
  let conditions = "18".to_string();

  let match_rows = insight::cmd::search::greater_than(
    path,
    column,
    conditions,
    0,
    true,
    true,
    insight::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 2);

  let context = std::fs::read_to_string(output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec!["name,age,gender", "Jerry,19,male", "Sandy,24,female"];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_ge() -> anyhow::Result<()> {
  let (temp_dir, path, output_path) = create_temp_csv()?;

  let column = "age".to_string();
  let conditions = "18".to_string();

  let match_rows = insight::cmd::search::greater_than_or_equal(
    path,
    column,
    conditions,
    0,
    true,
    true,
    insight::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 3);

  let context = std::fs::read_to_string(output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec![
    "name,age,gender",
    "Tom,18,male",
    "Jerry,19,male",
    "Sandy,24,female",
  ];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_lt() -> anyhow::Result<()> {
  let (temp_dir, path, output_path) = create_temp_csv()?;

  let column = "age".to_string();
  let conditions = "18".to_string();

  let match_rows = insight::cmd::search::less_than(
    path,
    column,
    conditions,
    0,
    true,
    true,
    insight::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 1);

  let context = std::fs::read_to_string(output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec!["name,age,gender", "Patrick,4,male"];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_le() -> anyhow::Result<()> {
  let (temp_dir, path, output_path) = create_temp_csv()?;

  let column = "age".to_string();
  let conditions = "18".to_string();

  let match_rows = insight::cmd::search::less_than_or_equal(
    path,
    column,
    conditions,
    0,
    true,
    true,
    insight::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 2);

  let context = std::fs::read_to_string(output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec!["name,age,gender", "Tom,18,male", "Patrick,4,male"];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_between() -> anyhow::Result<()> {
  let (temp_dir, path, output_path) = create_temp_csv()?;

  let column = "age".to_string();
  let conditions = vec!["18".to_string(), "19".to_string()];

  let match_rows = insight::cmd::search::between(
    path,
    column,
    conditions,
    0,
    true,
    true,
    insight::utils::MockEmitter::default(),
  )
  .await?
  .parse::<usize>()?;
  assert_eq!(match_rows, 2);

  let context = std::fs::read_to_string(output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec!["name,age,gender", "Tom,18,male", "Jerry,19,male"];
  assert_eq!(expected, result);

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
  let (temp_dir, path, _output_path) = create_temp_csv()?;

  let column = "name".to_string();
  let conditions = vec!["Tom".to_string(), "Jerry".to_string()];

  let match_rows = insight::cmd::search::equal_multi(
    path,
    column,
    conditions.clone(),
    0,
    true,
    true,
    insight::utils::MockEmitter::default(),
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
  let (temp_dir, path, _output_path) = create_temp_csv()?;

  let column = "name".to_string();
  let conditions = vec!["To".to_string(), "Jer".to_string()];

  let match_rows = insight::cmd::search::contains_multi(
    path,
    column,
    conditions.clone(),
    0,
    true,
    true,
    insight::utils::MockEmitter::default(),
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
  let (temp_dir, path, _output_path) = create_temp_csv()?;

  let column = "name".to_string();
  let conditions = vec!["Pa".to_string(), "San".to_string()];

  let match_rows = insight::cmd::search::starts_with_multi(
    path,
    column,
    conditions.clone(),
    0,
    true,
    true,
    insight::utils::MockEmitter::default(),
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
  let (temp_dir, path, _output_path) = create_temp_csv()?;

  let column = "name".to_string();
  let conditions = vec!["ick".to_string(), "dy".to_string()];

  let match_rows = insight::cmd::search::ends_with_multi(
    path,
    column,
    conditions.clone(),
    0,
    true,
    true,
    insight::utils::MockEmitter::default(),
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
