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

fn default_remote_whisper_url() -> String {
    "http://192.168.178.43:8000".to_string()
}

/// Ensure a user-entered URL has a scheme. Bare hostnames like
/// `whisper.example.com` get `https://` prepended; IPs on well-known LAN
/// ports stay `http://` only if the user typed that explicitly.
pub fn normalize_remote_url(url: &str) -> String {
    let trimmed = url.trim().trim_end_matches('/');
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    }
}

fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct General {
    pub autostart: bool,
    pub sounds: bool,
    pub overlay: bool,
    pub max_recording_seconds: u32,
    pub history_limit: u32,
    pub mic_device: Option<String>,
    #[serde(default = "default_remote_whisper_url")]
    pub remote_whisper_url: String,
    /// Cloudflare Access service-token Client-ID (public). Paired with a
    /// secret stored in the OS keyring under `named-cf_access_secret`.
    /// Empty string or None → no Cloudflare Access headers are sent.
    #[serde(default)]
    pub cf_access_client_id: String,
    #[serde(default = "default_true")]
    pub show_tooltips: bool,
    #[serde(default = "default_true")]
    pub check_updates: bool,
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
            remote_whisper_url: default_remote_whisper_url(),
            cf_access_client_id: String::new(),
            show_tooltips: true,
            check_updates: true,
        }
    }
}

pub fn config_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("de", "dss", "Dictatr")
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

    #[test]
    fn normalize_prepends_https_when_no_scheme() {
        assert_eq!(normalize_remote_url("whisper.example.com"), "https://whisper.example.com");
        assert_eq!(normalize_remote_url("whisper.example.com/"), "https://whisper.example.com");
        assert_eq!(normalize_remote_url("  whisper.example.com  "), "https://whisper.example.com");
    }

    #[test]
    fn normalize_keeps_explicit_scheme() {
        assert_eq!(normalize_remote_url("http://192.168.1.5:8000"), "http://192.168.1.5:8000");
        assert_eq!(normalize_remote_url("https://whisper.example.com"), "https://whisper.example.com");
        assert_eq!(normalize_remote_url("https://whisper.example.com/"), "https://whisper.example.com");
    }
}
