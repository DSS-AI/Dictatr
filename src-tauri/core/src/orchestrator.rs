use crate::audio::controller::AudioController;
use crate::config::profile::{HotkeyMode, Profile};
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
    /// Async-safe handle to the dedicated audio thread.
    pub audio: AudioController,
    /// All configured profiles, keyed by their UUID.
    pub profiles: HashMap<Uuid, Profile>,
    /// Preferred transcription backend (e.g. remote Whisper).
    pub primary_backend: Arc<dyn TranscriptionBackend>,
    /// Fallback backend used when the primary is unavailable.
    pub fallback_backend: Arc<dyn TranscriptionBackend>,
    /// Available LLM providers for post-processing, keyed by their UUID.
    pub llm_providers: HashMap<Uuid, Arc<dyn LlmProvider>>,
    /// Shared vocabulary injected into transcription hints and LLM prompts.
    pub vocabulary: Vec<String>,
    /// Persistent transcription history.
    pub history: Arc<HistoryStore>,
    /// Observable application state (Idle / Recording / Transcribing / …).
    pub state: Arc<Mutex<AppState>>,
    /// Which profile is currently active in Toggle mode (`None` = not toggled on).
    pub toggle_active_profile: Arc<Mutex<Option<Uuid>>>,
    /// Override microphone device name (`None` = OS default).
    pub mic_device: Option<String>,
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
        self.audio
            .start_recording(self.mic_device.clone())
            .await?;
        let mut s = self.state.lock();
        *s = (*s).apply(Transition::StartRecording);
        Ok(())
    }

    async fn stop_and_process(&mut self, profile: &Profile) -> Result<()> {
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
        let backend = self.select_backend().await;
        let transcription = backend
            .transcribe(&samples, profile.language.clone(), &self.vocabulary)
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
                        &self.vocabulary,
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

    /// Return the primary backend when available, otherwise the fallback.
    async fn select_backend(&self) -> Arc<dyn TranscriptionBackend> {
        if self.primary_backend.is_available().await {
            self.primary_backend.clone()
        } else {
            eprintln!("primary backend unavailable, using fallback");
            self.fallback_backend.clone()
        }
    }
}
