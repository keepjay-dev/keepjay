# Contributing to Scrubjay

Thanks for your interest! Scrubjay is in **Phase 0**: the developer alpha (0.2.0-alpha.1) is
out for testing. See [`docs/specs/11-roadmap.md`](docs/specs/11-roadmap.md) for current status.

> **We can't merge outside pull requests yet.** Contributions need a signed Contributor
> License Agreement (CLA), and its text is still in legal review. Until it's ready:
>
> - **Do** open issues: bug reports, alpha feedback, feature ideas and questions all help.
> - **Please don't** send code pull requests. We'd have to leave them unmerged, or close them,
>   until you could sign the CLA, and the code may change a lot by then.
> - Typo fixes are welcome as issues; we'll make the change and credit you.
>
> This notice goes away when the CLA is ready.

## Before you open a pull request

- **Contributor License Agreement (required).** Every external contribution needs a signed CLA
  before it can be merged (D-020, NFR-MAINT-007). The CLA keeps copyright with the project, which
  lets us publish the iOS app under App Store terms and offer commercial licences, while the code
  stays AGPL-3.0-or-later. **The CLA text is pending legal review**, so external pull requests
  can't be merged yet. Issues and discussions are very welcome in the meantime.
- Read the relevant spec file first (index: [`docs/specs/README.md`](docs/specs/README.md)) and
  reference requirement IDs (e.g. `SRS-005`, `CLI-007`) in commits and PRs.

## Development setup

Requirements: Rust (version pinned in `rust-toolchain.toml`). Node.js 22+ and pnpm 10+ are
only needed for formatting checks and the design tokens.

```sh
cargo test --workspace
cargo run -p scrubjay-cli -- status       # the CLI
pnpm install && pnpm lint                # design tokens and Prettier (optional)
```

## Checks (CI runs all of these)

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm lint && pnpm typecheck && pnpm format:check
cargo deny check licenses bans sources   # dependency licence policy (deny.toml)
scripts/check-spdx.sh                    # every source file has an SPDX licence header
```

## Conventions

- **Licence headers:** every source file starts with an SPDX header, e.g.
  `// SPDX-License-Identifier: AGPL-3.0-or-later` (MIT in `schemas/`, CC0 in `presets/`).
- **Git:** follow [`docs/GIT_WORKFLOW.md`](docs/GIT_WORKFLOW.md): Conventional Commits with the listed
  scopes, short-lived branches, squash-merge (the PR title becomes the commit and is checked
  in CI), e.g. `feat(cli): add review command (SRS-005, CLI-005)`.
- **Terminology:** use the glossary ([`docs/specs/12-glossary.md`](docs/specs/12-glossary.md)):
  a *fact* is not a *note*.
- **Dependencies:** only licences on the allowlist in `deny.toml`. If you need another, open an
  issue first.
