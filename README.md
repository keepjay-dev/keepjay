# Keepjay

**Keep what you learn while you code.** Keepjay is local-first spaced repetition for
developers: write flashcards as comments next to your code, review them while your tests
run, and know when a card goes out of date because the code under it changed.

```rust
// @card What does `Arc::clone` actually copy?
// Only the pointer: it increments the reference count; the data isn't cloned.
let shared = Arc::clone(&config);
```

```sh
keepjay source add .          # register a repository
keepjay scan                  # find its @card comments
keepjay while -- cargo test   # review while the tests run
```

- **Cards live in your files.** Code comments in 30+ languages, or Markdown card files. Plain
  text in Git, so they're reviewable, shareable and yours.
- **Cards notice when code changes.** Each card is anchored to the code below it; when that
  code changes, the card is flagged as stale instead of quietly going out of date.
- **Review in the gaps.** `while` reviews during a build or test run, `review --quick` takes
  a minute, and `status --format` puts what's due in your prompt.
- **Proven scheduling.** FSRS, as in Anki, and Anki decks import with their review history.
- **Local-first and private.** No account, and nothing leaves your machine unless you opt in
  to an anonymous weekly [usage ping](docs/TELEMETRY.md). Everything that runs locally is
  free, forever.

> **Early alpha.** It works and is tested on macOS, Linux and Windows, but expect rough
> edges. Feedback is very welcome as [issues](https://github.com/keepjay-dev/keepjay/issues).

## Install

On macOS or Linux:

```sh
curl -fsSL https://raw.githubusercontent.com/keepjay-dev/keepjay/main/install.sh | sh
```

On Windows, or to install by hand, download the archive for your platform from
[Releases](https://github.com/keepjay-dev/keepjay/releases) and put `keepjay` on your `PATH`.
Then follow [Getting started](docs/guide/getting-started.md). Releases before
0.2.0-alpha.3 installed the command under its old name, `scrubjay`.

## Documentation

- **Using Keepjay:** [Getting started](docs/guide/getting-started.md) ·
  [Writing cards](docs/guide/writing-cards.md) · [Reviewing](docs/guide/reviewing.md) ·
  [Commands](docs/guide/commands.md) · [FAQ](docs/guide/faq.md)
- **Project:** [Roadmap](ROADMAP.md) · [Changelog](CHANGELOG.md) · [Privacy](PRIVACY.md) ·
  [Security](SECURITY.md) · [Contributing](CONTRIBUTING.md) ·
  [Code of conduct](CODE_OF_CONDUCT.md)
- **Design:** the [specification](docs/specs/README.md): architecture, data model, CLI,
  requirements and the decision log.

*The name comes from food-caching jays, birds that remember what they hid, where and when
([D-037](docs/specs/decisions.md)). Formerly Scrubjay and, before that, Addisco.*

## Repository layout

| Path | What | Licence |
|---|---|---|
| `crates/study-core/` | Shared Rust core: storage, IDs, change log; later scheduling, workflows, metrics | AGPL-3.0-or-later |
| `apps/cli/` | `keepjay` command-line tool | AGPL-3.0-or-later |
| `packages/design-tokens/` | Colours, spacing and type as CSS variables (light/dark) | AGPL-3.0-or-later |
| `schemas/` | Data format schemas | MIT |
| `docs/` | Specification and documentation | CC BY 4.0 |

The sync server will be open source (AGPL) and self-hostable. The other hosted services (AI gateway, accounts and billing) are proprietary and live outside this repository ([D-020](docs/specs/decisions.md)). The name and logo are covered by the [trademark policy](TRADEMARKS.md).

## Development

Try it on this repository, which has its own cards:

```sh
keepjay source add . --deck Keepjay   # register the repo
keepjay scan                           # finds the @card comments and .keepjay/cards/
keepjay review --quick                 # a few cards, then back to work
keepjay while -- cargo test            # review while the tests run
```

Building from source needs Rust (pinned in `rust-toolchain.toml`); Node.js 22+ and pnpm 10+
are needed only for the formatting checks and design tokens (see
[CONTRIBUTING.md](CONTRIBUTING.md)).

```sh
cargo test --workspace                # core + CLI
cargo run -p keepjay-cli -- status
```

## Licence

Code: [GNU AGPL v3.0 or later](LICENSE). Schemas: MIT. Docs: CC BY 4.0. Contributions require a CLA (pending legal review). See [CONTRIBUTING.md](CONTRIBUTING.md).
