import { useState } from "react";

interface Props { value: string; onChange: (v: string) => void; }

const MULTIMEDIA_KEYS = [
  "LaunchMail",
  "LaunchApp1",
  "LaunchApp2",
  "LaunchMediaSelect",
  "MediaPlayPause",
  "MediaStop",
  "MediaNextTrack",
  "MediaPrevTrack",
  "VolumeMute",
  "VolumeDown",
  "VolumeUp",
  "BrowserBack",
  "BrowserForward",
  "BrowserRefresh",
  "BrowserStop",
  "BrowserSearch",
  "BrowserFavorites",
  "BrowserHome",
  "Sleep",
];

export default function HotkeyRecorder({ value, onChange }: Props) {
  const [recording, setRecording] = useState(false);

  const onKey = (e: React.KeyboardEvent) => {
    if (!recording) return;
    e.preventDefault();
    const parts: string[] = [];
    if (e.ctrlKey) parts.push("Ctrl");
    if (e.altKey) parts.push("Alt");
    if (e.shiftKey) parts.push("Shift");
    if (e.metaKey) parts.push("Meta");
    const key = e.key.length === 1 ? e.key.toUpperCase() : e.key;
    if (!["Control", "Alt", "Shift", "Meta"].includes(key)) {
      parts.push(key === " " ? "Space" : key);
      onChange(parts.join("+"));
      setRecording(false);
    }
  };

  const pickMultimediaKey = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const v = e.target.value;
    if (v) onChange(v);
    e.target.value = "";
  };

  return (
    <span style={{ display: "inline-flex", gap: 6, alignItems: "center" }}>
      <input
        readOnly
        value={recording ? "…drücken…" : value}
        onKeyDown={onKey}
        onFocus={() => setRecording(true)}
        onBlur={() => setRecording(false)}
      />
      <select value="" onChange={pickMultimediaKey} title="Multimedia-/Systemtaste wählen (z. B. LaunchMail)">
        <option value="">Systemtaste…</option>
        {MULTIMEDIA_KEYS.map(k => <option key={k} value={k}>{k}</option>)}
      </select>
    </span>
  );
}
