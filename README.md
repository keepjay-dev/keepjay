# Scrubjay

*Named after the scrub jay, the bird that remembers what it hid, where and when ([D-035](docs/specs/decisions.md)). Formerly Addisco.*

**Keep what you learn while you code.** Local-first spaced repetition for engineers and CS students:

- cards from your code comments and Markdown files, which notice when the code changes;
- reviews while your build runs;
- problem practice from your own solutions repo, ready by interview day;
- FSRS scheduling with lossless Anki import.

It starts as a CLI, then adds a VS Code extension and a mobile review app, all on one shared Rust core.

**Status:** Phase 0. The developer alpha (0.2.0-alpha.1) works: cards from `@card` comments and Markdown files, stale-card detection, `review --quick`, `while`, FSRS review and Anki import with history. Specification: [`docs/specs/`](docs/specs/README.md). Current status: [`docs/specs/11-roadmap.md`](docs/specs/11-roadmap.md).

## Repository layout

| Path | What | Licence |
|---|---|---|
| `crates/study-core/` | Shared Rust core: storage, IDs, change log; later scheduling, workflows, metrics | AGPL-3.0-or-later |
| `apps/cli/` | `scrubjay` command-line tool | AGPL-3.0-or-later |
| `packages/design-tokens/` | Colours, spacing and type as CSS variables (light/dark) | AGPL-3.0-or-later |
| `schemas/` | Data format schemas | MIT |
| `presets/` | Built-in presets | CC0-1.0 |
| `docs/` | Specification and documentation | CC BY 4.0 |

The sync server will be open source (AGPL) and self-hostable. The other hosted services (AI gateway, accounts and billing) are proprietary and live outside this repository ([D-020, D-033](docs/specs/decisions.md)). The name and logo are covered by the [trademark policy](TRADEMARKS.md).

## Install the CLI alpha

Download the archive for your platform from [Releases](https://github.com/mo-sajaad/scrubjay/releases), extract it, put `scrubjay` on your `PATH`, then run `scrubjay --help`. `scrubjay alias install` adds the short alias `sj`, unless another `sj` is already on your `PATH`. Alpha builds aren't code-signed yet; the release notes explain the one-time macOS and Windows prompts. See [CHANGELOG.md](CHANGELOG.md) for what works.

Try it on this repository, which has its own cards:

```sh
scrubjay source add . --deck Scrubjay   # register the repo
scrubjay scan                           # finds the @card comments and .scrubjay/cards/
scrubjay review --quick                 # a few cards, then back to work
scrubjay while -- cargo test            # review while the tests run
```

## Quick start (development)

Requirements: Rust (pinned in `rust-toolchain.toml`). Node.js 22+ and pnpm 10+ only for the
formatting checks and design tokens (see [CONTRIBUTING.md](CONTRIBUTING.md)).

```sh
cargo test --workspace                # core + CLI
cargo run -p scrubjay-cli -- status
```

## Licence

Code: [GNU AGPL v3.0 or later](LICENSE). Schemas: MIT. Presets: CC0. Docs: CC BY 4.0. Contributions require a CLA (pending legal review). See [CONTRIBUTING.md](CONTRIBUTING.md).
