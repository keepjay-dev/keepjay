# 06 — Command-Line Tool

> **Status:** Draft · **Version:** 0.2 · **Last updated:** 2026-11-19
> **Related:** [03-customization-model](03-customization-model.md) · [04-workflows](04-workflows.md) · [05-dashboards-and-widgets](05-dashboards-and-widgets.md) · [08-architecture](08-architecture.md)

The CLI is a first-class surface, not an afterthought. It links the same Rust core as the desktop app, reads and writes the same local database, and covers the full core study loop. The binary is `addisco`, with the optional short alias `addi` (CLI-018). The name is provisional and may change (D-019).

## 1. Principles

1. **Same core, same data.** No separate logic; no daemon required (D-010).
2. **Human by default, machine on request.** Pretty tables and a TUI on a TTY. `--json` everywhere for scripts.
3. **Fast.** Simple read commands start and finish in under 100 ms (NFR-PERF-001).
4. **Safe.** Destructive actions confirm (or need `--yes`). The CLI never prompts when not attached to a TTY.
5. **Composable.** Reads stdin, writes clean stdout, logs to stderr, and returns meaningful exit codes.
6. **Customisable within bounds.** Aliases, default command, output, colours and keymap live in `cli.toml` (§7). Study customisations (workflows, dashboards) are shared with the desktop.

## 2. Requirements (CLI)

| ID | Requirement | Pri |
|---|---|---|
| CLI-001 | A single `addisco` binary sharing the core and data directory with the desktop app. | P0 |
| CLI-002 | Works standalone: the desktop app doesn't need to be installed or running. | P0 |
| CLI-003 | Human-friendly output by default; `--json` on every read/list command, with a versioned schema (§6). | P0 |
| CLI-004 | Stable exit codes (§6.3). | P0 |
| CLI-005 | Interactive TUI for review, sessions and dashboard. `--plain` line mode for screen readers and dumb terminals. *Status (M0.4):* `addisco review` TUI and `--plain` implemented. `--plain` also accepts piped answers (an explicit flag counts as consent to read stdin). | P0 |
| CLI-006 | When stdin/stdout isn't a TTY: never prompt; fail with exit `2` if input is required. Destructive operations need confirmation or `--yes`. | P0 |
| CLI-007 | Items are addressable by full id, by **short id** (the last 8 characters, e.g. `k22k1m8e`; random, so unique in practice; ambiguity is an error listing the matches), or by unique name/path (`Biology::Cells`). | P0 |
| CLI-008 | `cli.toml` configuration with aliases and defaults; a base config is created on first run (§7). | P0 |
| CLI-009 | Shell completions for bash, zsh, fish and PowerShell (P0). Dynamic completion of deck, workflow and dashboard names (P1). | P0 / P1 |
| CLI-010 | `$EDITOR` integration for long text and documents (facts, notes, workflows, dashboards). | P0 |
| CLI-011 | Bulk input from stdin (CSV / NDJSON). | P0 |
| CLI-012 | `addisco doctor` diagnoses the data dir, DB integrity, schema version and desktop/CLI version skew. | P0 |
| CLI-013 | `addisco open <item>` deep-links into the desktop app. | P1 |
| CLI-014 | `--watch` for `status` and `dashboard`. | P1 |
| CLI-015 | Terminal image rendering (kitty / iTerm2 / sixel) for card images. Until then, images show as `[image: name]` with an open action. | P2 |
| CLI-018 | The binary uses the full product name. One official short alias (first 3–4 letters of the name) is offered at install and by `addisco alias install`. It's installed **only if no command of that name already exists on PATH**, and `addisco doctor` reports any later shadowing. See [research/naming.md §6](research/naming.md#6-cli-naming-conventions-applies-to-whichever-name-is-chosen). | P0 |
| CLI-016 | User-defined hooks on events (e.g. `session.completed` runs a shell command). See OQ-017. | P2 |

## 3. Global flags and environment

| Flag | Env var | Meaning |
|---|---|---|
| `-w, --workspace <name>` | `ADDISCO_WORKSPACE` | Workspace to act on (default: `cli.toml` → last used) |
| `--json` | — | Shorthand for `--format json` |
| `--format <table\|plain\|json\|ndjson>` | `ADDISCO_FORMAT` | Output format (`auto` = table on TTY, plain otherwise) |
| `--plain` | — | Line-oriented, no TUI, no box drawing |
| `-q, --quiet` / `-v, --verbose` | — | Less / more output (logs to stderr) |
| `--no-color` | `NO_COLOR` | Disable colour |
| `-y, --yes` | — | Assume yes for confirmations |
| `--data-dir <path>` | `ADDISCO_DATA_DIR` | Override the data directory |
| `--config <path>` | `ADDISCO_CONFIG` | Override `cli.toml` |

## 4. Command tree

