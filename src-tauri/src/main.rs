#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod overlay;
mod tray;

use dss_whisper_core::audio::controller::AudioController;
use dss_whisper_core::config;
use dss_whisper_core::history::HistoryStore;
use dss_whisper_core::hotkey::{HotkeyEvent, HotkeyRegistry};
use dss_whisper_core::llm::{anthropic::AnthropicProvider, openai_compat::OpenAiCompatProvider, LlmProvider};
use dss_whisper_core::orchestrator::Orchestrator;
use dss_whisper_core::secrets;
use dss_whisper_core::state::AppState;
use dss_whisper_core::transcription::{
    local::LocalWhisperBackend, remote::RemoteWhisperBackend, TranscriptionBackend,
};
use directories::ProjectDirs;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::mpsc;
use uuid::Uuid;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            tray::setup(&handle)?;

            let dirs = ProjectDirs::from("de", "dss", "Whisper")
                .expect("could not resolve project dirs");
            let db_path = dirs.config_dir().join("history.db");
            let history = Arc::new(HistoryStore::open(&db_path)
                .expect("failed to open history db"));
            app.manage(history.clone());

            let cfg = config::load().unwrap_or_default();

            let remote_url = std::env::var("DSS_WHISPER_REMOTE_URL")
                .unwrap_or_else(|_| "http://192.168.178.43:8503".into());
            let remote_token = std::env::var("DSS_WHISPER_REMOTE_TOKEN").unwrap_or_default();

            let primary: Arc<dyn TranscriptionBackend> = Arc::new(
                RemoteWhisperBackend::new(remote_url, remote_token)
            );

            // Fallback: use LocalWhisperBackend if model file is present, otherwise reuse primary.
            let model_path = dirs.data_dir().join("models").join("ggml-base.bin");
            let fallback: Arc<dyn TranscriptionBackend> = if model_path.exists() {
                match LocalWhisperBackend::new(model_path.clone()) {
                    Ok(be) => Arc::new(be),
                    Err(e) => {
                        eprintln!("local whisper init failed, falling back to remote: {e:?}");
                        primary.clone()
                    }
                }
            } else {
                eprintln!(
                    "local whisper model not found at {:?}, using remote only",
                    model_path
                );
                primary.clone()
            };

            let mut providers: HashMap<Uuid, Arc<dyn LlmProvider>> = HashMap::new();
            for p in &cfg.providers {
                if let Ok(key) = secrets::get_api_key(p.id) {
                    let prov: Arc<dyn LlmProvider> = match p.r#type {
                        dss_whisper_core::config::provider::ProviderType::Anthropic =>
                            Arc::new(AnthropicProvider::new(key)),
                        _ => Arc::new(OpenAiCompatProvider::new(p.base_url.clone(), key)),
                    };
                    providers.insert(p.id, prov);
                }
            }

            let profiles_map: HashMap<Uuid, _> = cfg.profiles.iter()
                .map(|p| (p.id, p.clone())).collect();

            let mut registry = HotkeyRegistry::new()
                .expect("could not initialize hotkey manager");
            for p in &cfg.profiles {
                if let Err(e) = registry.register(p.id, &p.hotkey) {
                    eprintln!("failed to register hotkey {}: {:?}", p.hotkey, e);
                }
            }

            let (tx, rx) = mpsc::unbounded_channel::<HotkeyEvent>();
            std::thread::spawn(move || registry.pump_into(tx));

            let audio = AudioController::spawn(cfg.general.max_recording_seconds);
            let level_ref = audio.level.clone();

            let handle_levels = handle.clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                    let v = *level_ref.lock();
                    let _ = handle_levels.emit("audio://level", v);
                }
            });

            let state = Arc::new(Mutex::new(AppState::Idle));
            let vocabulary = std::fs::read_to_string(dirs.config_dir().join("vocabulary.txt"))
                .ok()
                .map(|s| s.lines().map(|l| l.to_string()).filter(|l| !l.trim().is_empty()).collect())
                .unwrap_or_default();
            let mic_device = cfg.general.mic_device.clone();

            let mut orch = Orchestrator {
                audio,
                profiles: profiles_map,
                primary_backend: primary,
                fallback_backend: fallback,
                llm_providers: providers,
                vocabulary,
                history: history.clone(),
                state,
                toggle_active_profile: Arc::new(Mutex::new(None)),
                mic_device,
            };
            tauri::async_runtime::spawn(async move { orch.run_loop(rx).await; });

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
