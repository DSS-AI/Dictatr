# CLAUDE.md Template

A flexible template for creating global rules. Adapt sections based on your project type.

---

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

<!-- What is this project? One paragraph description -->

Projekt-Template-Starterkit für neue Claude Code Projekte. Enthält vorkonfigurierte Commands, Context-Dateien, Skills und Plugin-Integrationen. Dient als Ausgangspunkt für jedes neue Projekt.

---

## Tech Stack

<!-- List technologies used. Add/remove rows as needed -->

| Technology | Purpose |
|------------|---------|
| Bun | JavaScript-Runtime, benötigt für Claude Code Plugins (z.B. Telegram) |
| Claude Code Plugins | Erweiterungen für Claude Code (Telegram, Playwright, etc.) |

---

## Commands

<!-- Common commands for this project. Adjust based on your package manager and setup -->

```bash
# Bun installieren (falls nicht vorhanden)
curl -fsSL https://bun.sh/install | bash

# Telegram Plugin installieren
/plugin install telegram@claude-plugins-official

# Telegram Plugin konfigurieren (Bot-Token via /telegram:configure)
# Zugriff verwalten via /telegram:access
```

---

## Project Structure

<!-- Describe your folder organization. This varies greatly by project type -->

```
AA_ProjektTemplateStart/
├── .claude/
│   ├── commands/    # Slash-Commands (prime, create-plan, implement, etc.)
│   ├── context/     # Kontext-Dateien (business, personal, strategy, data)
│   └── skills/      # Skills (agent-browser, e2e-test, frontend-design, skill-creator)
├── outputs/         # Generierte Ausgaben
├── plans/           # Implementierungspläne
├── reference/       # Referenzmaterial
├── scripts/         # Automatisierungsskripte
└── CLAUDE.md        # Projekt-Regeln und -Konfiguration
```

---

## Architecture

<!-- Describe how the code is organized. Examples:
- Layered (routes → services → data)
- Component-based (features as self-contained modules)
- MVC pattern
- Event-driven
- etc.
-->

{Describe the architectural approach and data flow}

---

## Code Patterns

<!-- Key patterns and conventions used in this codebase -->

### Naming Conventions
- {convention}

### File Organization
- {pattern}

### Error Handling
- {approach}

---

## Testing

<!-- How to test and what patterns to follow -->

- **Run tests**: `{test-command}`
- **Test location**: `{test-directory}`
- **Pattern**: {describe test approach}

---

## Validation

<!-- Commands to run before committing -->

```bash
{validation-commands}
```

---

## Key Files

<!-- Important files to know about -->

| File | Purpose |
|------|---------|
| `CLAUDE.md` | Projekt-Regeln, Tech Stack, Plugin-Doku |
| `.gitignore` | Ignoriert Synology @eaDir, OS-Dateien, Python-Cache |
| `.claude/commands/prime.md` | Session-Start: Workspace verstehen inkl. Plugin-Status |
| `.claude/commands/shutdown.md` | Session-Ende: Aufräumen, Committen, Zusammenfassung |
| `.claude/context/` | Kontext-Dateien (Business, Personal, Strategie, Daten) |

---

## On-Demand Context

<!-- Optional: Reference docs for deeper context -->

| Topic | File |
|-------|------|
| {topic} | `{path}` |

---

## Plugins & Abhängigkeiten

### Voraussetzungen

| Abhängigkeit | Installation | Zweck |
|---|---|---|
| Bun | `curl -fsSL https://bun.sh/install \| bash` | Runtime für Claude Code Plugins |

### Installierte Plugins

| Plugin | Installationsbefehl | Zweck |
|---|---|---|
| Telegram | `/plugin install telegram@claude-plugins-official` | Telegram-Bot-Integration für Benachrichtigungen und Interaktion |
| Playwright | (vorinstalliert) | Browser-Automatisierung und E2E-Tests |

### Telegram Plugin Setup

1. **Bun** muss installiert sein (siehe oben)
2. Plugin installieren: `/plugin install telegram@claude-plugins-official`
3. Bot-Token konfigurieren: `/telegram:configure` (Telegram Bot-Token von @BotFather benötigt)
4. Zugriff verwalten: `/telegram:access`

---

## Notes

<!-- Any special instructions, constraints, or gotchas -->

- Bun ist eine zwingende Voraussetzung für Plugin-Installationen — vor dem ersten Plugin-Install prüfen ob `bun --version` funktioniert
- Telegram-Plugin benötigt einen Bot-Token von Telegram @BotFather