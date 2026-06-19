import { useEffect, useMemo, useState } from "react";
import { ipc } from "./ipc";
import type { TextBlock } from "./types";

/// Quick-pick popup opened by the Prompt-Manager hotkey. Lists the user's text
/// blocks grouped into category tabs; clicking one pastes it into the
/// previously-focused app (and leaves it on the clipboard). Management
/// (create/edit/delete) lives in the main window's "Textblöcke" tab.
export default function PromptPicker() {
  const [blocks, setBlocks] = useState<TextBlock[]>([]);
  const [activeCat, setActiveCat] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const reload = () => {
    ipc.getConfig()
      .then(cfg => setBlocks(cfg.text_blocks ?? []))
      .catch(e => setError(String(e)));
  };

  useEffect(() => {
    reload();
    // The popup window is reused (hidden, not destroyed); refresh its contents
    // each time it's shown again — the webview gets a DOM focus event.
    const onFocus = () => reload();
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") ipc.hidePromptWindow().catch(() => {});
    };
    window.addEventListener("focus", onFocus);
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("focus", onFocus);
      window.removeEventListener("keydown", onKey);
    };
  }, []);

  const categories = useMemo(() => {
    const seen: string[] = [];
    for (const b of blocks) {
      const c = b.category.trim() || "Sonstiges";
      if (!seen.includes(c)) seen.push(c);
    }
    return seen;
  }, [blocks]);

  // Keep a valid active category as the list changes.
  const current =
    activeCat && categories.includes(activeCat) ? activeCat : categories[0] ?? null;

  const visible = blocks.filter(b => (b.category.trim() || "Sonstiges") === current);

  const pick = (b: TextBlock) => {
    ipc.pasteTextBlock(b.content).catch(e => setError(String(e)));
  };

  return (
    <div className="picker">
      <header className="picker-head">
        <span>Prompt-Manager</span>
        <small>Klick → einfügen · Esc schließt</small>
      </header>

      {categories.length === 0 ? (
        <div className="picker-empty">
          Keine Textblöcke angelegt.<br />
          Im Hauptfenster unter <b>Textblöcke</b> anlegen.
        </div>
      ) : (
        <>
          <nav className="picker-tabs">
            {categories.map(c => (
              <button
                key={c}
                className={c === current ? "active" : ""}
                onClick={() => setActiveCat(c)}
              >
                {c}
              </button>
            ))}
          </nav>
          <ul className="picker-list">
            {visible.map(b => (
              <li key={b.id}>
                <button onClick={() => pick(b)} title={b.content}>
                  <span className="picker-title">{b.title || "(ohne Titel)"}</span>
                  <span className="picker-preview">
                    {b.content.replace(/\s+/g, " ").slice(0, 80)}
                  </span>
                </button>
              </li>
            ))}
            {visible.length === 0 && (
              <li className="picker-empty-cat">Keine Einträge in dieser Kategorie.</li>
            )}
          </ul>
        </>
      )}

      {error && <div className="picker-error">Fehler: {error}</div>}
    </div>
  );
}
