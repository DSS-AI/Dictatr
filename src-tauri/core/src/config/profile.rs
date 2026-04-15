use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HotkeyMode {
    PushToTalk,
    Toggle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    De,
    En,
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TranscriptionBackendId {
    RemoteWhisper,
    LocalWhisper,
    /// Use a chat-completion LLM with audio input (e.g. Gemini 2.5 Flash via
    /// OpenRouter, GPT-4o-audio-preview). Configured via `llm_transcription`.
    LlmTranscription,
}

fn default_llm_transcription() -> LlmTranscription {
    LlmTranscription { llm_provider_id: None, model: None }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LlmTranscription {
    pub llm_provider_id: Option<Uuid>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PostProcessing {
    pub enabled: bool,
    pub llm_provider_id: Option<Uuid>,
    pub model: Option<String>,
    pub system_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Profile {
    pub id: Uuid,
    pub name: String,
    pub hotkey: String,
    pub hotkey_mode: HotkeyMode,
    pub transcription_backend: TranscriptionBackendId,
    pub language: Language,
    pub post_processing: PostProcessing,
    #[serde(default = "default_llm_transcription")]
    pub llm_transcription: LlmTranscription,
    /// If true, only put the text on the clipboard — don't synthesize Ctrl+V.
    /// Useful for Remote Desktop sessions and other contexts where
    /// auto-paste isn't delivered to the right target (UIPI, RDP capture).
    #[serde(default)]
    pub clipboard_only: bool,
    /// If true, leave the transcribed text on the clipboard after injection
    /// (don't restore the user's previous clipboard content).
    #[serde(default)]
    pub keep_on_clipboard: bool,
}

impl Profile {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Profile name must not be empty".into());
        }
        if self.hotkey.trim().is_empty() {
            return Err("Hotkey must not be empty".into());
        }
        if self.post_processing.enabled && self.post_processing.llm_provider_id.is_none() {
            return Err("Post-processing enabled but no LLM provider selected".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_profile_json() {
        let json = r#"{
            "id": "00000000-0000-0000-0000-000000000001",
            "name": "Standard",
            "hotkey": "Ctrl+Alt+Space",
            "hotkey_mode": "push_to_talk",
            "transcription_backend": "remote_whisper",
            "language": "de",
            "post_processing": { "enabled": false, "llm_provider_id": null, "model": null, "system_prompt": null }
        }"#;

        let p: Profile = serde_json::from_str(json).unwrap();
        assert_eq!(p.name, "Standard");
        assert_eq!(p.hotkey_mode, HotkeyMode::PushToTalk);
        assert_eq!(p.language, Language::De);
        assert!(!p.post_processing.enabled);
    }

    #[test]
    fn validates_empty_name() {
        let p = Profile {
            id: Uuid::nil(),
            name: "  ".into(),
            hotkey: "Ctrl+Alt+Space".into(),
            hotkey_mode: HotkeyMode::Toggle,
            transcription_backend: TranscriptionBackendId::RemoteWhisper,
            language: Language::De,
            post_processing: PostProcessing {
                enabled: false, llm_provider_id: None, model: None, system_prompt: None,
            },
            llm_transcription: LlmTranscription { llm_provider_id: None, model: None },
        };
        assert!(p.validate().is_err());
    }

    #[test]
    fn validates_post_processing_without_provider() {
        let p = Profile {
            id: Uuid::nil(),
            name: "X".into(),
            hotkey: "Ctrl+Alt+X".into(),
            hotkey_mode: HotkeyMode::Toggle,
            transcription_backend: TranscriptionBackendId::RemoteWhisper,
            language: Language::Auto,
            post_processing: PostProcessing {
                enabled: true, llm_provider_id: None, model: None, system_prompt: None,
            },
            llm_transcription: LlmTranscription { llm_provider_id: None, model: None },
        };
        assert!(p.validate().is_err());
    }
}
