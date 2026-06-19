# Handoff — Prompt-Manager + Hotkey-Fix (E2E abgeschlossen 2026-06-18)

**Branch:** `feat/phase1-mvp`. **Noch nicht committet** (kein Auto-Push — auf Ansage committen).

## Status: ✅ funktioniert end-to-end

Prompt-Manager-Quick-Pick per eigenem Hotkey + Verwaltung im Tab „Textblöcke" sind fertig
und im laufenden App-Build verifiziert. Der zähe „Picker öffnet erst beim zweiten Druck"-Bug
ist gelöst. Voller Umfang + Root-Cause-Analyse: `docs/CHANGELOG.md` → `Unreleased`.

## Verifiziert (E2E im `bun run tauri dev`-Build, Windows)
- **F6-Picker** öffnet **jedes** Mal beim ersten Druck (vorher: ab dem 3. Öffnen Doppel-Druck nötig).
- **F8** Diktat push-to-talk sauber; **push-to-talk ↔ toggle** greift live beim Speichern (kein Neustart).
- **F6/F8** werden global geschluckt (auch im Terminal), kein WebView-Dialog (F7 vermieden).
- `cargo check -p dictatr` sauber; `cargo test -p dictatr-core` 36/37 (nur flaky `transcribes_against_mock_server`).

## Die zwei Kern-Fixes (Details im CHANGELOG)

1. **Picker-Zwei-Druck-Bug → eigentliche Ursache war der Low-Level-Hook, nicht das Fenster.**
   Die Auto-Repeat-Dedup (`hotkey_ll.rs`) hängte sich auf, wenn beim Öffnen (Fokuswechsel)
   das `WM_KEYUP` der F-Taste verloren ging → `pressed[vk]` blieb `true` → nächster echter
   Druck als Auto-Repeat unterdrückt. **Fix:** zeitbasierte Auto-Repeat-Erkennung
   (`REPEAT_GAP = 600 ms`, `pressed` speichert `Instant` statt `bool`) → selbstheilend.
   ⚠️ **Lehre:** „Hotkey reagiert nicht" *zuerst* am Event-Pfad/Hook instrumentieren.
   Die gesamte Fokus-/Fenster-Trickserei (AttachThreadInput-Foreground-Grab, Grace-Period,
   Fokus-Reclaim, Fenster-Neuerzeugung) war eine Sackgasse und wurde wieder entfernt.

2. **Profil-Modus live:** `Orchestrator.profiles` jetzt `Arc<Mutex<…>>`, von `main.rs` über
   `ORCH_PROFILES` geteilt, von `reload_hotkeys` bei jedem `save_config` aktualisiert.

## Geänderte Dateien (Überblick)
- Rust: `core/src/config/{mod,text_block}.rs`, `core/src/hotkey.rs`, `core/src/hotkey_ll.rs`
  (zeitbasierte Dedup), `core/src/orchestrator.rs` (Profile hinter Lock), `src/{main,commands,prompt_window}.rs`.
- TS: `src/{main.tsx,PromptPicker.tsx,App.tsx,ipc.ts,types.ts,index.css}`, `src/pages/{TextBlocks,General}.tsx`.
- `src-tauri/Cargo.toml`: unverändert (das temporär für den Foreground-Trick ergänzte
  `windows-sys` wurde mit dem Trick wieder entfernt).

## Offene Punkte / Hinweise
- **F7 meiden** (WebView2 Caret-Browsing). Empfehlung: F6/F8/F9. Browser-reservierte F-Tasten
  (F1/F3/F5/F7/F11/F12) generell vermeiden, da die UI selbst WebView2 ist.
- **F6 öffnet nur** (kein Toggle übers Schließen); Schließen per Esc/Block-Klick. Toggle bewusst
  nicht aktiv, weil `is_visible` beim wiederverwendeten Fenster unzuverlässig war.
- Neu angelegter **LLM-Provider** für ein Profil braucht weiter App-Neustart (Provider-Map ist Snapshot).
- Wenn Tests grün & zufrieden: committen (auf explizite Ansage). CLAUDE.md-Phasenstand ggf. um den
  Prompt-Manager ergänzen.
