# Dictatr

Diktat-Tool für Windows und macOS. Hotkey drücken, sprechen, loslassen — der Text wird transkribiert und direkt ins aktive Textfeld geschrieben.

Remote-Whisper, lokales whisper.cpp und LLM-Provider mit Audio-Input (Gemini 2.5, gpt-4o-audio) stehen als Backend zur Auswahl. Optionales LLM-Post-Processing bügelt Satzzeichen und Fachbegriffe glatt.

## Features

- **Hotkeys**: normale Kombos (Ctrl+Alt+Space …) per `global-hotkey`, zusätzlich Windows-Multimedia- und Launch-Tasten (LaunchMail, VolumeUp, BrowserHome, Media-Keys …) via Low-Level-Keyboard-Hook — Outlook & Co. werden dabei unterdrückt. Push-to-talk oder Toggle pro Profil.
- **Transkriptions-Backends** pro Profil wählbar:
  - **GPU-Server** — OpenAI-kompatibler Whisper-Server (z. B. `faster-whisper-server`) im LAN
  - **Lokal (whisper.cpp)** — heruntergeladenes `ggml-*.bin` (tiny / base / small / medium / large-v3), CPU
  - **LLM-Provider (Chat-Audio)** — Chat-Completion mit `input_audio`-Content-Part (Gemini 2.5 Flash/Pro via OpenRouter, gpt-4o-audio-preview)
- **Post-Processing** (optional): beliebiger OpenAI-kompatibler oder Anthropic-LLM korrigiert das Transkript (Großschreibung, Interpunktion, Fachbegriffe aus dem Wörterbuch, eigener System-Prompt möglich).
- **Modell-Manager**: Whisper-Modelle direkt aus der App von Huggingface laden, löschen, Größe & Status ablesen.
- **Wörterbuch**: Ein Begriff pro Zeile, live editierbar (kein Neustart), wird als Hint an Whisper und als Kontext an das Post-Processing-LLM gegeben.
- **Audio-Preview**: „Mikrofon testen"-Button mit Live-Pegelanzeige im Audio-Tab.
- **Sound-Cues** beim Aufnahme-Start/-Stopp (aufsteigender/absteigender Zwei-Ton-Chirp).
- **Tray-Menü** mit Settings und Beenden; Settings-Fenster versteckt sich beim Schließen, startet wieder über den Tray.
- **Sichere Key-Verwaltung**: API-Keys liegen im OS-Keyring (Windows Credential Manager / macOS Keychain), nicht in `config.json`.
- **History** der letzten Transkripte mit Copy-Icon und Löschen.
- **Tooltips** bei Hover auf erklärungsbedürftige Felder; abschaltbar unter „Allgemein".

## Build

| Plattform | Doku |
|-----------|------|
| Windows   | [`docs/BUILD-WINDOWS.md`](docs/BUILD-WINDOWS.md) |
| macOS     | [`docs/BUILD-MACOS.md`](docs/BUILD-MACOS.md)     |

Schnellstart (beide Plattformen, nach Voraussetzungen):

```bash
bun install
bun run tauri dev    # Development
bun run tauri build  # Release (.msi / .dmg)
```

## Konfiguration

Gespeichert in:

- Windows: `%APPDATA%\dss\Dictatr\config\config.json`
- macOS: `~/Library/Application Support/de.dss.dictatr/config.json`

Modelle in `…\data\models\ggml-*.bin` (Windows) bzw. `…/data/models/` (macOS).

API-Keys landen **nicht** in der config, sondern im OS-Keyring.

## Plattform-Unterschiede

| Feature                        | Windows | macOS |
|--------------------------------|---------|-------|
| Normale Hotkey-Kombos          | ✓       | ✓     |
| Multimedia-/Systemtasten       | ✓       | ✗ (Stub, nicht implementiert) |
| Remote-Whisper                 | ✓       | ✓     |
| LLM-Transkription (Chat-Audio) | ✓       | ✓     |
| Lokales whisper.cpp            | ✓       | ✓     |
| Mic-Level-Preview              | ✓       | ✓     |
| Text-Injection (Clipboard-Paste) | ✓     | ✓     |
| Keyring                        | Credential Manager | Keychain |

Details zu Änderungen: [`docs/CHANGELOG.md`](docs/CHANGELOG.md).

## Stack

- Tauri 2 + Rust (Orchestrator, Audio, Hotkeys, Transkription, Injection)
- React + TypeScript + Vite (Settings-UI)
- whisper-rs (lokal), reqwest (remote + LLM), rodio (Sounds), global-hotkey + windows-sys (Hotkey-Hooks), enigo + arboard (Text-Injection)
- bun als Paketmanager / Dev-Runner
