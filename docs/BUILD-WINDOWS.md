# Dictatr — Windows Build & Test Guide

Dieser Guide erklärt Schritt für Schritt, wie du den aktuellen Stand von `feat/phase1-mvp` auf einem Windows-Rechner baust, installierst und testest.

---

## 1. Voraussetzungen auf dem Windows-Host

Einmalig installieren (in dieser Reihenfolge):

### 1.1 Rust (MSVC Toolchain)

```powershell
# https://rustup.rs herunterladen und ausführen
# Standard-Installation wählen (MSVC ABI).
# Danach Powershell neu starten.
rustc --version
cargo --version
```

### 1.2 Microsoft Visual Studio Build Tools

```powershell
# https://visualstudio.microsoft.com/visual-cpp-build-tools/
# "Desktop development with C++" Workload installieren —
# enthält MSVC-Compiler, cmake und Windows SDK.
```

### 1.3 WebView2 Runtime

Meist bereits vorhanden unter Windows 11. Falls nicht:
https://developer.microsoft.com/microsoft-edge/webview2/

### 1.4 Bun

```powershell
# https://bun.sh/docs/installation
powershell -c "irm bun.sh/install.ps1 | iex"
bun --version
```

### 1.5 Git

https://git-scm.com/download/win

---

## 2. Repo klonen und Branch auschecken

```powershell
git clone <dein-git-remote> Dictatr
cd Dictatr
git checkout feat/phase1-mvp
```

Falls das Repo nur auf dem Synology/Debian-Host liegt, kopiere den Ordner per SMB/SCP herüber oder initialisiere ein Git-Remote auf dem Debian-Server:

```bash
# Auf Debian (Server):
cd /mnt/synology/Coding/Dictatr
git config --global --add safe.directory /mnt/synology/Coding/Dictatr
# SMB-Freigabe nutzen ODER remote pushen.
```

---

## 3. Frontend-Dependencies installieren

```powershell
cd Dictatr
bun install
```

Erwartete Ausgabe: ~70 packages, keine Fehler.

---

## 4. Dev-Build (schnell, zum Ausprobieren)

```powershell
bun run tauri dev
```

Erster Build dauert lange (5–15 min — whisper.cpp wird C++-kompiliert, cpal + reqwest + rusqlite ziehen einige Crates). Folgende Builds sind dank Inkremental-Compile schnell.

Wenn alles gut geht:
- Ein Tray-Icon erscheint im Windows-Infobereich.
- Das Settings-Fenster ist erstmal **unsichtbar** (`visible: false` in `tauri.conf.json`). Öffnen per Rechtsklick auf Tray → „Einstellungen".

Fehlerfälle:

| Fehler | Lösung |
|---|---|
| `cmake not found` | Visual Studio Build Tools Workload „C++" nachinstallieren |
| `linker error LNK2019` | Windows SDK nachinstallieren via VS Installer |
| `failed to load WebView2` | WebView2 Runtime installieren |
| `whisper-rs compilation error` | API-Breaking-Change — `src-tauri/core/src/transcription/local.rs` an aktuelle whisper-rs-Version anpassen |

---

## 5. Release-Build (MSI-Installer)

```powershell
bun run tauri build
```

Output: `src-tauri\target\release\bundle\msi\Dictatr_0.1.0_x64_en-US.msi`

Die MSI ist installierbar wie jede Standard-Windows-App.

---

## 6. Whisper-Modell herunterladen (für Offline-Diktat)

Das Modell `ggml-base.bin` (~140 MB) muss manuell abgelegt werden:

```powershell
# URL: https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin
$dest = "$env:APPDATA\Dictatr\models\ggml-base.bin"
New-Item -ItemType Directory -Force -Path (Split-Path $dest) | Out-Null
Invoke-WebRequest `
  "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin" `
  -OutFile $dest
```

Alternativ: größeres Modell für bessere Qualität:
- `ggml-small.bin` (~460 MB) — empfehlenswert für Deutsch
- `ggml-medium.bin` (~1.5 GB) — sehr gut, aber langsam auf CPU

