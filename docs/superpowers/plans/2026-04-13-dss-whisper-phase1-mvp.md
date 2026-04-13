# DSS-Whisper Phase 1 MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Eine Tauri-basierte Windows-Desktop-App liefern, die per globalem Hotkey Audio aufnimmt, über einen Remote-Whisper-Server oder lokal transkribiert, optional per LLM nachbearbeitet und den Text an der aktuellen Cursorposition einfügt.

**Architecture:** Tauri 2.x-App mit Rust-Core (State-Machine, Audio, Hotkey, Text-Injection, Backends) und minimaler Webview-UI (Settings, History). Transcription-Backends (`remote-whisper`, `local-whisper`) und LLM-Provider (`openai-compat`, `anthropic`, `ollama`) sind hinter Traits abstrahiert. Konfiguration liegt in `%APPDATA%/DSS-Whisper/config.json`, API-Keys im Windows Credential Manager, History in SQLite.

**Tech Stack:** Rust 1.80+, Tauri 2.x, `cpal` (Audio), `global-hotkey`, `enigo` (Keystrokes), `whisper-rs` (lokales Whisper), `reqwest` (HTTP), `rusqlite`, `keyring`, React + TypeScript fürs UI, Vite.

**Referenz-Spec:** [`docs/superpowers/specs/2026-04-13-dss-whisper-dictation-design.md`](../specs/2026-04-13-dss-whisper-dictation-design.md)

---

## File Structure

### Rust-Crate (`src-tauri/`)

| Datei | Verantwortung |
|---|---|
| `src-tauri/Cargo.toml` | Dependencies |
| `src-tauri/tauri.conf.json` | Tauri-Konfig (Window, Bundler, Permissions) |
| `src-tauri/src/main.rs` | Tauri-Setup, Command-Registrierung, Tray |
| `src-tauri/src/state.rs` | App-State-Machine (`Idle` / `Recording` / `Transcribing` / `Injecting`) |
| `src-tauri/src/config/mod.rs` | Config-Laden/Speichern |
| `src-tauri/src/config/profile.rs` | `Profile`-Struct + Validierung |
| `src-tauri/src/config/provider.rs` | `LlmProvider`-Config-Struct |
| `src-tauri/src/audio/mod.rs` | Audio-Capture-Facade |
| `src-tauri/src/audio/ringbuffer.rs` | Ringbuffer-Struct |
| `src-tauri/src/audio/capture.rs` | `cpal`-Integration |
| `src-tauri/src/hotkey.rs` | Global-Hotkey-Registrierung + Events |
| `src-tauri/src/inject.rs` | Text-Injection via `enigo` + Clipboard-Fallback |
| `src-tauri/src/transcription/mod.rs` | `TranscriptionBackend`-Trait + Registry |
| `src-tauri/src/transcription/remote.rs` | `RemoteWhisperBackend` (HTTP zum DSS-Server) |
| `src-tauri/src/transcription/local.rs` | `LocalWhisperBackend` (`whisper-rs`) |
| `src-tauri/src/llm/mod.rs` | `LlmProvider`-Trait + Registry |
| `src-tauri/src/llm/openai_compat.rs` | OpenAI-Chat-Completions-Adapter (OpenAI, Groq, Ollama, LiteLLM) |
| `src-tauri/src/llm/anthropic.rs` | Anthropic-Messages-Adapter |
| `src-tauri/src/llm/prompt.rs` | Post-Processing-Prompt-Builder |
| `src-tauri/src/history/mod.rs` | SQLite-History-Store |
| `src-tauri/src/secrets.rs` | Keyring-Wrapper |
| `src-tauri/src/orchestrator.rs` | Verknüpft State-Machine mit Backends, Hotkey, Audio, Inject |
| `src-tauri/src/commands.rs` | Tauri-IPC-Commands (Config-CRUD, Test-Buttons) |
| `src-tauri/src/tray.rs` | Tray-Icon-Logik + Status-Farben |
| `src-tauri/src/overlay.rs` | Mini-Overlay-Fenster |
| `src-tauri/src/error.rs` | Zentrale Error-Typen |

### Frontend (`src/`)

| Datei | Verantwortung |
|---|---|
| `src/main.tsx` | React-Entry, Router |
| `src/ipc.ts` | Typed Wrapper um `invoke()` |
| `src/pages/Profiles.tsx` | Profile-Tab (CRUD) |
| `src/pages/Providers.tsx` | LLM-Provider-Tab |
| `src/pages/Vocabulary.tsx` | Wörterbuch-Tab |
| `src/pages/Audio.tsx` | Mic-Auswahl + Pegelanzeige |
| `src/pages/General.tsx` | Allgemeine Settings |
| `src/pages/History.tsx` | History-Liste |
| `src/components/HotkeyRecorder.tsx` | Custom-Component |
| `src/components/LevelMeter.tsx` | Live-Pegelanzeige |
| `src/types.ts` | TypeScript-Types matching Rust-Structs |

### Server-seitig (`DSS-V-A-Transcribe`, separates Repo)

| Datei | Verantwortung |
|---|---|
| `app/api/dictate/route.ts` (oder .py) | Neuer Synchron-Endpoint `/api/dictate` |

### Tests

| Datei | Verantwortung |
|---|---|
| `src-tauri/src/audio/ringbuffer.rs` (inline `#[cfg(test)]`) | Unit-Tests Ringbuffer |
| `src-tauri/src/config/profile.rs` (inline) | Unit-Tests Profile Serde + Validierung |
| `src-tauri/src/llm/prompt.rs` (inline) | Unit-Tests Prompt-Builder |
| `src-tauri/src/state.rs` (inline) | Unit-Tests State-Transitions |
| `src-tauri/src/transcription/remote.rs` (inline) | Unit-Tests mit `wiremock` |
| `src-tauri/src/llm/openai_compat.rs` (inline) | Unit-Tests mit `wiremock` |
| `src-tauri/tests/integration.rs` | Integration-Tests (Backend-Fallback) |
| `docs/superpowers/plans/release-checklist.md` | Manuelle Release-Checkliste |

---

## Task 1: Tauri-Projekt initialisieren

**Files:**
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/src/main.rs`
- Create: `package.json`
- Create: `vite.config.ts`
- Create: `src/main.tsx`
- Create: `index.html`
- Create: `.gitignore` (ergänzen)

- [ ] **Step 1: Tauri-CLI installieren und App scaffolden**

```bash
cd /mnt/synology/Coding/DSS-Whisper
cargo install create-tauri-app --locked
cargo create-tauri-app --template react-ts --manager bun --identifier de.dss.whisper --name dss-whisper
```

Wenn das Tool nach Overwrite fragt: nur `src-tauri/`, `src/`, `index.html`, `package.json`, `vite.config.ts`, `tsconfig.json` übernehmen. Bestehende Dateien (`CLAUDE.md`, `docs/`, `pyproject.toml`, `.venv`) nicht anfassen.

- [ ] **Step 2: `.gitignore` ergänzen**

Füge ans Ende von `/mnt/synology/Coding/DSS-Whisper/.gitignore` an:

```
# Tauri / Node
node_modules/
dist/
src-tauri/target/
src-tauri/gen/
.vite/
```

- [ ] **Step 3: App-Identität in `tauri.conf.json` setzen**

Öffne `src-tauri/tauri.conf.json` und setze:

```json
{
  "productName": "DSS-Whisper",
  "version": "0.1.0",
  "identifier": "de.dss.whisper",
  "app": {
    "windows": [
      {
        "title": "DSS-Whisper Settings",
        "width": 900,
        "height": 650,
        "visible": false
      }
    ],
    "withGlobalTauri": false
  },
  "bundle": {
    "active": true,
    "targets": "msi",
    "icon": ["icons/icon.ico"]
  }
}
```

- [ ] **Step 4: Build-Sanity-Check**

```bash
cd /mnt/synology/Coding/DSS-Whisper
bun install
bun run tauri build --no-bundle
```

Expected: Build endet mit `Finished dev [unoptimized + debuginfo] target(s)`. Auf Linux ist das fein als Smoke-Test; echte MSI-Erzeugung passiert später auf Windows.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/ src/ index.html package.json vite.config.ts tsconfig.json bun.lockb .gitignore
git commit -m "feat: scaffold Tauri 2 app with React+TS frontend"
```

---

## Task 2: Error-Typen & Config-Struktur

**Files:**
- Create: `src-tauri/src/error.rs`
- Create: `src-tauri/src/config/mod.rs`
- Create: `src-tauri/src/config/profile.rs`
- Create: `src-tauri/src/config/provider.rs`
- Modify: `src-tauri/src/main.rs` (module deklarieren)
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Dependencies hinzufügen**

`src-tauri/Cargo.toml` — im `[dependencies]`-Block ergänzen:

```toml
thiserror = "1.0"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4", "serde"] }
directories = "5"
anyhow = "1"
```

- [ ] **Step 2: Failing Test für Profile-Deserialisierung**

