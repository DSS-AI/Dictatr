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
            .build()
            .unwrap();
        Self {
            base_url,
            bearer_token,
            client,
        }
    }

    fn lang_code(l: &Language) -> &'static str {
        match l {
            Language::De => "de",
            Language::En => "en",
            Language::Auto => "auto",
        }
    }
}

#[async_trait]
impl TranscriptionBackend for RemoteWhisperBackend {
    fn id(&self) -> &'static str {
        "remote-whisper"
    }

    async fn is_available(&self) -> bool {
        let url = format!("{}/api/health", self.base_url);
        self.client
            .get(&url)
            .timeout(Duration::from_secs(2))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    async fn transcribe(
        &self,
        samples: &[f32],
        language: Language,
        vocabulary: &[String],
    ) -> Result<Transcription> {
        let wav = pcm_to_wav_16k_mono(samples)?;
        let vocab = vocabulary.join(", ");

        let part = Part::bytes(wav)
            .file_name("audio.wav")
            .mime_str("audio/wav")
            .map_err(|e| AppError::Transcription(e.to_string()))?;
        let form = Form::new()
            .part("file", part)
            .text("language", Self::lang_code(&language).to_string())
            .text("vocabulary", vocab);

        let resp = self
            .client
            .post(format!("{}/api/dictate", self.base_url))
            .bearer_auth(&self.bearer_token)
            .multipart(form)
            .send()
            .await
            .map_err(|e| AppError::Transcription(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(AppError::Transcription(format!(
                "server status {}",
                resp.status()
            )));
        }
        let body: DictateResponse = resp
            .json()
            .await
            .map_err(|e| AppError::Transcription(e.to_string()))?;
        Ok(Transcription {
            text: body.text,
            duration_ms: body.duration_ms,
            backend_id: "remote-whisper",
        })
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
        Mock::given(method("POST"))
            .and(path("/api/dictate"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "text": "Hallo Welt",
                "duration_ms": 123,
                "backend": "mock"
            })))
            .mount(&server)
            .await;

        let be = RemoteWhisperBackend::new(server.uri(), "secret".into());
        let samples = vec![0.0f32; 16_000];
        let out = be.transcribe(&samples, Language::De, &[]).await.unwrap();
        assert_eq!(out.text, "Hallo Welt");
        assert_eq!(out.duration_ms, 123);
    }

    #[tokio::test]
    async fn returns_error_on_401() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/dictate"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&server)
            .await;

        let be = RemoteWhisperBackend::new(server.uri(), "wrong".into());
        let result = be.transcribe(&[0.0; 100], Language::De, &[]).await;
        assert!(result.is_err());
    }
}
