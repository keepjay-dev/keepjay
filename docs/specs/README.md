# Scrubjay — Project Specification

> **Status:** Draft · **Version:** 0.3 · **Last updated:** 2026-11-23

This folder is the source of truth for **what** we are building and **why**, plus the proposed **how**. It is split into focused files so that a person (or an AI assistant) can load only the file relevant to the current task instead of re-reading everything.

> Product name **Scrubjay** (CLI `scrubjay`, short alias `sj`, URL scheme `scrubjay://`), chosen in [D-035](decisions.md#d-035--product-name-scrubjay) to replace the provisional *Addisco*. D-035 has the list of identifiers a rename touches.

## Summary

Scrubjay is local-first spaced repetition for engineers and CS students: **"Keep what you learn while you code"** (D-027).

- **Capture where learning happens:** `@card` comments and Markdown card files in the user's own repositories, the terminal, the editor, coding agents (MCP) and solutions repos.
- **Cards that stay correct:** anchored to the code they're about, and flagged when that code changes.
- **Review in the gaps of the day:** build waits, VS Code, later the phone.
- **Ready on the day:** problem practice by pattern and ready-by-date targets for exams and interviews.
- **FSRS scheduling with lossless Anki import.**

It ships first as a **CLI**, then a **VS Code extension**, then a **mobile review app** with Pro (Phase 2), then the web, all on one Rust core and one local database (D-034).

The original general study app (workflows, dashboards, presets, planner) is **frozen**, not deleted (D-028). Specs 03, 04 and 05 carry frozen banners; 07 (desktop) is dropped.

## Index

| File | Covers | Read when… |
|---|---|---|
| [00-vision.md](00-vision.md) | Problem, vision, goals, non-goals, principles, success metrics | You need the "why" or are judging scope |
| [01-users-and-use-cases.md](01-users-and-use-cases.md) | Personas (PER-9…13 active), jobs-to-be-done, user journeys (J15–J20, J3, J8 active) | Designing a flow or writing acceptance tests |
| [02-features.md](02-features.md) | Feature catalogue: requirement IDs, priorities, module status (active / frozen) | Planning work; checking whether something is in scope |
| [03-customization-model.md](03-customization-model.md) | **Frozen** except configuration files: base versions, presets, settings layering, document model | Touching configuration or the document format |
| [04-workflows.md](04-workflows.md) | **Frozen:** sessions, workflow definitions, step types | Only if unfreezing |
| [05-dashboards-and-widgets.md](05-dashboards-and-widgets.md) | **Frozen** except the metrics behind `scrubjay stats` and the heatmap (STAT-008) | Working on stats |
| [06-cli.md](06-cli.md) | Command tree, flags, output contracts, config, hooks, TUI, parity matrix | Working on the CLI |
| [07-desktop-and-ux.md](07-desktop-and-ux.md) | **Dropped** (D-034); its design-system and accessibility sections apply to the shared review UI | Working on the shared review UI's look or accessibility |
| [08-architecture.md](08-architecture.md) | Tech stack, components, repo layout, concurrency, sync, platform strategy | Making technical or structural changes |
| [09-data-model.md](09-data-model.md) | Entities, storage conventions, query language, file/export formats | Touching the database, search, import/export |
| [10-non-functional-requirements.md](10-non-functional-requirements.md) | Performance, reliability, privacy, security, a11y, i18n, compatibility | Setting budgets; reviewing quality |
| [11-roadmap.md](11-roadmap.md) | Phases, milestones, exit criteria, **current status** | Deciding what to do next |
| [12-glossary.md](12-glossary.md) | Canonical terminology | Naming things in code, UI or docs |
| [13-code-sources.md](13-code-sources.md) | Sources, `@card` syntax, Markdown card files, anchors and change detection, deck repositories, code-native cards, problem practice, VS Code extension, MCP server (SRC, CODE, PROB, EDIT) | Working on the developer features |
| [decisions.md](decisions.md) | Decision log (`D-NNN`) | Before proposing or changing an approach |
| [open-questions.md](open-questions.md) | Unresolved questions (`OQ-NNN`) with proposed defaults | Before assuming an answer |

### Process

| File | Covers |
|---|---|
| [../GIT_WORKFLOW.md](../GIT_WORKFLOW.md) | Commit format, scopes, branches, PRs, squash-merge, releases (D-021) |

### Supporting research

| File | Covers |
|---|---|
| [research/naming.md](research/naming.md) | 153-name study (Latin, Greek, Arabic, others): current shortlist, scoring, registry/GitHub/command checks, web conflicts (OQ-001) |
| [research/core-language-rust-vs-typescript.md](research/core-language-rust-vs-typescript.md) | Rust vs TypeScript for the core, with measured CLI startup (D-002) |
| [research/market.md](research/market.md) | Market and viability: landscape, gap, risks, illustrative revenue, recommendation to narrow and validate |
| [research/licensing.md](research/licensing.md) | Licence options (open source vs closed vs fair source): growth, users, profitability, retention, comparables (OQ-020) |
| [research/developer-pivot.md](research/developer-pivot.md) | Pivot to developers and CS students: evaluation, risks, scope audit (freeze/cut/promote), new requirements, roadmap. Decided (D-027…D-032) |
| [research/naming-round-4.md](research/naming-round-4.md) | Naming round 4: 58 names from nine angles, a six-criterion rubric (adds brand feel and breadth), finalists Muista, Cofio and Scrubjay. Awaiting decision (OQ-001, D-019) |
| [research/naming-round-5.md](research/naming-round-5.md) | Naming round 5, after the Scrubjay conflicts (a paid local-first app and a same-story developer tool): 145 names checked over RDAP plus a live-product search; chose Keepjay (D-037) |
| [research/developer-strategy.md](research/developer-strategy.md) | Developer strategy: the angle ("Keep what you learn while you code"), personas, needed and loved features, gaps in other tools, business model, platforms. Decided (D-027, D-033, D-034) |

## Conventions

### Status lifecycle

`Draft` → `In review` → `Accepted` → (`Superseded`). Each file states its status in its header.

### Identifiers

IDs are stable. Never reuse a retired ID.

| Prefix | Meaning | Example | Defined in |
|---|---|---|---|
| `<MODULE>-NNN` | Functional requirement | `SRS-005` | 02 (and detail files 03–07, 13) |
| `NFR-<AREA>-NNN` | Non-functional requirement | `NFR-PERF-002` | 10 |
| `Jn` | User journey | `J4` | 01 |
| `D-NNN` | Decision | `D-003` | decisions.md |
| `OQ-NNN` | Open question | `OQ-002` | open-questions.md |

Reference IDs in issues, commit messages, PR descriptions and code comments (e.g. `// implements WF-005`).

### Where requirements live

Each module's requirements are defined in **one place only**:

| Modules | File |
|---|---|
| WS, SRS, NOTE, PLAN, GOAL, QUIZ, RES, NOTIF, SRCH, IO, SYNC, WEB, MOB, SHARE, AI, ACCT | 02-features.md |
| CUST | 03-customization-model.md |
| SES, WF | 04-workflows.md |
| DASH, STAT | 05-dashboards-and-widgets.md |
| CLI | 06-cli.md |
| DESK (dropped) | 07-desktop-and-ux.md |
| SRC, CODE, PROB, EDIT | 13-code-sources.md |
| NFR-* | 10-non-functional-requirements.md |

### Priorities

| Priority | Meaning |
|---|---|
| **P0** | Required for the MVP (first public beta: CLI + VS Code extension) |
| **P1** | Planned for v1.x, soon after the MVP |
| **P2** | Later / ecosystem; may move to a later phase |

### Requirement language

MUST / SHOULD / MAY, as in RFC 2119.

### Changing the spec

1. If a decision changes, update `decisions.md` first: add a new entry and mark the old one `Superseded`.
2. Update the affected files and bump their *Last updated* date.
3. When an open question is answered, move it to **Resolved** in `open-questions.md` and link the decision it produced.
4. Keep each file focused (aim for under ~500 lines). Split rather than grow.
