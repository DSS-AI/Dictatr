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
