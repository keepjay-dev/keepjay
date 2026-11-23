# Commands

`keepjay --help` and `keepjay <command> --help` show the same information in your terminal.
`kj` works in place of `keepjay` once you've run `keepjay alias install`.

## Everyday

| Command | What it does |
|---|---|
| `keepjay` or `keepjay status` | Today's summary: due, new and stale cards |
| `keepjay status --format '{due} due'` | One line for a shell prompt; fields `{due}` `{review}` `{learning}` `{new}` `{stale}` `{reviewed}` |
| `keepjay review` | Full-screen review (`--deck D`, `--limit N`, `--plain` for line mode) |
| `keepjay review --quick` | A few due cards, then exit (`-n N`, `--time 2m`); silent when nothing is due |
| `keepjay while -- CMD …` | Run a command and review until it finishes; exits with the command's code (`--deck`, `--limit`) |
| `keepjay scan` | Find cards in all sources and update them (`keepjay scan NAME` for one source, `-q` for hooks) |

## Sources

| Command | What it does |
|---|---|
| `keepjay source add PATH` | Register a folder or repository (`--name`, `--deck D`, `--tag T`, `--no-write-ids`) |
| `keepjay source list` | List sources |
| `keepjay source rm NAME` | Unregister a source; its cards are kept |
| `keepjay stale` | List stale cards |
| `keepjay stale keep ID` | The card is still right: clear the flag |
| `keepjay stale retire ID` | Remove the card's lines from its file and retire it, keeping its history (`--dry-run`, `--yes`) |

## Cards, decks and tags

| Command | What it does |
|---|---|
| `keepjay deck list` / `deck create PATH` | List decks, or create one (`Parent::Child` creates both) |
| `keepjay add card --deck D FRONT BACK` | Add a card without a file (`--type basic\|basic-reverse\|optional-reverse\|cloze\|type-in`, `--tag T`) |
| `keepjay tag list` | List tags |
| `keepjay import anki FILE` | Import an `.apkg` or `.colpkg` with its review history (`--into D`) |

## Setup and maintenance

| Command | What it does |
|---|---|
| `keepjay init` | Create your first workspace (other commands do this for you when needed) |
| `keepjay workspace list` / `workspace create NAME` | Workspaces keep separate sets of decks |
| `keepjay alias install` / `uninstall` / `status` | The `kj` short alias |
| `keepjay doctor` | Check the database and the alias |
| `keepjay telemetry status` / `on` / `off` | The anonymous weekly usage ping, off unless you turn it on; `status` shows exactly what it sends ([details](../TELEMETRY.md)) |

## Options for every command

| Option | Environment variable | Meaning |
|---|---|---|
| `--json` | | Machine-readable output |
| `--no-color` | `NO_COLOR` | No colour (any non-empty value of `NO_COLOR` works too) |
| `--data-dir PATH` | `KEEPJAY_DATA_DIR` | Use another data folder |
| `-w`, `--workspace NAME` | `KEEPJAY_WORKSPACE` | Act on another workspace |
| | `KEEPJAY_NO_TELEMETRY`, `DO_NOT_TRACK` | Never send the usage ping |

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Success, including "nothing to do" (e.g. no cards due) |
| 1 | Unexpected error |
| 2 | Usage error: bad flags or arguments, or input needed without a terminal |
| 3 | Not found |
| 4 | Validation failed (e.g. an invalid value) |
| 5 | Conflict (e.g. the database is busy, or another `kj` is on your `PATH`) |
| 6 | Your data was written by a newer Keepjay: update this one |
| 130 | Interrupted (Ctrl+C) |

`keepjay while` exits with its command's code instead.
