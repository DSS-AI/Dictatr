use crate::audio::controller::AudioController;
use crate::config::profile::{HotkeyMode, Profile, TranscriptionBackendId};
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

/// Central coordinator that drives the record → transcribe → post-process →
/// inject pipeline in response to hotkey events.
///
/// # Thread-safety
///
/// `cpal::Stream` is `!Send`. `AudioController` solves this by confining
/// `AudioCapture` (and its stream) to a dedicated OS thread while exposing an
/// async command interface. The `Orchestrator` therefore only holds
/// `AudioController` (which *is* `Send`) and can live inside a normal
/// `tokio::spawn` task.
pub struct Orchestrator {
    /// Async-safe handle to the dedicated audio thread (shared so the UI can
    /// also drive a mic-level preview via Tauri state).
    pub audio: Arc<AudioController>,
    /// All configured profiles, keyed by their UUID.
    pub profiles: HashMap<Uuid, Profile>,
    /// Remote Whisper (server) backend — always available.
    pub remote_backend: Arc<dyn TranscriptionBackend>,
    /// Local whisper.cpp backend — present only if a model file was found.
    pub local_backend: Option<Arc<dyn TranscriptionBackend>>,
    /// Available LLM providers for post-processing, keyed by their UUID.
    pub llm_providers: HashMap<Uuid, Arc<dyn LlmProvider>>,
    /// Raw provider configs + API keys, needed to build on-demand backends
    /// (currently `LlmTranscriptionBackend`). Keyed by the provider's UUID.
    pub llm_provider_keys: HashMap<Uuid, (crate::config::provider::LlmProviderConfig, String)>,
    /// Shared vocabulary injected into transcription hints and LLM prompts;
    /// live-updatable from the UI (Vocabulary tab) without restarting the app.
    pub vocabulary: Arc<Mutex<Vec<String>>>,
    /// Persistent transcription history.
    pub history: Arc<HistoryStore>,
    /// Observable application state (Idle / Recording / Transcribing / …).
    pub state: Arc<Mutex<AppState>>,
    /// Which profile is currently active in Toggle mode (`None` = not toggled on).
    pub toggle_active_profile: Arc<Mutex<Option<Uuid>>>,
    /// Override microphone device name (`None` = OS default).
    pub mic_device: Option<String>,
    /// Play audio cues on record start/stop when enabled.
    pub sounds_enabled: bool,
}

impl Orchestrator {
    /// Drive the orchestrator from a stream of hotkey events.
    ///
    /// Returns when the sender side of `rx` is dropped (i.e. the application
    /// is shutting down).
    pub async fn run_loop(&mut self, mut rx: UnboundedReceiver<HotkeyEvent>) {
        while let Some(event) = rx.recv().await {
            if let Err(e) = self.handle(event).await {
                eprintln!("orchestrator error: {e:?}");
                let mut s = self.state.lock();
                *s = (*s).apply(Transition::Fail);
            }
        }
    }

    // -------------------------------------------------------------------------
    // Private helpers
    // -------------------------------------------------------------------------

    async fn handle(&mut self, event: HotkeyEvent) -> Result<()> {
        match event {
            HotkeyEvent::Pressed(pid) => {
                let profile = self
                    .profiles
                    .get(&pid)
                    .cloned()
                    .ok_or_else(|| AppError::Config(format!("unknown profile {pid}")))?;

                match profile.hotkey_mode {
                    HotkeyMode::PushToTalk => {
                        self.start_recording(&profile).await?;
                    }
                    HotkeyMode::Toggle => {
                        // Take whatever profile was previously active.
                        let active_now = self.toggle_active_profile.lock().take();
                        if let Some(active_pid) = active_now {
                            // Already recording → stop and process.
                            let active_profile = self
                                .profiles
                                .get(&active_pid)
                                .cloned()
                                .ok_or_else(|| AppError::Config("toggle profile gone".into()))?;
                            self.stop_and_process(&active_profile).await?;
                        } else {
                            // Not recording → start.
                            *self.toggle_active_profile.lock() = Some(pid);
                            self.start_recording(&profile).await?;
                        }
                    }
                }
            }

            HotkeyEvent::Released(pid) => {
                let profile = match self.profiles.get(&pid).cloned() {
                    Some(p) => p,
                    None => return Ok(()),
                };
                // Only PushToTalk stops on key-release.
                if matches!(profile.hotkey_mode, HotkeyMode::PushToTalk) {
                    self.stop_and_process(&profile).await?;
                }
            }
        }
        Ok(())
    }

