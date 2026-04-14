#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod models;
mod overlay;
mod tray;

use dictatr_core::audio::controller::AudioController;
use dictatr_core::config;
use dictatr_core::history::HistoryStore;
use dictatr_core::hotkey::{HotkeyEvent, HotkeyRegistry};
use dictatr_core::llm::{anthropic::AnthropicProvider, openai_compat::OpenAiCompatProvider, LlmProvider};
use dictatr_core::orchestrator::Orchestrator;
use dictatr_core::secrets;
use dictatr_core::state::AppState;
use dictatr_core::transcription::{
    local::LocalWhisperBackend, remote::RemoteWhisperBackend, TranscriptionBackend,
};
use directories::ProjectDirs;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::mpsc;
use uuid::Uuid;

/// Return the path of the first installed ggml-*.bin under the models dir,
/// preferring the largest file (so large-v3 wins over base if both exist).
fn first_installed_model(dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let mut best: Option<(u64, std::path::PathBuf)> = None;
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let p = entry.path();
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.starts_with("ggml-") && name.ends_with(".bin") {
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            if best.as_ref().map_or(true, |(s, _)| size > *s) {
                best = Some((size, p));
            }
        }
    }
    best.map(|(_, p)| p)
}

fn main() {
    tauri::Builder::default()
        .on_window_event(|window, event| {
            // Keep the main window alive when the user closes it — hide instead
            // of destroy, so the tray icon can always reopen it.
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .setup(|app| {
            let handle = app.handle().clone();
            tray::setup(&handle)?;

            let dirs = ProjectDirs::from("de", "dss", "Dictatr")
                .expect("could not resolve project dirs");
            let db_path = dirs.config_dir().join("history.db");
            let history = Arc::new(HistoryStore::open(&db_path)
                .expect("failed to open history db"));
            app.manage(history.clone());

            let cfg = config::load().unwrap_or_default();

            let remote_url = std::env::var("DICTATR_REMOTE_URL")
                .unwrap_or_else(|_| "http://192.168.178.43:8000".into());
            let remote_token = std::env::var("DICTATR_REMOTE_TOKEN").unwrap_or_default();

            let remote_backend: Arc<dyn TranscriptionBackend> = Arc::new(
                RemoteWhisperBackend::new(remote_url, remote_token)
            );

            // Pick the first installed ggml-*.bin as the local model, if any.
            let models_dir = dirs.data_dir().join("models");
            let local_model = first_installed_model(&models_dir);
            let local_backend: Option<Arc<dyn TranscriptionBackend>> = match local_model {
                Some(path) => match LocalWhisperBackend::new(path.clone()) {
                    Ok(be) => {
                        eprintln!("loaded local whisper model: {:?}", path);
                        Some(Arc::new(be))
                    }
                    Err(e) => {
                        eprintln!("local whisper init failed: {e:?}");
                        None
                    }
                },
                None => {
                    eprintln!("no local whisper model installed under {:?}", models_dir);
                    None
                }
            };

            let mut providers: HashMap<Uuid, Arc<dyn LlmProvider>> = HashMap::new();
            for p in &cfg.providers {
                if let Ok(key) = secrets::get_api_key(p.id) {
                    use dictatr_core::config::provider::ProviderType;
                    let prov: Arc<dyn LlmProvider> = match p.r#type {
                        ProviderType::Anthropic => Arc::new(AnthropicProvider::new(key)),
                        ProviderType::OpenRouter | ProviderType::Openai
                        | ProviderType::OpenaiCompatible | ProviderType::Ollama =>
                            Arc::new(OpenAiCompatProvider::new(p.base_url.clone(), key)),
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
                    eprintln!("failed to register hotkey {} for {}: {:?}", p.hotkey, p.name, e);
                }
            }

            let (tx, rx) = mpsc::unbounded_channel::<HotkeyEvent>();
            let id_map = registry.id_map();
            let ll_keys = registry.ll_keys();
            // GlobalHotKeyManager is !Send (holds an HWND). Keep it alive on the
            // main thread by leaking it; the pump thread only needs the id map,
            // since GlobalHotKeyEvent::receiver() is a global channel.
            let _registry: &'static HotkeyRegistry = Box::leak(Box::new(registry));
            let tx_pump = tx.clone();
            std::thread::spawn(move || HotkeyRegistry::pump(id_map, tx_pump));

            if !ll_keys.is_empty() {
                match dictatr_core::hotkey_ll::start(ll_keys, tx.clone()) {
                    Ok(hook) => { Box::leak(Box::new(hook)); }
                    Err(e) => eprintln!("low-level hotkey hook failed: {e:?}"),
                }
            }
            drop(tx);

            let audio = Arc::new(AudioController::spawn(cfg.general.max_recording_seconds));
            app.manage(audio.clone());
            app.manage(Arc::new(models::DownloadState::new()));

            let state = Arc::new(Mutex::new(AppState::Idle));
            let vocabulary = std::fs::read_to_string(dirs.config_dir().join("vocabulary.txt"))
                .ok()
                .map(|s| s.lines().map(|l| l.to_string()).filter(|l| !l.trim().is_empty()).collect())
                .unwrap_or_default();
            let mic_device = cfg.general.mic_device.clone();

            let mut orch = Orchestrator {
                audio: audio.clone(),
                profiles: profiles_map,
                remote_backend,
                local_backend,
                llm_providers: providers,
                vocabulary,
                history: history.clone(),
                state,
                toggle_active_profile: Arc::new(Mutex::new(None)),
                mic_device,
                sounds_enabled: cfg.general.sounds,
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
            commands::test_llm_provider,
            commands::start_mic_preview,
            commands::stop_mic_preview,
            commands::get_audio_level,
            models::get_models_dir,
            models::list_models,
            models::start_model_download,
            models::get_download_progress,
            models::delete_model,
        ])
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
