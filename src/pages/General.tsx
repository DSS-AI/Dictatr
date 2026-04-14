import { useEffect, useState } from "react";
import { ipc } from "../ipc";
import type { AppConfig } from "../types";
import InfoTip from "../components/InfoTip";

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
      {(() => { const showTips = cfg.general.show_tooltips !== false; return <>
      <label><input type="checkbox" checked={cfg.general.autostart} onChange={e => save({ autostart: e.target.checked })} /> Mit Windows starten<InfoTip enabled={showTips} text="Dictatr beim Login automatisch in den Tray laden (aktuell manuell zu aktivieren via Windows-Autostart-Ordner)." /></label>
      <label><input type="checkbox" checked={cfg.general.sounds} onChange={e => save({ sounds: e.target.checked })} /> Sounds abspielen<InfoTip enabled={showTips} text="Kurzer Zwei-Ton-Chirp beim Start (aufsteigend) und Ende (absteigend) der Aufnahme." /></label>
      <label><input type="checkbox" checked={cfg.general.overlay} onChange={e => save({ overlay: e.target.checked })} /> Mini-Overlay einblenden<InfoTip enabled={showTips} text="Während der Aufnahme ein kleines, immer-oben-Fenster mit Status und Pegelanzeige zeigen." /></label>
      <label><input type="checkbox" checked={cfg.general.show_tooltips} onChange={e => save({ show_tooltips: e.target.checked })} /> Hilfe-Tooltips bei Mouse-Over anzeigen</label>
      <label>Maximale Aufnahmedauer (Sekunden)<InfoTip enabled={showTips} text="Obergrenze für eine einzelne Aufnahme. Der Ringbuffer verwirft beim Überschreiten die ältesten Samples." />
        <input type="number" value={cfg.general.max_recording_seconds}
        onChange={e => save({ max_recording_seconds: parseInt(e.target.value) || 120 })} /></label>
      <label>History-Länge<InfoTip enabled={showTips} text="Wie viele Transkripte im History-Tab aufbewahrt werden. Ältere werden automatisch gelöscht." />
        <input type="number" value={cfg.general.history_limit}
        onChange={e => save({ history_limit: parseInt(e.target.value) || 100 })} /></label>
      <label>GPU-Server-Adresse (für Backend „GPU-Server")<InfoTip enabled={showTips} text="Root-URL des OpenAI-kompatiblen Whisper-Servers (z. B. faster-whisper-server im LAN). Dictatr ruft /v1/audio/transcriptions an dieser Adresse auf." />
        <input value={cfg.general.remote_whisper_url}
          onChange={e => save({ remote_whisper_url: e.target.value })}
          placeholder="http://whisper:8000" />
        <small style={{ color: "#888" }}>Änderung greift nach App-Neustart.</small>
      </label>
      </>; })()}
    </div>
  );
}
