# Changelog

All notable changes are listed here. Versions follow [SemVer](https://semver.org/); entries are
generated from commit titles once the release workflow exists (docs/GIT_WORKFLOW.md §5).

## Unreleased

### Added
- `install.sh` for macOS and Linux: `curl -fsSL https://keepjay.com/install.sh | sh`
  installs the newest release in `~/.local/bin` after checking its checksum. It also avoids
  macOS's quarantine prompt for downloaded binaries.
- A Homebrew tap: `brew install keepjay-dev/tap/keepjay` installs `keepjay` and `kj` on macOS
  and Linux, and `brew upgrade keepjay` updates them.
- An **opt-in** anonymous weekly usage ping, off unless you say yes (D-040). Keepjay asks
  once, after `keepjay source add` or `keepjay init` in a terminal. It sends six fields, such
  as its version and a rough review count, and never content, names or paths:
  `keepjay telemetry status` shows exactly what. `keepjay telemetry on|off` changes it, and
  `KEEPJAY_NO_TELEMETRY` or `DO_NOT_TRACK` overrides it. See `docs/TELEMETRY.md`.
- The review screen draws card text as Markdown: code blocks appear behind a `│` bar
  with their language, inline `code` is coloured, and bold, italic, lists and links are
  shown as such (SRS-008, CODE-001). `NO_COLOR` turns the colour off.
- Code blocks on cards are syntax-coloured in reviews, using your terminal's palette, for
  about 75 languages named after the opening fence (```` ```rust ````) (CODE-001).

### Changed
- The review screen is at most 80 characters wide and centred on wider terminals, so long
  answers are easier to read. Text stays left-aligned, so code keeps its indentation.

## 0.2.0-alpha.3

The first release under the name **Keepjay**, and the first from the public repository
(`keepjay-dev/keepjay`). Upgrade from 0.2.0-alpha.2 by installing `keepjay` and removing the
old `scrubjay` binary; your data, cards and settings carry over (see below).

### Added
- A user guide in `docs/guide/` (getting started, writing cards, reviewing, commands and
  FAQ), plus `ROADMAP.md` and `PRIVACY.md`.

### Changed
- **Scrubjay is now Keepjay** (D-037). Another local-first desktop app is already sold as
  Scrubjay, and a developer tool uses the same story, so the name had to change. The
  command is now `keepjay`, the short alias `kj`, and environment variables start with
  `KEEPJAY_`.
  - Your data moves over by itself: on first run, an existing `Scrubjay` (or `Addisco`) data
    or config folder is renamed to `Keepjay`. The old `SCRUBJAY_*` and `ADDISCO_*` variables
    still work.
  - Cards in `.scrubjay/cards/` and rules in `.scrubjayignore` are still read, so nothing is
    retired. Rename them to `.keepjay/cards/` and `.keepjayignore` when convenient.
  - If you installed the `sj` alias, run `scrubjay alias uninstall` before upgrading, then
    `keepjay alias install` for `kj`. Otherwise delete `sj` by hand; it points at the old
    binary.
  - **Breaking:** JSON output uses the schema id `keepjay.cli/1` (was `scrubjay.cli/1`).

### Added
- `keepjay stale retire ID` for a stale card you no longer want: it removes the card's
  comment (or Markdown card) from your file, leaving the code alone, and retires the card
  with its review history. It shows the lines and asks first; use `--dry-run` to look, or
  `--yes` in scripts.
- `keepjay alias install` adds the short alias `kj` next to `keepjay` (a symlink on macOS
  and Linux, so it follows upgrades). It refuses if another program called `kj` is on your
  `PATH`. `keepjay alias uninstall` removes it, and `keepjay alias` shows what `kj` runs.
- `keepjay doctor` warns when another `kj` earlier on your `PATH` hides keepjay's.

## 0.2.0-alpha.2

Fixes from dogfooding and the first-run checks (J15, J16). Upgrade from 0.2.0-alpha.1 by
replacing the binary; nothing else changes.

### Added
- `@card:off` and `@card:on` comments (`<!-- @card:off -->` in Markdown card files) stop and
  restart finding cards, e.g. around tests full of `@card` examples. A misspelt switch is a
  warning.

### Fixed
- Re-scanning no longer rewrites every card's anchor. Before, each scan logged one change
  per card, which would also have had to sync.
- Scans are faster: files are read in parallel, and writing ids into files is about 4× faster
  on macOS. On 10,000 files, a re-scan takes about 0.3 s and a first scan 1.6 s.
- A file that can't be read is now a warning instead of stopping the scan.
- `scrubjay source add`, `deck create`, `add card` and `import anki` work on a fresh install
  without running `scrubjay init` first: they create your workspace.
- The 0.2.0-alpha.1 notes promised a short alias `sj`; it isn't installed yet. Until
  `scrubjay alias install` exists, add `alias sj=scrubjay` to your shell's startup file.

## 0.2.0-alpha.1 (developer alpha)

The first build for developers (M0.5, D-032): write cards next to the code they explain, and
Scrubjay tells you when that code changes. Terminal only, and your data stays on your machine.

### Renamed: Addisco is now **Scrubjay**
- The command is `scrubjay` (short alias `sj`). Your data moves on its own: on first run, an
  existing `Addisco` data folder is renamed to `Scrubjay`. `ADDISCO_DATA_DIR` still works;
  prefer `SCRUBJAY_DATA_DIR`.
- **Breaking for scripts:** `--json` output now says `"schema": "scrubjay.cli/1"`
  (was `addisco.cli/1`).

### Cards from your code
- `scrubjay source add PATH [--deck D] [--tag T…] [--no-write-ids]` registers a repository or
  folder; `source list` and `source rm` manage them.
- Write a card as a comment, in about 60 file types:
  ```rust
  // @card Why does `migrate` read the schema version again after taking the lock?
  // Another process may have migrated the database in the meantime.
  fn migrate(…) { … }
  ```
  Or put `Q:`/`A:` and `C:` (cloze) cards in `*.cards.md` files or `.scrubjay/cards/`.
- `scrubjay scan` finds them, respecting `.gitignore` and `.scrubjayignore`. It writes a short
  id into each card (`@card(k22k1m8e)`), so edits and moves keep your review history and a
  teammate's clone gets the same cards. Deleted cards retire, and come back with their history
  if you restore them.
- **Stale cards:** when the code under a card changes, the card is flagged.
  `scrubjay stale` lists them; `scrubjay stale keep ID` says the card is still right, or edit the
  card and scan again.

### Reviewing around your work
- `scrubjay review --quick [-n N] [--time D]`: review a few cards (5 by default) and exit.
  Prints nothing when nothing is due, so it can go in your shell's startup file.
- `scrubjay while -- cargo test`: review while a command runs. When it finishes you see the
  end of its output, and `scrubjay` exits with the command's exit code.
- `scrubjay status --format '{due}d {stale}s'` for shell prompts and status bars.

### Also new since 0.1.0-alpha.1
- `scrubjay import anki FILE [--into DECK]`: import Anki packages (`.apkg`, `.colpkg`) from
  both the current format (Anki 2.1.50+) and the legacy one, **including review history**,
  which is replayed through FSRS. Stock note types map to Scrubjay card types, even with
  translated names; custom note types are imported card by card from their templates.
  Suspended cards stay suspended. Importing the same package again skips notes you already
  have.
- Imported HTML formatting is kept and shown as clean text in the terminal.
- Tags: `scrubjay add card … --tag bio::cells` (repeatable) and `scrubjay tag list`. Anki tags
  are imported too. Tags match regardless of case, like in Anki.
- Anki images and sounds are imported into Scrubjay's media folder, stored once however
  often they're used. If a file name is already taken by a different file, the new one gets a
  short suffix and the imported cards are updated to match.

### Known limitations
- Cards are found in comments line by line, without a language parser: an `@card` example
  inside a string literal at the start of a line counts as a card. Exclude such files with
  `.scrubjayignore` (this repository does, for the parser's own tests).
- Without written ids (`--no-write-ids`), a card is matched by its file and exact text, so
  rewording it starts a new card.
- `stale retire`, Git hooks and editor integration come later.
- The terminal shows images and sounds as `[image: …]` / `[audio: …]`; playing audio comes
  later.
- Database schema v7. Older databases upgrade automatically, taking a backup first.

Data lives in `~/.local/share/scrubjay` (Linux), `~/Library/Application Support/Scrubjay`
(macOS) or `%LOCALAPPDATA%\Scrubjay` (Windows); override with `SCRUBJAY_DATA_DIR`.

## 0.1.0-alpha.1 (CLI alpha)

The first build for testers, as part of validation milestone M0.5 (D-024). Terminal only: the
desktop app is a scaffold, and data stays on your machine.

### What you can do
- `addisco init`, then `addisco deck create "Biology::Cells"` to set up decks.
- `addisco add card "Question" "Answer" -d <deck>`, with card types `basic`, `basic-reverse`,
  `optional-reverse`, `cloze` (`{{c1::...}}`) and `type-in`.
- `addisco review`: full-screen review with FSRS-7 scheduling and interval previews on the
  1–4 keys, or `--plain` for line mode and screen readers.
- `addisco` / `addisco status`: today's due cards, what you've done, and what to do next.
- `addisco doctor`: database health check. `--json` on every command for scripting.

### Known limitations
- No Anki import yet, no undo/edit/suspend during review, no sync, no desktop UI.
- Database schema v3. Future versions upgrade it automatically, taking a backup first.

Data lives in `~/.local/share/addisco` (Linux), `~/Library/Application Support/Addisco` (macOS)
or `%LOCALAPPDATA%\Addisco` (Windows); override with `ADDISCO_DATA_DIR`.
