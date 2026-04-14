# CLAUDE.md

Guidance für Claude Code in diesem Repo.

## Project Overview

**Dictatr** — leichtgewichtige Desktop-Diktier-App für Windows (später macOS). Self-hosted Alternative zu [wisprflow.ai](https://www.wisprflow.ai).

Läuft im Hintergrund, wird per globalem Hotkey aktiviert, transkribiert Sprache (Whisper) und fügt den Text an der aktuellen Cursor-Position ein. Multi-LLM-Post-Processing für Korrektur und Umformulierung. User wählt den LLM-Provider frei.

GitHub: https://github.com/DSS-AI/Dictatr

---

## Tech Stack

| Technology | Purpose |
|---|---|
| Rust 1.80+ | Backend-Core (Audio, Hotkey, Transcription, State-Machine) |
| Tauri 2.x | Desktop-App-Framework, cross-platform Win+Mac |
| React 18 + TypeScript | Settings-UI im Tauri-Webview |
| Bun | JS-Runtime + Paketmanager |
| Vite 5 | Build-Tool für Frontend |
| cpal | Audio-Capture cross-platform |
| whisper-rs | Lokales Whisper (whisper.cpp via FFI) |
| enigo | Keystroke-Simulation (Text-Injection) |
| global-hotkey | Globale Hotkey-Registrierung |
| rusqlite | Lokale History-DB |
| keyring | OS-Keyring (Credential Manager / Keychain) |

---

## Workspace-Architektur

Cargo-Workspace mit zwei Crates:

- **`dictatr-core`** (`src-tauri/core/`) — reine Logik, keine Tauri-Deps. Alle Module testbar ohne GUI-Stack. Enthält: audio, config, error, history, hotkey, inject, llm, orchestrator, secrets, state, transcription.
- **`dictatr`** (`src-tauri/`) — Tauri-Binary mit IPC-Commands, Tray, Overlay, main.rs-Verdrahtung.

Frontend-Code liegt in `src/` (React + TS).

---

## Commands

```bash
# Frontend-Deps installieren
bun install

# Dev-Build (Hot-Reload)
bun run tauri dev

# Release-Build (MSI auf Windows)
bun run tauri build

# Core-Tests (Linux: läuft ohne whisper-rs)
cd src-tauri && cargo test -p dictatr-core

# TypeScript-Check
bunx tsc --noEmit
```

---

## Project Structure

```
Dictatr/
├── .claude/                          # Projekt-lokale Commands/Skills (mostly global now)
├── CLAUDE.md                         # Diese Datei
├── package.json                      # Frontend-Deps + Tauri-Skripte
├── tsconfig*.json, vite.config.ts    # TS- und Vite-Konfig
├── index.html, public/               # Frontend-Entry, statische Assets
├── src/                              # React-App
│   ├── App.tsx, main.tsx, index.css
│   ├── ipc.ts, types.ts              # Tauri-IPC-Wrapper
│   ├── components/                   # HotkeyRecorder, LevelMeter
│   └── pages/                        # Profiles, Providers, Vocabulary, Audio, General, History
├── src-tauri/                        # Rust-Workspace-Root + Tauri-Binary
│   ├── Cargo.toml                    # Workspace + dictatr-binary-Manifest
│   ├── tauri.conf.json               # Tauri-App-Konfig
│   ├── icons/                        # App-Icons (icon.ico, tray-*.png)
│   ├── src/                          # Tauri-Binary (main.rs, commands.rs, tray.rs, overlay.rs)
│   └── core/                         # dictatr-core Crate (reine Logik, testbar)
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── audio/                # capture, controller, ringbuffer
│           ├── config/               # profile, provider, general
│           ├── error.rs, state.rs, secrets.rs, hotkey.rs, inject.rs
│           ├── history/              # SQLite-Store
│           ├── llm/                  # openai_compat, anthropic, prompt
│           └── transcription/        # remote (GPU-Server), local (whisper.cpp)
└── docs/
    ├── BUILD-WINDOWS.md              # Schritt-für-Schritt Windows-Build-Guide
    └── superpowers/                  # Design-Spec + Phase-1-Implementation-Plan
        ├── specs/2026-04-13-dss-whisper-dictation-design.md
        └── plans/2026-04-13-dss-whisper-phase1-mvp.md
```

---

## Architektur-Überblick

```
Global-Hotkey → AudioCapture (16 kHz mono, Ringbuffer)
             → State-Machine (Idle/Recording/Transcribing/Injecting)
             → TranscriptionBackend (remote GPU-Server ODER lokales whisper.cpp)
             → [optional] LLM-Post-Processing (openai-compat / anthropic / ollama)
             → Text-Injection via enigo (Clipboard-Fallback)
             → History (SQLite) + Tray-Icon-Update
```

Profile binden Hotkey + Backend + Sprache + Post-Processing-Einstellungen zusammen. Mehrere Profile parallel (z.B. „schnell", „E-Mail formal", „Englisch").

---

## Konfiguration & Datenpfade (Windows)

| Pfad | Inhalt |
|---|---|
| `%APPDATA%/Dictatr/config.json` | Profile, LLM-Provider-Metadaten, UI-Einstellungen |
| `%APPDATA%/Dictatr/vocabulary.txt` | Eigene Begriffe (eine Zeile pro Eintrag) |
| `%APPDATA%/Dictatr/history.db` | SQLite-History |
| `%APPDATA%/Dictatr/models/ggml-base.bin` | Lokales Whisper-Modell (~140 MB, separat runterladen) |
| Windows Credential Manager | API-Keys (Service „Dictatr") |

macOS-Äquivalent: `~/Library/Application Support/Dictatr/`.

---

## Testing

```bash
# Rust-Unit-Tests (dictatr-core)
cd src-tauri && cargo test -p dictatr-core

# TypeScript-Typecheck
bunx tsc --noEmit
```

Stand Phase 1 MVP: **24 Rust-Unit-Tests grün** (Ringbuffer, Resample, State-Machine, Hotkey-Parsing, Config-Serde, Profile-Validation, History, Prompt-Builder, wiremock für RemoteWhisper + OpenAI-compat). TS-Check 0 Fehler.

---

## Phasen-Stand

### Phase 1 MVP — ✅ abgeschlossen (Branch `feat/phase1-mvp`)

- Tauri-Skeleton, Settings-UI mit 6 Tabs, Tray-Icon, Overlay
- Audio-Capture + globale Hotkeys (push-to-talk + toggle)
- TranscriptionBackend-Trait + RemoteWhisperBackend (HTTP) + LocalWhisperBackend (whisper.cpp)
- LlmProvider-Trait + OpenAI-compat + Anthropic-Adapter + Prompt-Builder
- Orchestrator mit Auto-Fallback (Remote offline → Local)
- SQLite-History, OS-Keyring für API-Keys
- Text-Injection via enigo mit Clipboard-Fallback

### Deferred auf Phase 2 / externen Host

- **Server-Endpoint `/api/dictate`** in DSS-V-A-Transcribe (nur mit User-Freigabe; Projekt darf nicht ohne Weisung verändert werden)
- **MSI-Installer** (nur auf Windows-Host baubar)
- **macOS-Port** (Accessibility-Permissions, Notarization)
- **Auto-Updater** (Tauri-Updater)
- **Kontext-aware Prompts** + Command-Mode

---

## Key Files

| File | Purpose |
|---|---|
| `src-tauri/src/main.rs` | Tauri-Setup, verdrahtet Orchestrator + Audio + Hotkey + Tray |
| `src-tauri/src/commands.rs` | IPC-Commands (Config-CRUD, History, API-Keys) |
| `src-tauri/core/src/orchestrator.rs` | Kern-Event-Loop: Hotkey → Audio → Transcribe → LLM → Inject |
| `src-tauri/core/src/audio/controller.rs` | Send-sicherer Wrapper um AudioCapture |
| `src-tauri/core/src/transcription/{remote,local}.rs` | Backend-Implementierungen |
| `src-tauri/core/src/llm/{openai_compat,anthropic}.rs` | LLM-Provider-Adapter |
| `src/pages/Profiles.tsx` | Zentrale Settings-UI für Dictation-Profile |
| `docs/BUILD-WINDOWS.md` | Windows-Build-Guide für neue Umgebungen |
| `docs/superpowers/specs/2026-04-13-*-design.md` | Design-Dokument (historisch unter DSS-Whisper) |
| `docs/superpowers/plans/2026-04-13-*-phase1-mvp.md` | Implementation-Plan mit 20 Tasks |

---

## Notes

- **Branch-Strategie:** `master` enthält den initialen Template-Stand + Design-Docs. `feat/phase1-mvp` ist der aktuelle Arbeitszweig. PR/Merge nach Windows-Build-Verifikation.
- **Build-Abhängigkeiten auf Linux:** cpal braucht `libasound2-dev`, enigo braucht `libxdo-dev`, whisper-rs braucht `cmake` + `clang` + `libclang-dev` (nicht alle auf diesem Debian-Host installiert — Builds auf Windows-Host verschieben, wo MSVC alles mitbringt).
- **Vom NAS-Mount bauen ist verboten:** Build-Artefakte gehören nicht auf die NAS (Locking + Performance). Unter Windows lokal nach `C:\Dev\Dictatr\` klonen.
- **DSS-V-A-Transcribe bleibt unangetastet:** Das Remote-Whisper-Backend spricht gegen den bestehenden Port 8503, ein neuer `/api/dictate`-Endpoint wird nur mit ausdrücklicher Freigabe implementiert.

---

## Dev-Workflow: Windows als primäre Umgebung

Dictatr wird primär auf einem Windows-Rechner entwickelt (MSVC, `bun run tauri dev` / `build`). Der Linux-Workspace (`/mnt/synology/Coding/DSS-Whisper` auf dem Debian-Host) dient der Codebase-Analyse, Rust-Core-Tests und Claude-Code-Assistenz — er zieht Code vom Remote, pusht aber selten.

### Auto-Sync vom Remote

Bei jedem Claude-Code-Session-Start im Projekt läuft ein `SessionStart`-Hook, der automatisch `git fetch` + `git pull --ff-only` ausführt.

| Komponente | Ort | Zweck |
|---|---|---|
| Hook-Definition | `.claude/settings.local.json` (gitignored, lokal) | Verdrahtet den Hook ins Claude-Code-Runtime |
| Sync-Skript | `.claude/hooks/git-sync.sh` | Führt die Sync-Logik aus (chmod +x erforderlich) |

**Safety-Verhalten:**
- Clean Working Tree + Commits ahead → Fast-Forward-Pull + Commit-Log-Output
- Clean + up-to-date → Meldung „up-to-date"
- Uncommitted Changes → Skip mit Warnung (kein Auto-Merge)
- Divergent (non-FF) → Skip mit Fehlermeldung (kein Force)
- Offline → Skip mit „fetch fehlgeschlagen"

**Wenn der Hook nicht greift:** Settings-Watcher liest `.claude/settings.local.json` nur beim Session-Start. `claude` neu starten oder einmal `/hooks` öffnen, um die Config neu zu laden.

**Manuell syncen:** `./.claude/hooks/git-sync.sh` direkt aufrufen.