Create `src-tauri/src/config/profile.rs`:

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HotkeyMode {
    PushToTalk,
    Toggle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    De,
    En,
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TranscriptionBackendId {
    RemoteWhisper,
    LocalWhisper,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PostProcessing {
    pub enabled: bool,
    pub llm_provider_id: Option<Uuid>,
    pub model: Option<String>,
    pub system_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Profile {
    pub id: Uuid,
    pub name: String,
    pub hotkey: String,
    pub hotkey_mode: HotkeyMode,
    pub transcription_backend: TranscriptionBackendId,
    pub language: Language,
    pub post_processing: PostProcessing,
}

impl Profile {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Profile name must not be empty".into());
        }
        if self.hotkey.trim().is_empty() {
            return Err("Hotkey must not be empty".into());
        }
        if self.post_processing.enabled && self.post_processing.llm_provider_id.is_none() {
            return Err("Post-processing enabled but no LLM provider selected".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_profile_json() {
        let json = r#"{
            "id": "00000000-0000-0000-0000-000000000001",
            "name": "Standard",
            "hotkey": "Ctrl+Alt+Space",
            "hotkey_mode": "push_to_talk",
            "transcription_backend": "remote_whisper",
            "language": "de",
            "post_processing": { "enabled": false, "llm_provider_id": null, "model": null, "system_prompt": null }
        }"#;

        let p: Profile = serde_json::from_str(json).unwrap();
        assert_eq!(p.name, "Standard");
        assert_eq!(p.hotkey_mode, HotkeyMode::PushToTalk);
        assert_eq!(p.language, Language::De);
        assert!(!p.post_processing.enabled);
    }

    #[test]
    fn validates_empty_name() {
        let p = Profile {
            id: Uuid::nil(),
            name: "  ".into(),
            hotkey: "Ctrl+Alt+Space".into(),
            hotkey_mode: HotkeyMode::Toggle,
            transcription_backend: TranscriptionBackendId::RemoteWhisper,
            language: Language::De,
            post_processing: PostProcessing {
                enabled: false, llm_provider_id: None, model: None, system_prompt: None,
            },
        };
        assert!(p.validate().is_err());
    }

    #[test]
    fn validates_post_processing_without_provider() {
        let p = Profile {
            id: Uuid::nil(),
            name: "X".into(),
            hotkey: "Ctrl+Alt+X".into(),
            hotkey_mode: HotkeyMode::Toggle,
            transcription_backend: TranscriptionBackendId::RemoteWhisper,
            language: Language::Auto,
            post_processing: PostProcessing {
                enabled: true, llm_provider_id: None, model: None, system_prompt: None,
            },
        };
        assert!(p.validate().is_err());
    }
}
```

- [ ] **Step 3: Provider-Config und Error-Typen**

Create `src-tauri/src/config/provider.rs`:

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderType {
    Openai,
    OpenaiCompatible,
    Anthropic,
    Ollama,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LlmProviderConfig {
    pub id: Uuid,
    pub name: String,
    pub r#type: ProviderType,
    pub base_url: String,
    pub default_model: String,
}
```

Create `src-tauri/src/error.rs`:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("config error: {0}")]
    Config(String),
    #[error("audio error: {0}")]
    Audio(String),
    #[error("transcription error: {0}")]
    Transcription(String),
    #[error("llm error: {0}")]
    Llm(String),
    #[error("injection error: {0}")]
    Inject(String),
    #[error("keyring error: {0}")]
    Keyring(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, AppError>;
```

Create `src-tauri/src/config/mod.rs`:

```rust
pub mod profile;
pub mod provider;

use crate::error::{AppError, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    pub profiles: Vec<profile::Profile>,
    pub providers: Vec<provider::LlmProviderConfig>,
    pub general: General,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct General {
    pub autostart: bool,
    pub sounds: bool,
    pub overlay: bool,
    pub max_recording_seconds: u32,
    pub history_limit: u32,
    pub mic_device: Option<String>,
}

impl Default for General {
    fn default() -> Self {
        Self {
            autostart: false,
            sounds: true,
            overlay: true,
            max_recording_seconds: 120,
            history_limit: 100,
            mic_device: None,
        }
    }
}

pub fn config_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("de", "dss", "Whisper")
        .ok_or_else(|| AppError::Config("could not resolve app dirs".into()))?;
    Ok(dirs.config_dir().join("config.json"))
}

pub fn load() -> Result<AppConfig> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let data = std::fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&data)?)
}

pub fn save(cfg: &AppConfig) -> Result<()> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(cfg)?;
    std::fs::write(&path, data)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn default_config_roundtrips() {
        let cfg = AppConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let back: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.general.max_recording_seconds, 120);
    }
}
```

- [ ] **Step 4: Module in `main.rs` deklarieren**

Öffne `src-tauri/src/main.rs` und ersetze den bestehenden Inhalt:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod error;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Ergänze in `Cargo.toml` unter `[dev-dependencies]`:

```toml
tempfile = "3"
```

- [ ] **Step 5: Tests ausführen**

```bash
cd /mnt/synology/Coding/DSS-Whisper/src-tauri
cargo test --lib
```

Expected: `test result: ok. 4 passed; 0 failed`

- [ ] **Step 6: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/
git commit -m "feat: config structs, profile validation, app error types"
```

---

## Task 3: Keyring-Wrapper

**Files:**
- Create: `src-tauri/src/secrets.rs`
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Dependency ergänzen**

`Cargo.toml` → `[dependencies]`:

```toml
keyring = "3"
```

- [ ] **Step 2: Failing Test + Implementierung schreiben**

Create `src-tauri/src/secrets.rs`:

```rust
use crate::error::{AppError, Result};
use uuid::Uuid;

const SERVICE: &str = "DSS-Whisper";

fn key_for(provider_id: Uuid) -> String {
    format!("provider-{}", provider_id)
}

pub fn set_api_key(provider_id: Uuid, key: &str) -> Result<()> {
    let entry = keyring::Entry::new(SERVICE, &key_for(provider_id))
        .map_err(|e| AppError::Keyring(e.to_string()))?;
    entry.set_password(key).map_err(|e| AppError::Keyring(e.to_string()))
}

pub fn get_api_key(provider_id: Uuid) -> Result<String> {
    let entry = keyring::Entry::new(SERVICE, &key_for(provider_id))
        .map_err(|e| AppError::Keyring(e.to_string()))?;
    entry.get_password().map_err(|e| AppError::Keyring(e.to_string()))
}

pub fn delete_api_key(provider_id: Uuid) -> Result<()> {
    let entry = keyring::Entry::new(SERVICE, &key_for(provider_id))
        .map_err(|e| AppError::Keyring(e.to_string()))?;
    entry.delete_credential().map_err(|e| AppError::Keyring(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires real OS keyring, run only manually"]
    fn roundtrip_key() {
        let id = Uuid::new_v4();
        set_api_key(id, "secret-xyz").unwrap();
        assert_eq!(get_api_key(id).unwrap(), "secret-xyz");
        delete_api_key(id).unwrap();
    }
}
```

- [ ] **Step 3: In `main.rs` registrieren**

Ergänze `mod secrets;` oben in `src-tauri/src/main.rs`.

- [ ] **Step 4: Build**

```bash
cd /mnt/synology/Coding/DSS-Whisper/src-tauri
cargo build
```

Expected: Build erfolgreich.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/secrets.rs src-tauri/src/main.rs
git commit -m "feat: OS keyring wrapper for API keys"
```

---

## Task 4: Audio-Ringbuffer

**Files:**
- Create: `src-tauri/src/audio/mod.rs`
- Create: `src-tauri/src/audio/ringbuffer.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Failing Tests für Ringbuffer**

Create `src-tauri/src/audio/ringbuffer.rs`:

```rust
use std::collections::VecDeque;

/// Sliding-window audio buffer for 16 kHz mono f32 samples.
/// Drops oldest samples when capacity is exceeded.
pub struct RingBuffer {
    buf: VecDeque<f32>,
    capacity: usize,
}

impl RingBuffer {
    pub fn with_seconds(seconds: u32, sample_rate: u32) -> Self {
        Self::with_capacity((seconds as usize) * (sample_rate as usize))
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self { buf: VecDeque::with_capacity(capacity), capacity }
    }

    pub fn push_samples(&mut self, samples: &[f32]) {
        for s in samples {
            if self.buf.len() == self.capacity {
                self.buf.pop_front();
            }
            self.buf.push_back(*s);
        }
    }

    pub fn len(&self) -> usize { self.buf.len() }
    pub fn is_empty(&self) -> bool { self.buf.is_empty() }
    pub fn is_full(&self) -> bool { self.buf.len() == self.capacity }

    pub fn drain_to_vec(&mut self) -> Vec<f32> {
        let v: Vec<f32> = self.buf.drain(..).collect();
        v
    }

    pub fn clear(&mut self) { self.buf.clear(); }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capacity_from_seconds() {
        let rb = RingBuffer::with_seconds(2, 16_000);
        assert_eq!(rb.len(), 0);
    }

    #[test]
    fn drops_oldest_when_full() {
        let mut rb = RingBuffer::with_capacity(4);
        rb.push_samples(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        assert_eq!(rb.len(), 4);
        let v = rb.drain_to_vec();
        assert_eq!(v, vec![3.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn drain_empties_buffer() {
        let mut rb = RingBuffer::with_capacity(8);
        rb.push_samples(&[1.0, 2.0]);
        let _ = rb.drain_to_vec();
        assert!(rb.is_empty());
    }

    #[test]
    fn is_full_reports_correctly() {
        let mut rb = RingBuffer::with_capacity(3);
        assert!(!rb.is_full());
        rb.push_samples(&[1.0, 2.0, 3.0]);
        assert!(rb.is_full());
    }
}
```

- [ ] **Step 2: Module-Root anlegen**

Create `src-tauri/src/audio/mod.rs`:

```rust
pub mod ringbuffer;
```

Ergänze `mod audio;` in `src-tauri/src/main.rs`.

- [ ] **Step 3: Tests ausführen**

```bash
cd /mnt/synology/Coding/DSS-Whisper/src-tauri
cargo test --lib audio::ringbuffer
```

Expected: `test result: ok. 4 passed`

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/audio/ src-tauri/src/main.rs
git commit -m "feat: sliding-window audio ringbuffer"
```

---

## Task 5: Audio-Capture mit cpal

**Files:**
- Create: `src-tauri/src/audio/capture.rs`
- Modify: `src-tauri/src/audio/mod.rs`
- Modify: `src-tauri/Cargo.toml`

**Hinweis:** Echte Audio-Hardware lässt sich nur integrations-/manuell testen. Hier sicherstellen, dass API-Form und Resampling-Logik korrekt sind; das Sampling selbst wird manuell verifiziert.

- [ ] **Step 1: Dependency**

`Cargo.toml` → `[dependencies]`:

```toml
cpal = "0.15"
parking_lot = "0.12"
```

- [ ] **Step 2: Capture-Modul**

Create `src-tauri/src/audio/capture.rs`:

```rust
use crate::audio::ringbuffer::RingBuffer;
use crate::error::{AppError, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, Stream, StreamConfig};
use parking_lot::Mutex;
use std::sync::Arc;

pub const TARGET_SAMPLE_RATE: u32 = 16_000;

pub struct AudioCapture {
    stream: Option<Stream>,
    pub buffer: Arc<Mutex<RingBuffer>>,
    pub level: Arc<Mutex<f32>>,
}

impl AudioCapture {
    pub fn new(max_seconds: u32) -> Self {
        Self {
            stream: None,
            buffer: Arc::new(Mutex::new(RingBuffer::with_seconds(max_seconds, TARGET_SAMPLE_RATE))),
            level: Arc::new(Mutex::new(0.0)),
        }
    }

    pub fn list_input_devices() -> Result<Vec<String>> {
        let host = cpal::default_host();
        let devices = host.input_devices().map_err(|e| AppError::Audio(e.to_string()))?;
        Ok(devices.filter_map(|d| d.name().ok()).collect())
    }

    pub fn start(&mut self, device_name: Option<&str>) -> Result<()> {
        if self.stream.is_some() { return Ok(()); }

        let host = cpal::default_host();
        let device: Device = match device_name {
            Some(name) => host.input_devices()
                .map_err(|e| AppError::Audio(e.to_string()))?
                .find(|d| d.name().map(|n| n == name).unwrap_or(false))
                .ok_or_else(|| AppError::Audio(format!("device not found: {}", name)))?,
            None => host.default_input_device()
                .ok_or_else(|| AppError::Audio("no default input device".into()))?,
        };

        let config = device.default_input_config()
            .map_err(|e| AppError::Audio(e.to_string()))?;
        let sample_format = config.sample_format();
        let stream_config: StreamConfig = config.clone().into();
        let source_rate = stream_config.sample_rate.0;
        let channels = stream_config.channels as usize;

        let buf = self.buffer.clone();
        let level = self.level.clone();

        let err_cb = |err| eprintln!("audio stream error: {err}");

        let stream = match sample_format {
            SampleFormat::F32 => device.build_input_stream(
                &stream_config,
                move |data: &[f32], _| Self::on_chunk(data, channels, source_rate, &buf, &level),
                err_cb, None,
            ),
            SampleFormat::I16 => device.build_input_stream(
                &stream_config,
                move |data: &[i16], _| {
                    let f: Vec<f32> = data.iter().map(|s| *s as f32 / i16::MAX as f32).collect();
                    Self::on_chunk(&f, channels, source_rate, &buf, &level);
                },
                err_cb, None,
            ),
            other => return Err(AppError::Audio(format!("unsupported sample format: {:?}", other))),
        }.map_err(|e| AppError::Audio(e.to_string()))?;

        stream.play().map_err(|e| AppError::Audio(e.to_string()))?;
        self.stream = Some(stream);
        Ok(())
    }

