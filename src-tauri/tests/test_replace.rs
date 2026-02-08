use insight::io::csv::{config::CsvConfigBuilder, options::CsvOptions};

#[tokio::test]
async fn test_replace() -> anyhow::Result<()> {
  use std::io::Write;

  let temp_dir = tempfile::TempDir::new()?;

  let data = vec![
    "",
    "name,age,gender",
    "Jerry,19,male",
    "Patrick,4,male",
    "Sandy,24,female",
  ];
  let file_path = temp_dir.path().join("input.csv");
  let mut file = std::fs::File::create(&file_path)?;
  for line in &data {
    writeln!(file, "{}", line)?;
  }

  let mut opts = CsvOptions::new(file_path.to_string_lossy().to_string());
  opts.set_skiprows(1);
  let (_sep, reader) = opts.skiprows_and_delimiter()?;
  let output_path = temp_dir.path().join(format!(
    "{}_replace.csv",
    file_path.file_stem().unwrap().to_str().unwrap()
  ));
  let config = CsvConfigBuilder::new().build();
  let rdr = config.build_reader(reader);
  let wtr = config.build_writer(&output_path)?;

  insight::cmd::replace::regex_replace(
    rdr,
    wtr,
    opts,
    "age".to_string(),
    r"^\d+$".to_string(),
    "XX".to_string(),
    false,
    insight::utils::MockEmitter::default(),
  )
  .await?;

  let context = std::fs::read_to_string(&output_path)?;
  let result = context.lines().collect::<Vec<_>>();
  let expected = vec![
    "name,age,gender",
    "Jerry,XX,male",
    "Patrick,XX,male",
    "Sandy,XX,female",
  ];
  assert_eq!(expected, result);

  Ok(temp_dir.close()?)
}
