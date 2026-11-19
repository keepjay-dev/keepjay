# Changelog

All notable changes are listed here. Versions follow [SemVer](https://semver.org/); entries are
generated from commit titles once the release workflow exists (docs/GIT_WORKFLOW.md §5).

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
