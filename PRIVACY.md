# Privacy

Keepjay is local-first: your cards and your review history live on your machine.

## What Keepjay stores, and where

| What | Where |
|---|---|
| Cards, review history, decks, tags and settings | A SQLite database, `study.db`, in your data folder |
| Images and sounds imported from Anki | `media/` in your data folder |
| A random device id (used later for sync) | `device.toml` in your data folder |
| CLI settings | `cli.toml` in your config folder |

| OS | Data folder | Config folder |
|---|---|---|
| macOS | `~/Library/Application Support/Keepjay` | the same |
| Linux | `~/.local/share/keepjay` | `~/.config/keepjay` |
| Windows | `%LOCALAPPDATA%\Keepjay` | `%APPDATA%\Keepjay` |

`KEEPJAY_DATA_DIR` and `--data-dir` choose another data folder.

## What Keepjay changes in your files

- On the first scan of a source, it writes each card's short id into the card's comment
  (`@card(k22k1m8e)`) or adds an id line to a Markdown card, so the card keeps its history
  when you move it. Turn this off for a source with `keepjay source add PATH --no-write-ids`.
- `keepjay stale retire` removes a card's own lines from its file, after asking you.

Nothing else in your files is changed. Keepjay reads only the folders you register as
sources, respects `.gitignore` and `.keepjayignore`, and never runs code from your files.

## Network

The Keepjay CLI makes **no network connections**: no telemetry, no update checks, no
accounts. It runs your system's `git` locally to identify repositories.

If an optional, anonymous usage ping is added later, it will be **off unless you turn it
on**, every field it sends will be documented here first, and Keepjay will work fully with
it off. Future hosted services (sync, backups) will be optional and documented the same way.

## Questions

Email **hello@keepjay.com**. Security issues: see [SECURITY.md](SECURITY.md).
