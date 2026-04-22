#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod models;
mod overlay;
mod tray;

use dictatr_core::audio::controller::AudioController;
use dictatr_core::config;
use dictatr_core::history::HistoryStore;
use dictatr_core::config::profile::Profile;
use dictatr_core::hotkey::{HotkeyEvent, HotkeyRegistry, SharedIdMap};
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

fn apply_profiles(registry: &mut HotkeyRegistry, profiles: &[Profile]) {
    for p in profiles {
        if let Err(e) = registry.register(p.id, &p.hotkey) {
            eprintln!(
                "failed to register hotkey {} for {}: {:?}",
                p.hotkey, p.name, e
            );
        }
    }
}

// The GlobalHotKeyManager creates a hidden HWND that only receives WM_HOTKEY
// on the thread running a Win32 message pump. Tauri's main thread pumps, so we
// pin the registry there via a thread_local. The shared id_map lets the pump
// thread resolve new IDs after a reload without touching the registry itself.
thread_local! {
    static HOTKEY_REGISTRY: std::cell::RefCell<Option<HotkeyRegistry>> =
        const { std::cell::RefCell::new(None) };
}
static HOTKEY_ID_MAP: std::sync::OnceLock<SharedIdMap> = std::sync::OnceLock::new();

pub(crate) fn reload_hotkeys(profiles: &[Profile]) {
    HOTKEY_REGISTRY.with(|cell| {
        if let Some(reg) = cell.borrow_mut().as_mut() {
            reg.clear();
            apply_profiles(reg, profiles);
            if let Some(map) = HOTKEY_ID_MAP.get() {
                *map.lock() = reg.id_map();
            }
            dictatr_core::hotkey_ll::update_mapping(reg.ll_keys());
        }
    });
}

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
            #[cfg(target_os = "macos")]
            {
                dictatr_core::inject::prompt_microphone_if_needed();
                dictatr_core::inject::prompt_accessibility_if_needed();
            }

            let handle = app.handle().clone();
            tray::setup(&handle)?;

            let dirs = ProjectDirs::from("de", "dss", "Dictatr")
                .expect("could not resolve project dirs");
            let db_path = dirs.config_dir().join("history.db");
            let history = Arc::new(HistoryStore::open(&db_path)
                .expect("failed to open history db"));
            app.manage(history.clone());

            let cfg = config::load().unwrap_or_default();

            // Sync autostart registry entry with config
            {
                use tauri_plugin_autostart::ManagerExt;
                let autostart = app.autolaunch();
                if cfg.general.autostart {
                    let _ = autostart.enable();
                } else {
                    let _ = autostart.disable();
                }
            }

            let remote_url = std::env::var("DICTATR_REMOTE_URL")
                .unwrap_or_else(|_| cfg.general.remote_whisper_url.clone());
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
            let mut provider_keys: HashMap<Uuid, (dictatr_core::config::provider::LlmProviderConfig, String)> = HashMap::new();
            for p in &cfg.providers {
                if let Ok(key) = secrets::get_api_key(p.id) {
                    use dictatr_core::config::provider::ProviderType;
                    let prov: Arc<dyn LlmProvider> = match p.r#type {
                        ProviderType::Anthropic => Arc::new(AnthropicProvider::new(key.clone())),
                        ProviderType::OpenRouter | ProviderType::Openai
                        | ProviderType::OpenaiCompatible | ProviderType::Ollama =>
                            Arc::new(OpenAiCompatProvider::new(p.base_url.clone(), key.clone())),
                    };
                    providers.insert(p.id, prov);
                    provider_keys.insert(p.id, (p.clone(), key));
                }
            }

            let profiles_map: HashMap<Uuid, _> = cfg.profiles.iter()
                .map(|p| (p.id, p.clone())).collect();

            let (tx, rx) = mpsc::unbounded_channel::<HotkeyEvent>();
            let id_map_shared: SharedIdMap = Arc::new(Mutex::new(HashMap::new()));
            let _ = HOTKEY_ID_MAP.set(id_map_shared.clone());

            let pump_map = id_map_shared.clone();
            let pump_tx = tx.clone();
            std::thread::Builder::new()
                .name("dictatr-hotkey-pump".into())
                .spawn(move || HotkeyRegistry::pump_shared(pump_map, pump_tx))
                .expect("spawn hotkey pump");

            // Manager + HWND live on the Tauri main thread, which pumps Win32
            // messages for us; reloads come back here via run_on_main_thread.
            let mut registry = HotkeyRegistry::new()
                .expect("could not initialize hotkey manager");
            apply_profiles(&mut registry, &cfg.profiles);
            *id_map_shared.lock() = registry.id_map();

            // LL hook starts once with the current multimedia-key mapping and
            // stays alive for the app lifetime; Reload only swaps its mapping.
            match dictatr_core::hotkey_ll::start(registry.ll_keys(), tx.clone()) {
                Ok(hook) => { Box::leak(Box::new(hook)); }
                Err(e) => eprintln!("low-level hotkey hook failed: {e:?}"),
            }

            HOTKEY_REGISTRY.with(|cell| *cell.borrow_mut() = Some(registry));
            drop(tx);

            let audio = Arc::new(AudioController::spawn(cfg.general.max_recording_seconds));
            app.manage(audio.clone());
            app.manage(Arc::new(models::DownloadState::new()));

            let state = Arc::new(Mutex::new(AppState::Idle));
            let vocab_path = dirs.config_dir().join("vocabulary.txt");
            let vocab_initial: Vec<String> = std::fs::read_to_string(&vocab_path)
                .ok()
                .map(|s| s.lines().map(|l| l.to_string()).filter(|l| !l.trim().is_empty()).collect())
                .unwrap_or_default();
            let vocabulary = Arc::new(Mutex::new(vocab_initial));
            app.manage(vocabulary.clone());
            app.manage(commands::VocabularyPath(vocab_path));
            let mic_device = cfg.general.mic_device.clone();

            // Recording-indicator overlay: the observer shows/hides the overlay
            // on state transitions. The overlay itself polls get_audio_level via
            // IPC invoke — the Tauri v2 event bus (emit/emit_to) turned out to
            // be unreliable to the webview in this setup (see CHANGELOG v0.1.x
            // "Mic-Level-Meter"), so all level data goes through the polling
            // path that LevelMeter.tsx already uses in the main window.
            let obs_handle = app.handle().clone();
            let state_observer: Arc<dyn Fn(AppState) + Send + Sync> =
                Arc::new(move |state: AppState| {
                    let h = obs_handle.clone();
                    match state {
                        AppState::Recording => {
                            let _ = obs_handle.run_on_main_thread(move || {
                                let _ = overlay::show(&h);
                            });
                        }
                        _ => {
                            let _ = obs_handle.run_on_main_thread(move || {
                                overlay::hide(&h);
                            });
                        }
                    }
                });

            let mut orch = Orchestrator {
                audio: audio.clone(),
                profiles: profiles_map,
                remote_backend,
                local_backend,
                llm_providers: providers,
                llm_provider_keys: provider_keys,
                vocabulary,
                history: history.clone(),
                state,
                toggle_active_profile: Arc::new(Mutex::new(None)),
                mic_device,
                sounds_enabled: cfg.general.sounds,
                state_observer: Some(state_observer),
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
            commands::test_remote_whisper,
            commands::start_mic_preview,
            commands::stop_mic_preview,
            commands::get_audio_level,
            commands::get_vocabulary,
            commands::save_vocabulary,
            models::get_models_dir,
            models::list_models,
            models::start_model_download,
            models::get_download_progress,
            models::delete_model,
        ])
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
