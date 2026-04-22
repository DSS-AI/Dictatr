# Dictatr — Änderungs-Log

## Unreleased

## v0.1.9 — 2026-04-22 — Cloudflare Access + URL-Normalisierung

- **Cloudflare Access Service-Token für den GPU-Server:** Zwei neue Felder im Allgemein-Tab („Client-ID" + „Client-Secret"). Die Client-ID landet in `config.json`, das Secret im OS-Keyring (`named-cf_access_secret`). `RemoteWhisperBackend` hängt bei belegten Credentials `CF-Access-Client-Id` + `CF-Access-Client-Secret` an beide Request-Sites an (`/v1/models`-Probe + `/v1/audio/transcriptions`). Erlaubt den Betrieb des Whisper-Servers hinter einem Cloudflare-Tunnel mit Access-Policy.
- **URL-Schema optional:** Das GPU-Server-Feld akzeptiert jetzt bloße Hostnamen; fehlt das Schema, wird `https://` angenommen (`dictatr_core::config::normalize_remote_url`). Gilt sowohl für den Verbindungstest als auch für den tatsächlichen Transkriptions-Call zur Laufzeit.
- **`test_remote_whisper` hat CF-Header-Parameter:** Der Test-Button schickt die aktuell im UI eingetragenen CF-Credentials mit — ein leeres Secret-Feld bei gesetzter Client-ID fällt automatisch auf den Keyring-Wert zurück, damit der Test nach einem Neustart ohne erneute Secret-Eingabe klappt.

## v0.1.8 — 2026-04-22 — GPU-Server-Verbindungstest

- **„Verbindung testen"-Button im Allgemein-Tab** unter dem Feld „GPU-Server-Adresse". Neuer Tauri-Command `test_remote_whisper` probed `GET {url}/v1/models` mit 3 s Timeout und meldet Treffer inkl. Modell-Liste oder einen spezifischen Fehler (Timeout / Verbindung abgelehnt / HTTP-Status / ungültiges JSON). So ist vor dem ersten Diktat sichtbar, ob der OpenAI-kompatible Whisper-Server tatsächlich erreichbar ist — gerade praktisch beim Betrieb von verschiedenen Rechnern aus, wo der LAN-Hostname nicht überall auflöst.

## v0.1.7 — 2026-04-22 — Aufnahme-Indikator-Overlay

