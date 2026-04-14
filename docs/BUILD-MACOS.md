# Dictatr — macOS Build

Gilt für Phase 1 (`feat/phase1-mvp`). Das gleiche Feature-Set wie Windows, mit einer Ausnahme: Multimedia-/Systemtasten (LaunchMail, VolumeUp, …) sind Windows-only (`WH_KEYBOARD_LL`) und auf macOS stubbed — die Einträge im Dropdown erscheinen, tun aber nichts. Normale Hotkey-Kombos (`Ctrl+Alt+Space` etc.) funktionieren über `global-hotkey`.

## Voraussetzungen

```bash
# Xcode Command Line Tools (clang, make, etc.)
xcode-select --install

# Rust
curl https://sh.rustup.rs -sSf | sh

# bun
curl -fsSL https://bun.sh/install | bash

# cmake (whisper-rs-sys baut whisper.cpp via cmake)
brew install cmake
```

## Build

```bash
git clone https://github.com/DSS-AI/Dictatr.git
cd Dictatr
git checkout feat/phase1-mvp
bun install

# Development
bun run tauri dev

# Produktion (DMG)
bun run tauri build
```

Das fertige `.app` / `.dmg` liegt in `src-tauri/target/release/bundle/`.

## Berechtigungen (Systemeinstellungen)

macOS fragt beim ersten Start nach:
- **Mikrofon** (für die Aufnahme)
- **Eingabeüberwachung** (für `global-hotkey` und `enigo`-Paste)
- **Bedienungshilfen** (für `enigo`, wenn Ctrl+V / Cmd+V geschickt wird)

Nach dem Erteilen muss die App einmal neu gestartet werden.

## Known Issues

- Auf macOS sendet `enigo` Cmd+V (nicht Ctrl+V). Der aktuelle Inject-Code verwendet `Key::Control` — das entspricht auf macOS Ctrl und nicht Cmd. Falls die Paste auf macOS nicht greift, muss in `core/src/inject.rs` plattformabhängig auf `Key::Meta` gewechselt werden.
- Multimedia-/Systemtasten sind wie oben erwähnt Windows-only (Dropdown-Optionen sichtbar aber wirkungslos). Für macOS-Support wäre `CGEventTap` nötig — nicht im Scope von Phase 1.
