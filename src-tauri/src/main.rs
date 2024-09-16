#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use lib::sqlp;
use lib::cat;
use lib::convert;
use lib::count;
use lib::rename;
use lib::select;
use lib::search;

fn main() {
  tauri::Builder
    ::default()
    .invoke_handler(tauri::generate_handler![
      sqlp::get, 
      sqlp::query,
      cat::concat,
      convert::switch_excel,
      convert::switch_csv,
      count::count,
      rename::get_headers,
      rename::rename,
      select::has_headers,
      select::select,
      search::search_headers,
      search::search,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
