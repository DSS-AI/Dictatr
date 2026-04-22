# Dictatr

Diktat-Tool für Windows und macOS. Hotkey drücken, sprechen, loslassen — der Text wird transkribiert und direkt ins aktive Textfeld geschrieben.

Remote-Whisper, lokales whisper.cpp und LLM-Provider mit Audio-Input (Gemini 2.5, gpt-4o-audio) stehen als Backend zur Auswahl. Optionales LLM-Post-Processing bügelt Satzzeichen und Fachbegriffe glatt.

## Features

- **Hotkeys**: normale Kombos (Ctrl+Alt+Space …) per `global-hotkey`, zusätzlich Windows-Multimedia- und Launch-Tasten (LaunchMail, VolumeUp, BrowserHome, Media-Keys …) via Low-Level-Keyboard-Hook — Outlook & Co. werden dabei unterdrückt. Push-to-talk oder Toggle pro Profil.
- **Transkriptions-Backends** pro Profil wählbar. Für deutsches Diktat empfohlen in dieser Reihenfolge:
  1. **LLM-Provider (Chat-Audio)** — Chat-Completion mit `input_audio`-Content-Part. Gemini 2.5 Flash/Pro via OpenRouter oder gpt-4o-audio-preview via OpenAI. Höchste Qualität, minimale Halluzinationen, Namen/Zahlen sitzen. Kostet pro Request, aber bei Dictatr-Nutzungs-Volumen vernachlässigbar. **Empfohlene Standard-Wahl.**
  2. **GPU-Server** — OpenAI-kompatibler Whisper-Server (z. B. `faster-whisper-server`) im LAN. Server-seitig lässt sich Silero-VAD + `suppress_tokens`-Config sauber einrichten, was die Qualität gegenüber lokalem whisper.cpp deutlich anhebt.
  3. **Lokal (whisper.cpp)** — heruntergeladenes `ggml-*.bin` (tiny / base / small / medium / large-v3), rein CPU (Metal-Beschleunigung wird von `whisper-rs` 0.12 auf Apple Silicon noch nicht zuverlässig unterstützt). **Nur als Offline-Fallback gedacht, sehr experimentell.** Kann auf Stille Trainingsdaten-Floskeln halluzinieren („Danke fürs Zuschauen", „Untertitel im Auftrag des ZDF", „Schreibt es in die Kommentare", „SWR 2020" usw.) und auf kurzen Utterances in Repeat-Loops kippen. Im Code sind mehrere Post-Filter aktiv (`collapse_repetitions`, `strip_trailing_hallucinations`, `temperature_inc`-Fallback), die die gröbsten Kanten abfangen — die Qualität kommt aber nicht an die LLM- oder Remote-GPU-Backends heran. Für produktive Nutzung nicht empfohlen.
- **Post-Processing** (optional): beliebiger OpenAI-kompatibler oder Anthropic-LLM korrigiert das Transkript (Großschreibung, Interpunktion, Fachbegriffe aus dem Wörterbuch, eigener System-Prompt möglich).
- **Modell-Manager**: Whisper-Modelle direkt aus der App von Huggingface laden, löschen, Größe & Status ablesen.
- **Wörterbuch**: Ein Begriff pro Zeile, live editierbar (kein Neustart), wird als Hint an Whisper und als Kontext an das Post-Processing-LLM gegeben.
- **Audio-Preview**: „Mikrofon testen"-Button mit Live-Pegelanzeige im Audio-Tab.
- **Sound-Cues** beim Aufnahme-Start/-Stopp (aufsteigender/absteigender Zwei-Ton-Chirp).
- **Aufnahme-Indikator**: Kleines Overlay am unteren Bildschirmrand (Primary Monitor, mittig) zeigt während jeder Aufnahme einen pulsierenden REC-Dot plus eine Oszilloskop-Waveform, die live auf den Mikrofonpegel reagiert. Beim Ende der Aufnahme blendet sich das Overlay aus. Gilt für Push-to-talk und Toggle gleichermaßen.
- **Zwischenablage-Modi pro Profil**:
  - „Nur in Zwischenablage (kein Auto-Einfügen)" — Text landet nach dem Diktat nur im Clipboard, du fügst selbst mit Strg+V ein. Sinnvoll für Remote-Desktop-Fenster oder elevierte Apps (PowerShell als Admin), in die das automatische Einfügen durch Windows-UIPI oder RDP-Keyboard-Capture nicht durchkommt.
  - „Text ins Clipboard kopieren" — Auto-Einfügen läuft wie gewohnt, aber der transkribierte Text bleibt zusätzlich in der Zwischenablage liegen (normal würde der alte Clipboard-Inhalt wiederhergestellt).
