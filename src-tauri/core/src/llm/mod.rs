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