```
addisco                                  # default command (cli.toml: default_command, base: "status")
addisco init [--preset <id>] [--workspace <name>] [--sample]
addisco status [--watch]                 # today: due, tasks, goals, active session

# Flashcards
addisco review [QUERY] [--limit N] [--minutes M] [--cram]   # TUI review
addisco learn [QUERY] [--limit N]                           # new cards only
addisco add card [FRONT] [BACK] [--deck D] [--type T] [--tag T…] [--field k=v…] [--edit]
addisco add card --from-csv FILE|-  [--map front=1,back=2]
addisco card   list|show|edit|rm|suspend|unsuspend|flag|move|reschedule  [QUERY|ID]
addisco deck   list|create|rename|move|rm|options [get|set|edit]
addisco type   list|show|export|import          # card types (edit = P1)

# Notes, planner, goals
addisco note   new|list|show|edit|rm|export  [--subject S] [--tag T]
addisco task   add|list|done|edit|rm  [--due DATE] [--priority P] [--subject S]
addisco exam   add|list|show|edit|rm
addisco goal   list|add|show|edit|rm|progress
addisco subject list|create|rename|move|archive
addisco tag    list|rename|merge|rm

# Sessions & workflows
addisco session start [WORKFLOW] [--param k=v…] [--detach]
addisco session status|pause|resume|skip|extend [DURATION]|stop
addisco session log DURATION [--subject S] [--at DATETIME] [--note TEXT]   # manual log
addisco session list|show [ID]
addisco timer [DURATION]                 # shortcut: runs builtin:free-focus
addisco workflow list|show|new|edit|validate|export|import|reset|rm|dry-run  [NAME|FILE]

# Dashboards & stats
addisco dashboard [NAME] [--watch]       # P1 text rendering
addisco dashboard list|export|import|reset|set-home
addisco stats [METRIC] [--scope QUERY] [--range R] [--by DIM]
addisco search QUERY [--type card|note|task|exam|resource]

# Customisation
addisco preset list|show|apply|reapply|export
addisco config get|set|unset|list|edit|path  [KEY] [VALUE] [--explain] [--scope app|workspace|device]
addisco workspace list|create|use|rename|archive|rm

# Data
addisco import anki|csv|archive FILE [options]
addisco export archive|csv|md FILE|DIR [QUERY]
addisco backup create|list|restore [ID]
addisco doctor [--fix]
addisco open [ITEM]                      # P1: deep link into desktop
addisco completions bash|zsh|fish|powershell
addisco sync status|now|pause            # Phase 2, Pro (SYNC-*)
addisco account login|status|logout      # Phase 2, device-code login (ACCT-008)
addisco ask [PROMPT] [--context QUERY]   # Phase 2, Pro: AI study chat (AI-006)
```

`--detach` on `session start` returns immediately and leaves the session running in the data store, so it can be controlled with `addisco session …` or from the desktop.

## 5. Examples

```console
$ addisco
Today · Uni (university)                                   Thu 19 Nov
  Cards     42 due · 10 new             Goal   35 / 90 min  ███░░░░░
  Tasks     2 due today · 1 overdue     Exam   Biochem Final in 23 days
  Session   none — `addisco session start daily-routine`

$ addisco add card "What produces ATP?" "Mitochondria" --deck Biology --tag cells
Added card k22k1m8e to Biology (Basic)

$ addisco session start pomodoro --param focus=40m
$ addisco stats reviews --range 7d --by day --json | jq '.data.series'
$ addisco card list 'deck:Biology lapses:>3' --json | jq -r '.data[].id' | xargs addisco card suspend
$ addisco workflow export daily-routine > ~/dotfiles/addisco/daily-routine.toml
$ addi r                      # short alias + built-in alias: same as `addisco review`
```

## 6. Output contracts

### 6.1 JSON envelope

Every `--json` response on **stdout**:

```json
{ "schema": "addisco.cli/1", "ok": true, "data": { }, "warnings": [] }
```

Errors with `--json` are also written to stdout (with a non-zero exit). Human-readable error text always goes to stderr.

```json
{ "schema": "addisco.cli/1", "ok": false,
  "error": { "code": "not_found", "message": "No deck named 'Bio'", "details": { "suggestions": ["Biology"] } } }
```

`--format ndjson` streams one object per line for large lists. JSON Schemas for every payload are generated from core types and published in `schemas/cli/` (D-006). Payload changes are additive within `addisco.cli/1`; breaking changes bump the version.

### 6.2 Interactive vs non-interactive

| Condition | Behaviour |
|---|---|
| TTY on stdin and stdout | TUI and prompts allowed |
| `--plain`, or `TERM=dumb` | Line mode: numbered prompts, no full-screen UI |
| Not a TTY | No prompts. Missing input → exit `2`. Destructive without `--yes` → exit `2`. |

### 6.3 Exit codes

| Code | Meaning |
|---|---|
| 0 | Success (including "nothing to do", e.g. no cards due) |
| 1 | Unexpected error |
| 2 | Usage error: bad flags/args, or input required in non-interactive mode |
| 3 | Not found |
| 4 | Validation failed (document, query, value) |
| 5 | Conflict: another active session, DB busy beyond timeout, revision mismatch |
| 6 | Version incompatibility: DB schema newer than this binary — update required |
| 130 | Interrupted (Ctrl-C) |

## 7. Configuration (`cli.toml`)

