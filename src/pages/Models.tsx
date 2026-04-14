import { useEffect, useState } from "react";
import { ipc, type ModelInfo, type DownloadProgress } from "../ipc";

function formatBytes(b: number): string {
  if (b < 1024) return `${b} B`;
  if (b < 1024 * 1024) return `${(b / 1024).toFixed(1)} KB`;
  if (b < 1024 * 1024 * 1024) return `${(b / 1024 / 1024).toFixed(1)} MB`;
  return `${(b / 1024 / 1024 / 1024).toFixed(2)} GB`;
}

export default function Models() {
  const [dir, setDir] = useState<string>("");
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [progress, setProgress] = useState<DownloadProgress | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refresh = () => {
    ipc.listModels().then(setModels).catch(e => setError(String(e)));
  };

  useEffect(() => {
    ipc.getModelsDir().then(setDir).catch(console.error);
    refresh();
    const id = setInterval(() => {
      ipc.getDownloadProgress()
        .then(p => {
          setProgress(p);
          if (p.done) refresh();
        })
        .catch(() => {});
    }, 500);
    return () => clearInterval(id);
  }, []);

  const startDownload = async (name: string) => {
    setError(null);
    try {
      await ipc.startModelDownload(name);
    } catch (e) {
      setError(String(e));
    }
  };

  const remove = async (name: string) => {
    if (!confirm(`Modell ${name} wirklich löschen?`)) return;
    try {
      await ipc.deleteModel(name);
      refresh();
    } catch (e) { setError(String(e)); }
  };

  const downloading = progress?.name && !progress.done;
  const pct = progress && progress.total > 0
    ? Math.min(100, Math.round((progress.downloaded / progress.total) * 100))
    : 0;

  return (
    <div>
      <h1>Whisper-Modelle (lokal)</h1>
      <p style={{ color: "#888" }}>
        Werden für das lokale Whisper-Backend benötigt. Nach dem Download bitte
        die App einmal neu starten (Tray → Beenden → App aus dem Startmenü
        starten), damit das Modell geladen wird.
      </p>
      <div style={{ fontFamily: "monospace", fontSize: 12, background: "#f0f0f0", color: "#333", padding: 8, borderRadius: 4, marginBottom: 12 }}>
        Pfad: {dir || "…"}
      </div>

      {downloading && (
        <div style={{ padding: 10, background: "#eef6ff", border: "1px solid #99c", borderRadius: 4, marginBottom: 12 }}>
          <div>Lade <b>{progress!.name}</b> …</div>
          <div style={{ background: "#ddd", height: 8, borderRadius: 4, overflow: "hidden", marginTop: 6 }}>
            <div style={{ width: `${pct}%`, height: "100%", background: "#28a", transition: "width 300ms" }} />
          </div>
          <small>
            {formatBytes(progress!.downloaded)}
            {progress!.total > 0 ? ` / ${formatBytes(progress!.total)}` : ""}
            {progress!.total > 0 ? ` (${pct}%)` : ""}
          </small>
        </div>
      )}
      {progress?.error && (
        <p style={{ color: "#b23" }}>Download-Fehler: {progress.error}</p>
      )}
      {error && <p style={{ color: "#b23" }}>{error}</p>}

      <table style={{ width: "100%", borderCollapse: "collapse" }}>
        <thead>
          <tr style={{ textAlign: "left", borderBottom: "1px solid #555" }}>
            <th style={{ padding: 6 }}>Modell</th>
            <th style={{ padding: 6 }}>Größe</th>
            <th style={{ padding: 6 }}>Status</th>
            <th style={{ padding: 6 }}>Aktion</th>
          </tr>
        </thead>
        <tbody>
          {models.map(m => (
            <tr key={m.name} style={{ borderBottom: "1px solid #333" }}>
              <td style={{ padding: 6 }}>{m.name}</td>
              <td style={{ padding: 6 }}>~{m.size_mb} MB</td>
              <td style={{ padding: 6 }}>
                {m.installed
                  ? <span style={{ color: "#2d7" }}>✓ installiert ({formatBytes(m.installed_bytes)})</span>
                  : <span style={{ color: "#888" }}>– nicht installiert</span>}
              </td>
              <td style={{ padding: 6 }}>
                {m.installed
                  ? <button className="danger" onClick={() => remove(m.name)} disabled={!!downloading}>Löschen</button>
                  : <button onClick={() => startDownload(m.name)} disabled={!!downloading}>Herunterladen</button>}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
