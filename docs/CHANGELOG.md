# Dictatr — Änderungs-Log

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
