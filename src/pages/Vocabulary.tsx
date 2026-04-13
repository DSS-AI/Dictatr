import { useState } from "react";

// Vocabulary is stored in a separate file; for Phase 1 use a simple localStorage-ish approach
// via a dedicated IPC command. For MVP we just edit it as a big textarea and rely on main.rs
// reading vocabulary.txt. Add IPC commands later if needed — for now show a TODO.

export default function Vocabulary() {
  const [text, setText] = useState("");

  // TODO: IPC command get_vocabulary / save_vocabulary when implemented

  const save = () => {
    alert("Vokabular-Speicherung kommt in späterer Task — aktuell manuell bearbeitbar in config-ordner/vocabulary.txt");
  };

  return (
    <div>
      <h1>Wörterbuch</h1>
      <p style={{ color: "#888" }}>Ein Begriff pro Zeile. Wird dem LLM als Kontext mitgegeben.</p>
      <textarea value={text} onChange={e => setText(e.target.value)} style={{ width: "100%", minHeight: 300 }} />
      <button onClick={save}>Speichern</button>
    </div>
  );
}
