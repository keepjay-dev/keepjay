# Addisco — Project Specification

> **Status:** Draft · **Version:** 0.2 · **Last updated:** 2026-11-19

This folder is the source of truth for **what** we are building and **why**, plus the proposed **how**. It is split into focused files so that a person (or an AI assistant) can load only the file relevant to the current task instead of re-reading everything.

> Product name **Addisco** (CLI `addisco`, short alias `addi`, URL scheme `addisco://`) is **provisional** and may change: see [D-019](decisions.md#d-019--product-name-addisco-provisional) for the list of identifiers a rename touches.

## Summary

Addisco is a local-first, highly customisable study environment. Users start from a ready-made **base setup** (chosen from presets in under two minutes) and then reshape their **workflows** (sequences of focus, review, break and reflection steps), **dashboards** (grids of configurable widgets), card types, goals and settings. All of this happens within a bounded, validated set of building blocks, with no code required. It ships first as a **desktop app plus a first-class CLI** that share one Rust core and one local database. It then expands to a **web app** (with optional sync) and **native mobile apps**.

## Index

| File | Covers | Read when… |
|---|---|---|
| [00-vision.md](00-vision.md) | Problem, vision, goals, non-goals, principles, success metrics | You need the "why" or are judging scope |
| [01-users-and-use-cases.md](01-users-and-use-cases.md) | Personas, jobs-to-be-done, key user journeys (J1–J14) | Designing a flow or writing acceptance tests |
| [02-features.md](02-features.md) | Feature catalogue: requirement IDs and priorities | Planning work; checking whether something is in scope |
| [03-customization-model.md](03-customization-model.md) | Base versions, presets, settings layering, document model, limits | Touching anything a user can configure |
| [04-workflows.md](04-workflows.md) | Sessions, workflow definitions, step types, runtime, base workflows | Working on study sessions / the workflow engine |
| [05-dashboards-and-widgets.md](05-dashboards-and-widgets.md) | Dashboard layout, widget catalogue, metrics, analytics | Working on dashboards, stats, widgets, goals progress |
| [06-cli.md](06-cli.md) | Command tree, flags, output contracts, config, TUI, parity matrix | Working on the CLI |
| [07-desktop-and-ux.md](07-desktop-and-ux.md) | Navigation, key screens, onboarding, design system, accessibility | Working on the desktop UI or UX |
| [08-architecture.md](08-architecture.md) | Tech stack, components, repo layout, concurrency, sync, platform strategy | Making technical or structural changes |
| [09-data-model.md](09-data-model.md) | Entities, storage conventions, query language, file/export formats | Touching the database, search, import/export |
| [10-non-functional-requirements.md](10-non-functional-requirements.md) | Performance, reliability, privacy, security, a11y, i18n, compatibility | Setting budgets; reviewing quality |
| [11-roadmap.md](11-roadmap.md) | Phases, milestones, exit criteria, **current status** | Deciding what to do next |
| [12-glossary.md](12-glossary.md) | Canonical terminology | Naming things in code, UI or docs |
| [decisions.md](decisions.md) | Decision log (`D-NNN`) | Before proposing or changing an approach |
| [open-questions.md](open-questions.md) | Unresolved questions (`OQ-NNN`) with proposed defaults | Before assuming an answer |

### Supporting research

| File | Covers |
|---|---|
| [research/naming.md](research/naming.md) | 153-name study (Latin, Greek, Arabic, others): current shortlist, scoring, registry/GitHub/command checks, web conflicts (OQ-001) |
| [research/core-language-rust-vs-typescript.md](research/core-language-rust-vs-typescript.md) | Rust vs TypeScript for the core, with measured CLI startup (D-002) |
| [research/licensing.md](research/licensing.md) | Licence options (open source vs closed vs fair source): growth, users, profitability, retention, comparables (OQ-020) |

## Conventions

### Status lifecycle

`Draft` → `In review` → `Accepted` → (`Superseded`). Each file states its status in its header.

### Identifiers

IDs are stable. Never reuse a retired ID.

| Prefix | Meaning | Example | Defined in |
|---|---|---|---|
| `<MODULE>-NNN` | Functional requirement | `SRS-005` | 02 (and detail files 03–07) |
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
| DESK | 07-desktop-and-ux.md |
| NFR-* | 10-non-functional-requirements.md |

### Priorities

| Priority | Meaning |
|---|---|
| **P0** | Required for the MVP (first public beta of desktop + CLI) |
| **P1** | Planned for v1.x, soon after the MVP |
| **P2** | Later / ecosystem; may move to a later phase |

### Requirement language

MUST / SHOULD / MAY, as in RFC 2119.

### Changing the spec

1. If a decision changes, update `decisions.md` first: add a new entry and mark the old one `Superseded`.
2. Update the affected files and bump their *Last updated* date.
3. When an open question is answered, move it to **Resolved** in `open-questions.md` and link the decision it produced.
4. Keep each file focused (aim for under ~500 lines). Split rather than grow.