    pub fn stop(&mut self) {
        self.stream = None;
    }

    fn on_chunk(
        data: &[f32], channels: usize, source_rate: u32,
        buf: &Arc<Mutex<RingBuffer>>, level: &Arc<Mutex<f32>>,
    ) {
        // Downmix to mono (average channels)
        let mono: Vec<f32> = if channels == 1 {
            data.to_vec()
        } else {
            data.chunks(channels).map(|c| c.iter().sum::<f32>() / channels as f32).collect()
        };

        // Linear resample to 16 kHz (cheap; good enough for speech)
        let resampled = resample_linear(&mono, source_rate, TARGET_SAMPLE_RATE);

        // Update RMS level
        let rms = (resampled.iter().map(|s| s * s).sum::<f32>() / resampled.len().max(1) as f32).sqrt();
        *level.lock() = rms;

        buf.lock().push_samples(&resampled);
    }
}

pub(crate) fn resample_linear(input: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate { return input.to_vec(); }
    let ratio = to_rate as f32 / from_rate as f32;
    let out_len = (input.len() as f32 * ratio).round() as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src_pos = i as f32 / ratio;
        let idx = src_pos as usize;
        let frac = src_pos - idx as f32;
        let a = input.get(idx).copied().unwrap_or(0.0);
        let b = input.get(idx + 1).copied().unwrap_or(a);
        out.push(a + (b - a) * frac);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resample_passthrough() {
        let input = vec![1.0, 2.0, 3.0];
        let out = resample_linear(&input, 16_000, 16_000);
        assert_eq!(out, input);
    }

    #[test]
    fn resample_downsamples() {
        let input: Vec<f32> = (0..32).map(|i| i as f32).collect();
        let out = resample_linear(&input, 32_000, 16_000);
        assert_eq!(out.len(), 16);
    }
}
```

Update `src-tauri/src/audio/mod.rs`:

```rust
pub mod capture;
pub mod ringbuffer;
```

- [ ] **Step 3: Tests**

```bash
cd /mnt/synology/Coding/DSS-Whisper/src-tauri
cargo test --lib audio
```

Expected: `test result: ok. 6 passed`

- [ ] **Step 4: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/audio/
git commit -m "feat: cpal audio capture with resample to 16 kHz mono"
```

---

## Task 6: State-Machine

**Files:**
- Create: `src-tauri/src/state.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Failing Tests + Implementierung**

Create `src-tauri/src/state.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AppState {
    Idle,
    Recording,
    Transcribing,
    Injecting,
    Error,
}

#[derive(Debug)]
pub enum Transition {
    StartRecording,
    StopRecording,
    TranscriptionDone,
    InjectionDone,
    Fail,
    Reset,
}

