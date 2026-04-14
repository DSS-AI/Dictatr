# Dictatr — Änderungs-Log

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
