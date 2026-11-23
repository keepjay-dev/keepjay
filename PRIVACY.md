# Privacy

Keepjay is local-first: your cards and your review history live on your machine.

## What Keepjay stores, and where

| What | Where |
|---|---|
| Cards, review history, decks, tags and settings | A SQLite database, `study.db`, in your data folder |
| Images and sounds imported from Anki | `media/` in your data folder |
| A random device id (used later for sync) | `device.toml` in your data folder |
| Your answer about the usage ping, its random id and when it was last sent | `telemetry.toml` in your data folder |
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

Keepjay makes **no network connections** unless you turn on the usage ping. There are no
update checks and no accounts. It runs your system's `git` locally to identify repositories.

| Endpoint | When | What |
|---|---|---|
| `https://ping.keepjay.com/v1/ping` | Only if you said yes to the usage ping; at most once a week | Six anonymous fields, listed in [docs/TELEMETRY.md](docs/TELEMETRY.md): no content, names or paths |

The ping is **off unless you turn it on**. Keepjay asks once, the first time you run
`keepjay source add` or `keepjay init` in a terminal; the answer defaults to no.
`keepjay telemetry status` shows exactly what it would send, `keepjay telemetry off` turns
it off, and `KEEPJAY_NO_TELEMETRY=1` or `DO_NOT_TRACK=1` overrides it. Keepjay works the
same with it off. Future hosted services (sync, backups) will be optional and documented
the same way.

## Questions

Email **hello@keepjay.com**. Security issues: see [SECURITY.md](SECURITY.md).
