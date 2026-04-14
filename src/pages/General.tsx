import { useEffect, useState } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { ipc } from "../ipc";
import type { AppConfig } from "../types";
import InfoTip from "../components/InfoTip";
import { checkForUpdate, installUpdate, type DownloadProgress } from "../lib/updater";
import type { Update } from "@tauri-apps/plugin-updater";

type UpdateState =
  | { kind: "idle" }
  | { kind: "checking" }
  | { kind: "current" }
  | { kind: "available"; update: Update }
  | { kind: "installing"; update: Update; progress: DownloadProgress | null }
  | { kind: "error"; message: string };

export default function General() {
  const [cfg, setCfg] = useState<AppConfig | null>(null);
  const [version, setVersion] = useState<string>("");
  const [upd, setUpd] = useState<UpdateState>({ kind: "idle" });

  useEffect(() => { ipc.getConfig().then(setCfg).catch(console.error); }, []);
  useEffect(() => { getVersion().then(setVersion).catch(() => setVersion("?")); }, []);

  if (!cfg) return <div>Lade…</div>;

  const save = (patch: Partial<AppConfig["general"]>) => {
    const next = { ...cfg, general: { ...cfg.general, ...patch } };
    setCfg(next);
    ipc.saveConfig(next);
  };

  const runCheck = async () => {
    setUpd({ kind: "checking" });
    try {
      const found = await checkForUpdate();
      setUpd(found ? { kind: "available", update: found } : { kind: "current" });
    } catch (e) {
      setUpd({ kind: "error", message: String(e) });
    }
  };

  const runInstall = async () => {
    if (upd.kind !== "available") return;
    const update = upd.update;
    setUpd({ kind: "installing", update, progress: null });
    try {
      await installUpdate(update, (progress) =>
        setUpd({ kind: "installing", update, progress }),
      );
    } catch (e) {
      setUpd({ kind: "error", message: String(e) });
    }
  };

  const showTips = cfg.general.show_tooltips !== false;
  return (
    <div>
      <h1>Allgemein</h1>
      <label><input type="checkbox" checked={cfg.general.autostart} onChange={e => save({ autostart: e.target.checked })} /> Mit Windows starten<InfoTip enabled={showTips} text="Dictatr beim Login automatisch in den Tray laden (aktuell manuell zu aktivieren via Windows-Autostart-Ordner)." /></label>
      <label><input type="checkbox" checked={cfg.general.sounds} onChange={e => save({ sounds: e.target.checked })} /> Sounds abspielen<InfoTip enabled={showTips} text="Kurzer Zwei-Ton-Chirp beim Start (aufsteigend) und Ende (absteigend) der Aufnahme." /></label>
      <label><input type="checkbox" checked={cfg.general.overlay} onChange={e => save({ overlay: e.target.checked })} /> Mini-Overlay einblenden<InfoTip enabled={showTips} text="Während der Aufnahme ein kleines, immer-oben-Fenster mit Status und Pegelanzeige zeigen." /></label>
      <label><input type="checkbox" checked={cfg.general.show_tooltips} onChange={e => save({ show_tooltips: e.target.checked })} /> Hilfe-Tooltips bei Mouse-Over anzeigen</label>
      <label><input type="checkbox" checked={cfg.general.check_updates !== false} onChange={e => save({ check_updates: e.target.checked })} /> Beim Start nach Updates suchen<InfoTip enabled={showTips} text="Ruft beim App-Start die latest.json von GitHub ab und zeigt bei neuer Version einen Banner. Deaktiviert = kein Netzwerk-Traffic ohne manuellen Klick." /></label>
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

      <fieldset>
        <legend>Updates</legend>
        <p style={{ margin: "4px 0 10px" }}>
          Aktuelle Version: <code>{version || "…"}</code>
        </p>
        <button
          className="secondary"
          onClick={runCheck}
          disabled={upd.kind === "checking" || upd.kind === "installing"}
        >
          {upd.kind === "checking" ? "Suche…" : "Nach Updates suchen"}
        </button>
        {upd.kind === "current" && (
          <small style={{ display: "block", marginTop: 8, color: "var(--success)" }}>
            Du hast bereits die aktuellste Version.
          </small>
        )}
        {upd.kind === "available" && (
          <small style={{ display: "block", marginTop: 8 }}>
            Version {upd.update.version} verfügbar.{" "}
            <button style={{ marginLeft: 6 }} onClick={runInstall}>
              Jetzt installieren
            </button>
          </small>
        )}
        {upd.kind === "installing" && (
          <small style={{ display: "block", marginTop: 8, color: "var(--text-dim)" }}>
            Installiere {upd.update.version}
            {upd.progress && upd.progress.total
              ? ` — ${Math.round((upd.progress.downloaded / upd.progress.total) * 100)} %`
              : "…"}
          </small>
        )}
        {upd.kind === "error" && (
          <small style={{ display: "block", marginTop: 8, color: "#f0a0a0" }}>
            Fehler: {upd.message}
          </small>
        )}
      </fieldset>
    </div>
  );
}
