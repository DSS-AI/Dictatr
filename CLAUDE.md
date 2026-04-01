# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Projekt-Template-Starterkit für neue Claude Code Projekte. Enthält vorkonfigurierte Commands, Context-Dateien, Skills und Plugin-Integrationen. Dient als Ausgangspunkt für jedes neue Projekt. Bei Projektstart Abschnitte anpassen und Platzhalter ersetzen.

---

## Tech Stack

| Technology | Purpose |
|------------|---------|
| Bun | JavaScript-Runtime, benötigt für Claude Code Plugins (z.B. Telegram) |
| Claude Code Plugins | Erweiterungen für Claude Code (Telegram, Playwright, etc.) |
| Python >= 3.11 + uv | Runtime und Paketmanager für browser-use |
| browser-use | KI-gesteuerte Browser-Automatisierung via Chrome DevTools Protocol (Standard-Tool) |

---

## Commands

```bash
# Bun installieren (falls nicht vorhanden)
curl -fsSL https://bun.sh/install | bash

# Python + uv installieren (falls nicht vorhanden)
curl -LsSf https://astral.sh/uv/install.sh | sh

# browser-use installieren
uv add browser-use && uv sync

# Chromium für browser-use installieren
uvx browser-use install

# Installation prüfen
browser-use doctor

# Telegram Plugin installieren
/plugin install telegram@claude-plugins-official

# Telegram Plugin konfigurieren (Bot-Token via /telegram:configure)
# Zugriff verwalten via /telegram:access
```

---

## Project Structure

```
AA_ProjektTemplateStart/
├── .claude/
│   ├── commands/       # Slash-Commands (prime, shutdown, create-plan, implement, etc.)
│   ├── context/        # Kontext-Dateien (business, personal, strategy, data)
│   └── skills/         # Skills (browser-use, agent-browser, e2e-test, frontend-design, skill-creator)
├── outputs/            # Generierte Ausgaben
├── plans/              # Implementierungspläne
├── reference/          # Referenzmaterial
├── scripts/            # Automatisierungsskripte
├── pyproject.toml      # Python-Dependencies (browser-use)
└── CLAUDE.md           # Projekt-Regeln und -Konfiguration
```

---

## Architecture

Dieses Projekt ist ein Template — es hat keine eigene Architektur. Bei Projektstart diesen Abschnitt mit der Architektur des neuen Projekts ersetzen.

---

## Code Patterns

Bei Projektstart mit den Patterns des neuen Projekts ersetzen.

---

## Testing

Bei Projektstart mit den Test-Commands des neuen Projekts ersetzen.

---

## Key Files

| File | Purpose |
|------|---------|
| `CLAUDE.md` | Projekt-Regeln, Tech Stack, Plugin-Doku |
| `pyproject.toml` | Python-Dependencies (browser-use) |
| `.gitignore` | Ignoriert Synology @eaDir, OS-Dateien, Python-Cache, .venv, .env |
| `.claude/commands/prime.md` | Session-Start: Workspace verstehen inkl. Plugin-Status |
| `.claude/commands/shutdown.md` | Session-Ende: Aufräumen, Committen, Zusammenfassung |
| `.claude/context/` | Kontext-Dateien (Business, Personal, Strategie, Daten) |
| `.claude/skills/browser-use/` | browser-use Skill: CLI-Referenz für Browser-Automatisierung |

---

## Verfügbare Slash-Commands

| Command | Zweck |
|---------|-------|
| `/prime` | Session-Start: Workspace verstehen |
| `/shutdown` | Session-Ende: Aufräumen, Committen |
| `/create-plan` | Implementierungsplan erstellen |
| `/implement` | Plan implementieren |
| `/execute` | Plan ausführen |
| `/plan-feature` | Feature-Plan mit Codebase-Analyse |
| `/create-prd` | Product Requirements Document erstellen |
| `/commit` | Atomare Commits mit Tags |
| `/init-project` | Projekt lokal initialisieren |
| `/create-rules` | Globale Regeln aus Codebase erstellen |
| `/review` | Systematisches Code-Review |
| `/debug` | Systematisches Debugging |
| `/refactor` | Geführtes Refactoring |

---

## Verfügbare Skills

| Skill | Zweck |
|-------|-------|
| `browser-use` | KI-gesteuerte Browser-Automatisierung via CLI (Standard-Tool) |
| `agent-browser` | Browser-Interaktionen für Tests und Datenextraktion |
| `e2e-test` | End-to-End-Tests |
| `frontend-design` | Frontend-Design und UI-Komponenten |
| `skill-creator` | Neue Skills erstellen und verwalten |

---

## Plugins & Abhängigkeiten

### Voraussetzungen

| Abhängigkeit | Installation | Zweck |
|---|---|---|
| Bun | `curl -fsSL https://bun.sh/install \| bash` | Runtime für Claude Code Plugins |
| Python >= 3.11 | System-Python oder pyenv | Runtime für browser-use |
| uv | `curl -LsSf https://astral.sh/uv/install.sh \| sh` | Python-Paketmanager |

### Installierte Plugins

| Plugin | Installationsbefehl | Zweck |
|---|---|---|
| Telegram | `/plugin install telegram@claude-plugins-official` | Telegram-Bot-Integration |
| Playwright | (vorinstalliert) | E2E-Tests |

### browser-use (Standard-Tool für Browser-Automatisierung)

browser-use ist das Standard-Tool für KI-gesteuerte Browser-Automatisierung in allen Projekten. Es nutzt das Chrome DevTools Protocol (CDP) und ermöglicht einem LLM-Agenten, einen echten Browser zu steuern. Vollständige CLI-Referenz im Skill: `.claude/skills/browser-use/SKILL.md`

**Installation:**
```bash
uv add browser-use && uv sync
uvx browser-use install   # Chromium installieren
browser-use doctor        # Prüfen
```

**CLI-Kurzreferenz (Aliase: `browser-use`, `bu`, `browseruse`, `browser`):**
```bash
browser-use open https://example.com    # URL öffnen
browser-use state                       # Klickbare Elemente anzeigen
browser-use click 5                     # Element per Index klicken
browser-use type "Hello"                # Text eingeben
browser-use screenshot page.png         # Screenshot
browser-use close                       # Browser schließen
```

**Konfiguration:** LLM-API-Keys in `.env` setzen (z.B. `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`).

### Telegram Plugin Setup

1. **Bun** muss installiert sein (siehe oben)
2. Plugin installieren: `/plugin install telegram@claude-plugins-official`
3. Bot-Token konfigurieren: `/telegram:configure` (Telegram Bot-Token von @BotFather benötigt)
4. Zugriff verwalten: `/telegram:access`

---

## Notes

- Dies ist ein **Template-Starterkit** — Platzhalter-Abschnitte (Architecture, Code Patterns, Testing) bei Projektstart ersetzen
- **browser-use** ist das Standard-Tool für Browser-Automatisierung (ersetzt direkte Playwright-Nutzung für KI-gesteuerte Aufgaben). Für einfache E2E-Tests bleibt Playwright verfügbar
- browser-use benötigt Python >= 3.11 und Chromium — bei Projektstart `uvx browser-use install` ausführen
- Bun ist Voraussetzung für Plugin-Installationen — vor dem ersten Plugin-Install `bun --version` prüfen
- Telegram-Plugin benötigt einen Bot-Token von Telegram @BotFather
