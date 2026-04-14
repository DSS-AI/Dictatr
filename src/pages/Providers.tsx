import { useEffect, useState } from "react";
import { ipc } from "../ipc";
import type { AppConfig, LlmProviderConfig, ProviderType } from "../types";

type TestState = { status: "idle" | "running" | "ok" | "error"; message?: string };

const PROVIDER_PRESETS: Partial<Record<ProviderType, { base_url: string; default_model: string; model_hint?: string }>> = {
  openai:            { base_url: "https://api.openai.com",         default_model: "gpt-4o-mini" },
  openai_compatible: { base_url: "http://localhost:11434",         default_model: "llama3.1" },
  open_router:       { base_url: "https://openrouter.ai/api",      default_model: "anthropic/claude-3.5-sonnet",
                       model_hint: "Format: vendor/model — z. B. anthropic/claude-3.5-sonnet, openai/gpt-4o, google/gemini-2.5-pro" },
  anthropic:         { base_url: "https://api.anthropic.com",      default_model: "claude-3-5-sonnet-latest" },
  ollama:            { base_url: "http://localhost:11434",         default_model: "llama3.1" },
};

export default function Providers() {
  const [cfg, setCfg] = useState<AppConfig | null>(null);
  const [apiKeyInputs, setApiKeyInputs] = useState<Record<string, string>>({});
  const [testState, setTestState] = useState<Record<string, TestState>>({});

  useEffect(() => { ipc.getConfig().then(setCfg).catch(console.error); }, []);

  if (!cfg) return <div>Lade…</div>;

  const save = (next: AppConfig) => { setCfg(next); ipc.saveConfig(next); };

  const add = () => {
    const preset = PROVIDER_PRESETS.open_router!;
    const pr: LlmProviderConfig = {
      id: crypto.randomUUID(),
      name: "Neuer Provider",
      type: "open_router",
      base_url: preset.base_url,
      default_model: preset.default_model,
    };
    save({ ...cfg, providers: [...cfg.providers, pr] });
  };

  const update = (i: number, patch: Partial<LlmProviderConfig>) => {
    const providers = cfg.providers.map((p, idx) => idx === i ? { ...p, ...patch } : p);
    save({ ...cfg, providers });
  };

  const changeType = (i: number, t: ProviderType) => {
    const current = cfg.providers[i];
    const preset = PROVIDER_PRESETS[t];
    const currentPreset = PROVIDER_PRESETS[current.type];
    const patch: Partial<LlmProviderConfig> = { type: t };
    // Only overwrite fields that still hold the previous preset default — if
    // the user customized base_url / default_model, keep their value.
    if (preset) {
      if (!currentPreset || current.base_url === currentPreset.base_url) {
        patch.base_url = preset.base_url;
      }
      if (!currentPreset || current.default_model === currentPreset.default_model) {
        patch.default_model = preset.default_model;
      }
    }
    update(i, patch);
  };

  const remove = (i: number) =>
    save({ ...cfg, providers: cfg.providers.filter((_, idx) => idx !== i) });

  const saveApiKey = async (id: string) => {
    const k = apiKeyInputs[id];
    if (!k) return;
    await ipc.setApiKey(id, k);
    setApiKeyInputs({ ...apiKeyInputs, [id]: "" });
    setTestState(s => ({ ...s, [id]: { status: "idle" } }));
    alert("API-Key gespeichert.");
  };

  const testKey = async (id: string) => {
    setTestState(s => ({ ...s, [id]: { status: "running" } }));
    try {
      const reply = await ipc.testLlmProvider(id);
      setTestState(s => ({ ...s, [id]: { status: "ok", message: reply } }));
    } catch (e) {
      setTestState(s => ({ ...s, [id]: { status: "error", message: String(e) } }));
    }
  };

  return (
    <div>
      <h1>LLM-Anbieter (Post-Processing)</h1>
      <p style={{ color: "#888", marginTop: -8 }}>
        Pro Profil aktivierbar im Reiter „Profile" → „Post-Processing aktiv".
      </p>
      <button onClick={add}>+ Neuer Provider</button>
      {cfg.providers.map((p, i) => {
        const preset = PROVIDER_PRESETS[p.type];
        const ts = testState[p.id] ?? { status: "idle" as const };
        return (
          <fieldset key={p.id}>
            <legend>{p.name}</legend>
            <label>Name<input value={p.name} onChange={e => update(i, { name: e.target.value })} /></label>
            <label>Typ
              <select value={p.type} onChange={e => changeType(i, e.target.value as ProviderType)}>
                <option value="openai">OpenAI</option>
                <option value="open_router">OpenRouter</option>
                <option value="anthropic">Anthropic</option>
                <option value="ollama">Ollama</option>
                <option value="openai_compatible">OpenAI-kompatibel (Groq, LiteLLM, Custom)</option>
              </select>
            </label>
            <label>Base-URL<input value={p.base_url} onChange={e => update(i, { base_url: e.target.value })} /></label>
            <label>Default-Modell<input value={p.default_model} onChange={e => update(i, { default_model: e.target.value })} /></label>
            {preset?.model_hint && <small style={{ color: "#888" }}>{preset.model_hint}</small>}
            <label>API-Key (wird im OS-Keyring gespeichert, nicht in config.json)
              <input type="password" value={apiKeyInputs[p.id] ?? ""}
                onChange={e => setApiKeyInputs({ ...apiKeyInputs, [p.id]: e.target.value })} />
            </label>
            <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
              <button onClick={() => saveApiKey(p.id)}>API-Key speichern</button>
              <button onClick={() => testKey(p.id)} disabled={ts.status === "running"}>
                {ts.status === "running" ? "Teste…" : "API-Key testen"}
              </button>
              <button className="danger" onClick={() => remove(i)}>Provider löschen</button>
            </div>
            {ts.status === "ok" && (
              <p style={{ color: "#2d7a2d" }}>✓ OK — Antwort: {ts.message}</p>
            )}
            {ts.status === "error" && (
              <p style={{ color: "#b23" }}>✗ Fehler: {ts.message}</p>
            )}
          </fieldset>
        );
      })}
    </div>
  );
}
