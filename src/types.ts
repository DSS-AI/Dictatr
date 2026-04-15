export type HotkeyMode = "push_to_talk" | "toggle";
export type Language = "de" | "en" | "auto";
export type TranscriptionBackendId = "remote_whisper" | "local_whisper" | "llm_transcription";

export interface LlmTranscription {
  llm_provider_id: string | null;
  model: string | null;
}
export type ProviderType = "openai" | "openai_compatible" | "open_router" | "anthropic" | "ollama";

export interface PostProcessing {
  enabled: boolean;
  llm_provider_id: string | null;
  model: string | null;
  system_prompt: string | null;
}

export interface Profile {
  id: string;
  name: string;
  hotkey: string;
  hotkey_mode: HotkeyMode;
  transcription_backend: TranscriptionBackendId;
  language: Language;
  post_processing: PostProcessing;
  llm_transcription: LlmTranscription;
  clipboard_only: boolean;
  keep_on_clipboard: boolean;
}

export interface LlmProviderConfig {
  id: string;
  name: string;
  type: ProviderType;
  base_url: string;
  default_model: string;
}

export interface General {
  autostart: boolean;
  sounds: boolean;
  overlay: boolean;
  max_recording_seconds: number;
  history_limit: number;
  mic_device: string | null;
  remote_whisper_url: string;
  show_tooltips: boolean;
  check_updates: boolean;
}

export interface AppConfig {
  profiles: Profile[];
  providers: LlmProviderConfig[];
  general: General;
}

export interface HistoryEntry {
  id: number;
  timestamp: string;
  profile_id: string;
  profile_name: string;
  backend_id: string;
  duration_ms: number;
  text: string;
}