- **Aufnahme-Indikator-Overlay:** Kleines, transparentes Pill-Overlay am unteren Rand des Primary Monitors (320×56, 40 px Bottom-Margin, `alwaysOnTop`, `skipTaskbar`, `focus:false`) wird während jeder laufenden Aufnahme eingeblendet. Zeigt einen pulsierenden roten REC-Dot plus eine symmetrische Oszilloskop-Waveform in Neon-Grün, die live auf den Mikrofon-RMS reagiert (sqrt-Perceptual-Curve, so dass normale Sprache bei ~0.1 RMS sichtbar ausschlägt ohne bei Peaks anzuschlagen). Das Overlay existierte als Window-Shell + HTML-Stub bereits seit Phase 1, war aber nie verdrahtet.
  - **State-Observer im Core** (`dictatr-core::orchestrator`): Neues optionales `state_observer: Option<Arc<dyn Fn(AppState) + Send + Sync>>`-Feld. Neue interne Helper `transition()` und `set_state()` feuern den Callback nach jeder State-Mutation, **außerhalb** des State-Locks, um Reentranz-Deadlocks auszuschließen. Core bleibt Tauri-frei (reiner `Fn`-Trait-Objekt-Callback).
  - **Observer-Closure in `main.rs`**: Arc'd Closure reagiert auf `AppState::Recording` → `overlay::show` / sonst → `overlay::hide`, jeweils via `run_on_main_thread`, damit Window-Calls immer auf dem Tauri-Main-Thread landen.
  - **Positionierung**: `overlay::show` berechnet bei jedem Aufruf die Primary-Monitor-Bottom-Center-Position frisch (inkl. Scale-Factor für HiDPI-Displays); Monitor-Hotplug und DPI-Wechsel werden ohne App-Restart korrekt bedient.
  - **Level-Transport via Polling, nicht Events:** Der erste Anlauf über `AppHandle::emit("audio://level", rms)` + Event-Listener im Webview war tot — exakt die gleiche Falle wie beim Mic-Level-Meter (siehe Einträge von Phase 1, „Der Tauri v2 Event-Bus kommt in dieser Konstellation nicht zuverlässig im Webview an"). Umstellung auf das bereits bewährte Muster: `overlay.html` pollt alle 30 ms `invoke("get_audio_level")` via `window.__TAURI_INTERNALS__.invoke`. Kein Multi-Entry-Bundling für das Overlay nötig; kein `withGlobalTauri`-Toggle nötig.
- **Profile-Test-Fixtures repariert:** Die in v0.1.6 ergänzten Felder `clipboard_only` / `keep_on_clipboard` fehlten in zwei Struct-Literals in `profile.rs`' Unit-Tests, wodurch `cargo test -p dictatr-core` seit dem Release nicht mehr durchkompilierte. Tests wieder grün (30/32; der wiremock-basierte `transcribes_against_mock_server` bleibt flaky — nicht Teil dieses Patchs).

## v0.1.6 — 2026-04-16 — Autostart + Clipboard-Modi

- **Autostart funktioniert jetzt tatsächlich:** `tauri-plugin-autostart` integriert — das Häkchen „Automatisch starten" registriert die App jetzt im Windows-Autostart (Registry `HKCU\...\Run`) bzw. macOS LaunchAgent. Vorher wurde nur ein Flag in der Config gespeichert, ohne OS-Effekt.
- **Zwischenablage-Modi pro Profil:**
  - `clipboard_only` (Checkbox „Nur in Zwischenablage (kein Auto-Einfügen)") — Text landet ausschließlich in der Zwischenablage, kein `Ctrl+V`-Chord wird synthetisiert. Fix für Remote-Desktop-Fenster (RDP-Client fängt Tastatur-Events vor lokalem Hook ab) und elevierte Zielprozesse (Windows-UIPI blockt Low-Integrity-Eingaben an High-Integrity-Fenster).
  - `keep_on_clipboard` (Checkbox „Text ins Clipboard kopieren") — lässt den transkribierten Text nach dem Auto-Paste auch in der Zwischenablage liegen, statt den vorher gespeicherten Clipboard-Inhalt zu restaurieren.
- Beide Schalter sind pro Profil konfigurierbar; Default ist das alte Verhalten (Auto-Paste + Restore). Tooltips direkt in der UI erklären den Zweck.

## v0.1.5 — 2026-04-15 — Hotfix: Hotkeys lösten in 0.1.4 nicht aus

In v0.1.4 wurde der `GlobalHotKeyManager` auf einen eigenen Owner-Thread verschoben — das war falsch: dessen HWND empfängt `WM_HOTKEY` nur auf einem Thread mit laufendem Win32-Message-Pump. Der Owner blockierte stattdessen auf `mpsc::recv()`, also kamen keine Events an. Hotkeys wie `Ctrl+F7` oder `Ctrl+Alt+F8` waren registriert, haben aber nie gefeuert; nur LaunchMail & Co. funktionierten weiter, weil der LL-Hook einen eigenen Message-Pump-Thread unterhält.

**Fix:** Manager wieder auf Tauri-Main-Thread pinnen (der pumpt Messages), Registry über `thread_local!` + `run_on_main_thread`-Reload. Live-Reload bleibt erhalten.

## v0.1.4 — 2026-04-15 — Hotkey-Live-Reload + Tray-Neustart

- **Hotkey-Änderungen greifen ohne App-Neustart:** `save_config` schickt einen Reload-Command an einen dedizierten Owner-Thread, der den `!Send` `GlobalHotKeyManager` hält und die Registry + shared ID-Map + LL-Hook-Mapping atomar neu aufbaut. Die bisherige Mechanik (Registry via `Box::leak` einfrieren) war tot — `save_config` schrieb nur die Datei, Hotkeys blieben auf dem Stand vom App-Start.
- **`HotkeyRegistry::clear()` entfernt jetzt tatsächlich** die registrierten HotKeys. Vorher: `unregister_all(&[])` mit leerer Slice war ein no-op, was auch die bisherige Reload-Intention unterwandert hätte.
- **Tray-Menü um „Neustart" erweitert** — nutzt `AppHandle::restart()`. Deckt die Fälle ab, die kein Live-Reload haben (Backend-Wechsel, LLM-Provider, API-Key-Updates), weil der Orchestrator nach wie vor einen Profile-Snapshot hält.

## v0.1.3 — 2026-04-15 — Updater funktionsfähig

- **ACL-Capabilities ergänzt** (`src-tauri/capabilities/default.json`): Ohne diese Datei blockt Tauri 2 alle Plugin-Commands aus dem Webview. Fixt `Command plugin:updater|check not allowed by ACL` beim Klick auf „Nach Updates suchen" und sorgt nebenbei dafür, dass `getVersion()` in der Allgemein-Seite die App-Version liefert (dazu musste auch `core:default` gegrantet werden).
- **`bundle.createUpdaterArtifacts: true`** in `tauri.conf.json` aktiviert: Ohne dieses Opt-In markiert `tauri-cli` die `.msi.zip`/`.app.tar.gz`-Updater-Bundles nicht als solche, `tauri-action` findet die `.sig`-Files nicht und überspringt den `latest.json`-Upload mit `Signature not found for the updater JSON. Skipping upload...`. Genau das war in v0.1.2 passiert — Release hatte nur Binaries, keine Signaturen, keine `latest.json` → Updater konnte nichts finden.
- **Release-Workflow-Matrix** (aus `feat/macos-port`): `.github/workflows/release.yml` baut jetzt parallel `windows-latest` + `macos-latest (aarch64-apple-darwin)`, erzeugt MSI + DMG + `.app.tar.gz` + jeweilige `.sig`-Files; `tauri-action` mergt die `latest.json`-Plattform-Einträge automatisch.

**v0.1.2 bleibt als unvollständiges Release liegen** (Binaries ohne Signatur/`latest.json`, Updater-untauglich). Erster funktionierender Auto-Updater-Sprung ist von installiertem v0.1.3 auf alles Spätere; wer noch auf v0.1.1/v0.1.2 sitzt, muss die v0.1.3-MSI einmal manuell installieren.

## Phase 3 — macOS-Port Follow-ups (in Arbeit)

### TCC-Permissions auf macOS 26 beta

- **Root-Cause identifiziert:** Tauri's Bundler aktiviert per Default Hardened Runtime (`--options runtime`). Kombination ad-hoc-Signatur + Hardened Runtime auf macOS 26 blockt TCC-Permission-Dialoge für Mikrofon und Bedienungshilfen **stumm** — `AVCaptureDevice.requestAccess` resolvet als „denied", ohne dass der User den Dialog je sieht. Dictatr taucht in System Settings nie auf, Mic-Test zeigt flachen Pegel-Balken ohne Fehler.
- **Persistenter Fix:** `tools/macos-resign.sh` re-signiert das Bundle ad-hoc ohne Runtime-Flag. Pflicht-Schritt nach jedem `bun run tauri build` auf macOS 26 beta. Mit Developer-ID-Signatur + Notarization entfällt das.
- **Diagnose-Rezept:** `codesign -dv <Dictatr.app> | grep flags` — `0x10002(adhoc,runtime)` heißt kaputt, `0x2(adhoc)` heißt OK.
- `docs/BUILD-MACOS.md` um den Pflicht-Schritt ergänzt.

### Mic-Preview UX

- `start_mic_preview` prüft vor cpal-Start den AVCaptureDevice-Auth-Status (`dictatr_core::inject::microphone_auth_status`) und liefert bei Denied/Restricted einen klaren Fehlertext in die UI statt stumm Null-Buffer durchzureichen. Bei NotDetermined triggert der Command `prompt_microphone_if_needed()`, damit der System-Dialog auch über den Button erzwungen werden kann.

### Whisper-Halluzinations-Fixes (lokales Backend)

Das `ggml-small`-Modell halluzinierte bei Stille/leiser Aufnahme Tokens wie `[Musik]`, `[Zwischenruf]`, `[Applaus]` und geriet in Repeat-Loops (`[Zwischenruf] [Zwischenruf] [Zwischenruf] ...`). `LocalWhisperBackend::transcribe` setzt jetzt:

- `suppress_non_speech_tokens(true)` — unterdrückt Non-Speech-Tags direkt im Sampler.
- `suppress_blank(true)` — keine leeren Segmente.
- `no_context(true)` — verhindert, dass sich Halluzinationen über Segmentgrenzen ziehen (der eigentliche Repeat-Loop-Trigger).
- `temperature(0.0)` — deterministisches Greedy-Sampling.
- `no_speech_thold(0.6)` — strengeres Silence-Gating.
- `temperature_inc(0.2)` + `entropy_thold(3.0)` + `logprob_thold(-0.5)` — aktiviert whisper.cpp's Fallback-Mechanismus: Segmente mit niedriger Token-Entropie (= Repeat-Loop wie „Ja, ich habe das. Ja, ich habe das…") oder niedriger Confidence werden mit steigender Temperature (+0.2 bis 1.0) neu dekodiert. Ohne `temperature_inc > 0` gibt es keinen Retry, der Greedy-Loop bleibt als Endergebnis stehen.
- `SamplingStrategy::BeamSearch { beam_size: 5 }` statt `Greedy` — fundamental robuster gegen Repeat-Loops, besonders bei Utterances unter 2 Sekunden, wo das `small`-Modell auf CPU sonst gerne in Wiederholungen kippt (z. B. „Die Bekleidung wird mit einem Kohlenthalter…" × 7).
- Post-Filter `collapse_repetitions()` als Safety-Net: erkennt eine Phrase von ≥ 4 Wörtern, die ≥ 3× direkt hintereinander steht, und reduziert sie auf ein Vorkommen. Fängt die Fälle ab, bei denen whisper.cpp's Fallback trotz aller Parameter nicht greift. Unit-getestet.
- **BeamSearch in whisper-rs 0.12 ist als „WIP" markiert und produziert unabhängige Halluzinationen** (z. B. „Das ist der erste Teil der Strecke." für beliebiges Input). Greedy-Sampler bleibt die einzig brauchbare Strategie in dieser Version.
- Post-Filter `strip_trailing_hallucinations()`: Whisper ist stark auf YouTube-Audio trainiert und hängt gerne Schlussformeln an Diktate an („Danke fürs Zuschauen", „Untertitel im Auftrag des ZDF", „Abonniert den Kanal", „Bis zum nächsten Video" usw., DE + EN). Wir strippen eine kuratierte Liste von bekannten Trailing-Floskeln, inklusive Lowercase-Match und Trim von umgebender Interpunktion. Unicode-sicherer zeichenweiser Cut für Fälle wie „ß" ≠ „ss".

## Phase 2 — Auto-Updater (in Arbeit)

- `tauri-plugin-updater` + `tauri-plugin-process` eingebunden (Rust + npm).
- `tauri.conf.json`: `plugins.updater.endpoints` → `https://github.com/DSS-AI/Dictatr/releases/latest/download/latest.json`; `bundle.publisher = "DSS-Siegmund"`. `pubkey` wird beim Setup auf dem Windows-Host via `bunx @tauri-apps/cli signer generate` gesetzt.
- Neuer `general.check_updates`-Bool (Default `true`): Silent-Check beim App-Start; deaktivierbar im Allgemein-Tab.
- `UpdateBanner`-Komponente im Settings-Fenster mit Download-Progress und Relaunch nach Install.
- Allgemein-Tab zeigt aktuelle Version und „Nach Updates suchen"-Button mit Inline-Statusmeldung.
- GitHub-Actions-Workflow `.github/workflows/release.yml`: Tag-Push `v*` → baut signiert auf `windows-latest` (LLVM 18, Bun, Cargo-Cache), erstellt Release, lädt MSI + `.msi.sig` + `latest.json` via `tauri-apps/tauri-action@v0` (`includeUpdaterJson: true`).
- Release-Prozess in `docs/RELEASE.md` dokumentiert — Weg A (CI) als Standard, Weg B (manueller Build) als Fallback.

## Phase 1 MVP — Session vom 14.04.2026

End-to-End-Pipeline zum Laufen gebracht (Hotkey → Recording → Transkription → Text-Injection), plus mehrere Folge-Features. Alle Commits auf `feat/phase1-mvp`.

### Build / Tauri v2 Kompatibilität

- **whisper-rs-sys**: benötigt LLVM 18 + `libclang.dll` zum Bindgen → siehe `BUILD-WINDOWS.md`. Version 19 funktioniert nicht mit dem gepinnten `bindgen`.
- **tauri features**: `image-png` und `image-ico` müssen explizit aktiviert werden, sonst fehlt `tauri::image::Image::from_path` (Feature-gated).
- **Tray `Image::from_path`**: funktioniert erst nach Aktivierung obiger Features.
- **`use tauri::Emitter`**: Für `AppHandle::emit` nötig (in v2 traitbasiert).
- **GlobalHotKeyManager ist `!Send`** auf Windows (hält ein HWND). Registry bleibt via `Box::leak` auf dem Main-Thread; der Pump-Thread bekommt nur eine Kopie der ID-Map (`GlobalHotKeyEvent::receiver()` ist ein process-globaler Channel).
- **Tray-Fenster hochholen**: `show` + `unminimize` + `set_always_on_top(true) / false`-Toggle + `set_focus` — Windows stiehlt sonst den Fokus.
- **Fenster-Close**: `prevent_close` + `hide`, sonst wird das Hauptfenster zerstört und der Tray kann es nicht mehr öffnen.

### Persistenz / Config

- **keyring v3 ohne Feature-Flag = Mock-Store**: API-Keys gingen nach jedem Prozess-Restart verloren. Fix: `keyring = { version = "3", features = ["windows-native", "apple-native", "linux-native-sync-persistent", "sync-secret-service"] }`.

### Icons

- Alle Icons waren 0-Byte-Platzhalter → `tauri::generate_context!` Panic. Fix: Icon-Suite via `bun run tauri icon <source.png>` generieren. Tray-Icons (tray-*.png) werden separat erzeugt (System.Drawing-Script unter `beispiele/` oder manuell).

### Audio

- **Mic-Level-Meter**: Der Tauri v2 Event-Bus (`AppHandle::emit` / `emit_to`) kommt in dieser Konstellation nicht zuverlässig im Webview an. Umstellung auf Polling: `get_audio_level`-Command, Frontend polled alle 60 ms.
- **AudioController als `Arc<AudioController>`** im Tauri-State + Orchestrator, damit der Audio-Tab einen Live-Preview-Stream starten kann (unabhängig vom Hotkey-Pfad).
- **Sound-Cues** beim Record-Start/-Stop: `rodio` mit Two-Tone-Chirps (800→1200 Hz rising, 1200→800 Hz falling), gespawned auf eigenem Thread → keine Hotkey-Latenz. Toggle über `general.sounds`.

### Hotkeys

- **Multimedia- und Systemtasten** (LaunchMail, VolumeUp, BrowserHome, Media-Keys, Sleep, LaunchApp1/2): `global-hotkey` kann diese nicht abfangen. Stattdessen **WH_KEYBOARD_LL**-Hook auf einem Dedicated-Thread mit Win32-Message-Loop (via `windows-sys`). Hook-Proc retourniert `1` → unterdrückt die OS-Standardaktion (Outlook öffnet nicht mehr). Windows-only — stub für non-Windows.
- `HotkeyRecorder`: Dropdown für Systemtasten, da Browser/Webview diese Tasten nicht als `KeyboardEvent` sieht.

### Transkription

- **Remote-Whisper** spricht jetzt OpenAI-kompatibles `/v1/audio/transcriptions` (Multipart mit `file`, `model=whisper-1`, optionales `language`, `prompt` aus Vocabulary). Availability-Probe: `/v1/models`. Default-URL konfigurierbar in Allgemein (`general.remote_whisper_url`).
- **LLM-Provider als Transkriptions-Backend** (`TranscriptionBackendId::LlmTranscription`): sendet das WAV als base64 `input_audio`-Content-Part in eine Chat-Completion. Funktioniert mit Gemini 2.5 Flash/Pro (OpenRouter) und gpt-4o-audio-preview. Provider + Modell pro Profil wählbar.
- **Lokales whisper.cpp**: `first_installed_model()` wählt automatisch das größte installierte `ggml-*.bin`. `local_backend` ist jetzt `Option<Arc<dyn TranscriptionBackend>>`; Orchestrator gibt klaren Fehler aus, wenn Profil „lokal" wählt und kein Modell da ist.
- **Modell-Manager-Tab**: Download tiny/base/small/medium/large-v3 von Huggingface mit atomarem `.part → rename`, Progress-Polling (weil Event-Bus unzuverlässig), Delete-Option.

### Text-Injection

- `enigo.text()` verschluckt das erste und letzte Zeichen wenn die Ziel-App noch nicht akzeptiert. Fix: **Clipboard + Ctrl+V**. Alter Clipboard-Inhalt wird nach 250 ms wiederhergestellt.

### Vocabulary

- Live-editierbar: `Arc<Mutex<Vec<String>>>` im Orchestrator, Snapshot vor `.await` (parking_lot-MutexGuard ist `!Send`), Tauri-Commands `get_vocabulary`/`save_vocabulary`. Änderungen greifen ohne App-Neustart.

### UI / Theming

- Violette Farbpalette nach Referenz-Screenshot: Feld-Rand `#7a28ee` → `#b47dff`, Buttons `#9d5fff` → `#b47dff`, Sidebar-Aktiv auch violett (statt gold).
- Kompakte Paddings/Margins — Settings-Fenster (960×820) scrollt nicht mehr.
- `OpenRouter` als dedizierter Provider-Typ mit Preset + Model-Format-Hint; Preset überschreibt `default_model` nur noch, wenn der User das Feld nicht bereits angepasst hat.
- Provider-Page: Überschrift „LLM-Anbieter (Post-Processing)", Kontext-Hinweis auf Profil-Checkbox.
- „API-Key testen"-Button: minimaler `complete(…)`-Ping mit Default-Modell, zeigt Antwort oder Serverfehler.

### UI-Feinschliff (Session-Ende 2026-04-14)

- Tooltip-System: neue `InfoTip`-Komponente (kleines „?"-Badge, ~13×13 px lila, custom Popup per Maus-Hover). Abschaltbar via `general.show_tooltips`-Checkbox im Allgemein-Tab. Eingesetzt an allen erklärungsbedürftigen Feldern in Profile / Providers / Allgemein.
- History-Tab: kleines Copy-Icon (SVG, zwei überlappende Rechtecke) pro Eintrag; wechselt nach Klick kurz auf ein ✓-Häkchen. Text-Zelle selbst ist ebenfalls klickbar. Tabelle nutzt jetzt `table-layout: fixed` mit festen Spaltenbreiten → kein horizontaler Scrollbar.
- Sidebar von 220 → 150 px (+ kompakteres Padding), dadurch mehr Platz für den Content.
- Settings-Fenster final auf 960×920, `fieldset`-Padding weiter reduziert — keine Scrollbars mehr bei voll expandiertem Profil.

### Plattform-Status

| Feature                       | Windows | macOS        |
|-------------------------------|---------|--------------|
| Normale Hotkey-Kombos         | ✓       | ✓            |
| Multimedia-/Systemtasten      | ✓       | ✗ (Stub)     |
| Remote-Whisper                | ✓       | ✓            |
| LLM-Transkription             | ✓       | ✓            |
| Lokales whisper.cpp           | ✓       | ✓            |
| Mic-Level-Meter               | ✓       | ✓            |
| Text-Injection (Clipboard)    | ✓       | ✓            |
| Keyring                       | ✓ (Credential Manager) | ✓ (Keychain) |
| Tray + Close→Hide             | ✓       | ✓            |

Für macOS-Build siehe `BUILD-MACOS.md`.