- **Tray-Menü** mit „Einstellungen", „Neustart" und „Beenden"; „Neustart" hilft, wenn größere Config-Änderungen (Backend, LLM-Provider, API-Keys) einen sauberen Restart brauchen.
- **Hotkey-Änderungen greifen live** — nach „Speichern" wird die globale Hotkey-Registrierung im laufenden Prozess ausgetauscht, kein App-Neustart nötig.
- **Sichere Key-Verwaltung**: API-Keys liegen im OS-Keyring (Windows Credential Manager / macOS Keychain), nicht in `config.json`.
- **History** der letzten Transkripte mit Copy-Icon und Löschen.
- **Tooltips** bei Hover auf erklärungsbedürftige Felder; abschaltbar unter „Allgemein".

## Download

| Plattform | Installer |
|-----------|-----------|
| Windows (x64)        | [Aktuelle Release auf GitHub](https://github.com/DSS-AI/Dictatr/releases/latest) — `.msi` herunterladen und ausführen |
| macOS (Apple Silicon)| Ab nächstem Release als `.dmg` verfügbar. Aktuell nur Build aus Source, siehe [`docs/BUILD-MACOS.md`](docs/BUILD-MACOS.md). Auf macOS 26 beta ist nach `bun run tauri build` zusätzlich `./tools/macos-resign.sh` nötig (ad-hoc-Re-Sign ohne Hardened Runtime), sonst blockiert TCC stumm die Mic- und Bedienungshilfen-Dialoge. |

Nach der Installation aktualisiert sich Dictatr bei neuen Releases automatisch (Banner im Settings-Fenster oder Button „Nach Updates suchen" im Allgemein-Tab).

## Empfohlenes Setup (für deutsches Diktat)

1. **Transkriptions-Backend:** LLM-Chat-Audio mit Gemini 2.5 Flash via OpenRouter (günstig, schnell, sehr robust für Deutsch) oder gpt-4o-audio-preview via OpenAI. Key unter Settings → Provider hinterlegen, Profil auf „LLM (Chat-Audio)" stellen.
2. **Hotkey:** Push-to-talk auf eine Taste, die nicht in Editor/Terminal kollidiert (z. B. eine unbenutzte Multimedia-/Funktionstaste oder `Ctrl+Alt+Space`).
3. **Wörterbuch** mit wiederkehrenden Eigennamen/Fachbegriffen befüllen — wird als Kontext-Hint an das Backend übergeben.
4. **Lokales whisper.cpp** nur als Offline-Fallback konfigurieren, falls Internet ausfällt; nicht als primären Pfad.

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
| Lokales whisper.cpp            | ✓ (experimentell) | ✓ (experimentell, CPU-only) |
| Mic-Level-Preview              | ✓       | ✓     |
| Text-Injection (Clipboard-Paste) | ✓     | ✓ (auf macOS 26 beta ohne Developer ID: Text landet in Zwischenablage, manueller Cmd+V) |
| Keyring                        | Credential Manager | Keychain |

Details zu Änderungen: [`docs/CHANGELOG.md`](docs/CHANGELOG.md).

## Stack

- Tauri 2 + Rust (Orchestrator, Audio, Hotkeys, Transkription, Injection)
- React + TypeScript + Vite (Settings-UI)
- whisper-rs (lokal), reqwest (remote + LLM), rodio (Sounds), global-hotkey + windows-sys (Hotkey-Hooks), enigo + arboard (Text-Injection)
- bun als Paketmanager / Dev-Runner
