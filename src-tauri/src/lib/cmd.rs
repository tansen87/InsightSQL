use std::collections::HashMap;

use anyhow::Result;

use crate::utils::CsvOptions;

#[tauri::command]
pub async fn map_headers(path: String, skip_rows: String) -> Result<Vec<HashMap<String, String>>, String> {
    let mut csv_options = CsvOptions::new(path);
    csv_options.set_skip_rows(skip_rows.parse::<usize>().map_err(|e| e.to_string())?);

    async { csv_options.map_headers().map_err(|e| e.to_string()) }.await
}