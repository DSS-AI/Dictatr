# Dictatr — Release-Prozess

Zwei Wege: **automatisiert via GitHub Actions** (empfohlen — ein `git push --tags` reicht) oder **manuell vom Windows-Host** (als Fallback).

Installierte Dictatr-Instanzen holen sich das Update beim nächsten Start automatisch über `tauri-plugin-updater` gegen GitHub Releases.

---

## A) Automatisiert — GitHub Actions (empfohlen)

Workflow: `.github/workflows/release.yml`. Triggert auf `v*`-Tag-Push und `workflow_dispatch`.

### Einmaliges Setup der Repo-Secrets

Auf https://github.com/DSS-AI/Dictatr/settings/secrets/actions folgende **Repository Secrets** anlegen:

| Secret | Wert |
|---|---|
| `TAURI_SIGNING_PRIVATE_KEY` | Kompletter Inhalt der lokalen `updater.key` (inkl. `untrusted comment`-Zeile) |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Passwort, das bei `signer generate` vergeben wurde |

Der Public Key muss bereits in `src-tauri/tauri.conf.json` unter `plugins.updater.pubkey` stehen (siehe Schritt 1 im manuellen Prozess unten).

### Pflicht-Konfig in `tauri.conf.json`

- `bundle.createUpdaterArtifacts: true` — **ohne das Flag** tagged `tauri-cli` die `.msi.zip` / `.app.tar.gz`-Bundles nicht als Updater-Artefakte, `tauri-action` meldet `Signature not found for the updater JSON. Skipping upload...` und im Release liegen am Ende nur Binaries, **keine `.sig`-Files und keine `latest.json`**. Bug-Symptom im Client: `Could not fetch a valid release JSON from the remote`. (Genau das war in v0.1.2 passiert — siehe CHANGELOG.)
- `plugins.updater.endpoints` / `plugins.updater.pubkey` — siehe unten.

### ACL-Capabilities (Pflicht für Tauri 2)

`src-tauri/capabilities/default.json` muss dem Main-Window mindestens `core:default`, `updater:default`, `process:default` granten, sonst:

- `plugin:updater|check` / `plugin:updater|download-and-install` / `plugin:process|restart` brechen mit „not allowed by ACL" ab.
- `getVersion()` (`core:app:allow-version`) liefert still nichts → Versionsanzeige zeigt `?`.

Die Datei ist im Repo; neue Capabilities ergänzen, wenn Plugin-Commands aus dem Webview nicht greifen.

### Release raushauen

```powershell
# 1. Version in 3 Dateien bumpen (Snippet siehe weiter unten)
# 2. CHANGELOG pflegen
git commit -am "release: v0.2.0"
git tag v0.2.0
git push && git push --tags
```

Danach läuft die Action auf einem `windows-latest`-Runner: LLVM 18 installieren, `bun install`, `bun run tauri build`, signieren, GitHub-Release erstellen, MSI + `.msi.sig` + `latest.json` hochladen. Dauer: ~12–20 min (Cold-Cache), ~5–8 min mit Cargo-Cache.

Status der Action: https://github.com/DSS-AI/Dictatr/actions

### Manuell auslösen (ohne Tag-Push)

Action-Tab → „Release"-Workflow → „Run workflow" → Branch wählen. Praktisch für Test-Builds ohne Release.

---

## B) Manuell vom Windows-Host (Fallback)

### Einmaliges Setup auf dem Windows-Host

Auch für den CI-Weg nötig, weil der Public Key ins Repo muss und der Private Key aus einer lokalen Key-Generierung kommt.

#### 1. Updater-Keypair erzeugen

```powershell
bunx @tauri-apps/cli signer generate -w C:\Users\dss\.dictatr\updater.key
```

- **`updater.key`** ist der **private Schlüssel** — bleibt ausschließlich auf diesem Rechner, nie ins Repo.
- Die Kommando-Ausgabe zeigt den **Public Key** (Base64). Kopiere ihn in `src-tauri/tauri.conf.json` unter `plugins.updater.pubkey` (ersetzt den Platzhalter `REPLACE_WITH_PUBKEY_FROM_TAURI_SIGNER_GENERATE`).

#### 2. Signing-Environment setzen

