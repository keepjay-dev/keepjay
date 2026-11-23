# Roadmap

Keepjay is an early alpha. This is the plan in broad strokes; it changes as we learn from
people using it. Requirement IDs (e.g. SRC-007) point into the [specification](docs/specs/README.md).

## Now: the CLI alpha

What works today (see the [changelog](CHANGELOG.md)):

- Cards from `@card` comments in your code and from Markdown card files, with ids written
  back so cards keep their history when you move them.
- Stale cards: when the code under a card changes, the next scan flags it. Keep it, edit it,
  or retire it.
- FSRS scheduling, `review`, `review --quick`, and `while`, which reviews while your build or
  tests run.
- `status --format` for shell prompts, lossless Anki import with review history, and the
  `kj` short alias.

## Next

- **Sources, finished:** deck repositories you can share, a Git hook that scans on commit,
  editing stale cards in place, undo and suspend in review, automatic backups and export.
- **Code-aware cards:** syntax highlighting, and anchors that follow a function or type
  rather than line numbers (tree-sitter, starting with Python, TypeScript/JavaScript, Rust
  and Go).
- **Composability and VS Code:** queries and filtered review, JSON output for scripting,
  shell completions, an MCP server so coding agents can draft cards, and a **VS Code
  extension**.
- **Interview mode and ready-by-a-date:** turn your solutions repo into practice by
  pattern, and schedule a deck to be ready for an exam or interview date.
- **Beta:** package managers (Homebrew, Scoop/winget, AUR, Nix, `cargo install`), signed
  binaries and a docs site.

## Later

- **Review on your phone**, with optional hosted sync (Keepjay Pro). A self-hostable sync
  server will be open source. Everything that runs locally stays free (D-017).
- A web app, a JetBrains plugin and community editor plugins.

Ideas and feedback are welcome as [issues](https://github.com/keepjay-dev/keepjay/issues).
