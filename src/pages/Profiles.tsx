import { useEffect, useState } from "react";
import { ipc } from "../ipc";
import type { AppConfig, Profile } from "../types";
import HotkeyRecorder from "../components/HotkeyRecorder";

export default function Profiles() {
  const [cfg, setCfg] = useState<AppConfig | null>(null);

  useEffect(() => { ipc.getConfig().then(setCfg).catch(console.error); }, []);

  if (!cfg) return <div>Lade…</div>;

  const save = (next: AppConfig) => { setCfg(next); ipc.saveConfig(next); };

  const add = () => {
    const p: Profile = {
      id: crypto.randomUUID(),
      name: "Neues Profil",
      hotkey: "Ctrl+Alt+Space",
      hotkey_mode: "push_to_talk",
      transcription_backend: "remote_whisper",
      language: "de",
      post_processing: { enabled: false, llm_provider_id: null, model: null, system_prompt: null },
    };
    save({ ...cfg, profiles: [...cfg.profiles, p] });
  };

  const update = (i: number, patch: Partial<Profile>) => {
    const profiles = cfg.profiles.map((p, idx) => idx === i ? { ...p, ...patch } : p);
    save({ ...cfg, profiles });
  };

  const remove = (i: number) =>
    save({ ...cfg, profiles: cfg.profiles.filter((_, idx) => idx !== i) });

  return (
    <div>
      <h1>Profile</h1>
      <button onClick={add}>+ Neues Profil</button>
      {cfg.profiles.length === 0 && <p style={{ color: "#888" }}>Noch keine Profile angelegt.</p>}
      {cfg.profiles.map((p, i) => (
        <fieldset key={p.id}>
          <legend>{p.name}</legend>
          <label>Name<input value={p.name} onChange={e => update(i, { name: e.target.value })} /></label>
          <label>Hotkey<HotkeyRecorder value={p.hotkey} onChange={v => update(i, { hotkey: v })} /></label>
          <label>Modus
            <select value={p.hotkey_mode} onChange={e => update(i, { hotkey_mode: e.target.value as Profile["hotkey_mode"] })}>
              <option value="push_to_talk">Push-to-talk</option>
              <option value="toggle">Toggle</option>
            </select>
          </label>
          <label>Backend
            <select value={p.transcription_backend} onChange={e => update(i, { transcription_backend: e.target.value as Profile["transcription_backend"] })}>
              <option value="remote_whisper">GPU-Server</option>
              <option value="local_whisper">Lokal (whisper.cpp)</option>
            </select>
          </label>
          <label>Sprache
            <select value={p.language} onChange={e => update(i, { language: e.target.value as Profile["language"] })}>
              <option value="de">Deutsch</option>
              <option value="en">Englisch</option>
              <option value="auto">Auto</option>
            </select>
          </label>
          <label>
            <input type="checkbox" checked={p.post_processing.enabled}
              onChange={e => update(i, { post_processing: { ...p.post_processing, enabled: e.target.checked } })} />
            Post-Processing aktiv
          </label>
          {p.post_processing.enabled && (
            <>
              <label>LLM-Provider
                <select value={p.post_processing.llm_provider_id ?? ""}
                  onChange={e => update(i, { post_processing: { ...p.post_processing, llm_provider_id: e.target.value || null } })}>
                  <option value="">— wählen —</option>
                  {cfg.providers.map(pr => <option key={pr.id} value={pr.id}>{pr.name}</option>)}
                </select>
              </label>
              <label>Modell<input value={p.post_processing.model ?? ""}
                onChange={e => update(i, { post_processing: { ...p.post_processing, model: e.target.value || null } })} /></label>
              <label>Eigener System-Prompt (optional)<textarea value={p.post_processing.system_prompt ?? ""}
                onChange={e => update(i, { post_processing: { ...p.post_processing, system_prompt: e.target.value || null } })} /></label>
            </>
          )}
          <button className="danger" onClick={() => remove(i)}>Profil löschen</button>
        </fieldset>
      ))}
    </div>
  );
}
