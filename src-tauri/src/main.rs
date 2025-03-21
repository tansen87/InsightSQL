#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

#[cfg(target_os = "windows")]
use lib::access;
use lib::apply;
use lib::cat;
use lib::cmd;
use lib::convert;
use lib::count;
use lib::dbf;
use lib::enumerate;
use lib::fill;
use lib::join;
use lib::offset;
use lib::pinyin;
use lib::rename;
use lib::replace;
use lib::reverse;
use lib::search;
use lib::select;
use lib::skip;
use lib::slice;
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
      #[cfg(target_os = "windows")]
      access::access,
      apply::apply,
      skip::skip,
      cat::concat,
      cmd::map_headers,
      cmd::inter_headers,
      cmd::dupli_headers,
      convert::switch_excel,
      convert::map_excel_sheets,
      convert::switch_csv,
      count::count,
      dbf::dbf,
      enumerate::enumer,
      fill::fill,
      join::join,
      offset::get_offset_headers,
      offset::offset,
      pinyin::pinyin,
      rename::get_rename_headers,
      rename::rename,
      replace::replace,
      reverse::reverse,
      search::search,
      select::get_select_headers,
      select::select,
      slice::slice,
      sort::sort,
      split::split,
      sqlp::query,
      traverse::traverse,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
