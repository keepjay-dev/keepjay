# 06 — Command-Line Tool

> **Status:** Draft · **Version:** 0.3 · **Last updated:** 2026-11-22
> **Related:** [13-code-sources](13-code-sources.md) · [08-architecture](08-architecture.md) · [09-data-model](09-data-model.md) · decisions D-028, D-030, D-031

The CLI is the first surface and the reference shell (D-031). It links the Rust core directly and shares the local database with the VS Code extension (which bundles it) and the MCP server. It covers the full developer loop: capture, scan, review, and practise. The binary is `addisco`, with the optional short alias `addi` (CLI-018). The name is provisional and may change (D-019).

## 1. Principles

1. **Same core, same data.** No separate logic; no daemon required (D-010).
2. **Human by default, machine on request.** Pretty tables and a TUI on a TTY. `--json` everywhere for scripts.
3. **Fast.** Simple read commands start and finish in under 100 ms (NFR-PERF-001).
4. **Safe.** Destructive actions confirm (or need `--yes`). The CLI never prompts when not attached to a TTY.
5. **Composable.** Reads stdin, writes clean stdout, logs to stderr, and returns meaningful exit codes.
6. **Customisable within bounds.** Aliases, default command, output, colours, keymap, comment prefixes and hooks live in `cli.toml` (§7).

## 2. Requirements (CLI)

