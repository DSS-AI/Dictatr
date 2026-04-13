import { useEffect, useState } from "react";
import { ipc } from "../ipc";
import type { AppConfig } from "../types";
import LevelMeter from "../components/LevelMeter";

export default function Audio() {
  const [cfg, setCfg] = useState<AppConfig | null>(null);
  const [devices, setDevices] = useState<string[]>([]);

  useEffect(() => {
    ipc.getConfig().then(setCfg).catch(console.error);
    ipc.listInputDevices().then(setDevices).catch(console.error);
  }, []);

  if (!cfg) return <div>Lade…</div>;

  const save = (micDevice: string | null) => {
    const next = { ...cfg, general: { ...cfg.general, mic_device: micDevice } };
    setCfg(next);
    ipc.saveConfig(next);
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
      <label>Live-Pegelanzeige
        <LevelMeter />
      </label>
    </div>
  );
}
