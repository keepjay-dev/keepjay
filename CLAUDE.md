# CLAUDE.md

Cross-platform, highly customisable study app (**Addisco**, provisional name — D-019; CLI `addisco`, alias `addi`). Desktop app + CLI first, then web, then native mobile, all on one shared Rust core. Audience: university students → postgrads → lifelong academics. Free at launch, then freemium (paid = hosted sync, AI chat). **Current phase: 0 (specification). No code yet.**

## Start here (keep context small)

1. Read `docs/specs/11-roadmap.md` → **Current status** table: what phase and milestone we're in and what's next.
2. Use the index in `docs/specs/README.md` to open **only** the spec files relevant to the task.
3. Before proposing an approach, check `docs/specs/decisions.md`. Before assuming an answer, check `docs/specs/open-questions.md` (the spec assumes each question's *proposed default*).

## Spec map (one line each)

| File | Use for |
|---|---|
| `00-vision` | Goals, non-goals, principles |
| `01-users-and-use-cases` | Personas, journeys J1–J14 (acceptance tests) |
| `02-features` | Requirement catalogue (WS, SRS, NOTE, PLAN, GOAL, QUIZ, RES, NOTIF, SRCH, IO, SYNC, WEB, MOB, SHARE, AI, ACCT) |
| `03-customization-model` | Base versions, presets, settings layers, document format, limits (CUST) |
| `04-workflows` | Sessions and the workflow engine (SES, WF) |
| `05-dashboards-and-widgets` | Dashboards, widgets, metrics (DASH, STAT) |
| `06-cli` | CLI commands, output/exit codes, `cli.toml`, TUI (CLI) |
| `07-desktop-and-ux` | Desktop screens, onboarding, design system, a11y (DESK) |
| `08-architecture` | Stack, core modules, repo layout, multi-process, sync design |
| `09-data-model` | Entities, query language, archive format, paths |
| `10-non-functional-requirements` | Perf/reliability/privacy/security/a11y budgets (NFR-*) |
| `12-glossary` | Canonical terms (e.g. *fact* ≠ *note*) |
| `research/` | Naming study; Rust vs TypeScript; licensing options |

## Conventions

- Requirement IDs `<MODULE>-NNN`, `NFR-<AREA>-NNN`; decisions `D-NNN`; open questions `OQ-NNN`; journeys `Jn`. Reference them in commits, PRs and code comments.
- Priorities: P0 = MVP, P1 = v1.x, P2 = later.
- Use glossary terms in code and UI.
- When changing the spec: decisions first, then affected files; bump *Last updated*; update the roadmap status table when a milestone moves.
- Keep spec files focused (< ~500 lines). Split rather than grow.
