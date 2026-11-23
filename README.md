# Keepjay

*Named after food-caching jays, birds that remember what they hid, where and when ([D-037](docs/specs/decisions.md)). Formerly Scrubjay and, before that, Addisco.*

**Keep what you learn while you code.** Local-first spaced repetition for engineers and CS students:

- cards from your code comments and Markdown files, which notice when the code changes;
- reviews while your build runs;
- problem practice from your own solutions repo, ready by interview day;
- FSRS scheduling with lossless Anki import.

It starts as a CLI, then adds a VS Code extension and a mobile review app, all on one shared Rust core.

**Status:** Phase 0, an early alpha. The developer alpha works: cards from `@card` comments and Markdown files, stale-card detection, `review --quick`, `while`, FSRS review and Anki import with history. Specification: [`docs/specs/`](docs/specs/README.md). Current status: [`docs/specs/11-roadmap.md`](docs/specs/11-roadmap.md).

## Repository layout

| Path | What | Licence |
|---|---|---|
| `crates/study-core/` | Shared Rust core: storage, IDs, change log; later scheduling, workflows, metrics | AGPL-3.0-or-later |
| `apps/cli/` | `keepjay` command-line tool | AGPL-3.0-or-later |
| `packages/design-tokens/` | Colours, spacing and type as CSS variables (light/dark) | AGPL-3.0-or-later |
| `schemas/` | Data format schemas | MIT |
| `docs/` | Specification and documentation | CC BY 4.0 |

The sync server will be open source (AGPL) and self-hostable. The other hosted services (AI gateway, accounts and billing) are proprietary and live outside this repository ([D-020, D-033](docs/specs/decisions.md)). The name and logo are covered by the [trademark policy](TRADEMARKS.md).

## Install the CLI alpha

Download the archive for your platform from [Releases](https://github.com/mo-sajaad/scrubjay/releases), extract it, put `keepjay` on your `PATH`, then run `keepjay --help`. `keepjay alias install` adds the short alias `kj`, unless another `kj` is already on your `PATH`. Alpha builds aren't code-signed yet; the release notes explain the one-time macOS and Windows prompts. See [CHANGELOG.md](CHANGELOG.md) for what works. Releases up to 0.2.0-alpha.2 still install the command as `scrubjay` (the old name, D-037); the next release ships it as `keepjay`.

Try it on this repository, which has its own cards:

```sh
keepjay source add . --deck Keepjay   # register the repo
keepjay scan                           # finds the @card comments and .keepjay/cards/
keepjay review --quick                 # a few cards, then back to work
keepjay while -- cargo test            # review while the tests run
```

## Quick start (development)

Requirements: Rust (pinned in `rust-toolchain.toml`). Node.js 22+ and pnpm 10+ only for the
formatting checks and design tokens (see [CONTRIBUTING.md](CONTRIBUTING.md)).

```sh
cargo test --workspace                # core + CLI
cargo run -p keepjay-cli -- status
```

## Licence

Code: [GNU AGPL v3.0 or later](LICENSE). Schemas: MIT. Docs: CC BY 4.0. Contributions require a CLA (pending legal review). See [CONTRIBUTING.md](CONTRIBUTING.md).
