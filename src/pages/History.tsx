import { useEffect, useState } from "react";
import { ipc } from "../ipc";
import type { HistoryEntry } from "../types";

export default function History() {
  const [entries, setEntries] = useState<HistoryEntry[]>([]);

  const load = () => ipc.listHistory(100).then(setEntries).catch(console.error);
  useEffect(() => { load(); }, []);

  const remove = async (id: number) => {
    await ipc.deleteHistory(id);
    load();
  };

  return (
    <div>
      <h1>History</h1>
      <button onClick={load}>Aktualisieren</button>
      <table>
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
              <td style={{ maxWidth: 400, whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}>{e.text}</td>
              <td><button className="danger" onClick={() => remove(e.id)}>×</button></td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
