<!--
External contributors: we can't merge pull requests until the CLA is ready
(see CONTRIBUTING.md). Please open an issue instead.

PR title = the squash commit message: <type>(<scope>): <summary>
e.g. feat(cli): add review command (SRS-005)   See docs/GIT_WORKFLOW.md
-->

## What and why

<!-- What changes, and why. Link the spec: requirement IDs, decisions, open questions. -->

Spec: <!-- e.g. SRS-005, CLI-005, J3 -->

## How it was tested

<!-- Commands run, tests added, manual checks (screenshots for UI). -->

## Checklist

- [ ] PR title follows the commit format (CI checks this)
- [ ] Tests added or updated; `cargo test --workspace` and `pnpm lint` pass locally
- [ ] Spec, roadmap or decisions updated if behaviour or plans changed
- [ ] New source files have an SPDX licence header
- [ ] No new dependency with a licence outside `deny.toml`
- [ ] Breaking change? Title has `!` and the body has a `BREAKING CHANGE:` note (DB schema / CLI JSON / document format)
- [ ] External contributors: CLA signed