    async fn start_recording(&mut self, _profile: &Profile) -> Result<()> {
        if self.sounds_enabled {
            crate::sound::play_start();
        }
        self.audio
            .start_recording(self.mic_device.clone())
            .await?;
        let mut s = self.state.lock();
        *s = (*s).apply(Transition::StartRecording);
        Ok(())
    }

    async fn stop_and_process(&mut self, profile: &Profile) -> Result<()> {
        if self.sounds_enabled {
            crate::sound::play_stop();
        }
        let samples = self.audio.stop_and_drain().await?;
        {
            let mut s = self.state.lock();
            *s = (*s).apply(Transition::StopRecording);
        }

        if samples.is_empty() {
            *self.state.lock() = AppState::Idle;
            return Ok(());
        }

        // --- Transcribe ---------------------------------------------------------
        // Snapshot the vocabulary outside the .await so the MutexGuard (which
        // is !Send for parking_lot) doesn't cross the suspension point.
        let vocab_snapshot: Vec<String> = self.vocabulary.lock().clone();
        let backend = self.backend_for(profile)?;
        let transcription = backend
            .transcribe(&samples, profile.language.clone(), &vocab_snapshot)
            .await?;

        let mut final_text = transcription.text.clone();

        // --- Post-process (optional LLM correction) ----------------------------
        if profile.post_processing.enabled {
            if let (Some(provider_id), Some(model)) = (
                profile.post_processing.llm_provider_id,
                profile.post_processing.model.as_deref(),
            ) {
                if let Some(provider) = self.llm_providers.get(&provider_id) {
                    let prompt = build_prompt(
                        &transcription.text,
                        &vocab_snapshot,
                        profile.post_processing.system_prompt.as_deref(),
                    );
                    match provider.complete(&prompt.system, &prompt.user, model).await {
                        Ok(corrected) => final_text = corrected,
                        Err(e) => {
                            eprintln!("post-processing failed, using raw text: {e:?}");
                        }
                    }
                }
            }
        }

        {
            let mut s = self.state.lock();
            *s = (*s).apply(Transition::TranscriptionDone);
        }

        // --- Inject -------------------------------------------------------------
        if let Err(e) = TextInjector::inject(&final_text) {
            eprintln!("injection failed, falling back to clipboard: {e:?}");
            TextInjector::clipboard_fallback(&final_text)?;
        }

        {
            let mut s = self.state.lock();
            *s = (*s).apply(Transition::InjectionDone);
        }

        // --- Persist to history -------------------------------------------------
        self.history.insert(&HistoryEntry {
            id: 0,
            timestamp: Utc::now(),
            profile_id: profile.id,
            profile_name: profile.name.clone(),
            backend_id: transcription.backend_id.to_string(),
            duration_ms: transcription.duration_ms,
            text: final_text,
        })?;

        Ok(())
    }

    /// Pick the backend the profile asked for. `LocalWhisper` errors out
    /// with an actionable message when no model is installed; `RemoteWhisper`
    /// always returns the remote (availability is the caller's concern).
    fn backend_for(&self, profile: &Profile) -> Result<Arc<dyn TranscriptionBackend>> {
        match profile.transcription_backend {
            TranscriptionBackendId::RemoteWhisper => Ok(self.remote_backend.clone()),
            TranscriptionBackendId::LocalWhisper => self.local_backend.clone().ok_or_else(|| {
                AppError::Transcription(
                    "Lokales Whisper-Modell nicht installiert — bitte unter \"Modelle\" herunterladen oder Remote-Backend wählen".into(),
                )
            }),
            TranscriptionBackendId::LlmTranscription => {
                let pid = profile.llm_transcription.llm_provider_id.ok_or_else(|| {
                    AppError::Transcription("LLM-Provider im Profil nicht gesetzt".into())
                })?;
                let (cfg, key) = self.llm_provider_keys.get(&pid).ok_or_else(|| {
                    AppError::Transcription(
                        "LLM-Provider hat keinen API-Key oder wurde gelöscht".into(),
                    )
                })?;
                let model = profile.llm_transcription.model.clone()
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or_else(|| cfg.default_model.clone());
                if model.trim().is_empty() {
                    return Err(AppError::Transcription("Kein Modell angegeben".into()));
                }
                Ok(Arc::new(crate::transcription::llm::LlmTranscriptionBackend::new(
                    cfg.base_url.clone(), key.clone(), model,
                )))
            }
        }
    }
}
