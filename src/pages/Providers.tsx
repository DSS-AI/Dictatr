import { useEffect, useState } from "react";
import { ipc } from "../ipc";
import type { AppConfig, LlmProviderConfig } from "../types";

export default function Providers() {
  const [cfg, setCfg] = useState<AppConfig | null>(null);
  const [apiKeyInputs, setApiKeyInputs] = useState<Record<string, string>>({});

  useEffect(() => { ipc.getConfig().then(setCfg).catch(console.error); }, []);

  if (!cfg) return <div>Lade…</div>;

  const save = (next: AppConfig) => { setCfg(next); ipc.saveConfig(next); };

  const add = () => {
    const pr: LlmProviderConfig = {
      id: crypto.randomUUID(),
      name: "Neuer Provider",
      type: "openai_compatible",
      base_url: "https://api.openai.com",
      default_model: "gpt-4o-mini",
    };
    save({ ...cfg, providers: [...cfg.providers, pr] });
  };

  const update = (i: number, patch: Partial<LlmProviderConfig>) => {
    const providers = cfg.providers.map((p, idx) => idx === i ? { ...p, ...patch } : p);
    save({ ...cfg, providers });
  };

  const remove = (i: number) =>
    save({ ...cfg, providers: cfg.providers.filter((_, idx) => idx !== i) });

  const saveApiKey = async (id: string) => {
    const k = apiKeyInputs[id];
    if (!k) return;
    await ipc.setApiKey(id, k);
    setApiKeyInputs({ ...apiKeyInputs, [id]: "" });
    alert("API-Key gespeichert.");
  };

  return (
    <div>
      <h1>LLM-Anbieter</h1>
      <button onClick={add}>+ Neuer Provider</button>
      {cfg.providers.map((p, i) => (
        <fieldset key={p.id}>
          <legend>{p.name}</legend>
          <label>Name<input value={p.name} onChange={e => update(i, { name: e.target.value })} /></label>
          <label>Typ
            <select value={p.type} onChange={e => update(i, { type: e.target.value as LlmProviderConfig["type"] })}>
              <option value="openai">OpenAI</option>
              <option value="openai_compatible">OpenAI-kompatibel (Groq, Ollama, LiteLLM, Custom)</option>
              <option value="anthropic">Anthropic</option>
              <option value="ollama">Ollama</option>
            </select>
          </label>
          <label>Base-URL<input value={p.base_url} onChange={e => update(i, { base_url: e.target.value })} /></label>
          <label>Default-Modell<input value={p.default_model} onChange={e => update(i, { default_model: e.target.value })} /></label>
          <label>API-Key (wird im OS-Keyring gespeichert, nicht in config.json)
            <input type="password" value={apiKeyInputs[p.id] ?? ""}
              onChange={e => setApiKeyInputs({ ...apiKeyInputs, [p.id]: e.target.value })} />
          </label>
          <button onClick={() => saveApiKey(p.id)}>API-Key speichern</button>
          <button className="danger" onClick={() => remove(i)}>Provider löschen</button>
        </fieldset>
      ))}
    </div>
  );
}
