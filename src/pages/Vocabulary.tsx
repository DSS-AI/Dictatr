import { useEffect, useState } from "react";
import { ipc } from "../ipc";

export default function Vocabulary() {
  const [text, setText] = useState("");
  const [loaded, setLoaded] = useState("");
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    ipc.getVocabulary()
      .then(v => { setText(v); setLoaded(v); })
      .catch(e => setError(String(e)));
  }, []);

  const dirty = text !== loaded;

  const save = async () => {
    setError(null); setStatus(null);
    try {
      await ipc.saveVocabulary(text);
      setLoaded(text);
      const count = text.split("\n").filter(l => l.trim()).length;
      setStatus(`Gespeichert (${count} Einträge). Wirkt ab dem nächsten Hotkey-Druck.`);
      setTimeout(() => setStatus(null), 4000);
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div>
      <h1>Wörterbuch</h1>
      <p>
        Ein Begriff pro Zeile. Wird sowohl als Whisper-Prompt (Transkription) als
        auch als Kontext für das Post-Processing-LLM mitgegeben — hilfreich für
        Namen, Fachbegriffe und Akronyme.
      </p>
      <textarea
        value={text}
        onChange={e => setText(e.target.value)}
        placeholder={"z.B.\nInvoice Ninja\nISO 10218-1\nwhisper.cpp"}
        style={{ width: "100%", minHeight: 360, fontFamily: "ui-monospace, Consolas, monospace", fontSize: 13 }}
      />
      <div style={{ display: "flex", gap: 10, alignItems: "center", marginTop: 10 }}>
        <button onClick={save} disabled={!dirty}>Speichern</button>
        <button className="secondary" onClick={() => setText(loaded)} disabled={!dirty}>Zurücksetzen</button>
        <small style={{ color: "#888" }}>
          {text.split("\n").filter(l => l.trim()).length} Einträge
          {dirty ? " · ungespeichert" : ""}
        </small>
      </div>
      {status && <p style={{ color: "#2d7" }}>{status}</p>}
      {error && <p style={{ color: "#e55" }}>Fehler: {error}</p>}
    </div>
  );
}