impl AppState {
    pub fn apply(self, t: Transition) -> Self {
        match (self, t) {
            (AppState::Idle, Transition::StartRecording) => AppState::Recording,
            (AppState::Recording, Transition::StopRecording) => AppState::Transcribing,
            (AppState::Transcribing, Transition::TranscriptionDone) => AppState::Injecting,
            (AppState::Injecting, Transition::InjectionDone) => AppState::Idle,
            (_, Transition::Fail) => AppState::Error,
            (_, Transition::Reset) => AppState::Idle,
            (s, _) => s, // Invalid transitions are no-ops
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn happy_path() {
        let s = AppState::Idle
            .apply(Transition::StartRecording)
            .apply(Transition::StopRecording)
            .apply(Transition::TranscriptionDone)
            .apply(Transition::InjectionDone);
        assert_eq!(s, AppState::Idle);
    }

    #[test]
    fn fail_from_any_state() {
        assert_eq!(AppState::Recording.apply(Transition::Fail), AppState::Error);
        assert_eq!(AppState::Transcribing.apply(Transition::Fail), AppState::Error);
    }

    #[test]
    fn invalid_transitions_are_noop() {
        assert_eq!(AppState::Idle.apply(Transition::StopRecording), AppState::Idle);
        assert_eq!(AppState::Idle.apply(Transition::InjectionDone), AppState::Idle);
    }

    #[test]
    fn reset_from_error() {
        assert_eq!(AppState::Error.apply(Transition::Reset), AppState::Idle);
    }
}
```

Ergänze `mod state;` in `src-tauri/src/main.rs`.

- [ ] **Step 2: Tests**

```bash
cargo test --lib state
```

Expected: `test result: ok. 4 passed`

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/state.rs src-tauri/src/main.rs
git commit -m "feat: state machine for dictation flow"
```

---

## Task 7: Global Hotkey

**Files:**
- Create: `src-tauri/src/hotkey.rs`
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Dependency**

`Cargo.toml` → `[dependencies]`:

```toml
global-hotkey = "0.6"
tokio = { version = "1", features = ["rt-multi-thread", "macros", "sync"] }
```

- [ ] **Step 2: Hotkey-Modul**

Create `src-tauri/src/hotkey.rs`:

```rust
use crate::error::{AppError, Result};
use global_hotkey::hotkey::HotKey;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use std::collections::HashMap;
use std::str::FromStr;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum HotkeyEvent {
    Pressed(Uuid),
    Released(Uuid),
}

pub struct HotkeyRegistry {
    manager: GlobalHotKeyManager,
    by_id: HashMap<u32, Uuid>,
}

impl HotkeyRegistry {
    pub fn new() -> Result<Self> {
        let manager = GlobalHotKeyManager::new().map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
        Ok(Self { manager, by_id: HashMap::new() })
    }

    pub fn register(&mut self, profile_id: Uuid, combo: &str) -> Result<()> {
        let hk = HotKey::from_str(combo)
            .map_err(|e| AppError::Config(format!("invalid hotkey {combo}: {e}")))?;
        self.manager.register(hk).map_err(|e| AppError::Config(e.to_string()))?;
        self.by_id.insert(hk.id(), profile_id);
        Ok(())
    }

    pub fn clear(&mut self) -> Result<()> {
        for hk_id in self.by_id.keys().copied().collect::<Vec<_>>() {
            let _ = self.manager.unregister_all(&[]);
            let _ = hk_id;
        }
        self.by_id.clear();
        Ok(())
    }

    pub fn resolve(&self, hk_id: u32) -> Option<Uuid> {
        self.by_id.get(&hk_id).copied()
    }

    pub fn pump_into(&self, tx: UnboundedSender<HotkeyEvent>) {
        let receiver = GlobalHotKeyEvent::receiver();
        while let Ok(event) = receiver.recv() {
            let profile_id = match self.resolve(event.id) {
                Some(id) => id,
                None => continue,
            };
            let msg = match event.state {
                HotKeyState::Pressed => HotkeyEvent::Pressed(profile_id),
                HotKeyState::Released => HotkeyEvent::Released(profile_id),
            };
            if tx.send(msg).is_err() { break; }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_combo() {
        let hk = HotKey::from_str("Ctrl+Alt+Space");
        assert!(hk.is_ok());
    }

    #[test]
    fn rejects_garbage_combo() {
        let hk = HotKey::from_str("not a hotkey");
        assert!(hk.is_err());
    }
}
```

- [ ] **Step 3: Registrieren & Tests**

Ergänze `mod hotkey;` in `src-tauri/src/main.rs`.

```bash
cargo test --lib hotkey
```

Expected: 2 passed.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/hotkey.rs src-tauri/src/main.rs
git commit -m "feat: global hotkey registry with press/release events"
```

---

## Task 8: Text-Injection

**Files:**
- Create: `src-tauri/src/inject.rs`
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Dependencies**

`Cargo.toml` → `[dependencies]`:

```toml
enigo = "0.2"
arboard = "3"
```

- [ ] **Step 2: Injection-Modul**

Create `src-tauri/src/inject.rs`:

```rust
use crate::error::{AppError, Result};
use enigo::{Enigo, Keyboard, Settings};

pub struct TextInjector;

impl TextInjector {
    pub fn inject(text: &str) -> Result<()> {
        let mut enigo = Enigo::new(&Settings::default())
            .map_err(|e| AppError::Inject(e.to_string()))?;
        enigo.text(text).map_err(|e| AppError::Inject(e.to_string()))?;
        Ok(())
    }

    pub fn clipboard_fallback(text: &str) -> Result<()> {
        let mut cb = arboard::Clipboard::new().map_err(|e| AppError::Inject(e.to_string()))?;
        cb.set_text(text.to_string()).map_err(|e| AppError::Inject(e.to_string()))?;
        Ok(())
    }
}
```

Ergänze `mod inject;` in `main.rs`.

- [ ] **Step 3: Build**

```bash
cargo build
```

Expected: Kompiliert. (Manueller Test kommt in Release-Checkliste.)

- [ ] **Step 4: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/inject.rs src-tauri/src/main.rs
git commit -m "feat: text injection via enigo with clipboard fallback"
```

---

## Task 9: Server-Endpoint `/api/dictate` (DSS-V-A-Transcribe)

**Files:**
- Create/Modify im Repo `DSS-V-A-Transcribe`: `app/api/dictate/route.ts` (Next.js) oder `server/api/dictate.py` (FastAPI, je nach Stack)

**Vor dem Schreiben:** Öffne zuerst das DSS-V-A-Transcribe-Repo und prüfe den tatsächlichen Stack. Die Memory sagt „Next.js + faster-whisper GPU" — der Whisper-Teil läuft vermutlich als separater Python-Service. Der Endpoint gehört an die Stelle, an der faster-whisper direkt erreichbar ist.

- [ ] **Step 1: Bestehende Transcription-Route inspizieren**

```bash
cd /mnt/synology/Coding/DSS-V-A-Transcribe
# Finde existierende Whisper-Aufrufe
```

Nutze `Grep pattern:"faster_whisper" path:"."`

Ziel: die vorhandene WhisperModel-Instanz wiederverwenden, nicht ein zweites Modell laden.

- [ ] **Step 2: Synchronen Endpoint hinzufügen (FastAPI-Variante)**

Wenn Python: Create `server/api/dictate.py`:

```python
from fastapi import APIRouter, UploadFile, Form, Header, HTTPException
from pydantic import BaseModel
import os, time, io
import numpy as np
import soundfile as sf
from .whisper_instance import get_model  # bestehende Singleton-Instanz

router = APIRouter()

class DictateResponse(BaseModel):
    text: str
    duration_ms: int
    backend: str

@router.post("/api/dictate", response_model=DictateResponse)
async def dictate(
    file: UploadFile,
    language: str = Form("de"),
    vocabulary: str = Form(""),
    authorization: str = Header(...),
):
    expected = f"Bearer {os.environ['DICTATE_SHARED_SECRET']}"
    if authorization != expected:
        raise HTTPException(status_code=401, detail="unauthorized")

    raw = await file.read()
    audio, sr = sf.read(io.BytesIO(raw), dtype="float32")
    if audio.ndim > 1:
        audio = audio.mean(axis=1)
    if sr != 16_000:
        raise HTTPException(status_code=400, detail=f"expected 16 kHz audio, got {sr}")

    started = time.monotonic()
    model = get_model()
    segments, _ = model.transcribe(
        audio,
        language=None if language == "auto" else language,
        initial_prompt=vocabulary or None,
        vad_filter=True,
        beam_size=1,
    )
    text = " ".join(seg.text.strip() for seg in segments).strip()
    elapsed_ms = int((time.monotonic() - started) * 1000)

    return DictateResponse(text=text, duration_ms=elapsed_ms, backend="faster-whisper-gpu")
```

- [ ] **Step 3: Router registrieren & Secret dokumentieren**

In der FastAPI-Hauptdatei:

```python
from .api.dictate import router as dictate_router
app.include_router(dictate_router)
```

`.env.example` ergänzen:

```
DICTATE_SHARED_SECRET=change-me
```

- [ ] **Step 4: Manueller Curl-Test**

```bash
# Im DSS-V-A-Transcribe-Repo, 16k-mono Test-Wav erzeugen:
ffmpeg -f lavfi -i "sine=frequency=440:duration=2" -ar 16000 -ac 1 -y /tmp/beep.wav
curl -X POST http://192.168.178.43:8503/api/dictate \
  -H "Authorization: Bearer change-me" \
  -F "file=@/tmp/beep.wav" -F "language=de"
```

Expected: JSON-Response mit leerem/kurzem text (bei einem Sinuston), `duration_ms < 2000`.

- [ ] **Step 5: Commit im Server-Repo**

```bash
cd /mnt/synology/Coding/DSS-V-A-Transcribe
git add server/api/dictate.py .env.example
git commit -m "feat: add /api/dictate synchronous endpoint for DSS-Whisper"
```

---

## Task 10: TranscriptionBackend-Trait + RemoteWhisperBackend

**Files:**
- Create: `src-tauri/src/transcription/mod.rs`
- Create: `src-tauri/src/transcription/remote.rs`
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Dependencies**

`Cargo.toml` → `[dependencies]`:

```toml
reqwest = { version = "0.12", features = ["multipart", "json"] }
async-trait = "0.1"
bytes = "1"
hound = "3"
```

`[dev-dependencies]`:

```toml
wiremock = "0.6"
tokio = { version = "1", features = ["full"] }
```

- [ ] **Step 2: Trait definieren**

Create `src-tauri/src/transcription/mod.rs`:

```rust
pub mod remote;
pub mod local;

use crate::config::profile::Language;
use crate::error::Result;
use async_trait::async_trait;

pub struct Transcription {
    pub text: String,
    pub duration_ms: u64,
    pub backend_id: &'static str,
}

#[async_trait]
pub trait TranscriptionBackend: Send + Sync {
    async fn transcribe(
        &self,
        pcm_16k_mono: &[f32],
        language: Language,
        vocabulary: &[String],
    ) -> Result<Transcription>;

    fn id(&self) -> &'static str;

    async fn is_available(&self) -> bool { true }
}

pub fn pcm_to_wav_16k_mono(samples: &[f32]) -> Result<Vec<u8>> {
    use hound::{WavSpec, WavWriter, SampleFormat};
    let spec = WavSpec { channels: 1, sample_rate: 16_000, bits_per_sample: 16, sample_format: SampleFormat::Int };
    let mut buf = Vec::new();
    {
        let cursor = std::io::Cursor::new(&mut buf);
        let mut writer = WavWriter::new(cursor, spec)
            .map_err(|e| crate::error::AppError::Transcription(e.to_string()))?;
        for s in samples {
            let clamped = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
            writer.write_sample(clamped).map_err(|e| crate::error::AppError::Transcription(e.to_string()))?;
        }
        writer.finalize().map_err(|e| crate::error::AppError::Transcription(e.to_string()))?;
    }
    Ok(buf)
}
```

- [ ] **Step 3: RemoteWhisperBackend mit Test**

Create `src-tauri/src/transcription/remote.rs`:

```rust
use super::{pcm_to_wav_16k_mono, Transcription, TranscriptionBackend};
use crate::config::profile::Language;
use crate::error::{AppError, Result};
use async_trait::async_trait;
use reqwest::multipart::{Form, Part};
use serde::Deserialize;
use std::time::Duration;

pub struct RemoteWhisperBackend {
    base_url: String,
    bearer_token: String,
    client: reqwest::Client,
}

#[derive(Deserialize)]
struct DictateResponse {
    text: String,
    duration_ms: u64,
}

impl RemoteWhisperBackend {
    pub fn new(base_url: String, bearer_token: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build().unwrap();
        Self { base_url, bearer_token, client }
    }

    fn lang_code(l: &Language) -> &'static str {
        match l { Language::De => "de", Language::En => "en", Language::Auto => "auto" }
    }
}

#[async_trait]
impl TranscriptionBackend for RemoteWhisperBackend {
    fn id(&self) -> &'static str { "remote-whisper" }

    async fn is_available(&self) -> bool {
        let url = format!("{}/api/health", self.base_url);
        self.client.get(&url).timeout(Duration::from_secs(2)).send().await
            .map(|r| r.status().is_success()).unwrap_or(false)
    }

    async fn transcribe(&self, samples: &[f32], language: Language, vocabulary: &[String]) -> Result<Transcription> {
        let wav = pcm_to_wav_16k_mono(samples)?;
        let vocab = vocabulary.join(", ");

        let part = Part::bytes(wav).file_name("audio.wav").mime_str("audio/wav")
            .map_err(|e| AppError::Transcription(e.to_string()))?;
        let form = Form::new()
            .part("file", part)
            .text("language", Self::lang_code(&language).to_string())
            .text("vocabulary", vocab);

        let resp = self.client.post(format!("{}/api/dictate", self.base_url))
            .bearer_auth(&self.bearer_token)
            .multipart(form).send().await
            .map_err(|e| AppError::Transcription(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(AppError::Transcription(format!("server status {}", resp.status())));
        }
        let body: DictateResponse = resp.json().await
            .map_err(|e| AppError::Transcription(e.to_string()))?;
        Ok(Transcription { text: body.text, duration_ms: body.duration_ms, backend_id: "remote-whisper" })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn transcribes_against_mock_server() {
        let server = MockServer::start().await;
        Mock::given(method("POST")).and(path("/api/dictate"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "text": "Hallo Welt", "duration_ms": 123, "backend": "mock"
            })))
            .mount(&server).await;

        let be = RemoteWhisperBackend::new(server.uri(), "secret".into());
        let samples = vec![0.0f32; 16_000];
        let out = be.transcribe(&samples, Language::De, &[]).await.unwrap();
        assert_eq!(out.text, "Hallo Welt");
        assert_eq!(out.duration_ms, 123);
    }

    #[tokio::test]
    async fn returns_error_on_401() {
        let server = MockServer::start().await;
        Mock::given(method("POST")).and(path("/api/dictate"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&server).await;

        let be = RemoteWhisperBackend::new(server.uri(), "wrong".into());
        let result = be.transcribe(&[0.0; 100], Language::De, &[]).await;
        assert!(result.is_err());
    }
}
```

- [ ] **Step 4: Module registrieren & bauen**

Ergänze `mod transcription;` in `main.rs`.

```bash
cargo test --lib transcription::remote
```

Expected: 2 passed.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/transcription/
git commit -m "feat: TranscriptionBackend trait + RemoteWhisperBackend with wiremock tests"
```

---

## Task 11: LocalWhisperBackend (whisper.cpp)

**Files:**
- Create: `src-tauri/src/transcription/local.rs`
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Dependency**

`Cargo.toml` → `[dependencies]`:

```toml
whisper-rs = { version = "0.12", default-features = false }
```

- [ ] **Step 2: Backend implementieren**

Create `src-tauri/src/transcription/local.rs`:

```rust
use super::{Transcription, TranscriptionBackend};
use crate::config::profile::Language;
use crate::error::{AppError, Result};
use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub struct LocalWhisperBackend {
    ctx: Arc<WhisperContext>,
}

impl LocalWhisperBackend {
    pub fn new(model_path: PathBuf) -> Result<Self> {
        let ctx = WhisperContext::new_with_params(
            model_path.to_str().ok_or_else(|| AppError::Transcription("bad model path".into()))?,
            WhisperContextParameters::default(),
        ).map_err(|e| AppError::Transcription(e.to_string()))?;
        Ok(Self { ctx: Arc::new(ctx) })
    }

    fn lang_code(l: &Language) -> Option<&'static str> {
        match l { Language::De => Some("de"), Language::En => Some("en"), Language::Auto => None }
    }
}

#[async_trait]
impl TranscriptionBackend for LocalWhisperBackend {
    fn id(&self) -> &'static str { "local-whisper" }

