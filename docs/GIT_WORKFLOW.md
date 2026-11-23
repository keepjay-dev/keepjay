# Git workflow

> **Status:** Accepted (D-021) · **Last updated:** 2026-11-23
> Applies to everyone, human or AI assistant. Summary for `CLAUDE.md` and `CONTRIBUTING.md`.

## 1. Rules at a glance

1. `main` is always releasable. Nobody pushes to it directly; everything lands through a pull request.
2. One topic per branch, one pull request per branch.
3. Pull requests are **squash-merged**. The **PR title becomes the commit message**, so it must follow the commit format (§2). CI checks it.
4. Reference spec IDs (`SRS-005`, `D-020`, `OQ-006`…) in the PR body and, where useful, in commit messages.
5. Never force-push to `main`, and never rewrite history someone else has pulled.

## 2. Commit message format

[Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/):

```
<type>(<scope>): <summary in imperative mood, lowercase, no full stop>

<body: what and why, wrapped at ~72 chars, spec IDs>

<trailers>
```

Example:

```
feat(cli): add `keepjay review` with TUI (SRS-005, CLI-005)

Review due cards in the terminal with 1–4 grading, undo and edit.
Uses the core queue builder; no scheduling logic in the CLI.

Refs: J3
```

### Types

| Type | Use for | In changelog |
|---|---|---|
| `feat` | New user-visible capability | Features |
| `fix` | Bug fix | Fixes |
| `perf` | Performance improvement | Performance |
| `refactor` | Code change with no behaviour change | — |
| `test` | Tests only | — |
| `docs` | Documentation and spec | — |
| `build` | Build system, dependencies, packaging | — |
| `ci` | CI configuration | — |
| `chore` | Maintenance that fits nowhere else | — |
| `revert` | Reverts a previous commit (`revert: <original title>`) | Fixes |

**Breaking changes:** add `!` after the type or scope (`feat(core)!: …`) and a `BREAKING CHANGE:` trailer explaining the migration. Before v1.0 this still matters for the CLI JSON schema (`keepjay.cli/N`), document formats and the database schema.

### Scopes

Optional but encouraged. Use one, or several separated by commas (`feat(core,cli): …`).

| Scope | Covers |
|---|---|
| `core` | `crates/study-core` |
| `cli` | `apps/cli` |
| `web` | `apps/web` (Phase 2) |
| `mobile` | `apps/ios`, `apps/android` (Phase 3) |
| `ui` | Shared UI packages (`packages/ui`) |
| `tokens` | `packages/design-tokens` |
| `schemas` | `schemas/` |
| `db` | Database migrations (always also mention the schema version) |
| `spec` | `docs/specs/` |
| `research` | `docs/specs/research/` |
| `ci` | `.github/` (use type `ci` too) |
| `deps` | Dependency updates |
| `release` | Version bumps and changelogs |
| `repo` | Repo-wide config (licences, editor config, tooling) |

New scopes are added here first, then in the CI check (`.github/workflows/ci.yml`, job `pr-title`).

### Trailers

| Trailer | When |
|---|---|
| `Refs: <IDs>` | Spec IDs or issues not already in the title |
| `Closes #<n>` | The PR fully resolves an issue |
| `BREAKING CHANGE: <text>` | With `!` (see above) |
| `Co-Authored-By: <name> <email>` | Each additional author, **including AI assistants** that wrote part of the change |
| `Claude-Session: <url>` | Commits made from a Claude Code session (traceability) |

When a PR is squash-merged, keep the useful trailers from its commits (authors, `Refs`, `BREAKING CHANGE`) in the squash commit body.

## 3. Branches

| Branch | Purpose | Rules |
|---|---|---|
| `main` | Always releasable | Protected: PR required, CI must pass, no force-push, linear history |
| `<type>/<short-topic>` | Normal work, e.g. `feat/srs-review`, `fix/wal-checkpoint` | Lowercase, hyphens, no spaces. Short-lived: aim to merge within a few days. |
| `claude/<topic>-<id>` | Branches created by Claude Code sessions | Same rules as above |
| `release/vX.Y` | Created only when a release needs stabilising or patching while `main` moves on (from v1.0) | Only `fix` and `release` changes, merged back to `main` |

