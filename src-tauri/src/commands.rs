use dictatr_core::audio::capture::AudioCapture;
use dictatr_core::audio::controller::AudioController;
use dictatr_core::config::{self, provider::ProviderType, AppConfig};
use dictatr_core::history::{HistoryEntry, HistoryStore};
use dictatr_core::llm::{
    anthropic::AnthropicProvider, openai_compat::OpenAiCompatProvider, LlmProvider,
};
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

#[tauri::command]
pub async fn test_llm_provider(provider_id: Uuid) -> std::result::Result<String, String> {
    let cfg = config::load().map_err(|e| e.to_string())?;
    let p = cfg.providers.iter().find(|p| p.id == provider_id)
        .ok_or_else(|| "Provider nicht in Konfiguration gefunden".to_string())?;
    let key = secrets::get_api_key(provider_id)
        .map_err(|e| format!("Kein API-Key gespeichert ({e})"))?;

    let model = if p.default_model.trim().is_empty() {
        return Err("Kein Default-Modell gesetzt".to_string());
    } else {
        p.default_model.clone()
    };

    let provider: Arc<dyn LlmProvider> = match p.r#type {
        ProviderType::Anthropic => Arc::new(AnthropicProvider::new(key)),
        ProviderType::OpenRouter | ProviderType::Openai
        | ProviderType::OpenaiCompatible | ProviderType::Ollama =>
            Arc::new(OpenAiCompatProvider::new(p.base_url.clone(), key)),
    };

    let reply = provider
        .complete("You are a healthcheck. Reply with a single word: ok", "ping", &model)
        .await
        .map_err(|e| e.to_string())?;
    Ok(reply.trim().chars().take(200).collect())
}

#[tauri::command]
pub async fn start_mic_preview(
    audio: State<'_, Arc<AudioController>>,
    device: Option<String>,
) -> std::result::Result<(), String> {
    audio.start_recording(device).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_audio_level(audio: State<'_, Arc<AudioController>>) -> f32 {
    audio.level_snapshot()
}

#[tauri::command]
pub async fn stop_mic_preview(
    audio: State<'_, Arc<AudioController>>,
) -> std::result::Result<(), String> {
    // Drain and discard — we only wanted the live level meter.
    audio.stop_and_drain().await.map(|_| ()).map_err(|e| e.to_string())
}
