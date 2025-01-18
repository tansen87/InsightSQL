use std::path::Path;
use std::time::Instant;

use anyhow::{anyhow, Result};
use csv::StringRecord;
use lazy_static::lazy_static;
use odbc_api::{buffers::TextRowSet, ConnectionOptions, Cursor, Environment, ResultSetMetadata};

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

async fn access_to_csv(path: String, sep: String) -> Result<()> {
  let driver = "{Microsoft Access Driver (*.mdb, *.accdb)}";
  let batch_size = 5000;

  let sep = if sep == "\\t" {
    b'\t'
  } else {
    sep.as_bytes()[0]
  };

  let vec_path: Vec<&str> = path.split('|').collect();
  let parent_path = Path::new(&vec_path[0])
    .parent()
    .map(|parent| parent.to_string_lossy())
    .unwrap_or_else(|| "Default Path".to_string().into());

  for fp in vec_path.iter() {
    let c = format!("Driver={};Dbq={};", driver, fp);
    let conn = connection(&c)?;
    let tables = get_all_table(&conn)?;

    for table in tables.iter() {
      let fname = format!("{}/accessDB_{}.csv", parent_path, table);
      let query = format!("select * from {}", table);

      match conn.execute(&query, ())? {
        Some(mut cursor) => {
          let mut writer = csv::WriterBuilder::new().delimiter(sep).from_path(&fname)?;
          let headers: Vec<String> = cursor.column_names()?.collect::<Result<_, _>>()?;
          writer.write_record(headers)?;

          let mut buffers = TextRowSet::for_cursor(batch_size, &mut cursor, Some(4096))?;
          let mut row_set_cursor = cursor.bind_buffer(&mut buffers)?;
          while let Some(batch) = row_set_cursor.fetch()? {
            for row_index in 0..batch.num_rows() {
              let record = (0..batch.num_cols())
                .map(|col_index| batch.at(col_index, row_index).unwrap_or(&[]));
              let record = StringRecord::from_byte_record_lossy(record.collect());
              writer.write_record(record.as_byte_record())?;
            }
          }
          writer.flush()?;
        }
        None => return Err(anyhow!("Query came back empty.")),
      }
    }
  }

  Ok(())
}

#[tauri::command]
pub async fn access(path: String, sep: String) -> Result<String, String> {
  let start_time = Instant::now();

  match access_to_csv(path, sep).await {
    Ok(_) => {
      let end_time = Instant::now();
      let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
      Ok(format!("{elapsed_time:.2}"))
    }
    Err(err) => Err(format!("access failed: {err}")),
  }
}
