import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export type DownloadProgress = {
  downloaded: number;
  total: number | null;
};

export async function checkForUpdate(): Promise<Update | null> {
  const update = await check();
  return update ?? null;
}

export async function installUpdate(
  update: Update,
  onProgress?: (p: DownloadProgress) => void,
): Promise<void> {
  let downloaded = 0;
  let total: number | null = null;
  await update.downloadAndInstall((event) => {
    if (event.event === "Started") {
      total = event.data.contentLength ?? null;
      onProgress?.({ downloaded: 0, total });
    } else if (event.event === "Progress") {
      downloaded += event.data.chunkLength;
      onProgress?.({ downloaded, total });
    } else if (event.event === "Finished") {
      onProgress?.({ downloaded: total ?? downloaded, total });
    }
  });
  await relaunch();
}
