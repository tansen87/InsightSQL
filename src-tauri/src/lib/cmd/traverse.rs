use std::{fs, path::Path};

use anyhow::Result;

fn traverse_directory<P: AsRef<Path>>(path: P, prefix: String) -> Result<Vec<String>> {
  let mut names = Vec::new();

  let entries = fs::read_dir(&path)?;

  for entry in entries {
    let entry = entry?;
    let path = entry.path();
    let metadata = entry.metadata()?;
    let file_type = metadata.file_type();

    if file_type.is_file() {
      let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
      let prefixed_name = format!("{}{}", prefix, filename);
      names.push(prefixed_name);
    } else if file_type.is_dir() {
      let dirname = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
      let new_prefix = format!("{}{}", prefix, dirname);

      // 递归调用 traverse_directory，并将结果合并到 names 中
      let sub_names = traverse_directory(&path, format!("{new_prefix}|"))?;
      names.extend(sub_names);
    }
  }

  Ok(names)
}

fn write_xlsx(data: Vec<String>, output: String) -> Result<()> {
  let mut workbook = rust_xlsxwriter::Workbook::new();
  let worksheet = workbook.add_worksheet();
  worksheet.write_string(0, 0, "FileName".to_string())?;
  for (idx, value) in data.iter().enumerate() {
    worksheet.write_string((idx + 1).try_into()?, 0, value)?;
  }
  workbook.save(output)?;

  Ok(())
}

#[tauri::command]
pub async fn traverse(folder_path: String, output: String) -> Result<String, String> {
  let directory_path = Path::new(folder_path.as_str());

  let data = match (async { traverse_directory(directory_path, String::new()) }).await {
    Ok(result) => Ok(result),
    Err(err) => Err(format!("{err}")),
  };

  match (async { write_xlsx(data.map_err(|e| e.to_string()).unwrap(), output) }).await {
    Ok(_) => Ok("traverse done".to_string()),
    Err(err) => Err(format!("traverse failed: {err}")),
  }
}