Nach dem Download muss der Pfad im Code aktuell auf `ggml-base.bin` zeigen (`src-tauri/src/main.rs`). Für andere Modelle passt du die Zeile `model_path = dirs.data_dir().join("models").join("ggml-base.bin");` an.

---

## 7. Erste Konfiguration nach Installation

1. Tray-Icon → Rechtsklick → „Einstellungen"
2. Tab **LLM-Anbieter** → „+ Neuer Provider"
   - Beispiel OpenAI: Typ „OpenAI", Base-URL `https://api.openai.com`, Default-Modell `gpt-4o-mini`
   - API-Key eingeben → „API-Key speichern" (landet im Windows Credential Manager, NICHT in `config.json`)
3. Tab **Profile** → „+ Neues Profil"
   - Name: „Standard"
   - Hotkey: In das Feld klicken, dann gewünschte Tastenkombi drücken (z.B. `Ctrl+Alt+Space`)
   - Modus: Push-to-talk
   - Backend: „Lokal (whisper.cpp)" (funktioniert ohne Server) oder „GPU-Server" (falls Remote-Endpoint existiert)
   - Sprache: Deutsch
   - Post-Processing: erstmal aus — später einschalten und LLM-Profil wählen

4. Tab **Audio** → Mikrofon auswählen, Pegel testen (beim Reinsprechen sollte der grüne Balken ausschlagen — aktuell sieht man nur was, wenn auch Aufnahme läuft, das ist in einer Version 0.2 als Live-Test zu verbessern).

---

## 8. Diktieren testen

1. Cursor in Notepad oder VS Code setzen.
2. Hotkey halten (`Ctrl+Alt+Space`) → Tray-Icon wird rot (konzeptionell — aktuell noch keine dynamischen Icons).
3. Reinsprechen.
4. Loslassen → ca. 1–3 s Pause → Text erscheint an der Cursorposition.

Wenn's nicht funktioniert: `%APPDATA%\Dictatr\logs\` anschauen (Logs sind aktuell auf stdout — für Dev-Build im Terminal sichtbar).

---

## 9. Was im aktuellen MVP noch NICHT funktioniert

- **Remote-Whisper-Backend:** der `/api/dictate`-Endpoint existiert im DSS-V-A-Transcribe-Server noch nicht. Nur Local-Whisper testbar.
- **Pro-Hotkey-Tray-Icon-Farben:** Struktur ist da, Icons selbst noch Platzhalter.
- **Sounds beim Start/Stop:** nicht implementiert.
- **Live-Level-Meter im Audio-Tab:** zeigt nur während aktiver Aufnahme einen Wert.
- **Auto-Update:** Phase 2.
- **macOS-Build:** Phase 2.

---

## 10. Troubleshooting-Cheatsheet

| Symptom | Ursache | Fix |
|---|---|---|
| App startet, Tray-Icon fehlt | Bundler-Icon nicht erzeugt | `src-tauri/icons/icon.ico` mit echtem Icon ersetzen |
| Hotkey reagiert nicht | Konflikt mit System-Shortcut (z.B. Win+Space) | anderen Hotkey wählen |
| „device not found" | Mic-Name in Config vs. aktuelles Gerät | Audio-Tab: auf „System-Standard" umstellen |
| Text wird nicht eingefügt | Zielfenster akzeptiert keine Keystrokes (UAC-Dialog, abgesicherter Modus) | Clipboard-Fallback greift automatisch — per `Strg+V` einfügen |
| Transcription dauert >10s | Modell zu groß für CPU | `ggml-tiny.bin` oder `ggml-base.bin` statt `ggml-medium.bin` |

---

## 11. Weiterentwicklungs-Pfad

Siehe `docs/superpowers/plans/2026-04-13-dictatr-phase1-mvp.md` für die MVP-Tasks, und `docs/superpowers/specs/2026-04-13-dictatr-dictation-design.md` für Phase 2 (macOS, Auto-Update, Command-Mode).
