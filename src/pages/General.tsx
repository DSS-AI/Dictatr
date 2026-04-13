import { useEffect, useState } from "react";
import { ipc } from "../ipc";
import type { AppConfig } from "../types";

export default function General() {
  const [cfg, setCfg] = useState<AppConfig | null>(null);

  useEffect(() => { ipc.getConfig().then(setCfg).catch(console.error); }, []);

  if (!cfg) return <div>Lade…</div>;

  const save = (patch: Partial<AppConfig["general"]>) => {
    const next = { ...cfg, general: { ...cfg.general, ...patch } };
    setCfg(next);
    ipc.saveConfig(next);
  };

  return (
    <div>
      <h1>Allgemein</h1>
      <label><input type="checkbox" checked={cfg.general.autostart} onChange={e => save({ autostart: e.target.checked })} /> Mit Windows starten</label>
      <label><input type="checkbox" checked={cfg.general.sounds} onChange={e => save({ sounds: e.target.checked })} /> Sounds abspielen</label>
      <label><input type="checkbox" checked={cfg.general.overlay} onChange={e => save({ overlay: e.target.checked })} /> Mini-Overlay einblenden</label>
      <label>Maximale Aufnahmedauer (Sekunden)<input type="number" value={cfg.general.max_recording_seconds}
        onChange={e => save({ max_recording_seconds: parseInt(e.target.value) || 120 })} /></label>
      <label>History-Länge<input type="number" value={cfg.general.history_limit}
        onChange={e => save({ history_limit: parseInt(e.target.value) || 100 })} /></label>
    </div>
  );
}
