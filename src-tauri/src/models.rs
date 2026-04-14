//! Whisper GGML model download manager.
//!
//! Provides Tauri commands to list installed models, start a download from
//! huggingface, and poll progress. Progress is held in a shared state
//! because the Tauri v2 event bus did not reach the webview in this setup.

use parking_lot::Mutex;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::State;

pub struct DownloadState {
    pub progress: Mutex<DownloadProgress>,
    pub in_progress: AtomicBool,
}

impl DownloadState {
    pub fn new() -> Self {
        Self {
            progress: Mutex::new(DownloadProgress::default()),
            in_progress: AtomicBool::new(false),
        }
    }
}

#[derive(Default, Clone, Serialize)]
pub struct DownloadProgress {
    pub name: Option<String>,
    pub downloaded: u64,
    pub total: u64,
    pub done: bool,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct ModelInfo {
    pub name: String,
    pub filename: String,
    pub size_mb: u32,
    pub installed: bool,
    pub installed_bytes: u64,
}

/// Known whisper.cpp models (name → expected size in MB).
const MODELS: &[(&str, u32)] = &[
    ("tiny", 75),
    ("base", 142),
    ("small", 466),
    ("medium", 1500),
    ("large-v3", 3094),
];

fn models_dir() -> PathBuf {
    let dirs = directories::ProjectDirs::from("de", "dss", "Dictatr")
        .expect("project dirs");
    dirs.data_dir().join("models")
}

fn file_for(name: &str) -> (String, PathBuf) {
    let filename = format!("ggml-{name}.bin");
    let path = models_dir().join(&filename);
    (filename, path)
}

#[tauri::command]
pub fn get_models_dir() -> String {
    models_dir().to_string_lossy().into_owned()
}

#[tauri::command]
pub fn list_models() -> Vec<ModelInfo> {
    MODELS
        .iter()
        .map(|(name, size_mb)| {
            let (filename, path) = file_for(name);
            let installed_bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            ModelInfo {
                name: (*name).to_string(),
                filename,
                size_mb: *size_mb,
                installed: installed_bytes > 0,
                installed_bytes,
            }
        })
        .collect()
}

#[tauri::command]
pub async fn start_model_download(
    state: State<'_, Arc<DownloadState>>,
    name: String,
) -> Result<(), String> {
    if !MODELS.iter().any(|(n, _)| *n == name) {
        return Err(format!("unbekanntes Modell: {name}"));
    }
    if state
        .in_progress
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err("Ein Download läuft bereits".into());
    }

    let state_clone = state.inner().clone();
    let name_clone = name.clone();
    {
        let mut p = state.progress.lock();
        *p = DownloadProgress {
            name: Some(name.clone()),
            downloaded: 0,
            total: 0,
            done: false,
            error: None,
        };
    }

    tauri::async_runtime::spawn(async move {
        let result = download_model(&name_clone, &state_clone).await;
        {
            let mut p = state_clone.progress.lock();
            match result {
                Ok(()) => p.done = true,
                Err(e) => {
                    p.error = Some(e);
                    p.done = true;
                }
            }
        }
        state_clone.in_progress.store(false, Ordering::SeqCst);
    });

    Ok(())
}

#[tauri::command]
pub fn get_download_progress(state: State<'_, Arc<DownloadState>>) -> DownloadProgress {
    state.progress.lock().clone()
}

async fn download_model(name: &str, state: &Arc<DownloadState>) -> Result<(), String> {
    use tokio::io::AsyncWriteExt;

    let (_filename, dest) = file_for(name);
    std::fs::create_dir_all(dest.parent().unwrap())
        .map_err(|e| format!("Verzeichnis anlegen fehlgeschlagen: {e}"))?;

    let url = format!(
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{name}.bin"
    );
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60 * 30))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let total = resp.content_length().unwrap_or(0);
    {
        let mut p = state.progress.lock();
        p.total = total;
    }

    // Write atomically via a .part file, rename on success so a crashed
    // download doesn't leave a truncated ggml-*.bin that the app would then
    // try to load.
    let tmp = dest.with_extension("bin.part");
    let mut out = tokio::fs::File::create(&tmp)
        .await
        .map_err(|e| format!("Datei anlegen: {e}"))?;

    let mut stream = resp.bytes_stream();
    let mut downloaded: u64 = 0;
    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        out.write_all(&chunk).await.map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;
        state.progress.lock().downloaded = downloaded;
    }
    out.flush().await.map_err(|e| e.to_string())?;
    drop(out);

    tokio::fs::rename(&tmp, &dest)
        .await
        .map_err(|e| format!("Rename fehlgeschlagen: {e}"))?;

    Ok(())
}

#[tauri::command]
pub fn delete_model(name: String) -> Result<(), String> {
    let (_filename, path) = file_for(&name);
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}
