use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("config error: {0}")]
    Config(String),
    #[error("audio error: {0}")]
    Audio(String),
    #[error("transcription error: {0}")]
    Transcription(String),
    #[error("llm error: {0}")]
    Llm(String),
    #[error("injection error: {0}")]
    Inject(String),
    #[error("keyring error: {0}")]
    Keyring(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, AppError>;