Device-scoped, never synced. Location: `<config_dir>/cli.toml` ([09 §8](09-data-model.md#8-on-disk-layout)). It's created with these **base values** on first run; `addisco config edit --cli` opens it.

```toml
# Addisco CLI configuration (base version)
default_workspace = ""          # empty = last used
default_command = "status"      # what bare `addisco` runs
output = "auto"                 # auto | table | plain | json
color = "auto"                  # auto | always | never
editor = ""                     # empty = $VISUAL / $EDITOR / platform default
date_format = "locale"          # locale | iso

[aliases]                       # user-extensible; expands before parsing
r  = "review"
l  = "learn"
s  = "session start"
p  = "session start pomodoro"
t  = "timer"
a  = "add card"

[review]
keymap = "default"              # default | vim
show_intervals = true
show_progress = true

[session]
notify = true                   # OS notifications while TUI is open
bell = true                     # terminal bell on step change
confirm_stop = true

[tui]
theme = "auto"                  # auto | dark | light | high-contrast
unicode = true                  # false = ASCII-only box drawing
```

Workspace-scoped and app-scoped settings (shared with the desktop) are managed with `addisco config get|set KEY` and are stored in the database, not in this file ([03 §5](03-customization-model.md#5-settings-layering)).

## 8. Interoperability with the desktop app

- **Shared data dir and DB.** SQLite in WAL mode with short transactions supports both processes at once ([08 §8](08-architecture.md#8-multi-process-access-desktop--cli)).
- **Live updates.** Changes made by the CLI appear in an open desktop window within ~1 s (DESK-007), and vice versa during a TUI session.
- **One active session.** It can be started in one surface and controlled from the other (SES-004).
- **Install paths.** The desktop installer offers to put `addisco` on PATH (DESK-014). Standalone install via Homebrew, winget, Scoop, `.deb`/`.rpm`, and `cargo install` (OQ-016).
- **Version skew.** Both binaries check the DB schema version on open. Older schema → migrate with a backup first. Newer schema → refuse writes and exit `6` with update instructions. `addisco doctor` reports skew.
- **Deep links** (P1): `addisco open card k22k1m8e` → `addisco://card/k22k1m8e`.

## 9. TUI screens

### Review

```
┌ Review · Biology::Cells ────────────────────────── 23 due · 4 new ┐
│                                                                   │
│   What organelle produces most of the cell's ATP?                 │
│                                                                   │
│ ───────────────────────────────────────────────────────────────── │
│   Mitochondria                                                    │
│                                                                   │
├───────────────────────────────────────────────────────────────────┤
│ [1] Again <10m   [2] Hard 2d   [3] Good 4d   [4] Easy 9d          │
│ space reveal · u undo · e edit · s suspend · f flag · q quit · ?  │
└───────────────────────────────────────────────────────────────────┘
```

Default keys: `space`/`enter` reveal, `1`–`4` grade, `q`/`Esc` quit (implemented in M0.4). Still to do: `u` undo, `e` edit in `$EDITOR`, `s` suspend, `b` bury, `f` flag. The Vim keymap adds `j/k`, `:` for commands, and `ZZ` to quit.

### Session runner

```
┌ Daily Routine ─────────────────────────────────── 00:41:12 elapsed ┐
│ ✓ Warm-up checklist                                                │
│ ✓ Reviews            38 cards · 92% correct                        │
│ ▶ Learn new          4 / 10                                        │
│ · Focus 25m          Top task: Problem set 4                       │
│ · Break 5m                                                         │
│ · Reflect                                                          │
├────────────────────────────────────────────────────────────────────┤
│ p pause · n skip step · + extend 5m · x end session · ? help       │
└────────────────────────────────────────────────────────────────────┘
```

`review`/`learn` steps embed the review screen. `focus`/`break` show a large countdown and progress bar.

## 10. Feature parity matrix (Phase 1)

✓ full · ◐ partial · — not available

| Capability | Desktop | CLI | CLI notes |
|---|---|---|---|
| Onboarding / presets | ✓ | ✓ | `init`, `preset apply` |
| Review & learn | ✓ | ✓ | TUI; images as placeholders until CLI-015 |
| Add / edit cards | ✓ | ✓ | Flags, `$EDITOR`, CSV stdin |
| Card browser + bulk ops | ✓ | ✓ | Query + bulk subcommands |
| Notes | ✓ | ◐ | `$EDITOR`, list, search, export; no live preview |
| Tasks, exams | ✓ | ✓ | List-based; calendar via `agenda` (P1) |
| Goals | ✓ | ✓ | |
| Run workflows / sessions | ✓ | ✓ | |
| Edit workflows | ✓ visual | ✓ | TOML in `$EDITOR` |
| View dashboards | ✓ | ◐ | Text-capable widgets (P1) |
| Edit dashboards | ✓ visual | ◐ | TOML in `$EDITOR` |
| Stats | ✓ charts | ✓ | Tables, sparklines, JSON |
| Settings | ✓ | ✓ | `config` |
| Themes | ✓ | ◐ | TUI colour scheme only |
| Notifications | ✓ | ◐ | While a TUI is open; terminal bell |
| Import / export / backup | ✓ | ✓ | |
