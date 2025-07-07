#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use lib::command;

#[cfg(target_os = "windows")]
use lib::cmd::access;
use lib::cmd::apply;
use lib::cmd::cat;
use lib::cmd::count;
use lib::cmd::dbf;
use lib::cmd::enumerate;
use lib::cmd::extsort;
use lib::cmd::fill;
use lib::cmd::idx;
use lib::cmd::join;
use lib::cmd::pinyin;
use lib::cmd::rename;
use lib::cmd::replace;
use lib::cmd::reverse;
use lib::cmd::search;
use lib::cmd::select;
use lib::cmd::skip;
use lib::cmd::slice;
use lib::cmd::sort;
use lib::cmd::split;
use lib::cmd::sqlp;
use lib::cmd::to_csv;
use lib::cmd::to_excel;
use lib::cmd::transpose;
use lib::cmd::traverse;

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
      command::from_headers,
      command::map_headers,
      command::inter_headers,
      command::dupli_headers,
      command::to_json,
      #[cfg(target_os = "windows")]
      access::access,
      apply::apply,
      skip::skip,
      cat::concat,
      count::count,
      dbf::dbf,
      enumerate::enumer,
      extsort::extsort,
      fill::fill,
      idx::idx,
      join::join,
      pinyin::pinyin,
      rename::rename,
      replace::replace,
      reverse::reverse,
      search::search,
      select::select,
      slice::slice,
      sort::sort,
      split::split,
      sqlp::query,
      to_csv::excel2csv,
      to_csv::map_excel_sheets,
      to_excel::csv2xlsx,
      transpose::transpose,
      traverse::traverse,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
