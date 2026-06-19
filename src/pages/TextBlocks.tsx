import { useEffect, useState } from "react";
import { ipc } from "../ipc";
import type { AppConfig, TextBlock } from "../types";
import InfoTip from "../components/InfoTip";

export default function TextBlocks() {
  const [cfg, setCfg] = useState<AppConfig | null>(null);

  useEffect(() => { ipc.getConfig().then(setCfg).catch(console.error); }, []);

  if (!cfg) return <div>Lade…</div>;

  const blocks = cfg.text_blocks ?? [];
  const save = (next: AppConfig) => { setCfg(next); ipc.saveConfig(next); };

  // Distinct categories already in use — fed into the datalist for quick reuse.
  const categories = Array.from(
    new Set(blocks.map(b => b.category.trim()).filter(Boolean)),
  );

  const add = () => {
    const b: TextBlock = {
      id: crypto.randomUUID(),
      category: categories[0] ?? "Prompts",
      title: "Neuer Textblock",
      content: "",
    };
    save({ ...cfg, text_blocks: [...blocks, b] });
  };

  const update = (i: number, patch: Partial<TextBlock>) => {
    const text_blocks = blocks.map((b, idx) => idx === i ? { ...b, ...patch } : b);
    save({ ...cfg, text_blocks });
  };

  const remove = (i: number) =>
    save({ ...cfg, text_blocks: blocks.filter((_, idx) => idx !== i) });

  const showTips = cfg.general.show_tooltips !== false;

  return (
    <div>
      <h1>Textblöcke</h1>
      <p>
        Wiederkehrende Texte/Prompts, gruppiert nach frei wählbaren Kategorien. Über den
        Prompt-Manager-Hotkey (Tab <b>Allgemein</b>) blitzschnell abrufbar und ins aktive Fenster
        einfügbar.
      </p>
      <button onClick={add}>+ Neuer Textblock</button>
      {blocks.length === 0 && <p style={{ color: "#888" }}>Noch keine Textblöcke angelegt.</p>}

      <datalist id="tb-categories">
        {categories.map(c => <option key={c} value={c} />)}
      </datalist>

      {blocks.map((b, i) => (
        <fieldset key={b.id}>
          <legend>{b.title || "(ohne Titel)"}</legend>
          <label>Titel<input value={b.title} onChange={e => update(i, { title: e.target.value })} /></label>
          <label>Kategorie<InfoTip enabled={showTips} text="Frei wählbar (z. B. Prompts, Emails, Sonstiges). Blöcke mit gleicher Kategorie erscheinen im Picker unter demselben Reiter." />
            <input list="tb-categories" value={b.category}
              onChange={e => update(i, { category: e.target.value })}
              placeholder="z. B. Prompts" />
          </label>
          <label>Inhalt<textarea value={b.content}
            style={{ minHeight: 120, fontFamily: "ui-monospace, Consolas, monospace" }}
            onChange={e => update(i, { content: e.target.value })} /></label>
          <button className="danger" onClick={() => remove(i)}>Textblock löschen</button>
        </fieldset>
      ))}
    </div>
  );
}
