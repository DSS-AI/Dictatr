import { useEffect, useState } from "react";
import { ipc } from "../ipc";
import type { HistoryEntry } from "../types";

export default function History() {
  const [entries, setEntries] = useState<HistoryEntry[]>([]);
  const [copiedId, setCopiedId] = useState<number | null>(null);

  const load = () => ipc.listHistory(100).then(setEntries).catch(console.error);
  useEffect(() => { load(); }, []);

  const remove = async (id: number) => {
    await ipc.deleteHistory(id);
    load();
  };

  const copy = async (id: number, text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopiedId(id);
      setTimeout(() => setCopiedId(prev => (prev === id ? null : prev)), 1500);
    } catch (e) {
      console.error("clipboard write failed", e);
    }
  };

  return (
    <div>
      <h1>History</h1>
      <button onClick={load}>Aktualisieren</button>
      <table style={{ tableLayout: "fixed", width: "100%" }}>
        <colgroup>
          <col style={{ width: 140 }} />
          <col style={{ width: 90 }} />
          <col style={{ width: 110 }} />
          <col style={{ width: 60 }} />
          <col />
          <col style={{ width: 80 }} />
        </colgroup>
        <thead>
          <tr><th>Zeit</th><th>Profil</th><th>Backend</th><th>Dauer</th><th>Text</th><th></th></tr>
        </thead>
        <tbody>
          {entries.map(e => (
            <tr key={e.id}>
              <td>{new Date(e.timestamp).toLocaleString("de-DE")}</td>
              <td>{e.profile_name}</td>
              <td>{e.backend_id}</td>
              <td>{e.duration_ms}ms</td>
              <td
                title={e.text}
                style={{ whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis", cursor: "pointer" }}
                onClick={() => copy(e.id, e.text)}
              >
                {e.text}
              </td>
              <td style={{ whiteSpace: "nowrap", paddingRight: 4 }}>
                <button
                  onClick={() => copy(e.id, e.text)}
                  title={copiedId === e.id ? "Kopiert" : "In Zwischenablage kopieren"}
                  style={{ padding: "4px 6px", margin: 0, marginRight: 4 }}
                  aria-label="Kopieren"
                >
                  {copiedId === e.id ? (
                    <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                      <polyline points="3 8 7 12 13 4" />
                    </svg>
                  ) : (
                    <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
                      <rect x="5" y="5" width="9" height="9" rx="1.5" />
                      <path d="M3 11V3.5A1.5 1.5 0 0 1 4.5 2H11" />
                    </svg>
                  )}
                </button>
                <button className="danger" onClick={() => remove(e.id)} style={{ padding: "4px 8px", margin: 0 }} title="Eintrag löschen">×</button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
