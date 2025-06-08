#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use lib::cmd;

#[cfg(target_os = "windows")]
use lib::command::access;
use lib::command::apply;
use lib::command::cat;
use lib::command::convert;
use lib::command::count;
use lib::command::dbf;
use lib::command::enumerate;
use lib::command::extsort;
use lib::command::fill;
use lib::command::idx;
use lib::command::join;
use lib::command::offset;
use lib::command::pinyin;
use lib::command::rename;
use lib::command::replace;
use lib::command::reverse;
use lib::command::search;
use lib::command::select;
use lib::command::skip;
use lib::command::slice;
use lib::command::sort;
use lib::command::split;
use lib::command::sqlp;
use lib::command::transpose;
use lib::command::traverse;

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
      cmd::from_headers,
      cmd::map_headers,
      cmd::inter_headers,
      cmd::dupli_headers,
      cmd::to_json,
      #[cfg(target_os = "windows")]
      access::access,
      apply::apply,
      skip::skip,
      cat::concat,
      convert::switch_excel,
      convert::map_excel_sheets,
      convert::switch_csv,
      count::count,
      dbf::dbf,
      enumerate::enumer,
      extsort::extsort,
      fill::fill,
      idx::idx,
      join::join,
      offset::get_offset_headers,
      offset::offset,
      pinyin::pinyin,
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
      transpose::transpose,
      traverse::traverse,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
