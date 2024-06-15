#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use lib::sqlp;

fn main() {
  tauri::Builder
    ::default()
    .invoke_handler(tauri::generate_handler![sqlp::get, sqlp::query])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
