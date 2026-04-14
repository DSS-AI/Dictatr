import { invoke } from "@tauri-apps/api/core";
import type { AppConfig, HistoryEntry } from "./types";

export const ipc = {
  getConfig: () => invoke<AppConfig>("get_config"),
  saveConfig: (cfg: AppConfig) => invoke<void>("save_config", { cfg }),
  setApiKey: (providerId: string, key: string) =>
    invoke<void>("set_api_key", { providerId, key }),
  listInputDevices: () => invoke<string[]>("list_input_devices"),
  listHistory: (limit: number) => invoke<HistoryEntry[]>("list_history", { limit }),
  deleteHistory: (id: number) => invoke<void>("delete_history", { id }),
  testLlmProvider: (providerId: string) =>
    invoke<string>("test_llm_provider", { providerId }),
  startMicPreview: (device: string | null) =>
    invoke<void>("start_mic_preview", { device }),
  stopMicPreview: () => invoke<void>("stop_mic_preview"),
  getAudioLevel: () => invoke<number>("get_audio_level"),
  getModelsDir: () => invoke<string>("get_models_dir"),
  listModels: () => invoke<ModelInfo[]>("list_models"),
  startModelDownload: (name: string) => invoke<void>("start_model_download", { name }),
  getDownloadProgress: () => invoke<DownloadProgress>("get_download_progress"),
  deleteModel: (name: string) => invoke<void>("delete_model", { name }),
};

export interface ModelInfo {
  name: string;
  filename: string;
  size_mb: number;
  installed: boolean;
  installed_bytes: number;
}

export interface DownloadProgress {
  name: string | null;
  downloaded: number;
  total: number;
  done: boolean;
  error: string | null;
}
