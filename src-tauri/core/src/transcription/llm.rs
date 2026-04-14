//! Transcription backend that uses a chat-completion LLM with audio input.
//!
//! OpenAI-compatible servers (OpenAI itself, OpenRouter, Groq, DeepInfra, …)
//! accept audio via the `input_audio` content part. Gemini 2.5 Flash via
//! OpenRouter and `gpt-4o-audio-preview` directly at OpenAI both understand
//! this payload shape. Plain text-only models (Claude, older GPT-4) will
//! return an error — that's surfaced to the user in the orchestrator.
use super::{pcm_to_wav_16k_mono, Transcription, TranscriptionBackend};
use crate::config::profile::Language;
use crate::error::{AppError, Result};
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;

pub struct LlmTranscriptionBackend {
    base_url: String,
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl LlmTranscriptionBackend {
    pub fn new(base_url: String, api_key: String, model: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .unwrap();
        Self { base_url, api_key, model, client }
    }

    fn language_hint(lang: &Language) -> &'static str {
        match lang {
            Language::De => "Deutsch",
            Language::En => "English",
            Language::Auto => "der gesprochenen Sprache",
        }
    }
}

#[derive(Deserialize)]
struct ChatResponse { choices: Vec<Choice> }
#[derive(Deserialize)]
struct Choice { message: ChoiceMessage }
#[derive(Deserialize)]
struct ChoiceMessage { content: String }

#[async_trait]
impl TranscriptionBackend for LlmTranscriptionBackend {
    fn id(&self) -> &'static str { "llm" }

    async fn is_available(&self) -> bool { true }

    async fn transcribe(
        &self,
        samples: &[f32],
        language: Language,
        vocabulary: &[String],
    ) -> Result<Transcription> {
        let started = std::time::Instant::now();
        let wav = pcm_to_wav_16k_mono(samples)?;
        let b64 = B64.encode(&wav);

        let mut instructions = format!(
            "Transkribiere das Audio wörtlich in {}. Antworte ausschließlich mit dem reinen Transkript — keine Einleitungen, keine Erklärungen, kein Markdown.",
            Self::language_hint(&language)
        );
        if !vocabulary.is_empty() {
            instructions.push_str("\nBerücksichtige diese Fachbegriffe beim Transkribieren: ");
            instructions.push_str(&vocabulary.join(", "));
            instructions.push('.');
        }

        let body = json!({
            "model": self.model,
            "modalities": ["text"],
            "messages": [{
                "role": "user",
                "content": [
                    { "type": "input_audio", "input_audio": { "data": b64, "format": "wav" } },
                    { "type": "text", "text": instructions }
                ]
            }],
            "temperature": 0.0
        });

        let resp = self.client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Transcription(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Transcription(format!(
                "LLM-Transkription {status}: {body}"
            )));
        }
        let parsed: ChatResponse = resp
            .json()
            .await
            .map_err(|e| AppError::Transcription(e.to_string()))?;
        let text = parsed.choices.into_iter().next()
            .map(|c| c.message.content.trim().to_string())
            .ok_or_else(|| AppError::Transcription("Leere LLM-Antwort".into()))?;

        Ok(Transcription {
            text,
            duration_ms: started.elapsed().as_millis() as u64,
            backend_id: "llm",
        })
    }
}
