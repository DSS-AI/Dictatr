# Dictatr — Design-Dokument

**Datum:** 2026-04-13
**Status:** Draft (zum Review)
**Autor:** DSS-AI (via Claude Code Brainstorming)

---

## 1. Zielsetzung

Dictatr ist eine **leichtgewichtige Desktop-Diktier-App** für Windows (später macOS), die als self-hosted Alternative zu [wisprflow.ai](https://www.wisprflow.ai) dient. Sie läuft im Hintergrund, wird per konfigurierbarem Hotkey aktiviert, transkribiert Sprache in Text und fügt diesen an der aktuellen Cursorposition ein. Der User soll den LLM-Provider frei wählen können — inklusive eigener GPU-Infrastruktur (DSS-V-A-Transcribe auf 192.168.178.43).

**Nicht-Ziele:**
- Kein Cloud-Service / keine Team-Features
- Keine Web-Version
- Kein Audio-Archiv (nur Text-History)

---

## 2. User-Stories

- *Als User möchte ich einen globalen Hotkey drücken, sprechen und nach dem Loslassen den Text dort sehen, wo mein Cursor steht.*
- *Als User möchte ich mehrere Profile haben, um z.B. „schnelles Diktat" vs. „formelle E-Mail" mit unterschiedlichen Hotkeys und LLM-Korrekturen zu trennen.*
- *Als User möchte ich meinen eigenen GPU-Server für die Transkription nutzen, aber offline automatisch auf ein lokales Whisper ausweichen können.*
- *Als User möchte ich eigene Fachbegriffe (DSS-Siegmund, Invoice Ninja, ISO 10218-1) in ein Wörterbuch legen, damit sie korrekt geschrieben werden.*
- *Als User möchte ich die letzten ~100 Diktate einsehen, einzelne löschen oder erneut einfügen.*

---

## 3. Tech-Stack

| Bereich | Wahl | Begründung |
|---|---|---|
| App-Framework | **Tauri 2.x** (Rust + System-Webview) | ~5-15 MB Installer, cross-platform Windows+macOS ohne Mehraufwand, native Performance |
| UI | HTML/TypeScript + kleines UI-Framework (React oder Vanilla TS) | Webview-UI, identisch auf beiden OS |
| Audio-Capture | `cpal` crate | Etabliert, cross-platform, 16 kHz mono PCM |
| Global Hotkey | `global-hotkey` crate | Unterstützt Windows + macOS |
| Text-Injection | `enigo` crate | Keystroke-Simulation cross-platform |
| Lokales Whisper | `whisper.cpp` via Rust-Binding (`whisper-rs`) | Eingebettet, offline, GPU-optional |
| Keyring | `keyring` crate | Windows Credential Manager / macOS Keychain |
| History-DB | `rusqlite` (SQLite) | Lokal, keine externen Abhängigkeiten |
| Build/Packaging | Tauri Bundler (MSI für Windows, DMG für macOS) | Standardweg |

---

## 4. Architektur

### 4.1 Komponenten

```
┌─────────────────────────────────────────────────────────┐
│                    Tauri-App (Rust Core)                │
│                                                         │
│  ┌─────────────┐  ┌──────────────┐  ┌───────────────┐   │
│  │  Hotkey     │  │   Audio      │  │  Text         │   │
│  │  Listener   │  │   Capture    │  │  Injector     │   │
│  └──────┬──────┘  └──────┬───────┘  └───────▲───────┘   │
│         │                │                  │           │
│         ▼                ▼                  │           │
│  ┌─────────────────────────────────────────────────┐   │
│  │            State Machine (Orchestrator)         │   │
│  │   idle → recording → transcribing → injecting   │   │
│  └──────┬───────────────────┬──────────────────────┘   │
│         │                   │                           │
│         ▼                   ▼                           │
│  ┌────────────────┐   ┌────────────────┐                │
│  │ Transcription  │   │ Post-Processor │                │
│  │ Backend Trait  │   │ (LLM)          │                │
│  │ ──────────────  │   │ ──────────────  │                │
│  │ remote-whisper │   │ openai-compat  │                │
│  │ local-whisper  │   │ anthropic      │                │
│  │ cloud-api      │   │ gemini         │                │
│  └────────────────┘   └────────────────┘                │
│         │                   │                           │
│  ┌────────────────────────────────────────────────┐    │
│  │       Storage: config.json, history.db,        │    │
│  │               OS-Keyring (API-Keys)            │    │
│  └────────────────────────────────────────────────┘    │
│                                                         │
│  ┌────────────────┐  ┌────────────────────────────┐    │
│  │  Tray Icon     │  │  Settings-UI (Webview)     │    │
│  │  + Overlay     │  │  (bei Bedarf sichtbar)     │    │
│  └────────────────┘  └────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

### 4.2 Transcription-Backend-Plugin-Interface

```rust
pub trait TranscriptionBackend: Send + Sync {
    async fn transcribe(
        &self,
        audio: &[f32],          // 16 kHz mono PCM
        language: Language,      // De | En | Auto
        vocabulary: &[String],   // Optional hint
    ) -> Result<Transcription, BackendError>;

    fn id(&self) -> &'static str;         // "remote-whisper" | ...
    fn is_available(&self) -> bool;       // Heartbeat-Check für Fallback
}

pub struct Transcription {
    pub text: String,
    pub duration_ms: u64,
    pub backend_id: &'static str,
}
```

**Implementierungen im MVP:**
- `RemoteWhisperBackend` → HTTP POST `http://192.168.178.43:8503/api/dictate`
- `LocalWhisperBackend` → `whisper-rs` mit mitgeliefertem `ggml-base.bin` Modell
- `CloudApiBackend` (Phase 2) → Groq Whisper / OpenAI Whisper

### 4.3 LLM-Post-Processor

```rust
pub trait LlmProvider: Send + Sync {
    async fn complete(
        &self,
        system: &str,
        user: &str,
        model: &str,
    ) -> Result<String, LlmError>;
}
```

**Mapping auf OpenAI-kompatibles Chat-Completions-Interface**; separate Adapter für Anthropic Messages API und Gemini. Ollama und LiteLLM funktionieren out-of-the-box über das OpenAI-kompatible Interface.

---

## 5. Ablauf eines Diktats

### 5.1 Happy Path

1. User drückt Hotkey des gewählten Profils (z.B. `Strg+Alt+Leer`)
2. State → `recording`: Tray-Icon rot, optionaler Sound, Mini-Overlay unten rechts mit Pegelanzeige
3. Audio wird in Ringbuffer (16 kHz mono, max 2 min) geschrieben
4. User lässt los (Push-to-talk) bzw. drückt erneut (Toggle)
5. State → `transcribing`: Tray-Icon gelb
6. Audio geht an das Profil-Backend (z.B. `remote-whisper`)
7. Falls Profil Post-Processing an hat: Raw-Text + Vokabular an LLM-Profil, gibt korrigierten Text zurück
8. State → `injecting`: Text per `enigo` an aktueller Cursorposition eingefügt
9. State → `idle`: Tray-Icon grün (1 s), dann grau. History-Eintrag (Text, Timestamp, Profil, Backend, Dauer).

### 5.2 Fehler-Fälle

| Szenario | Verhalten |
|---|---|
| GPU-Server unerreichbar (Heartbeat fehlschlägt) | Auto-Fallback auf `local-whisper`, Toast „GPU offline, nutze lokal" |
| Mic fehlt / Permission verweigert | Toast + Settings-UI öffnen beim Audio-Tab |
| Max-Aufnahmedauer (2 min) erreicht | Auto-Stop, normale Verarbeitung wie bei Release |
| LLM-Provider Fehler (Timeout, 4xx/5xx) | Raw-Whisper-Text einfügen + Toast „Korrektur fehlgeschlagen: {grund}" |
| whisper.cpp lokal nicht initialisiert | Toast „Modell fehlt, Download starten?" → Settings-UI |
| Zielfenster akzeptiert keine Keystrokes (z.B. UAC) | Toast + Text in Zwischenablage legen, Hinweis „Bitte manuell einfügen" |

---

## 6. Profile & Settings

### 6.1 Profil-Datenmodell

```json
{
  "id": "uuid",
  "name": "Email formal",
  "hotkey": "Ctrl+Alt+M",
  "hotkey_mode": "push_to_talk" | "toggle",
  "transcription_backend": "remote-whisper" | "local-whisper" | "cloud-api",
  "language": "de" | "en" | "auto",
  "post_processing": {
    "enabled": true,
    "llm_provider_id": "uuid",
    "model": "claude-sonnet-4-6",
    "system_prompt": "..."
  }
}
```

### 6.2 LLM-Provider-Datenmodell

```json
{
  "id": "uuid",
  "name": "Claude Sonnet",
  "type": "anthropic" | "openai" | "openai_compatible" | "gemini" | "ollama",
  "base_url": "https://api.anthropic.com",
  "api_key_ref": "keyring:dictatr/provider-uuid",
  "default_model": "claude-sonnet-4-6"
}
```

### 6.3 Settings-UI-Tabs

| Tab | Inhalt |
|---|---|
| Profile | CRUD mit allen Feldern oben, Hotkey-Recorder |
| LLM-Anbieter | Provider hinzufügen/entfernen, API-Key-Eingabe (Keyring) |
| Wörterbuch | Textliste, ein Begriff pro Zeile, optional Kategorien |
| Audio | Mic-Auswahl-Dropdown, Live-Pegelanzeige, Test-Button |
| Allgemein | Autostart, Sounds, Overlay, Max-Dauer (Default 120 s), History-Länge (Default 100) |
| History | Suche, Tabelle (Timestamp/Profil/Text-Preview), Re-Inject-Button, Löschen |

### 6.4 Konfigurations-Dateien

- `%APPDATA%/Dictatr/config.json` (Profile, Provider-Metadaten, UI-Einstellungen)
- `%APPDATA%/Dictatr/vocabulary.txt` (eine Zeile pro Begriff)
- `%APPDATA%/Dictatr/history.db` (SQLite)
- API-Keys: OS-Keyring, Referenz via `api_key_ref` in `config.json`
- Logs: `%APPDATA%/Dictatr/logs/dictatr.log` (rotierend)

macOS-Pfad-Äquivalent: `~/Library/Application Support/Dictatr/`

---

## 7. Server-seitige Integration (DSS-V-A-Transcribe)

Ein **neuer Endpoint** wird in DSS-V-A-Transcribe ergänzt, der kurze Audio-Clips synchron (ohne Queue) verarbeitet:

```
POST http://192.168.178.43:8503/api/dictate
Headers:
  Authorization: Bearer <shared_secret>
  Content-Type: multipart/form-data
Body:
  file: audio.wav (16 kHz mono, ≤ 2 min)
  language: "de" | "en" | "auto"
  vocabulary: "DSS-Siegmund, Invoice Ninja, ..." (optional, wird als Whisper initial_prompt genutzt)

Response 200:
  { "text": "...", "duration_ms": 1234, "backend": "faster-whisper-gpu" }

Response 4xx/5xx:
  { "error": "...", "code": "..." }
```

**Performance-Ziel:** < 2 s Latenz für 10 s Audio (Ende-zu-Ende).

**Authentifizierung:** Einfaches Bearer-Shared-Secret, in Client-Keyring und Server-Env-Var. Läuft nur im LAN, kein OAuth nötig.

---

## 8. Post-Processing-Prompt (Default)

```
System:
Du korrigierst diktierten Text. Verändere den Inhalt nicht.
Korrigiere ausschließlich Rechtschreibung, Grammatik, Zeichensetzung und
offensichtlich falsche Wort-Erkennungen. Gib ausschließlich den korrigierten
Text zurück, ohne Kommentare oder Anführungszeichen.

Verwende folgendes Vokabular korrekt, wenn es vorkommt:
{vocabulary}

User:
{raw_whisper_text}
```

Pro Profil ist der System-Prompt überschreibbar (z.B. „formeller E-Mail-Stil, Sie-Form").

---

## 9. Sicherheit & Privatsphäre

- API-Keys ausschließlich im OS-Keyring, niemals in `config.json` oder Logs.
- Audio-Daten nur im RAM, niemals persistiert (auch nicht in History).
- History speichert nur finalen Text + Metadaten, kein Audio.
- Optional: „Privatmodus" pro Profil, der History-Eintrag unterdrückt.
- Netzwerk-Traffic an GPU-Server geht via HTTP im LAN (HTTPS optional via Reverse-Proxy).
- Telemetrie: **keine**.

---

## 10. Testing-Strategie

### 10.1 Automatisiert

- **Rust-Unit-Tests:**
  - Audio-Ringbuffer-Logik (Füllstand, Overflow, Reset)
  - Backend-Auswahl (Heartbeat → Fallback-Pfad)
  - Profil-Serialisierung / Deserialisierung
  - Prompt-Builder (Vokabular-Einbettung, Escaping)
  - Keyring-Wrapper (mit Mock)

- **Integration-Tests:**
  - `RemoteWhisperBackend` gegen Mock-HTTP-Server (wiremock)
  - `LocalWhisperBackend` mit kleinem Test-Audio-Clip
  - LLM-Provider-Adapter mit Mock-Responses

### 10.2 Manuell (Release-Checkliste)

- Hotkey-Injection in: Notepad, Word, Outlook, Chrome (Google Docs, Gmail), VS Code, Terminal, Slack, Teams
- Hotkey-Konflikte mit System-Shortcuts (z.B. Win+Leer Sprach-Umschaltung) prüfen
- Push-to-talk & Toggle beide durchtesten
- Auto-Fallback: GPU-Server stoppen → lokales Whisper greift
- Max-Aufnahmedauer: 2:05 min sprechen → Auto-Stop
- UAC-Dialog offen: Clipboard-Fallback greift
- Multi-Monitor: Overlay-Position korrekt

---

## 11. Phasen

### Phase 1 — MVP (Windows only)

| # | Aufgabe |
|---|---|
| 1 | Tauri-Projekt-Skeleton + CI (cargo test, cargo build --release) |
| 2 | Server-Endpoint `/api/dictate` in DSS-V-A-Transcribe |
| 3 | Audio-Capture (`cpal`) + Ringbuffer + Pegelmessung |
| 4 | Global-Hotkey (`global-hotkey`) mit push-to-talk & toggle |
| 5 | `TranscriptionBackend`-Trait + `RemoteWhisperBackend` |
| 6 | `LocalWhisperBackend` (`whisper-rs`, Modell-Download in Settings) |
| 7 | `LlmProvider`-Trait + 3 Adapter (openai-compat, anthropic, ollama) |
| 8 | State-Machine + Orchestrator |
| 9 | Text-Injection (`enigo`) + Clipboard-Fallback |
| 10 | Tray-Icon + Mini-Overlay |
| 11 | Settings-UI (alle 6 Tabs) |
| 12 | SQLite-History |
| 13 | OS-Keyring-Integration |
| 14 | MSI-Installer (Tauri Bundler) |
| 15 | Release-Checkliste manuell durchlaufen |

### Phase 2 — Erweiterungen

- macOS-Port (Accessibility-Permissions, Signierung, Notarization, DMG)
- `CloudApiBackend` (Groq, OpenAI Whisper direkt)
- Auto-Updater (Tauri Updater mit signierten Releases)
- Pro-Profil-Prompts
- Kontext-aware Prompts (aktives Fenster → Profil-Hint)
- Command-Mode („neue Zeile", „lösche das", „fettdruck")

---

## 12. Bewusste Nicht-Features (YAGNI)

- Streaming / Partial-Transkription (User will Final nach Release)
- Sprachbefehle außerhalb Command-Mode (keine Wake-Words)
- Team-Features, Cloud-Sync, geteilte Profile
- Audio-Archiv / Audio-Replay in History
- Browser-Erweiterung
- Mobile-App

---

## 13. Offene Punkte für Implementierungs-Phase

- Konkretes whisper.cpp-Modell für `local-whisper`: `ggml-base` (140 MB, schnell) vs `ggml-small` (460 MB, besser) — im Settings-UI wählbar + Download on demand.
- Hotkey-Recorder-UI: Eigene Capture-Komponente oder fertige Library?
- Overlay-Fenster: Transparent, always-on-top — Tauri-API-Details prüfen.
- DSS-V-A-Transcribe: Wie nah ist der neue `/api/dictate` am bestehenden Code? Evtl. vorhandene faster-whisper-Instanz wiederverwenden oder separate kleine Instanz?

Diese Punkte werden im Implementierungs-Plan (nächster Schritt via writing-plans-Skill) aufgelöst.
