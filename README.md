# Addisco

*Latin "I keep learning". Provisional name, see [D-019](docs/specs/decisions.md).*

A local-first, highly customisable study app: spaced-repetition flashcards, notes, timed study workflows, planning, goals and configurable dashboards. It starts with a desktop app plus a first-class CLI, then grows to a web app and native mobile apps on one shared core.

**Status:** Phase 0. The specification is complete ([`docs/specs/`](docs/specs/README.md)), and the repository scaffold (M0.3) is in place. Current status: [`docs/specs/11-roadmap.md`](docs/specs/11-roadmap.md).

## Repository layout

| Path | What | Licence |
|---|---|---|
| `crates/study-core/` | Shared Rust core: storage, IDs, change log; later scheduling, workflows, metrics | AGPL-3.0-or-later |
| `apps/cli/` | `addisco` command-line tool | AGPL-3.0-or-later |
| `apps/desktop/` | Desktop app: Tauri 2 (`src-tauri/`) + React/TypeScript UI (`src/`) | AGPL-3.0-or-later |
| `packages/design-tokens/` | Colours, spacing and type as CSS variables (light/dark) | AGPL-3.0-or-later |
| `schemas/` | Data format schemas | MIT |
| `presets/` | Built-in presets | CC0-1.0 |
| `docs/` | Specification and documentation | CC BY 4.0 |

Hosted services (sync, AI gateway, accounts) are proprietary and live outside this repository ([D-020](docs/specs/decisions.md)). The name and logo are covered by the [trademark policy](TRADEMARKS.md).

## Quick start (development)

Requirements: Rust (pinned in `rust-toolchain.toml`), Node.js 22+, pnpm 10+, and on Linux the Tauri system libraries (see [CONTRIBUTING.md](CONTRIBUTING.md)).

```sh
pnpm install && pnpm build            # design tokens + desktop frontend
cargo test --workspace                # core + CLI + desktop
cargo run -p addisco-cli -- init      # create your first workspace
cargo run -p addisco-cli -- status
pnpm desktop:dev                      # desktop app; changes made with the CLI appear live
```

## Licence

Code: [GNU AGPL v3.0 or later](LICENSE). Schemas: MIT. Presets: CC0. Docs: CC BY 4.0. Contributions require a CLA (pending legal review). See [CONTRIBUTING.md](CONTRIBUTING.md).