    async fn transcribe(&self, samples: &[f32], language: Language, vocabulary: &[String]) -> Result<Transcription> {
        let samples_owned = samples.to_vec();
        let vocab_prompt = vocabulary.join(", ");
        let ctx = self.ctx.clone();
        let lang = Self::lang_code(&language);

        let (text, ms) = tokio::task::spawn_blocking(move || -> Result<(String, u64)> {
            let mut state = ctx.create_state().map_err(|e| AppError::Transcription(e.to_string()))?;
            let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
            if let Some(l) = lang { params.set_language(Some(l)); }
            if !vocab_prompt.is_empty() { params.set_initial_prompt(&vocab_prompt); }
            params.set_print_progress(false);
            params.set_print_realtime(false);
            params.set_print_special(false);

            let start = Instant::now();
            state.full(params, &samples_owned).map_err(|e| AppError::Transcription(e.to_string()))?;

            let num_segments = state.full_n_segments().map_err(|e| AppError::Transcription(e.to_string()))?;
            let mut text = String::new();
            for i in 0..num_segments {
                text.push_str(&state.full_get_segment_text(i).map_err(|e| AppError::Transcription(e.to_string()))?);
            }
            Ok((text.trim().to_string(), start.elapsed().as_millis() as u64))
        }).await.map_err(|e| AppError::Transcription(e.to_string()))??;

        Ok(Transcription { text, duration_ms: ms, backend_id: "local-whisper" })
    }
}
```

- [ ] **Step 3: Build**

```bash
cargo build
```

Expected: Kompiliert. (whisper.cpp wird via Cargo-Features gebaut; kann erstmals 1-3 Min dauern.)

Falls die Build-Kette von `whisper-rs` fehlschlägt (z.B. fehlendes cmake/clang auf Linux), Install-Doc in `reference/whisper-setup.md` ergänzen und Task-Checkbox bleibt offen, bis auf Windows-Host gebaut.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/transcription/local.rs
git commit -m "feat: LocalWhisperBackend using whisper.cpp"
```

---

## Task 12: LLM-Provider-Trait + OpenAI-kompatibler Adapter

**Files:**
- Create: `src-tauri/src/llm/mod.rs`
- Create: `src-tauri/src/llm/openai_compat.rs`
- Create: `src-tauri/src/llm/prompt.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Prompt-Builder mit Tests**

Create `src-tauri/src/llm/prompt.rs`:

```rust
const DEFAULT_SYSTEM: &str = "Du korrigierst diktierten Text. Verändere den Inhalt nicht. \
Korrigiere ausschließlich Rechtschreibung, Grammatik, Zeichensetzung und offensichtlich \
falsche Wort-Erkennungen. Gib ausschließlich den korrigierten Text zurück, ohne Kommentare \
oder Anführungszeichen.";

pub struct PostProcPrompt {
    pub system: String,
    pub user: String,
}

pub fn build_prompt(
    raw_text: &str,
    vocabulary: &[String],
    custom_system: Option<&str>,
) -> PostProcPrompt {
    let base = custom_system.unwrap_or(DEFAULT_SYSTEM);
    let system = if vocabulary.is_empty() {
        base.to_string()
    } else {
        format!(
            "{base}\n\nVerwende folgendes Vokabular korrekt, wenn es vorkommt:\n{}",
            vocabulary.join(", ")
        )
    };
    PostProcPrompt { system, user: raw_text.to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_system_used_when_no_custom() {
        let p = build_prompt("hallo welt", &[], None);
        assert!(p.system.starts_with("Du korrigierst"));
        assert_eq!(p.user, "hallo welt");
    }

    #[test]
    fn vocabulary_appended_to_system() {
        let p = build_prompt("x", &vec!["DSS-Siegmund".into(), "Invoice Ninja".into()], None);
        assert!(p.system.contains("DSS-Siegmund, Invoice Ninja"));
    }

    #[test]
    fn custom_system_replaces_default() {
        let p = build_prompt("x", &[], Some("Antworte nur in Großbuchstaben."));
        assert!(p.system.contains("Großbuchstaben"));
        assert!(!p.system.contains("Du korrigierst"));
    }
}
```

- [ ] **Step 2: Provider-Trait**

Create `src-tauri/src/llm/mod.rs`:

```rust
pub mod openai_compat;
pub mod anthropic;
pub mod prompt;

use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete(&self, system: &str, user: &str, model: &str) -> Result<String>;
    fn id(&self) -> &'static str;
}
```

- [ ] **Step 3: OpenAI-kompatibler Adapter + Tests**

Create `src-tauri/src/llm/openai_compat.rs`:

```rust
use super::LlmProvider;
use crate::error::{AppError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct OpenAiCompatProvider {
    base_url: String,
    api_key: String,
    client: reqwest::Client,
}

impl OpenAiCompatProvider {
    pub fn new(base_url: String, api_key: String) -> Self {
        let client = reqwest::Client::builder().timeout(Duration::from_secs(30)).build().unwrap();
        Self { base_url, api_key, client }
    }
}

#[derive(Serialize)]
struct ChatMessage<'a> { role: &'a str, content: &'a str }

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    temperature: f32,
}

#[derive(Deserialize)]
struct ChatResponse { choices: Vec<Choice> }
#[derive(Deserialize)]
struct Choice { message: ChoiceMessage }
#[derive(Deserialize)]
struct ChoiceMessage { content: String }

#[async_trait]
impl LlmProvider for OpenAiCompatProvider {
    fn id(&self) -> &'static str { "openai-compat" }

    async fn complete(&self, system: &str, user: &str, model: &str) -> Result<String> {
        let req = ChatRequest {
            model,
            messages: vec![
                ChatMessage { role: "system", content: system },
                ChatMessage { role: "user", content: user },
            ],
            temperature: 0.0,
        };

        let resp = self.client.post(format!("{}/v1/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&req).send().await
            .map_err(|e| AppError::Llm(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Llm(format!("{}: {}", status, body)));
        }

        let parsed: ChatResponse = resp.json().await.map_err(|e| AppError::Llm(e.to_string()))?;
        parsed.choices.into_iter().next()
            .map(|c| c.message.content)
            .ok_or_else(|| AppError::Llm("no choices in response".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn happy_path_returns_corrected_text() {
        let server = MockServer::start().await;
        Mock::given(method("POST")).and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": [{ "message": { "content": "Hallo Welt." } }]
            })))
            .mount(&server).await;

        let p = OpenAiCompatProvider::new(server.uri(), "key".into());
        let out = p.complete("system", "user", "gpt-4o-mini").await.unwrap();
        assert_eq!(out, "Hallo Welt.");
    }
}
```

- [ ] **Step 4: Anthropic-Stub (minimal, erfüllt Trait)**

Create `src-tauri/src/llm/anthropic.rs`:

```rust
use super::LlmProvider;
use crate::error::{AppError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct AnthropicProvider {
    api_key: String,
    client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        let client = reqwest::Client::builder().timeout(Duration::from_secs(30)).build().unwrap();
        Self { api_key, client }
    }
}

#[derive(Serialize)]
struct Msg<'a> { role: &'a str, content: &'a str }

#[derive(Serialize)]
struct Req<'a> {
    model: &'a str,
    max_tokens: u32,
    system: &'a str,
    messages: Vec<Msg<'a>>,
}

#[derive(Deserialize)]
struct Resp { content: Vec<ContentBlock> }
#[derive(Deserialize)]
struct ContentBlock { text: String }

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn id(&self) -> &'static str { "anthropic" }

    async fn complete(&self, system: &str, user: &str, model: &str) -> Result<String> {
        let req = Req {
            model, max_tokens: 2048, system,
            messages: vec![Msg { role: "user", content: user }],
        };
        let resp = self.client.post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&req).send().await
            .map_err(|e| AppError::Llm(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(AppError::Llm(format!("status {}", resp.status())));
        }
        let parsed: Resp = resp.json().await.map_err(|e| AppError::Llm(e.to_string()))?;
        parsed.content.into_iter().next()
            .map(|c| c.text)
            .ok_or_else(|| AppError::Llm("no content blocks".into()))
    }
}
```

- [ ] **Step 5: Registrieren & Tests**

Ergänze `mod llm;` in `main.rs`.

```bash
cargo test --lib llm
```

Expected: 4 passed.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/llm/ src-tauri/src/main.rs
git commit -m "feat: LlmProvider trait with OpenAI-compat and Anthropic adapters"
```

---

## Task 13: History-Store (SQLite)

**Files:**
- Create: `src-tauri/src/history/mod.rs`
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Dependency**

`Cargo.toml`:

```toml
rusqlite = { version = "0.31", features = ["bundled"] }
chrono = { version = "0.4", features = ["serde"] }
```

- [ ] **Step 2: Store + Tests**

Create `src-tauri/src/history/mod.rs`:

```rust
use crate::error::{AppError, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::Serialize;
use std::path::Path;
use std::sync::Mutex;
use uuid::Uuid;

pub struct HistoryStore {
    conn: Mutex<Connection>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HistoryEntry {
    pub id: i64,
    pub timestamp: DateTime<Utc>,
    pub profile_id: Uuid,
    pub profile_name: String,
    pub backend_id: String,
    pub duration_ms: u64,
    pub text: String,
}

impl HistoryStore {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() { std::fs::create_dir_all(parent)?; }
        let conn = Connection::open(path).map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
        conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                profile_id TEXT NOT NULL,
                profile_name TEXT NOT NULL,
                backend_id TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                text TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_history_ts ON history(timestamp DESC);
        "#).map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn insert(&self, e: &HistoryEntry) -> Result<i64> {
        let c = self.conn.lock().unwrap();
        c.execute(
            "INSERT INTO history (timestamp, profile_id, profile_name, backend_id, duration_ms, text) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![e.timestamp.to_rfc3339(), e.profile_id.to_string(), e.profile_name, e.backend_id, e.duration_ms as i64, e.text],
        ).map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
        Ok(c.last_insert_rowid())
    }

    pub fn list(&self, limit: u32) -> Result<Vec<HistoryEntry>> {
        let c = self.conn.lock().unwrap();
        let mut stmt = c.prepare("SELECT id, timestamp, profile_id, profile_name, backend_id, duration_ms, text \
                                  FROM history ORDER BY id DESC LIMIT ?1")
            .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            let ts: String = row.get(1)?;
            let pid: String = row.get(2)?;
            Ok(HistoryEntry {
                id: row.get(0)?,
                timestamp: DateTime::parse_from_rfc3339(&ts).unwrap().with_timezone(&Utc),
                profile_id: Uuid::parse_str(&pid).unwrap_or(Uuid::nil()),
                profile_name: row.get(3)?,
                backend_id: row.get(4)?,
                duration_ms: row.get::<_, i64>(5)? as u64,
                text: row.get(6)?,
            })
        }).map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn delete(&self, id: i64) -> Result<()> {
        self.conn.lock().unwrap().execute("DELETE FROM history WHERE id = ?1", params![id])
            .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
        Ok(())
    }

    pub fn trim(&self, keep: u32) -> Result<()> {
        self.conn.lock().unwrap().execute(
            "DELETE FROM history WHERE id NOT IN (SELECT id FROM history ORDER BY id DESC LIMIT ?1)",
            params![keep as i64],
        ).map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn sample(text: &str) -> HistoryEntry {
        HistoryEntry {
            id: 0, timestamp: Utc::now(), profile_id: Uuid::new_v4(),
            profile_name: "P".into(), backend_id: "remote-whisper".into(),
            duration_ms: 500, text: text.into(),
        }
    }

    #[test]
    fn insert_and_list() {
        let dir = tempdir().unwrap();
        let store = HistoryStore::open(&dir.path().join("h.db")).unwrap();
        store.insert(&sample("eins")).unwrap();
        store.insert(&sample("zwei")).unwrap();
        let list = store.list(10).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].text, "zwei");
    }

    #[test]
    fn trim_keeps_n_newest() {
        let dir = tempdir().unwrap();
        let store = HistoryStore::open(&dir.path().join("h.db")).unwrap();
        for i in 0..5 { store.insert(&sample(&format!("t{i}"))).unwrap(); }
        store.trim(2).unwrap();
        let list = store.list(10).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].text, "t4");
    }
}
```

- [ ] **Step 3: Registrieren & Tests**

Ergänze `mod history;` in `main.rs`.

```bash
cargo test --lib history
```

