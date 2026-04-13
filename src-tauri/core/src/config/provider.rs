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
