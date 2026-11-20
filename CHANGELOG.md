# Changelog

All notable changes are listed here. Versions follow [SemVer](https://semver.org/); entries are
generated from commit titles once the release workflow exists (docs/GIT_WORKFLOW.md §5).

## Unreleased

### Added
- `addisco import anki FILE [--into DECK]`: import Anki packages (`.apkg`, `.colpkg`) from both
  the current format (Anki 2.1.50+) and the legacy one, **including review history**, which is
  replayed through FSRS. Stock note types map to Addisco card types, even with translated
  names; custom note types are imported card by card from their templates. Suspended cards
  stay suspended. Importing the same package again skips notes you already have.
- Imported HTML formatting is kept and shown as clean text in the terminal.
- Tags: `addisco add card … --tag bio::cells` (repeatable) and `addisco tag list`. Anki tags
  are imported too. Tags match regardless of case, like in Anki.
- Anki images and sounds are imported into Addisco's media folder, stored once however often
  they're used. If a file name is already taken by a different file, the new one gets a
  short suffix and the imported cards are updated to match.

### Known limitations
- The terminal shows images and sounds as `[image: …]` / `[audio: …]`; playing audio comes
  later.

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
