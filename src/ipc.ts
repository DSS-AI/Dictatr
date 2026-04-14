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
};
