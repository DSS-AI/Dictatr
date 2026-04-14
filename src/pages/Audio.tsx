import { useEffect, useState } from "react";
import { ipc } from "../ipc";
import type { AppConfig } from "../types";
import LevelMeter from "../components/LevelMeter";

export default function Audio() {
  const [cfg, setCfg] = useState<AppConfig | null>(null);
  const [devices, setDevices] = useState<string[]>([]);
  const [previewing, setPreviewing] = useState(false);
  const [previewError, setPreviewError] = useState<string | null>(null);

  useEffect(() => {
    ipc.getConfig().then(setCfg).catch(console.error);
    ipc.listInputDevices().then(setDevices).catch(console.error);
    return () => { ipc.stopMicPreview().catch(() => {}); };
  }, []);

  if (!cfg) return <div>Lade…</div>;

  const stopIfActive = async () => {
    if (previewing) {
      await ipc.stopMicPreview().catch(() => {});
      setPreviewing(false);
    }
  };

  const save = async (micDevice: string | null) => {
    await stopIfActive();
    const next = { ...cfg, general: { ...cfg.general, mic_device: micDevice } };
    setCfg(next);
    ipc.saveConfig(next);
  };

  const togglePreview = async () => {
    setPreviewError(null);
    if (previewing) {
      try { await ipc.stopMicPreview(); } catch (e) { setPreviewError(String(e)); }
      setPreviewing(false);
    } else {
      try {
        await ipc.startMicPreview(cfg.general.mic_device ?? null);
        setPreviewing(true);
      } catch (e) { setPreviewError(String(e)); }
    }
  };

  return (
    <div>
      <h1>Audio</h1>
      <label>Mikrofon
        <select value={cfg.general.mic_device ?? ""} onChange={e => save(e.target.value || null)}>
          <option value="">System-Standard</option>
          {devices.map(d => <option key={d} value={d}>{d}</option>)}
        </select>
      </label>
      <div>
        <label>Live-Pegelanzeige</label>
        <LevelMeter />
        <div style={{ display: "flex", gap: 8, marginTop: 8 }}>
          <button onClick={togglePreview}>
            {previewing ? "Mikrofon-Test stoppen" : "Mikrofon testen"}
          </button>
          {previewing && <small style={{ color: "#888" }}>Sprich ins Mikro — der Pegel sollte ausschlagen.</small>}
        </div>
        {previewError && <p style={{ color: "#b23" }}>Fehler: {previewError}</p>}
      </div>
    </div>
  );
}
