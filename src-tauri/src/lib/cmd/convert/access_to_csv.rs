use std::path::PathBuf;

use anyhow::{Result, anyhow};
use csv::{StringRecord, WriterBuilder};
use lazy_static::lazy_static;
use odbc_api::{ConnectionOptions, Cursor, Environment, ResultSetMetadata, buffers::TextRowSet};

use crate::io::csv::options::CsvOptions;

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
      let table_type = match row_set.at_as_str(3, row_index)? {
        Some(s) => s,
        None => continue,
      };
      let table_name = match row_set.at_as_str(2, row_index)? {
        Some(s) => s,
        None => continue,
      };

      if table_type == "TABLE" {
        tables.push(table_name.to_string());
      }
    }
  }

  Ok(tables)
}

pub async fn access_to_csv(path: &str, wtr_sep: String) -> Result<()> {
  let driver = "{Microsoft Access Driver (*.mdb, *.accdb)}";
  let batch_size = 5000;
  let sep = if wtr_sep == "\\t" {
    b'\t'
  } else {
    wtr_sep.as_bytes()[0]
  };

  let opts = CsvOptions::new(path);
  let parent_path = opts.parent_path()?;
  let file_stem = opts.file_stem()?;

  let c = format!("Driver={};Dbq={};", driver, path);
  let conn = connection(&c)?;
  let tables = get_all_table(&conn)?;

  for (idx, table) in tables.iter().enumerate() {
    let mut fname = PathBuf::from(parent_path);
    let tbl_name: String = table.chars().take(30).collect();
    fname.push(format!("{file_stem}.{idx:03}.{tbl_name}.csv"));
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