- Keep branches up to date by **rebasing on `main`** (or merging `main` in if others share the branch). Don't rebase a branch someone else has pulled without telling them.
- Delete branches after merging (GitHub setting: "Automatically delete head branches").
- **Stacked PRs** (a PR whose base is another PR's branch): merge from the bottom up. Let GitHub delete the merged branch: it then retargets the PRs stacked on it to `main`. If you delete a base branch by hand (for example `gh pr merge --delete-branch`, or `git push --delete`), first change the next PR's base to `main`. Otherwise GitHub closes that PR, and it can't be reopened once its branch has been force-pushed. After each squash merge, rebase the next branch onto `main` with only its own commits (`git rebase --onto main <old base tip>`).

## 4. Pull requests

- **Title:** the future squash commit message (§2). CI fails if it doesn't match.
- **Body:** use the template (`.github/pull_request_template.md`): what and why, spec IDs, test plan, checklist.
- **Size:** prefer small PRs (under ~400 changed lines excluding lockfiles and generated files). Split large work behind the spec's milestones.
- **Draft PRs** for early feedback. Mark ready when CI passes.
- **Before merge:** CI green on all three OSes; at least one approving review (once there's more than one maintainer); CLA signed for external contributors (NFR-MAINT-007); spec and roadmap updated if behaviour or plans changed.
- **Merge method:** **Squash and merge** only. Disable merge commits and rebase-merging in the repository settings.

## 5. Versions, tags and releases

- **SemVer** for the product (`vMAJOR.MINOR.PATCH`). Every shell shares one version (08 §14). Pre-1.0: minor versions may break things, but changes are still marked with `!`.
- **Tags:** annotated tags `vX.Y.Z` on `main` (or on `release/vX.Y`), created only by the release workflow (M5). Until M5 automates releases end to end, a maintainer tags by hand on `main` after the version-bump PR is merged: `git tag -a v0.1.0-alpha.1 -m "CLI alpha" && git push origin v0.1.0-alpha.1`. Pushing the tag runs `.github/workflows/release.yml`. It checks that the tag matches the Cargo version, builds and smoke-tests the CLI for Linux (static), macOS (Apple silicon, Intel) and Windows, and publishes a GitHub Release (a pre-release if the version has a `-` suffix) with checksums and the CHANGELOG section as notes.
- **Changelog:** generated from squash-commit titles (`feat`, `fix`, `perf`, `revert`, breaking changes), which is why PR titles matter.
- **Separate version numbers**, each recorded in the release notes when it changes:
  - database schema (`PRAGMA user_version`, D-009);
  - CLI JSON schema (`keepjay.cli/N`);
  - document schema per kind (03 §7).

## 6. History hygiene

- Never commit secrets, `.env` files, local databases (`*.db`) or build output. `.gitignore` covers the common cases.
- Fix mistakes on your own unpushed branch freely (amend, interactive rebase). After pushing a shared branch, add a fix-up commit instead of rewriting.
- To undo something on `main`, use `git revert` through a PR. Never reset or force-push.
- **Signed commits:** optional for now; **required for everyone once external contributors are accepted** (enable "Require signed commits" on `main` then).

## 7. Repository settings to apply on GitHub (maintainer action)

These can't be set from the repository files. Apply them in **Settings**:

- [x] **Rules → Rulesets → "Protect main"** (applied 2026-11-22) on the default branch: require a pull request (squash only, no approvals needed while there's one maintainer); require status checks (`Lint, format & licences`, `Build & test (ubuntu-latest)`, `Build & test (macos-latest)`, `Build & test (windows-latest)`, `PR title`); require linear history; block force pushes and deletion. Repository admins can bypass it for emergencies only; normal merges, including an agent's, wait for the checks.
- [x] **General → Pull Requests** (applied 2026-11-22): squash merging only; squash commit title = PR title, message = PR description; "Automatically delete head branches" and "Always suggest updating pull request branches" on.
- [x] **General** (applied 2026-11-22; moved 2026-11-23): the public repository is `mo-sajaad/keepjay` (D-037, D-038); the earlier private repository `mo-sajaad/scrubjay` is kept as an archive; description and topics set; wiki off (docs live in `docs/`).
- [ ] Before making the repository public: reserve the name (domain, crate, npm scope, GitHub org), decide what stays in `docs/specs/research/`, and say in CONTRIBUTING that outside contributions wait for the CLA.
- [ ] Later (with external contributors): require signed commits; require 1 approving review; enable the CLA bot.
