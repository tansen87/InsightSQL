#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use lib::access;
use lib::behead;
use lib::cat;
use lib::convert;
use lib::count;
use lib::dbf;
use lib::fill;
use lib::modify;
use lib::offset;
use lib::rename;
use lib::search;
use lib::select;
use lib::split;
use lib::sqlp;
use lib::traverse;

fn main() {
  tauri::Builder::default()
    .plugin(tauri_plugin_os::init())
    .plugin(tauri_plugin_http::init())
    .plugin(tauri_plugin_global_shortcut::Builder::new().build())
    .plugin(tauri_plugin_clipboard_manager::init())
    .plugin(tauri_plugin_notification::init())
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_process::init())
    .plugin(tauri_plugin_shell::init())
    .plugin(tauri_plugin_dialog::init())
    .invoke_handler(tauri::generate_handler![
      sqlp::get,
      sqlp::query,
      cat::concat,
      convert::switch_excel,
      convert::switch_csv,
      count::count,
      rename::get_rename_headers,
      rename::rename,
      select::get_select_headers,
      select::select,
      search::get_search_headers,
      search::search,
      fill::get_fill_headers,
      fill::fill,
      split::split,
      access::access,
      dbf::dbf,
      behead::behead,
      modify::modify,
      traverse::traverse,
      offset::get_offset_headers,
      offset::offset,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
