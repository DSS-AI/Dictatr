use dictatr_core::audio::capture::AudioCapture;
use dictatr_core::config::{self, AppConfig};
use dictatr_core::history::{HistoryEntry, HistoryStore};
use dictatr_core::secrets;
use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn get_config() -> std::result::Result<AppConfig, String> {
    config::load().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_config(cfg: AppConfig) -> std::result::Result<(), String> {
    config::save(&cfg).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_api_key(provider_id: Uuid, key: String) -> std::result::Result<(), String> {
    secrets::set_api_key(provider_id, &key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_input_devices() -> std::result::Result<Vec<String>, String> {
    AudioCapture::list_input_devices().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_history(store: State<'_, Arc<HistoryStore>>, limit: u32) -> std::result::Result<Vec<HistoryEntry>, String> {
    store.list(limit).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_history(store: State<'_, Arc<HistoryStore>>, id: i64) -> std::result::Result<(), String> {
    store.delete(id).map_err(|e| e.to_string())
}