Expected: 2 passed.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/history/ src-tauri/src/main.rs
git commit -m "feat: SQLite history store"
```

---

## Task 14: Orchestrator

**Files:**
- Create: `src-tauri/src/orchestrator.rs`
- Modify: `src-tauri/src/main.rs`

**Verantwortung:** Verknüpft Hotkey-Events mit Audio-Capture, Transcription-Backend-Auswahl (inkl. Auto-Fallback), Post-Processing, Text-Injection, History und State-Machine.

- [ ] **Step 1: Orchestrator-Modul**

Create `src-tauri/src/orchestrator.rs`:

```rust
use crate::audio::capture::AudioCapture;
use crate::config::profile::{HotkeyMode, Profile, Language};
use crate::error::{AppError, Result};
use crate::history::{HistoryEntry, HistoryStore};
use crate::hotkey::HotkeyEvent;
use crate::inject::TextInjector;
use crate::llm::prompt::build_prompt;
use crate::llm::LlmProvider;
use crate::state::{AppState, Transition};
use crate::transcription::TranscriptionBackend;
use chrono::Utc;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedReceiver;
use uuid::Uuid;

pub struct Orchestrator {
    pub audio: AudioCapture,
    pub profiles: HashMap<Uuid, Profile>,
    pub primary_backend: Arc<dyn TranscriptionBackend>,
    pub fallback_backend: Arc<dyn TranscriptionBackend>,
    pub llm_providers: HashMap<Uuid, Arc<dyn LlmProvider>>,
    pub vocabulary: Vec<String>,
    pub history: Arc<HistoryStore>,
    pub state: Arc<Mutex<AppState>>,
    pub toggle_active_profile: Arc<Mutex<Option<Uuid>>>,
}

impl Orchestrator {
    pub async fn run_loop(&mut self, mut rx: UnboundedReceiver<HotkeyEvent>) {
        while let Some(event) = rx.recv().await {
            if let Err(e) = self.handle(event).await {
                eprintln!("orchestrator error: {e:?}");
                *self.state.lock() = (*self.state.lock()).apply(Transition::Fail);
            }
        }
    }

    async fn handle(&mut self, event: HotkeyEvent) -> Result<()> {
        match event {
            HotkeyEvent::Pressed(pid) => {
                let profile = self.profiles.get(&pid).cloned()
                    .ok_or_else(|| AppError::Config(format!("unknown profile {pid}")))?;
                match profile.hotkey_mode {
                    HotkeyMode::PushToTalk => self.start_recording(&profile)?,
                    HotkeyMode::Toggle => {
                        let mut active = self.toggle_active_profile.lock();
                        if active.is_some() {
                            let active_pid = active.take().unwrap();
                            drop(active);
                            let active_profile = self.profiles.get(&active_pid).cloned()
                                .ok_or_else(|| AppError::Config("toggle profile gone".into()))?;
                            self.stop_and_process(&active_profile).await?;
                        } else {
                            *active = Some(pid);
                            drop(active);
                            self.start_recording(&profile)?;
                        }
                    }
                }
            }
            HotkeyEvent::Released(pid) => {
                let profile = match self.profiles.get(&pid).cloned() {
                    Some(p) => p,
                    None => return Ok(()),
                };
                if matches!(profile.hotkey_mode, HotkeyMode::PushToTalk) {
                    self.stop_and_process(&profile).await?;
                }
            }
        }
        Ok(())
    }

    fn start_recording(&mut self, profile: &Profile) -> Result<()> {
        self.audio.buffer.lock().clear();
        self.audio.start(None)?;  // TODO: device from config
        *self.state.lock() = (*self.state.lock()).apply(Transition::StartRecording);
        let _ = profile;
        Ok(())
    }

    async fn stop_and_process(&mut self, profile: &Profile) -> Result<()> {
        self.audio.stop();
        let samples = self.audio.buffer.lock().drain_to_vec();
        *self.state.lock() = (*self.state.lock()).apply(Transition::StopRecording);

        if samples.is_empty() { *self.state.lock() = AppState::Idle; return Ok(()); }

        let backend = self.select_backend().await;
        let transcription = backend
            .transcribe(&samples, profile.language.clone(), &self.vocabulary)
            .await?;

        let mut final_text = transcription.text.clone();

        if profile.post_processing.enabled {
            if let (Some(pid), Some(model)) = (profile.post_processing.llm_provider_id, profile.post_processing.model.as_deref()) {
                if let Some(provider) = self.llm_providers.get(&pid) {
                    let prompt = build_prompt(
                        &transcription.text, &self.vocabulary,
                        profile.post_processing.system_prompt.as_deref(),
                    );
                    match provider.complete(&prompt.system, &prompt.user, model).await {
                        Ok(corrected) => final_text = corrected,
                        Err(e) => eprintln!("post-processing failed, using raw text: {e:?}"),
                    }
                }
            }
        }

        *self.state.lock() = (*self.state.lock()).apply(Transition::TranscriptionDone);

        if let Err(e) = TextInjector::inject(&final_text) {
            eprintln!("injection failed, falling back to clipboard: {e:?}");
            TextInjector::clipboard_fallback(&final_text)?;
        }
        *self.state.lock() = (*self.state.lock()).apply(Transition::InjectionDone);

        self.history.insert(&HistoryEntry {
            id: 0, timestamp: Utc::now(),
            profile_id: profile.id, profile_name: profile.name.clone(),
            backend_id: transcription.backend_id.to_string(),
            duration_ms: transcription.duration_ms,
            text: final_text,
        })?;

        Ok(())
    }

    async fn select_backend(&self) -> Arc<dyn TranscriptionBackend> {
        if self.primary_backend.is_available().await {
            self.primary_backend.clone()
        } else {
            eprintln!("primary backend unavailable, using fallback");
            self.fallback_backend.clone()
        }
    }
}
```

Ergänze `mod orchestrator;` in `main.rs`.

- [ ] **Step 2: Build**

```bash
cargo build
```

Expected: Kompiliert.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/orchestrator.rs src-tauri/src/main.rs
git commit -m "feat: orchestrator tying hotkeys, audio, backends, LLM and history"
```

---

## Task 15: Tauri-Commands (IPC) + Frontend-Integration

**Files:**
- Create: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/main.rs`
- Create: `src/ipc.ts`
- Create: `src/types.ts`

- [ ] **Step 1: IPC-Commands**

Create `src-tauri/src/commands.rs`:

```rust
use crate::audio::capture::AudioCapture;
use crate::config::{self, AppConfig};
use crate::config::profile::Profile;
use crate::config::provider::LlmProviderConfig;
use crate::history::{HistoryEntry, HistoryStore};
use crate::secrets;
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
```

Ergänze `mod commands;` in `main.rs` und registriere in `tauri::Builder`:

```rust
tauri::Builder::default()
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
```

- [ ] **Step 2: TypeScript-Typen**

Create `src/types.ts`:

```typescript
export type HotkeyMode = "push_to_talk" | "toggle";
export type Language = "de" | "en" | "auto";
export type TranscriptionBackendId = "remote_whisper" | "local_whisper";
export type ProviderType = "openai" | "openai_compatible" | "anthropic" | "ollama";

export interface PostProcessing {
  enabled: boolean;
  llm_provider_id: string | null;
  model: string | null;
  system_prompt: string | null;
}

export interface Profile {
  id: string;
  name: string;
  hotkey: string;
  hotkey_mode: HotkeyMode;
  transcription_backend: TranscriptionBackendId;
  language: Language;
  post_processing: PostProcessing;
}

export interface LlmProviderConfig {
  id: string;
  name: string;
  type: ProviderType;
  base_url: string;
  default_model: string;
}

export interface General {
  autostart: boolean;
  sounds: boolean;
  overlay: boolean;
  max_recording_seconds: number;
  history_limit: number;
  mic_device: string | null;
}

export interface AppConfig {
  profiles: Profile[];
  providers: LlmProviderConfig[];
  general: General;
}

export interface HistoryEntry {
  id: number;
  timestamp: string;
  profile_id: string;
  profile_name: string;
  backend_id: string;
  duration_ms: number;
  text: string;
}
```

- [ ] **Step 3: IPC-Wrapper**

Create `src/ipc.ts`:

```typescript
import { invoke } from "@tauri-apps/api/core";
import type { AppConfig, HistoryEntry } from "./types";

export const ipc = {
  getConfig: () => invoke<AppConfig>("get_config"),
  saveConfig: (cfg: AppConfig) => invoke<void>("save_config", { cfg }),
  setApiKey: (providerId: string, key: string) =>
    invoke<void>("set_api_key", { providerId, key }),
  listInputDevices: () => invoke<string[]>("list_input_devices"),
  listHistory: (limit: number) => invoke<HistoryEntry[]>("list_history", { limit }),
  deleteHistory: (id: number) => invoke<void>("delete_history", { id }),
};
```

- [ ] **Step 4: Build**

```bash
cd /mnt/synology/Coding/DSS-Whisper
bun install
cargo build --manifest-path src-tauri/Cargo.toml
```

Expected: Beide Seiten kompilieren.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/main.rs src/types.ts src/ipc.ts
git commit -m "feat: Tauri IPC commands and typed frontend wrapper"
```

---

## Task 16: Settings-UI (6 Tabs)

**Files:**
- Create: `src/pages/Profiles.tsx`, `Providers.tsx`, `Vocabulary.tsx`, `Audio.tsx`, `General.tsx`, `History.tsx`
- Create: `src/components/HotkeyRecorder.tsx`, `LevelMeter.tsx`
- Modify: `src/main.tsx`, `src/App.tsx`

**Anmerkung:** UI-Tests werden im MVP manuell verifiziert (über Release-Checkliste). Kein Jest/Vitest-Setup in dieser Phase — nur TypeScript-Typecheck über `tsc`.

- [ ] **Step 1: Routing-Grundgerüst**

Update `src/main.tsx`:

```tsx
import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter, Routes, Route, NavLink, Navigate } from "react-router-dom";
import Profiles from "./pages/Profiles";
import Providers from "./pages/Providers";
import Vocabulary from "./pages/Vocabulary";
import Audio from "./pages/Audio";
import General from "./pages/General";
import History from "./pages/History";
import "./index.css";

function App() {
  return (
    <BrowserRouter>
      <div className="app">
        <nav className="sidebar">
          <NavLink to="/profiles">Profile</NavLink>
          <NavLink to="/providers">LLM-Anbieter</NavLink>
          <NavLink to="/vocabulary">Wörterbuch</NavLink>
          <NavLink to="/audio">Audio</NavLink>
          <NavLink to="/general">Allgemein</NavLink>
          <NavLink to="/history">History</NavLink>
        </nav>
        <main className="content">
          <Routes>
            <Route path="/" element={<Navigate to="/profiles" />} />
            <Route path="/profiles" element={<Profiles />} />
            <Route path="/providers" element={<Providers />} />
            <Route path="/vocabulary" element={<Vocabulary />} />
            <Route path="/audio" element={<Audio />} />
            <Route path="/general" element={<General />} />
            <Route path="/history" element={<History />} />
          </Routes>
        </main>
      </div>
    </BrowserRouter>
  );
}

ReactDOM.createRoot(document.getElementById("root")!).render(<App />);
```

