fn create_temp_csv() -> anyhow::Result<(
  tempfile::TempDir,
  csv::Reader<std::io::BufReader<Box<dyn std::io::Read + Send>>>,
  csv::Writer<std::io::BufWriter<std::fs::File>>,
  String,
)> {
  use std::io::Write;

  let temp_dir = tempfile::TempDir::new()?;

  let data = vec![
    "",
    "Patrick,4,male",
    "name,age,gender",
    "汤-姆-1,18,男",
    "杰-瑞-2,19,male",
    "Sa-n-dy,24,female",
  ];
  let file_path = temp_dir.path().join("input.csv");
  let mut file = std::fs::File::create(&file_path)?;
  for line in &data {
    writeln!(file, "{}", line)?;
  }

  let output_path = temp_dir
    .path()
    .join(format!(
      "{}_slice.csv",
      file_path.file_stem().unwrap().to_str().unwrap()
    ))
    .to_string_lossy()
    .to_string();

  let mut csv_options = insight::io::csv::options::CsvOptions::new(file_path);
  csv_options.set_skiprows(2);
  let rdr = csv_options.skiprows_reader()?;

  let output_file = std::fs::File::create(&output_path)?;
  let buf_writer = std::io::BufWriter::with_capacity(256_000, output_file);
  let wtr = csv::WriterBuilder::new().from_writer(buf_writer);

  Ok((temp_dir, rdr, wtr, output_path))
}

#[tokio::test]
async fn test_slice_left() -> anyhow::Result<()> {
  let (temp_dir, rdr, wtr, output_path) = create_temp_csv()?;

  insight::cmd::string::slice::slice_nchar(
    rdr,
    wtr,
    "name",
    1,
    false,
    "left".to_string(),
    true,
    insight::utils::MockEmitter::default(),
  )
  .await?;

  let context = std::fs::read_to_string(output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec![
    "name,age,gender,name_nchar",
    "汤-姆-1,18,男,汤",
    "杰-瑞-2,19,male,杰",
    "Sa-n-dy,24,female,S",
  ];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_slice_right() -> anyhow::Result<()> {
  let (temp_dir, rdr, wtr, output_path) = create_temp_csv()?;

  insight::cmd::string::slice::slice_nchar(
    rdr,
    wtr,
    "name",
    1,
    false,
    "right".to_string(),
    true,
    insight::utils::MockEmitter::default(),
  )
  .await?;

  let context = std::fs::read_to_string(&output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec![
    "name,age,gender,name_nchar",
    "汤-姆-1,18,男,1",
    "杰-瑞-2,19,male,2",
    "Sa-n-dy,24,female,y",
  ];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_split_n() -> anyhow::Result<()> {
  let (temp_dir, rdr, wtr, output_path) = create_temp_csv()?;

  insight::cmd::string::split::split_n(
    rdr,
    wtr,
    "name",
    2,
    "-".to_string(),
    true,
    insight::utils::MockEmitter::default(),
  )
  .await?;

  let context = std::fs::read_to_string(&output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec![
    "name,age,gender,name_nth",
    "汤-姆-1,18,男,姆",
    "杰-瑞-2,19,male,瑞",
    "Sa-n-dy,24,female,n",
  ];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_split_max() -> anyhow::Result<()> {
  let (temp_dir, rdr, wtr, output_path) = create_temp_csv()?;

  insight::cmd::string::split::split_max(
    rdr,
    wtr,
    "name".to_string(),
    2,
    "-".to_string(),
    true,
    insight::utils::MockEmitter::default(),
  )
  .await?;

  let context = std::fs::read_to_string(&output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec![
    "name,age,gender,name_max1,name_max2",
    "汤-姆-1,18,男,汤,姆",
    "杰-瑞-2,19,male,杰,瑞",
    "Sa-n-dy,24,female,Sa,n",
  ];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}

#[tokio::test]
async fn test_slice() -> anyhow::Result<()> {
  let (temp_dir, rdr, wtr, output_path) = create_temp_csv()?;

  insight::cmd::string::slice::slice(
    rdr,
    wtr,
    "name",
    2,
    3,
    false,
    true,
    insight::utils::MockEmitter::default(),
  )
  .await?;

  let context = std::fs::read_to_string(&output_path)?;
  let result = context.trim().split('\n').collect::<Vec<_>>();
  let expected = vec![
    "name,age,gender,name_slice",
    "汤-姆-1,18,男,-姆-",
    "杰-瑞-2,19,male,-瑞-",
    "Sa-n-dy,24,female,a-n",
  ];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}
