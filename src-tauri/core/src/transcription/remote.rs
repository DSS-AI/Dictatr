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
    cf_access_client_id: String,
    cf_access_client_secret: String,
    client: reqwest::Client,
}

#[derive(Deserialize)]
struct DictateResponse {
    text: String,
}

impl RemoteWhisperBackend {
    pub fn new(base_url: String, bearer_token: String) -> Self {
        Self::with_cf_access(base_url, bearer_token, String::new(), String::new())
    }

    pub fn with_cf_access(
        base_url: String,
        bearer_token: String,
        cf_access_client_id: String,
        cf_access_client_secret: String,
    ) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();
        Self {
            base_url,
            bearer_token,
            cf_access_client_id,
            cf_access_client_secret,
            client,
        }
    }

    fn apply_cf_headers(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if !self.cf_access_client_id.is_empty() && !self.cf_access_client_secret.is_empty() {
            req.header("CF-Access-Client-Id", &self.cf_access_client_id)
                .header("CF-Access-Client-Secret", &self.cf_access_client_secret)
        } else {
            req
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
        // Probe the models endpoint (every OpenAI-compatible server has it).
        let url = format!("{}/v1/models", self.base_url);
        let req = self.client.get(&url).timeout(Duration::from_secs(2));
        self.apply_cf_headers(req)
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
        let started = std::time::Instant::now();

        let part = Part::bytes(wav)
            .file_name("audio.wav")
            .mime_str("audio/wav")
            .map_err(|e| AppError::Transcription(e.to_string()))?;
        let mut form = Form::new()
            .part("file", part)
            .text("model", "whisper-1".to_string())
            .text("response_format", "json".to_string());
        // Only set language when the user picked a specific one; "auto" means
        // let Whisper detect.
        let lang = Self::lang_code(&language);
        if lang != "auto" {
            form = form.text("language", lang.to_string());
        }
        if !vocabulary.is_empty() {
            form = form.text("prompt", vocabulary.join(", "));
        }

        let mut req = self
            .client
            .post(format!("{}/v1/audio/transcriptions", self.base_url));
        if !self.bearer_token.is_empty() {
            req = req.bearer_auth(&self.bearer_token);
        }
        req = self.apply_cf_headers(req);
        let resp = req
            .multipart(form)
            .send()
            .await
            .map_err(|e| AppError::Transcription(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Transcription(format!(
                "server status {status}: {body}"
            )));
        }
        let body: DictateResponse = resp
            .json()
            .await
            .map_err(|e| AppError::Transcription(e.to_string()))?;
        Ok(Transcription {
            text: body.text,
            duration_ms: started.elapsed().as_millis() as u64,
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