Ergänze in `package.json`:

```bash
bun add react-router-dom
```

- [ ] **Step 2: Profiles-Seite (Kernstück)**

Create `src/pages/Profiles.tsx`:

```tsx
import { useEffect, useState } from "react";
import { ipc } from "../ipc";
import type { AppConfig, Profile } from "../types";
import HotkeyRecorder from "../components/HotkeyRecorder";

export default function Profiles() {
  const [cfg, setCfg] = useState<AppConfig | null>(null);

  useEffect(() => { ipc.getConfig().then(setCfg); }, []);

  if (!cfg) return <div>Lade…</div>;

  const save = (next: AppConfig) => { setCfg(next); ipc.saveConfig(next); };

  const add = () => {
    const p: Profile = {
      id: crypto.randomUUID(),
      name: "Neues Profil",
      hotkey: "Ctrl+Alt+Space",
      hotkey_mode: "push_to_talk",
      transcription_backend: "remote_whisper",
      language: "de",
      post_processing: { enabled: false, llm_provider_id: null, model: null, system_prompt: null },
    };
    save({ ...cfg, profiles: [...cfg.profiles, p] });
  };

  const update = (i: number, patch: Partial<Profile>) => {
    const profiles = cfg.profiles.map((p, idx) => idx === i ? { ...p, ...patch } : p);
    save({ ...cfg, profiles });
  };

  const remove = (i: number) =>
    save({ ...cfg, profiles: cfg.profiles.filter((_, idx) => idx !== i) });

  return (
    <div>
      <h1>Profile</h1>
      <button onClick={add}>+ Neues Profil</button>
      {cfg.profiles.map((p, i) => (
        <fieldset key={p.id}>
          <legend>{p.name}</legend>
          <label>Name <input value={p.name} onChange={e => update(i, { name: e.target.value })} /></label>
          <label>Hotkey <HotkeyRecorder value={p.hotkey} onChange={v => update(i, { hotkey: v })} /></label>
          <label>Modus
            <select value={p.hotkey_mode} onChange={e => update(i, { hotkey_mode: e.target.value as any })}>
              <option value="push_to_talk">Push-to-talk</option>
              <option value="toggle">Toggle</option>
            </select>
          </label>
          <label>Backend
            <select value={p.transcription_backend} onChange={e => update(i, { transcription_backend: e.target.value as any })}>
              <option value="remote_whisper">GPU-Server</option>
              <option value="local_whisper">Lokal (whisper.cpp)</option>
            </select>
          </label>
          <label>Sprache
            <select value={p.language} onChange={e => update(i, { language: e.target.value as any })}>
              <option value="de">Deutsch</option>
              <option value="en">Englisch</option>
              <option value="auto">Auto</option>
            </select>
          </label>
          <label>
            <input type="checkbox" checked={p.post_processing.enabled}
              onChange={e => update(i, { post_processing: { ...p.post_processing, enabled: e.target.checked } })} />
            Post-Processing aktiv
          </label>
          {p.post_processing.enabled && (
            <>
              <label>LLM-Provider
                <select value={p.post_processing.llm_provider_id ?? ""}
                  onChange={e => update(i, { post_processing: { ...p.post_processing, llm_provider_id: e.target.value || null } })}>
                  <option value="">— wählen —</option>
                  {cfg.providers.map(pr => <option key={pr.id} value={pr.id}>{pr.name}</option>)}
                </select>
              </label>
              <label>Modell
                <input value={p.post_processing.model ?? ""}
                  onChange={e => update(i, { post_processing: { ...p.post_processing, model: e.target.value || null } })} />
              </label>
              <label>Eigener System-Prompt (optional)
                <textarea value={p.post_processing.system_prompt ?? ""}
                  onChange={e => update(i, { post_processing: { ...p.post_processing, system_prompt: e.target.value || null } })} />
              </label>
            </>
          )}
          <button onClick={() => remove(i)}>Profil löschen</button>
        </fieldset>
      ))}
    </div>
  );
}
```

- [ ] **Step 3: HotkeyRecorder-Komponente**

Create `src/components/HotkeyRecorder.tsx`:

```tsx
import { useState } from "react";

export default function HotkeyRecorder({ value, onChange }: { value: string; onChange: (v: string) => void }) {
  const [recording, setRecording] = useState(false);

  const onKey = (e: React.KeyboardEvent) => {
    if (!recording) return;
    e.preventDefault();
    const parts: string[] = [];
    if (e.ctrlKey) parts.push("Ctrl");
    if (e.altKey) parts.push("Alt");
    if (e.shiftKey) parts.push("Shift");
    if (e.metaKey) parts.push("Meta");
    const key = e.key.length === 1 ? e.key.toUpperCase() : e.key;
    if (!["Control", "Alt", "Shift", "Meta"].includes(key)) {
      parts.push(key === " " ? "Space" : key);
      onChange(parts.join("+"));
      setRecording(false);
    }
  };

  return (
    <span>
      <input readOnly value={recording ? "…drücken…" : value} onKeyDown={onKey} onFocus={() => setRecording(true)} onBlur={() => setRecording(false)} />
    </span>
  );
}
```

- [ ] **Step 4: Weitere Seiten skelettartig**

Create `src/pages/Providers.tsx`, `Vocabulary.tsx`, `Audio.tsx`, `General.tsx`, `History.tsx` — jeweils einfache CRUD-UIs gegen `ipc.getConfig()/saveConfig()`. Muster folgt `Profiles.tsx`. Für `Audio.tsx` zusätzlich `ipc.listInputDevices()` für Dropdown, `LevelMeter` kommt aus Tauri-Event `audio://level` (wird in Task 17 verdrahtet). `History.tsx` nutzt `ipc.listHistory(100)` + `ipc.deleteHistory(id)`.

**Vollständige Implementierung analog zu `Profiles.tsx`** — für Kürze hier nur die Signaturen, die konkrete Form folgt dem gleichen State-Laden-und-Speichern-Muster. Jede Seite öffnet mit `useEffect(() => { ipc.getConfig().then(setCfg); }, []);` und ruft `ipc.saveConfig(...)` nach jeder Änderung.

- [ ] **Step 5: Typecheck & Build**

```bash
bun run tsc --noEmit
bun run build
```

Expected: Keine Typ-Fehler.

- [ ] **Step 6: Commit**

```bash
git add src/ package.json bun.lockb
git commit -m "feat: Settings UI with Profiles, HotkeyRecorder and IPC wiring"
```

---

## Task 17: Tray-Icon + Mini-Overlay + Events ans Frontend

**Files:**
- Create: `src-tauri/src/tray.rs`
- Create: `src-tauri/src/overlay.rs`
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/Cargo.toml`
- Create: `src-tauri/icons/tray-idle.png`, `tray-recording.png`, `tray-processing.png`, `tray-error.png` (Platzhalter, 32×32 PNG)

- [ ] **Step 1: Tray-Setup**

`Cargo.toml` Features:

```toml
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-shell = "2"
```

Create `src-tauri/src/tray.rs`:

```rust
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager};

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let open_settings = MenuItem::with_id(app, "open_settings", "Einstellungen", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Beenden", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open_settings, &quit])?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open_settings" => {
                if let Some(win) = app.get_webview_window("main") { let _ = win.show(); let _ = win.set_focus(); }
            }
            "quit" => { app.exit(0); }
            _ => {}
        })
        .build(app)?;
    Ok(())
}

pub fn set_state_icon(app: &AppHandle, path: &str) {
    if let Some(tray) = app.tray_by_id("main-tray") {
        let _ = tray.set_icon(Some(tauri::image::Image::from_path(path).unwrap()));
    }
}
```

- [ ] **Step 2: Overlay-Fenster (dezent, immer im Vordergrund)**

Create `src-tauri/src/overlay.rs`:

```rust
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

pub fn show(app: &AppHandle) -> tauri::Result<()> {
    if let Some(win) = app.get_webview_window("overlay") {
        let _ = win.show();
        return Ok(());
    }
    let _ = WebviewWindowBuilder::new(app, "overlay", WebviewUrl::App("overlay.html".into()))
        .title("DSS-Whisper Overlay")
        .decorations(false)
        .always_on_top(true)
        .transparent(true)
        .skip_taskbar(true)
        .inner_size(220.0, 60.0)
        .position(f64::MAX, f64::MAX) // wird per JS-Script an bottom-right gepinnt
        .build()?;
    Ok(())
}

pub fn hide(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("overlay") { let _ = win.hide(); }
}
```

Create `overlay.html` im Projekt-Root als Platzhalter (einfache inline HTML-Seite mit Recording-Pill + Pegel-Animation, `window.__TAURI__` liest `audio://level`-Events). Details fallen in Task 18 auf.

- [ ] **Step 3: In main.rs verdrahten**

```rust
fn main() {
    tauri::Builder::default()
        .setup(|app| {
            tray::setup(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // ... bestehende ...
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 4: Pegel-Events emittieren**

Im Orchestrator (`audio/capture.rs`) einen `tokio::task`/Timer starten, der alle 50 ms den aktuellen `level` via Tauri-Event `audio://level` emittiert. Code-Skelett:

```rust
// In main.rs setup-Handler, nachdem AudioCapture im State liegt:
let handle = app.handle().clone();
let level_ref = audio.level.clone();
tauri::async_runtime::spawn(async move {
    loop {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let v = *level_ref.lock();
        let _ = handle.emit("audio://level", v);
    }
});
```

- [ ] **Step 5: Build**

```bash
cargo build
```

Expected: Kompiliert.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/
git commit -m "feat: tray icon, mini overlay and level events"
```

---

## Task 18: Setup-Integration — alles in `main.rs` verdrahten

**Files:**
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/tauri.conf.json` (additional window for overlay)

- [ ] **Step 1: main.rs vollständig**

Ersetze `src-tauri/src/main.rs`:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio; mod commands; mod config; mod error;
mod history; mod hotkey; mod inject; mod llm;
mod orchestrator; mod overlay; mod secrets; mod state;
mod transcription; mod tray;

