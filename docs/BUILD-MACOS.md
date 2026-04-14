# Dictatr — macOS Build

Gilt für `feat/macos-port`. Feature-Parität mit Windows inklusive lokalem
Whisper, mit folgenden Einschränkungen:

- Multimedia-/Systemtasten (`LaunchMail`, `VolumeUp`, …) sind Windows-only
  (`WH_KEYBOARD_LL`) und auf macOS stubbed — die Einträge im Dropdown erscheinen,
  tun aber nichts. Normale Hotkey-Kombos (`Ctrl+Alt+Space` etc.) funktionieren
  über `global-hotkey`.
- Auf **macOS 26 beta** blockiert TCC synthetische Cmd+V-Events für ad-hoc
  signierte Apps. Die transkribierte Zeichenkette landet zuverlässig in der
  Zwischenablage — der Nutzer muss aktuell manuell `Cmd+V` drücken, bis eine
  Developer-ID-Signatur verwendet wird. Auf stabilem macOS sollte der Auto-Paste
  funktionieren.

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
git checkout feat/macos-port
bun install
```

macOS 26 beta und Apple Silicon brauchen ein paar Workarounds gegen kaputte
Toolchain-Flags (whisper.cpp passt `-mcpu=native`/`-mavx` unkonditional an den
Compiler — auf Apple clang 21 ungültig; die vendored whisper.cpp referenziert
dazu `tests/`/`examples/`-Verzeichnisse, die der Crate nicht einschließt).
Dafür liegen Wrapper-Scripts in `tools/` bei, die `cc`/`c++`/`cmake`-Aufrufe
vor der Weiterleitung umschreiben. Diese Env-Variablen vor dem Build setzen:

```bash
# macOS 26 beta: Command Line Tools haben kaputte C++-Header-Pfade —
# SDKROOT setzen, damit <mutex>/<string>/<atomic> gefunden werden.
export SDKROOT="$(xcrun --show-sdk-path)"
export MACOSX_DEPLOYMENT_TARGET=11.0
export CPLUS_INCLUDE_PATH="$SDKROOT/usr/include/c++/v1:$SDKROOT/usr/include"
export CPATH="$SDKROOT/usr/include"

# Clang/CMake-Wrapper: filtern x86-only Flags, ersetzen -mcpu=native durch
# -mcpu=apple-m1, injizieren WHISPER_BUILD_TESTS=OFF in whisper.cpp's CMake.
export CC="$(pwd)/tools/macos-clang-wrap.sh"
export CXX="$(pwd)/tools/macos-clangxx-wrap.sh"
export CMAKE="$(pwd)/tools/macos-cmake-wrap.sh"

# Development
bun run tauri dev

# Produktion (.app + .dmg)
bun run tauri build
```

Auf stabilem macOS (nicht 26 beta) sind die Env-Vars i.d.R. nicht nötig.

Das fertige `.app` / `.dmg` liegt in `src-tauri/target/release/bundle/`. Die
App wird automatisch ad-hoc signiert (`bundle.macOS.signingIdentity: "-"`).

**Pflicht-Schritt nach jedem Build auf macOS 26 beta:**

```bash
./tools/macos-resign.sh
# und falls bereits in /Applications installiert:
./tools/macos-resign.sh /Applications/Dictatr.app
```

Tauri's Bundler aktiviert Hardened Runtime per Default. Kombination
Hardened-Runtime + ad-hoc-Signatur blockiert auf macOS 26 TCC-Permission-Dialoge
(Mikrofon, Bedienungshilfen) stumm — `AVCaptureDevice.requestAccess` resolvet
als „denied", ohne dass der User den Dialog jemals zu sehen bekommt. Das
Script entfernt das Runtime-Flag per Re-Sign. Mit Developer-ID entfällt der
Schritt (Hardened Runtime funktioniert dann korrekt mit Entitlements).

## Berechtigungen (Systemeinstellungen)

macOS fragt beim ersten Start nach:
- **Mikrofon** — für die Aufnahme. Beim App-Start triggert die App einen
  `AVCaptureDevice.requestAccess`-Call, der den Dialog direkt auslöst.
- **Bedienungshilfen** — für `CGEventPost`-Tastendrücke (Cmd+V). Muss manuell
  unter System Settings → Datenschutz & Sicherheit → Bedienungshilfen
  hinzugefügt werden. "+" → Dictatr.app auswählen → aktivieren.
- **Eingabeüberwachung** — für globale Hotkeys. Wird beim ersten Hotkey
  automatisch abgefragt.

Nach dem Erteilen muss die App einmal neu gestartet werden.

## macOS 26 beta: TCC-Caveats bei ad-hoc signierten Builds

TCC invalidiert den Accessibility-Eintrag bei jedem Rebuild (Binary-Hash
ändert sich). Wenn `[inject] AXIsProcessTrusted = false` im Log steht obwohl
Dictatr in den Bedienungshilfen aktiviert ist:

```bash
# TCC-Eintrag resetten
tccutil reset All de.dss.dictatr

# App per open starten (wichtig: nicht direkt das Binary, damit
# LaunchServices den Prozess als Bundle registriert)
open /path/to/Dictatr.app

# In System Settings → Bedienungshilfen → "+" → Dictatr.app erneut hinzufügen
# Dann App nochmal neu starten.
```

Für einen Build, der diese Friktionen vermeidet, braucht es eine Apple
Developer ID-Signatur (`bundle.macOS.signingIdentity: "Developer ID Application: ..."`)
und Notarization.

## Known Issues (macOS)

- Auto-Paste (synthetisches Cmd+V via CGEventPost) schlägt auf macOS 26 beta
  ohne Developer-ID fehl. Fallback: Text bleibt in der Zwischenablage, manueller
  Cmd+V durch den Nutzer.
- Multimedia-/Systemtasten sind Windows-only (Dropdown-Optionen sichtbar aber
  wirkungslos). Für macOS-Support wäre `CGEventTap` nötig — eigener Task.
- Metal-Beschleunigung für whisper-rs 0.12 nicht aktiviert (0.12 unterstützt das
  Feature noch nicht sauber auf Apple Silicon). Small-Modell braucht auf CPU
  etwa 1x Audiolänge.
