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
        let path_str = model_path
            .to_str()
            .ok_or_else(|| AppError::Transcription("bad model path".into()))?;
        let ctx = WhisperContext::new_with_params(path_str, WhisperContextParameters::default())
            .map_err(|e| AppError::Transcription(e.to_string()))?;
        Ok(Self {
            ctx: Arc::new(ctx),
        })
    }

    fn lang_code(l: &Language) -> Option<&'static str> {
        match l {
            Language::De => Some("de"),
            Language::En => Some("en"),
            Language::Auto => None,
        }
    }
}

#[async_trait]
impl TranscriptionBackend for LocalWhisperBackend {
    fn id(&self) -> &'static str {
        "local-whisper"
    }

    async fn transcribe(
        &self,
        samples: &[f32],
        language: Language,
        vocabulary: &[String],
    ) -> Result<Transcription> {
        let samples_owned = samples.to_vec();
        let vocab_prompt = vocabulary.join(", ");
        let ctx = self.ctx.clone();
        let lang = Self::lang_code(&language);

        let (text, ms) =
            tokio::task::spawn_blocking(move || -> Result<(String, u64)> {
                let mut state = ctx
                    .create_state()
                    .map_err(|e| AppError::Transcription(e.to_string()))?;
                let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
                if let Some(l) = lang {
                    params.set_language(Some(l));
                }
                if !vocab_prompt.is_empty() {
                    params.set_initial_prompt(&vocab_prompt);
                }
                params.set_print_progress(false);
                params.set_print_realtime(false);
                params.set_print_special(false);

                let start = Instant::now();
                state
                    .full(params, &samples_owned)
                    .map_err(|e| AppError::Transcription(e.to_string()))?;

                let num_segments = state
                    .full_n_segments()
                    .map_err(|e| AppError::Transcription(e.to_string()))?;
                let mut text = String::new();
                for i in 0..num_segments {
                    text.push_str(
                        &state
                            .full_get_segment_text(i)
                            .map_err(|e| AppError::Transcription(e.to_string()))?,
                    );
                }
                Ok((text.trim().to_string(), start.elapsed().as_millis() as u64))
            })
            .await
            .map_err(|e| AppError::Transcription(e.to_string()))??;

        Ok(Transcription {
            text,
            duration_ms: ms,
            backend_id: "local-whisper",
        })
    }
}
