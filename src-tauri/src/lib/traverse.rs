use std::error::Error;
use std::path::Path;
use std::{fs, io};

use tauri::Emitter;

fn traverse_directory(path: &Path, prefix: String) -> io::Result<Vec<String>> {
  let mut names = Vec::new();

  let entries = fs::read_dir(path)?;

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

fn write_xlsx(data: Vec<String>, output: String) -> Result<(), Box<dyn Error>> {
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
pub async fn traverse(folder_path: String, output: String, window: tauri::Window) {
  let directory_path = Path::new(folder_path.as_str());

  let data = match (async { traverse_directory(directory_path, String::new()) }).await {
    Ok(result) => result,
    Err(error) => {
      window.emit("traverse_err", &error.to_string()).unwrap();
      return ();
    }
  };

  match (async { write_xlsx(data, output) }).await {
    Ok(result) => result,
    Err(error) => {
      window
        .emit("traverse_write_err", &error.to_string())
        .unwrap();
      return ();
    }
  }
}
