#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod overlay;
mod tray;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            tray::setup(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_config,
            commands::set_api_key,
            commands::list_input_devices,
            commands::list_history,
            commands::delete_history,
        ])
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
