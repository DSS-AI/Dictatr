import { useEffect, useState } from "react";
import { ipc } from "../ipc";
import type { AppConfig, Profile } from "../types";
import HotkeyRecorder from "../components/HotkeyRecorder";
import InfoTip from "../components/InfoTip";

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
      llm_transcription: { llm_provider_id: null, model: null },
      clipboard_only: false,
      keep_on_clipboard: false,
    };
    save({ ...cfg, profiles: [...cfg.profiles, p] });
  };

  const update = (i: number, patch: Partial<Profile>) => {
    const profiles = cfg.profiles.map((p, idx) => idx === i ? { ...p, ...patch } : p);
    save({ ...cfg, profiles });
  };

  const remove = (i: number) =>
    save({ ...cfg, profiles: cfg.profiles.filter((_, idx) => idx !== i) });

  const showTips = cfg.general.show_tooltips !== false;

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
          <label>Modus<InfoTip enabled={showTips} text="Push-to-talk: Hotkey gedrückt halten für die Aufnahme, Loslassen stoppt und transkribiert. Toggle: Einmal drücken startet, erneut drücken stoppt." />
            <select value={p.hotkey_mode} onChange={e => update(i, { hotkey_mode: e.target.value as Profile["hotkey_mode"] })}>
              <option value="push_to_talk">Push-to-talk</option>
              <option value="toggle">Toggle</option>
            </select>
          </label>
          <label>Backend<InfoTip enabled={showTips} text="GPU-Server: Remote-Whisper auf deinem LAN-Server (schnell, beste Qualität mit large-v3). Lokal: whisper.cpp mit heruntergeladenem Modell (offline, CPU — siehe Tab Modelle). LLM-Provider: Chat-Completion mit Audio-Input (Gemini 2.5 Flash via OpenRouter, gpt-4o-audio-preview)." />
            <select value={p.transcription_backend} onChange={e => update(i, { transcription_backend: e.target.value as Profile["transcription_backend"] })}>
              <option value="remote_whisper">GPU-Server</option>
              <option value="local_whisper">Lokal (whisper.cpp)</option>
              <option value="llm_transcription">LLM-Provider (Chat-Audio)</option>
            </select>
          </label>
          {p.transcription_backend === "llm_transcription" && (
            <>
              <label>LLM-Provider<InfoTip enabled={showTips} text="Der LLM-Anbieter der die Audio-Datei als Chat-Message mit input_audio-Content-Part erhält. Nur Modelle mit Audio-Input (Gemini 2.5 Flash/Pro, gpt-4o-audio-preview)." />
                <select value={p.llm_transcription.llm_provider_id ?? ""}
                  onChange={e => update(i, { llm_transcription: { ...p.llm_transcription, llm_provider_id: e.target.value || null } })}>
                  <option value="">— wählen —</option>
                  {cfg.providers.map(pr => <option key={pr.id} value={pr.id}>{pr.name}</option>)}
                </select>
              </label>
              <label>Modell (leer = Default des Providers)<InfoTip enabled={showTips} text="Audio-fähiges Modell. Beispiele: google/gemini-2.5-flash, google/gemini-2.5-pro (via OpenRouter), gpt-4o-audio-preview (direkt bei OpenAI). Text-only Modelle (Claude, GPT-4-Text) geben einen Fehler zurück." />
                <input value={p.llm_transcription.model ?? ""}
                  onChange={e => update(i, { llm_transcription: { ...p.llm_transcription, model: e.target.value || null } })}
                  placeholder="z. B. google/gemini-2.5-flash" />
              </label>
              <small style={{ color: "#888", display: "block", marginTop: -6, marginBottom: 10 }}>
                Funktioniert nur mit Modellen, die Audio-Input verstehen (Gemini 2.5 Flash/Pro via OpenRouter, gpt-4o-audio-preview, …).
              </small>
            </>
          )}
          <label>Sprache<InfoTip enabled={showTips} text="Sprache des gesprochenen Audios. 'Auto' lässt Whisper selbst erkennen — meist zuverlässig, kann aber bei kurzen Clips oder starkem Akzent danebenliegen." />
            <select value={p.language} onChange={e => update(i, { language: e.target.value as Profile["language"] })}>
              <option value="de">Deutsch</option>
              <option value="en">Englisch</option>
              <option value="auto">Auto</option>
            </select>
          </label>
          <label>
            <input type="checkbox" checked={p.clipboard_only ?? false}
              onChange={e => update(i, { clipboard_only: e.target.checked })} />
            Nur in Zwischenablage (kein Auto-Einfügen)<InfoTip enabled={showTips} text="Text landet nach dem Diktat nur in der Zwischenablage; du fügst ihn selbst mit Strg+V ein. Sinnvoll für Remote-Desktop-Fenster oder elevierte Apps (PowerShell als Admin), in die das automatische Einfügen nicht durchkommt." />
          </label>
          <label>
            <input type="checkbox" checked={p.keep_on_clipboard ?? false}
              onChange={e => update(i, { keep_on_clipboard: e.target.checked })} />
            Text ins Clipboard kopieren<InfoTip enabled={showTips} text="Lässt den transkribierten Text nach dem Auto-Einfügen auch in der Zwischenablage liegen (normal wird der vorherige Clipboard-Inhalt wiederhergestellt). Praktisch, wenn du den Text zusätzlich woanders einfügen willst." />
          </label>
          <label>
            <input type="checkbox" checked={p.post_processing.enabled}
              onChange={e => update(i, { post_processing: { ...p.post_processing, enabled: e.target.checked } })} />
            Post-Processing aktiv<InfoTip enabled={showTips} text="Nach der Transkription das Transkript durch ein LLM nachbearbeiten (Satzzeichen, Großschreibung, Fachbegriffe korrigieren)." />
          </label>
          {p.post_processing.enabled && (
            <>
              <label>LLM-Provider<InfoTip enabled={showTips} text="LLM-Anbieter der das Transkript nachbearbeitet (Satzzeichen, Großschreibung, Fachbegriffe aus dem Wörterbuch). Ein Text-Modell reicht — Audio-Input ist nicht nötig." />
                <select value={p.post_processing.llm_provider_id ?? ""}
                  onChange={e => update(i, { post_processing: { ...p.post_processing, llm_provider_id: e.target.value || null } })}>
                  <option value="">— wählen —</option>
                  {cfg.providers.map(pr => <option key={pr.id} value={pr.id}>{pr.name}</option>)}
                </select>
              </label>
              <label>Modell<InfoTip enabled={showTips} text="Konkretes Modell des LLM-Anbieters. Bei OpenRouter im Format vendor/model (z. B. anthropic/claude-3.5-sonnet). Leer lassen = Default-Modell des Providers." />
                <input value={p.post_processing.model ?? ""}
                onChange={e => update(i, { post_processing: { ...p.post_processing, model: e.target.value || null } })} /></label>
              <label>Eigener System-Prompt (optional)<InfoTip enabled={showTips} text="Überschreibt den Standard-Prompt für Post-Processing. Hier kannst du spezielle Regeln vorgeben (z. B. 'Keine Emojis', 'Förmliche Anrede')." />
                <textarea value={p.post_processing.system_prompt ?? ""}
                onChange={e => update(i, { post_processing: { ...p.post_processing, system_prompt: e.target.value || null } })} /></label>
            </>
          )}
          <button className="danger" onClick={() => remove(i)}>Profil löschen</button>
        </fieldset>
      ))}
    </div>
  );
}
