use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use csv::{StringRecord, WriterBuilder};
use lazy_static::lazy_static;
use odbc_api::{ConnectionOptions, Cursor, Environment, ResultSetMetadata, buffers::TextRowSet};

fn connection(odbc_conn: &str) -> Result<odbc_api::Connection<'_>, odbc_api::Error> {
  lazy_static! {
    static ref ENV: Environment = Environment::new().unwrap();
  }
  let conn = ENV.connect_with_connection_string(odbc_conn, ConnectionOptions::default())?;
  Ok(conn)
}

fn get_all_table(conn: &odbc_api::Connection) -> Result<Vec<String>> {
  let mut cursor = conn.tables("", "", "", "")?;
  let mut tables = Vec::new();
  let mut buffer = TextRowSet::for_cursor(100, &mut cursor, Some(4096))?;
  let mut row_set_cursor = cursor.bind_buffer(&mut buffer)?;

  while let Some(row_set) = row_set_cursor.fetch()? {
    for row_index in 0..row_set.num_rows() {
      let mut values = Vec::new();
      for col_index in 0..row_set.num_cols() {
        let value = row_set
          .at_as_str(col_index, row_index)
          .unwrap()
          .unwrap_or("NULL");
        values.push(value);
      }

      if values.len() >= 4 && values[3] == "TABLE" && values[2] != "NULL" {
        tables.push(values[2].to_string());
      }
    }
  }

  Ok(tables)
}

pub async fn access_to_csv(path: &str, sep: String) -> Result<()> {
  let driver = "{Microsoft Access Driver (*.mdb, *.accdb)}";
  let batch_size = 5000;

  let sep = if sep == "\\t" {
    b'\t'
  } else {
    sep.as_bytes()[0]
  };

  let parent_path = Path::new(path).parent().unwrap().to_str().unwrap();
  let file_stem = Path::new(path).file_stem().unwrap().to_str().unwrap();

  let c = format!("Driver={};Dbq={};", driver, path);
  let conn = connection(&c)?;
  let tables = get_all_table(&conn)?;

  for table in tables.iter() {
    let mut fname = PathBuf::from(parent_path);
    fname.push(format!("{file_stem}.access.csv"));
    let query = format!("select * from {table}");

    match conn.execute(&query, ())? {
      Some(mut cursor) => {
        let mut writer = WriterBuilder::new().delimiter(sep).from_path(&fname)?;
        let headers: Vec<String> = cursor.column_names()?.collect::<Result<_, _>>()?;
        writer.write_record(headers)?;

        let mut buffers = TextRowSet::for_cursor(batch_size, &mut cursor, Some(4096))?;
        let mut row_set_cursor = cursor.bind_buffer(&mut buffers)?;
        while let Some(batch) = row_set_cursor.fetch()? {
          for row_index in 0..batch.num_rows() {
            let record =
              (0..batch.num_cols()).map(|col_index| batch.at(col_index, row_index).unwrap_or(&[]));
            let record = StringRecord::from_byte_record_lossy(record.collect());
            writer.write_record(record.as_byte_record())?;
          }
        }
        writer.flush()?;
      }
      None => return Err(anyhow!("Query came back empty.")),
    }
  }

  Ok(())
}
