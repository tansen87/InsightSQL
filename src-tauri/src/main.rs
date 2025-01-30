#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

#[cfg(target_os = "windows")]
use lib::access;
use lib::apply;
use lib::behead;
use lib::cat;
use lib::convert;
use lib::count;
use lib::dbf;
use lib::enumerate;
use lib::fill;
use lib::join;
use lib::modify;
use lib::offset;
use lib::pinyin;
use lib::rename;
use lib::replace;
use lib::search;
use lib::select;
use lib::sort;
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
      sqlp::query,
      cat::get_cat_headers,
      cat::concat,
      convert::switch_excel,
      convert::map_excel_sheets,
      convert::switch_csv,
      count::count,
      rename::get_rename_headers,
      rename::rename,
      select::get_select_headers,
      select::select,
      search::search,
      fill::fill,
      join::join,
      split::split,
      #[cfg(target_os = "windows")]
      access::access,
      dbf::dbf,
      behead::behead,
      modify::modify,
      traverse::traverse,
      offset::get_offset_headers,
      offset::offset,
      enumerate::enumer,
      pinyin::pinyin,
      replace::replace,
      apply::apply,
      sort::sort,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
