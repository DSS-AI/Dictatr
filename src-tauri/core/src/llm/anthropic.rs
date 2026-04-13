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
