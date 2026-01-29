#[tokio::test]
async fn test_idx() -> anyhow::Result<()> {
  let temp_dir = tempfile::TempDir::new()?;

  let data = vec!["index,age,name", "1,18,AC", "3,19,AD", "2,24,AA", "4,20,AB"];
  let file_path = temp_dir.path().join("input.csv");
  let mut wtr = csv::Writer::from_path(&file_path)?;
  for line in &data {
    wtr.write_record(line.split(','))?;
  }
  wtr.flush()?;

  insight::cmd::idx::create_index(&file_path, true, 0).await?;

  let output_path = temp_dir.path().join(format!(
    "{}.csv.idx",
    file_path.file_stem().unwrap().to_str().unwrap()
  ));
  let idx_file = std::fs::File::open(output_path)?;
  let idx = csv_index::RandomAccessSimple::open(idx_file)?;
  assert_eq!(idx.len(), 5);

  Ok(temp_dir.close()?)
}
