import { useState } from "react";

interface Props { value: string; onChange: (v: string) => void; }

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

  return (
    <input
      readOnly
      value={recording ? "…drücken…" : value}
      onKeyDown={onKey}
      onFocus={() => setRecording(true)}
      onBlur={() => setRecording(false)}
    />
  );
}