Als **User-Environment-Variablen** (Windows: „Systemumgebungsvariablen bearbeiten"):

| Variable | Wert |
|---|---|
| `TAURI_SIGNING_PRIVATE_KEY` | **Inhalt** der Datei `updater.key` (nicht der Pfad) |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Passwort, das bei der Key-Generierung vergeben wurde |

`bun run tauri build` liest die beiden Variablen und signiert den MSI-Build automatisch.

Für CI (Weg A) den **gleichen Inhalt** zusätzlich als GitHub-Repo-Secrets hinterlegen (siehe oben).

**Gotcha:** Mit `bundle.createUpdaterArtifacts: true` **verlangt** `bun run tauri build` die Env-Vars auch lokal. Fehlen sie, wird die MSI noch fertig gebaut, aber der Bundler-Step danach scheitert mit `A public key has been found, but no private key.` und der Build-Prozess endet mit Exit-Code 1. Die MSI unter `src-tauri/target/release/bundle/msi/Dictatr_*_x64_en-US.msi` ist trotzdem valide und installierbar — nur die `.msi.zip.sig` fehlt.

---

### Pro manuellen Release

#### 1. Version bumpen

In **drei** Dateien synchron:

| Datei | Zeile |
|---|---|
| `package.json` | `"version": "0.2.0"` |
| `src-tauri/Cargo.toml` | `version = "0.2.0"` |
| `src-tauri/tauri.conf.json` | `"version": "0.2.0"` |

PowerShell-Snippet (setzt die Version in allen drei Dateien):

```powershell
$v = "0.2.0"
(Get-Content package.json)             -replace '"version": "[^"]+"', "`"version`": `"$v`""     | Set-Content package.json
(Get-Content src-tauri\tauri.conf.json) -replace '"version": "[^"]+"', "`"version`": `"$v`""    | Set-Content src-tauri\tauri.conf.json
(Get-Content src-tauri\Cargo.toml)     -replace '^version = "[^"]+"',  "version = `"$v`""      | Set-Content src-tauri\Cargo.toml
```

#### 2. Changelog-Eintrag

Neuen Abschnitt in `docs/CHANGELOG.md` — was hat sich seit dem letzten Release geändert.

#### 3. Commit + Tag + Push

```powershell
git commit -am "release: v0.2.0"
git tag v0.2.0
git push
git push --tags
```

#### 4. Bauen

```powershell
bun install
bun run tauri build
```

Output:

- `src-tauri\target\release\bundle\msi\Dictatr_0.2.0_x64_en-US.msi`
- `src-tauri\target\release\bundle\msi\Dictatr_0.2.0_x64_en-US.msi.sig`

Die `.sig`-Datei enthält eine einzige Base64-Zeile — die Tauri-Updater-Signatur.

#### 5. GitHub-Release erstellen

```powershell
gh release create v0.2.0 `
  "src-tauri\target\release\bundle\msi\Dictatr_0.2.0_x64_en-US.msi" `
  "src-tauri\target\release\bundle\msi\Dictatr_0.2.0_x64_en-US.msi.sig" `
  --title "v0.2.0" --notes-file docs\CHANGELOG.md
```

#### 6. `latest.json` befüllen und hochladen

Template (`docs/templates/latest.json`):

```json
{
  "version": "0.2.0",
  "notes": "siehe CHANGELOG.md",
  "pub_date": "2026-04-15T12:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "<inhalt von .msi.sig kopieren>",
      "url": "https://github.com/DSS-AI/Dictatr/releases/download/v0.2.0/Dictatr_0.2.0_x64_en-US.msi"
    }
  }
}
```

- **`signature`**: Kompletter Inhalt der `.msi.sig`-Datei (eine Zeile Base64).
- **`url`**: Direkter Download-Link zum MSI-Asset des Releases.
- **`pub_date`**: ISO-8601, aktuelle UTC-Zeit.

Dann hochladen:

```powershell
gh release upload v0.2.0 latest.json
```

#### 7. Verifikation

- Auf einem Rechner mit älterer Dictatr-Version App starten → Banner „Dictatr 0.2.0 ist verfügbar" erscheint.
- „Installieren" klicken → Download-Progress, MSI installiert, App startet neu.
- Allgemein-Tab → „Aktuelle Version: 0.2.0".
- „Nach Updates suchen" → „Du hast bereits die aktuellste Version."

---

## Troubleshooting

- **Banner erscheint nicht:** `latest.json` als Release-Asset prüfen, URL in `tauri.conf.json` (`plugins.updater.endpoints`) muss auf diese Datei zeigen. Tauri folgt `releases/latest/download/latest.json` — zeigt immer auf den neuesten Release.
- **„signature verification failed":** `pubkey` in `tauri.conf.json` passt nicht zum `TAURI_SIGNING_PRIVATE_KEY` des Builds. Pubkey aus `bunx @tauri-apps/cli signer sign --help` bzw. erneut generieren und committen.
- **Offline-Start:** Update-Check scheitert stillschweigend (nur `console.warn`) — das ist so gewollt, kein Error-Dialog.
- **Older version zeigt kein Update:** `version` in `latest.json` > installierter Version? Tauri vergleicht Semver.
