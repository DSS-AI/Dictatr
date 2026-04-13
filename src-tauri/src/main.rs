#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dss_whisper_core as _;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
