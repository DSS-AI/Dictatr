import { useState } from "react";
import type { Update } from "@tauri-apps/plugin-updater";
import { installUpdate, type DownloadProgress } from "../lib/updater";

type Props = {
  update: Update;
  onDismiss: () => void;
};

export default function UpdateBanner({ update, onDismiss }: Props) {
  const [installing, setInstalling] = useState(false);
  const [progress, setProgress] = useState<DownloadProgress | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleInstall = async () => {
    setInstalling(true);
    setError(null);
    try {
      await installUpdate(update, setProgress);
    } catch (e) {
      setInstalling(false);
      setError(String(e));
    }
  };

  const pct = progress && progress.total
    ? Math.min(100, Math.round((progress.downloaded / progress.total) * 100))
    : null;

  return (
    <div className="update-banner">
      <div className="update-banner-body">
        <span className="update-banner-title">
          Dictatr {update.version} ist verfügbar.
        </span>
        {installing && (
          <span className="update-banner-progress">
            {pct !== null ? `Lade… ${pct} %` : "Lade…"}
          </span>
        )}
        {error && <span className="update-banner-error">{error}</span>}
      </div>
      <div className="update-banner-actions">
        <button onClick={handleInstall} disabled={installing}>
          {installing ? "Installiere…" : "Installieren"}
        </button>
        <button
          className="secondary"
          onClick={onDismiss}
          disabled={installing}
          aria-label="Verwerfen"
        >
          ×
        </button>
      </div>
    </div>
  );
}
