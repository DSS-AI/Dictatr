export type HotkeyMode = "push_to_talk" | "toggle";
export type Language = "de" | "en" | "auto";
export type TranscriptionBackendId = "remote_whisper" | "local_whisper";
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
