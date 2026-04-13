pub mod profile;
pub mod provider;

use crate::error::{AppError, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub profiles: Vec<profile::Profile>,
    #[serde(default)]
    pub providers: Vec<provider::LlmProviderConfig>,
    #[serde(default)]
    pub general: General,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct General {
    pub autostart: bool,
    pub sounds: bool,
    pub overlay: bool,
    pub max_recording_seconds: u32,
    pub history_limit: u32,
    pub mic_device: Option<String>,
}

impl Default for General {
    fn default() -> Self {
        Self {
            autostart: false,
            sounds: true,
            overlay: true,
            max_recording_seconds: 120,
            history_limit: 100,
            mic_device: None,
        }
    }
}

pub fn config_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("de", "dss", "Whisper")
        .ok_or_else(|| AppError::Config("could not resolve app dirs".into()))?;
    Ok(dirs.config_dir().join("config.json"))
}

pub fn load() -> Result<AppConfig> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let data = std::fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&data)?)
}

pub fn save(cfg: &AppConfig) -> Result<()> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(cfg)?;
    std::fs::write(&path, data)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_roundtrips() {
        let cfg = AppConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let back: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.general.max_recording_seconds, 120);
    }
}