| ID | Requirement | Pri |
|---|---|---|
| CLI-001 | A single `addisco` binary sharing the core and data directory with the other shells (VS Code extension, MCP server, later mobile through sync). | P0 |
| CLI-002 | Works standalone: no other shell needs to be installed or running. | P0 |
| CLI-003 | Human-friendly output by default; `--json` on every read/list command, with a versioned schema (§6). | P0 |
| CLI-004 | Stable exit codes (§6.3). | P0 |
| CLI-005 | Interactive TUI for review (sessions and dashboard TUIs are frozen, D-028). `--plain` line mode for screen readers and dumb terminals. *Status (M0.4):* `addisco review` TUI and `--plain` implemented. `--plain` also accepts piped answers (an explicit flag counts as consent to read stdin). | P0 |
| CLI-006 | When stdin/stdout isn't a TTY: never prompt; fail with exit `2` if input is required. Destructive operations need confirmation or `--yes`. | P0 |
| CLI-007 | Items are addressable by full id, by **short id** (the last 8 characters, e.g. `k22k1m8e`; random, so unique in practice; ambiguity is an error listing the matches), or by unique name/path (`Biology::Cells`). | P0 |
| CLI-008 | `cli.toml` configuration with aliases and defaults; a base config is created on first run (§7). | P0 |
| CLI-009 | Shell completions for bash, zsh, fish and PowerShell (P0). Dynamic completion of deck, workflow and dashboard names (P1). | P0 / P1 |
| CLI-010 | `$EDITOR` integration for long text and documents (facts, notes, workflows, dashboards). | P0 |
| CLI-011 | Bulk input from stdin (CSV / NDJSON). | P0 |
| CLI-012 | `addisco doctor` diagnoses the data dir, DB integrity, schema version, version skew between shells, registered sources, and duplicate data dirs under Windows and WSL. | P0 |
| CLI-013 | `addisco open <item>` deep-links into the desktop app. *Superseded by SRC-007 (`open` jumps to the card's source in `$EDITOR`). Desktop deep links were dropped with the desktop app (D-034).* | — |
| CLI-014 | `--watch` for `status` and `dashboard`. | P1 |
| CLI-015 | Terminal image rendering (kitty / iTerm2 / sixel) for card images. Until then, images show as `[image: name]` with an open action. | P2 |
| CLI-018 | The binary uses the full product name. One official short alias (first 3–4 letters of the name) is offered at install and by `addisco alias install`. It's installed **only if no command of that name already exists on PATH**, and `addisco doctor` reports any later shadowing. See [research/naming.md §6](research/naming.md#6-cli-naming-conventions-applies-to-whichever-name-is-chosen). | P0 |
| CLI-016 | User-defined **hooks**: shell commands run on events (`review.completed`, `card.added`, `scan.completed`, `scan.stale_found`), configured in `cli.toml` `[hooks]`. The event's JSON is passed on stdin. Local, opt-in, CLI only (D-030, resolves OQ-017). | P0 |
| CLI-019 | **`addisco review --quick [-n N] [--time D]`**: review up to N due cards (default 5), or for D, then exit. Exits 0 silently when nothing is due. | P0 |
| CLI-020 | **`addisco while [--limit N] -- <command…>`**: run the command, review while it runs, and stop when it exits. Show the end of its output (captured to a temporary file) and return its exit code, so it composes in scripts (`addi while -- make && deploy`). Quitting the review leaves the command running in the foreground. | P0 |
| CLI-021 | **`addisco status --format '<template>'`** for shell prompts, tmux and starship (fields such as `{due}`, `{stale}`, `{target}`, `{target_days}`), within NFR-PERF-001. | P0 |

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
addisco init [--workspace <name>] [--sample]
addisco status [--format TEMPLATE] [--watch]   # today: due, stale, next target (CLI-021)

# Review
addisco review [QUERY] [--limit N] [--minutes M] [--cram]   # TUI review; QUERY = filtered review (SRS-010)
addisco review --quick [-n N] [--time D]                    # a few cards, then exit (CLI-019)
addisco while [--limit N] -- COMMAND…                       # review while COMMAND runs (CLI-020)
addisco learn [QUERY] [--limit N]                           # new cards only
addisco target set|list|rm NAME [--scope QUERY] [--date DATE] [--retention R]   # ready by a date (SRS-016)

# Sources and cards (13)
addisco source add PATH [--kind repo|folder|problems] [--deck D] [--tag T…] [--no-write-ids]
addisco source list|rm|hooks install|hooks uninstall [NAME]
addisco scan [SOURCE] [--quiet]                             # find @card comments and card files (SRC-002/003)
addisco stale [list|keep|retire] [ID]                       # cards whose code changed (SRC-006)
addisco edit CARD | addisco open CARD                       # $EDITOR at the card's file and line (SRC-007)
addisco add card [FRONT] [BACK] [--deck D] [--type T] [--tag T…] [--field k=v…] [--edit]
addisco card   list|show|edit|rm|suspend|unsuspend|flag|move|reschedule  [QUERY|ID]
addisco card   from-code PATH:SYMBOL --hide body|signature|return|params  # tree-sitter cloze (CODE-002)
addisco deck   list|create|rename|move|rm|options [get|set|edit]
addisco deck   subscribe URL [--name N] | pull [NAME] | diff [NAME]   # deck repositories (SRC-011)
addisco type   list|show|export|import          # card types (edit = P1)
addisco tag    list|rename|merge|rm

# Stats and search
addisco stats [METRIC] [--scope QUERY] [--range R] [--by DIM]
addisco stats heatmap [--svg FILE]               # P1 (STAT-008)
addisco search QUERY

# Configuration and integrations
addisco config get|set|unset|list|edit|path  [KEY] [VALUE]
addisco workspace list|create|use|rename|archive|rm
addisco mcp                                      # MCP server over stdio (AI-010)
addisco completions bash|zsh|fish|powershell
addisco alias install                            # the `addi` short alias (CLI-018)

# Data
addisco import anki|archive FILE [options]       # csv: P1 (IO-002)
addisco export archive|log|md FILE|DIR [QUERY]   # log = review history as NDJSON (D-029)
addisco backup create|list|restore [ID]
addisco doctor [--fix]

# Phase 2 (Pro)
addisco sync status|now|pause                    # SYNC-*
addisco account login|status|logout              # device-code login (ACCT-008)
```

Frozen with their specs (D-028): `note`, `task`, `exam`, `goal`, `subject`, `session`, `timer`, `workflow`, `dashboard` and `preset`. The hosted-chat command `ask` was cut (D-033).

## 5. Examples

```console
$ addisco
Today · Uni                               Thu 19 Nov
  Cards     42 due · 3 learning · 10 new
  Done      18 reviewed in 7 min
  Next      addisco review
  Data      ~/.local/share/addisco (schema v3)

$ addisco source add . --deck Addisco --tag repo::addisco
$ addisco scan
Scanned 214 files in 38 ms: 12 cards (3 new, ids written), 1 stale

$ addi while -- cargo test    # review until the tests finish; exits with cargo's code
$ addisco status --format '{due}d {stale}s'        # for a shell prompt or tmux
$ addisco stats reviews --range 7d --by day --json | jq '.data.series'
$ addisco card list 'lang:rust lapses:>3' --json | jq -r '.data[].id' | xargs addisco card suspend
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
q  = "review --quick"
w  = "while --"
l  = "learn"
s  = "scan"
a  = "add card"

[review]
keymap = "default"              # default | vim | emacs
show_intervals = true
show_progress = true
quick_count = 5                 # cards per `review --quick` (CLI-019)

[status]
format = ""                     # empty = full summary; e.g. "{due}d {stale}s" for prompts (CLI-021)

[scan]
extra_prefixes = {}             # e.g. { ".gleam" = "//" } adds comment prefixes (13 §3)

[hooks]                         # opt-in shell commands; the event JSON arrives on stdin (CLI-016)
# "scan.stale_found" = "notify-send 'Addisco' 'Some cards are stale'"

[tui]
theme = "auto"                  # auto | dark | light | high-contrast
unicode = true                  # false = ASCII-only box drawing
```

Workspace-scoped and app-scoped settings (shared with the VS Code extension) are managed with `addisco config get|set KEY` and are stored in the database, not in this file ([03 §5](03-customization-model.md#5-settings-layering)).

## 8. Interoperability with other shells

- **Shared data dir and DB.** The terminal CLI, the CLI bundled in the VS Code extension and the MCP server open the same database. SQLite in WAL mode with short transactions supports them all at once ([08 §8](08-architecture.md#8-multi-process-access-desktop--cli)).
- **Live updates.** A review in VS Code updates the due count in a running TUI or `status --watch` within ~1 s, and vice versa (NFR-PERF-006).
- **Install paths.** Standalone (GitHub releases, Homebrew, Scoop/winget, AUR, Nix, `cargo install`), and bundled per platform in the VS Code extension (OQ-016). There's no desktop installer (D-034).
- **Version skew.** Every binary checks the DB schema version on open. Older schema → migrate with a backup first. Newer schema → refuse writes and exit `6` with update instructions. `addisco doctor` reports skew, for example between the terminal CLI and the one bundled in VS Code.

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

### While (CLI-020)

```
┌ Review · while cargo test ─────────────────────── 12 due · running 00:41 ┐
│                                                                          │
│   Why does holding a std::sync::MutexGuard across .await risk deadlock?  │
│                                                                          │
├──────────────────────────────────────────────────────────────────────────┤
│ space reveal · 1–4 grade · q stop reviewing (cargo test keeps running)   │
└──────────────────────────────────────────────────────────────────────────┘
```

When the command exits, the screen closes, the last lines of its output are printed, and `addisco` exits with the command's code.

### Session runner (frozen with sessions, D-028)

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

## 10. Feature parity matrix

✓ full · ◐ partial · — not available

| Capability | CLI (Phase 1) | VS Code (Phase 1) | Mobile (Phase 2) |
|---|---|---|---|
| Review, quick review, filtered review | ✓ (TUI; images as placeholders until CLI-015) | ✓ (review panel) | ✓ |
| Review while a command runs | ✓ `while` | ◐ status bar + panel during tasks | — |
| Add cards | ✓ flags, `$EDITOR`, NDJSON stdin | ✓ from selection | ◐ quick capture |
| Sources, scan, stale queue | ✓ | ✓ scan on save, gutter marks | — (read-only cards) |
| Tree-sitter cloze | ✓ `card from-code` | ✓ from selection | — |
| Problem practice | ✓ | ✓ | ✓ review only |
| Targets (ready by a date) | ✓ | ◐ status bar countdown | ✓ countdown, widget |
| Stats | ✓ tables, JSON | ◐ summary | ◐ summary |
| Deck repositories | ✓ | ◐ through the CLI | — |
| Settings | ✓ `config`, `cli.toml` | ◐ extension settings | ◐ |
| Import / export / backup | ✓ | — | — |
| MCP server | ✓ `addisco mcp` | — (agents call the CLI) | — |