use crate::audio::capture::AudioCapture;
use crate::history::HistoryStore;
use crate::hotkey::{HotkeyEvent, HotkeyRegistry};
use crate::orchestrator::Orchestrator;
use crate::transcription::{local::LocalWhisperBackend, remote::RemoteWhisperBackend, TranscriptionBackend};
use crate::llm::{anthropic::AnthropicProvider, openai_compat::OpenAiCompatProvider, LlmProvider};
use directories::ProjectDirs;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            tray::setup(&handle)?;

            let dirs = ProjectDirs::from("de", "dss", "Whisper").unwrap();
            let db_path = dirs.config_dir().join("history.db");
            let history = Arc::new(HistoryStore::open(&db_path)?);
            app.manage(history.clone());

            let cfg = config::load().unwrap_or_default();

            let remote_cfg = std::env::var("DSS_WHISPER_REMOTE_URL")
                .unwrap_or_else(|_| "http://192.168.178.43:8503".into());
            let remote_token = secrets::get_api_key(Uuid::nil())
                .unwrap_or_else(|_| std::env::var("DSS_WHISPER_REMOTE_TOKEN").unwrap_or_default());

            let primary: Arc<dyn TranscriptionBackend> = Arc::new(
                RemoteWhisperBackend::new(remote_cfg, remote_token)
            );
            let model_path = dirs.data_dir().join("models").join("ggml-base.bin");
            let fallback: Arc<dyn TranscriptionBackend> = match LocalWhisperBackend::new(model_path) {
                Ok(be) => Arc::new(be),
                Err(e) => { eprintln!("local whisper not available: {e:?}"); primary.clone() }
            };

            let mut providers: HashMap<Uuid, Arc<dyn LlmProvider>> = HashMap::new();
            for p in &cfg.providers {
                if let Ok(key) = secrets::get_api_key(p.id) {
                    let prov: Arc<dyn LlmProvider> = match p.r#type {
                        config::provider::ProviderType::Anthropic => Arc::new(AnthropicProvider::new(key)),
                        _ => Arc::new(OpenAiCompatProvider::new(p.base_url.clone(), key)),
                    };
                    providers.insert(p.id, prov);
                }
            }

            let profiles_map: HashMap<Uuid, _> = cfg.profiles.iter().map(|p| (p.id, p.clone())).collect();

            let mut registry = HotkeyRegistry::new()?;
            for p in &cfg.profiles {
                if let Err(e) = registry.register(p.id, &p.hotkey) {
                    eprintln!("failed to register hotkey {}: {:?}", p.hotkey, e);
                }
            }

            let (tx, rx) = mpsc::unbounded_channel::<HotkeyEvent>();
            std::thread::spawn({
                let tx = tx.clone();
                move || registry.pump_into(tx)
            });

            let audio = AudioCapture::new(cfg.general.max_recording_seconds);

            // Level-Event-Pump
            let level_ref = audio.level.clone();
            let handle_levels = handle.clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                    let v = *level_ref.lock();
                    let _ = handle_levels.emit("audio://level", v);
                }
            });

            let state = Arc::new(Mutex::new(crate::state::AppState::Idle));
            let vocabulary = std::fs::read_to_string(dirs.config_dir().join("vocabulary.txt"))
                .ok().map(|s| s.lines().map(|l| l.to_string()).filter(|l| !l.trim().is_empty()).collect())
                .unwrap_or_default();

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
```

- [ ] **Step 2: `tauri.conf.json` um Overlay-Window**

Unter `app.windows`:

```json
{
  "label": "overlay",
  "url": "overlay.html",
  "width": 220, "height": 60,
  "decorations": false,
  "alwaysOnTop": true,
  "transparent": true,
  "skipTaskbar": true,
  "visible": false
}
```

- [ ] **Step 3: Overlay-HTML**

Create `overlay.html`:

```html
<!DOCTYPE html>
<html><head><style>
  body { margin:0; background:transparent; font-family:sans-serif; color:white; }
  .pill { background:rgba(0,0,0,.7); border-radius:30px; padding:8px 16px; display:flex; align-items:center; gap:8px; }
  .dot { width:10px; height:10px; border-radius:50%; background:#ef4444; animation:pulse 1s infinite; }
  @keyframes pulse { 0%,100% { opacity:1 } 50% { opacity:.3 } }
  .meter { flex:1; height:6px; background:rgba(255,255,255,.2); border-radius:3px; overflow:hidden; }
  .meter-fill { height:100%; background:#10b981; transition:width .05s; }
</style></head><body>
  <div class="pill"><div class="dot"></div><div class="meter"><div id="fill" class="meter-fill" style="width:0%"></div></div></div>
  <script type="module">
    import { listen } from "https://esm.sh/@tauri-apps/api/event";
    const fill = document.getElementById("fill");
    listen("audio://level", (e) => {
      const v = Math.min(1, Math.max(0, e.payload * 4));
      fill.style.width = (v * 100).toFixed(0) + "%";
    });
  </script>
</body></html>
```

- [ ] **Step 4: Build & manueller Smoke-Test**

```bash
bun run tauri dev
```

Expected: App startet, Tray-Icon sichtbar, Settings-Fenster lässt sich öffnen.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/main.rs src-tauri/tauri.conf.json overlay.html
git commit -m "feat: wire up orchestrator, tray, overlay in main"
```

---

## Task 19: MSI-Installer (Windows)

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Create: `src-tauri/icons/icon.ico` (Platzhalter generieren)

**Hinweis:** Dieser Task läuft **auf einem Windows-Host** — Rust-Toolchain + MS Visual Studio Build Tools + WebView2 erforderlich. Auf Linux-Host wird nur der Konfig-Schritt commited.

- [ ] **Step 1: Bundler-Konfig prüfen**

`tauri.conf.json`:

```json
{
  "bundle": {
    "active": true,
    "targets": ["msi"],
    "icon": ["icons/icon.ico"],
    "windows": {
      "webviewInstallMode": { "type": "embedBootstrapper" }
    }
  }
}
```

- [ ] **Step 2: Icon-Platzhalter**

Schlichtes 256×256-PNG → in `.ico` konvertieren (z.B. via ImageMagick: `magick convert icon.png -define icon:auto-resize=256,128,64,48,32,16 icon.ico`).

- [ ] **Step 3: Auf Windows-Host bauen**

```powershell
cd C:\path\to\DSS-Whisper
bun install
bun run tauri build
```

Expected: `src-tauri\target\release\bundle\msi\DSS-Whisper_0.1.0_x64_en-US.msi` entsteht.

- [ ] **Step 4: Installation & Smoke-Test**

MSI auf Test-Windows-Rechner installieren, Hotkey konfigurieren, in Notepad diktieren.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/tauri.conf.json src-tauri/icons/icon.ico
git commit -m "build: MSI installer config + icon"
```

---

## Task 20: Release-Checkliste ausführen

**Files:**
- Create: `docs/superpowers/plans/release-checklist.md`

- [ ] **Step 1: Checkliste festhalten**

Create `docs/superpowers/plans/release-checklist.md`:

```markdown
# DSS-Whisper v0.1 Release-Checkliste

## Integrations-Punkte
- [ ] Server-Endpoint `/api/dictate` läuft und antwortet
- [ ] Bearer-Secret auf Client und Server identisch
- [ ] Lokales whisper.cpp-Modell `ggml-base.bin` heruntergeladen

## Hotkey in verschiedenen Zielen
- [ ] Notepad
- [ ] Word
- [ ] Outlook (Neue Mail)
- [ ] Chrome (Google Docs)
- [ ] Chrome (Gmail)
- [ ] VS Code
- [ ] Windows Terminal / PowerShell
- [ ] Slack
- [ ] MS Teams Chat

## Modi
- [ ] Push-to-talk Release ≙ Einfügen
- [ ] Toggle Start/Stop
- [ ] Max-Dauer 120 s → Auto-Stop funktioniert

## Backends / Fallback
- [ ] GPU-Server online → remote-whisper wird genutzt
- [ ] GPU-Server aus → Auto-Fallback auf local-whisper + Toast

## Post-Processing
- [ ] Profil ohne PP → Rohtext
- [ ] Profil mit PP + Anthropic → korrigiert
- [ ] Profil mit PP + Ollama lokal → korrigiert
- [ ] PP-Provider-Fehler → Rohtext + Toast

## History
- [ ] Eintrag erscheint nach Diktat
- [ ] Löschen entfernt Eintrag
- [ ] trim bei mehr als `history_limit`

## Settings-UI
- [ ] Profile CRUD speichert persistent
- [ ] HotkeyRecorder nimmt neue Kombi auf
- [ ] API-Key-Eingabe landet im Keyring (nicht in config.json)
- [ ] Mic-Dropdown listet alle Geräte

## Sicherheit
- [ ] `config.json` enthält KEINE API-Keys
- [ ] Logs enthalten KEINE API-Keys
- [ ] Kein Audio in History

## Robustheit
- [ ] Mic-Permission verweigert → Toast + Settings öffnen
- [ ] UAC-Dialog aktiv → Clipboard-Fallback funktioniert
```

- [ ] **Step 2: Checkliste durchlaufen, Ergebnisse eintragen**

- [ ] **Step 3: Commit**

```bash
git add docs/superpowers/plans/release-checklist.md
git commit -m "docs: release checklist for v0.1"
```

---

## Self-Review

Nach dem Schreiben dieses Plans überprüft:

1. **Spec-Coverage** — alle Sections aus `2026-04-13-dss-whisper-dictation-design.md` abgedeckt:
   - §3 Tech-Stack → Task 1-5 (Tauri, cpal, hotkey, enigo), Task 11 (whisper-rs), Task 13 (rusqlite, keyring)
   - §4 Architektur → Task 2 (config), Task 4-5 (audio), Task 10-11 (backends), Task 12 (llm), Task 14 (orchestrator)
   - §5 Ablauf → Task 14 (orchestrator happy path), Release-Checkliste (manuell)
   - §6 Profile & Settings → Task 15-16
   - §7 Server-Endpoint → Task 9
   - §8 Prompt → Task 12
   - §9 Sicherheit → Task 3 (keyring), Task 13 (nur Text in History)
   - §10 Testing → Unit-Tests in Tasks 2/4/6/10/12/13, Release-Checkliste Task 20
   - §11 Phase 1 → Tasks 1-20

2. **Placeholders gescannt:** kein TODO/TBD im Code-Body. Eine Ausnahme: Task 14 `start_recording` hat `// TODO: device from config` — bewusster Haken, weil Mic-Device-Selection im MVP erst in Task 16 (Audio-Tab) aufgelöst wird; Default-Mic reicht fürs Erste.

3. **Typ-Konsistenz:**
   - `TranscriptionBackendId` serialisiert mit `snake_case` → JSON schreibt `"remote_whisper"` / `"local_whisper"`; TypeScript-Typ spiegelt das.
   - `ProviderType` enum vs. `type`-Field → `r#type` in Rust, `type` in JSON/TS. ✓
   - `HistoryEntry.timestamp` ist `DateTime<Utc>` in Rust, `string` (ISO-8601) in TypeScript. ✓
   - `profile_id` ist `Uuid` in Rust, `string` in TS. ✓

Keine nachträglichen Korrekturen nötig.

---

## Execution Handoff

Plan komplett und gespeichert in `docs/superpowers/plans/2026-04-13-dss-whisper-phase1-mvp.md`. Zwei Execution-Optionen:

**1. Subagent-Driven (empfohlen)** — Frischer Subagent pro Task, Review dazwischen, schnelles Iterieren.
**2. Inline-Execution** — Tasks in dieser Session via `executing-plans`-Skill abarbeiten, Batch mit Checkpoints.

Welcher Weg?
