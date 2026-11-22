# CLAUDE.md

Local-first spaced repetition for engineers and CS students (**Addisco**, provisional name — D-019; CLI `addisco`, alias `addi`): "Keep what you learn while you code" (D-027).

- **Surfaces (D-034):** CLI first, then a VS Code extension, then a mobile review app (Phase 2), then web, all on one shared Rust core. No standalone desktop app.
- **Audience:** enthusiast engineers → interview candidates and CS students → engineers learning on the job.
- **Business model (D-033):** local features free forever; Pro = hosted sync, backups and AI drafting.
- **Current phase: 0.** The core and CLI alpha exist (FSRS review, Anki import with history, tags, media). Next is the M0.5 developer slice.
- **Frozen (D-028):** workflows, dashboards, presets, notes and the planner (specs 03, 04, 05). 07 is dropped. Don't build frozen features without a new decision.

## Start here (keep context small)

1. Read `docs/specs/11-roadmap.md` → **Current status** table: what phase and milestone we're in and what's next.
2. Use the index in `docs/specs/README.md` to open **only** the spec files relevant to the task.
3. Before proposing an approach, check `docs/specs/decisions.md`. Before assuming an answer, check `docs/specs/open-questions.md` (the spec assumes each question's *proposed default*).

## Spec map (one line each)

| File | Use for |
|---|---|
| `00-vision` | Goals, non-goals, principles |
| `01-users-and-use-cases` | Personas (PER-9…13 active), journeys J15–J20 plus J3, J8 (acceptance tests) |
| `02-features` | Requirement catalogue with module status (WS, SRS, NOTE, PLAN, GOAL, QUIZ, RES, NOTIF, SRCH, IO, SYNC, WEB, MOB, SHARE, AI, ACCT) |
| `03-customization-model` | Frozen except config files (CUST) |
| `04-workflows` | Frozen: sessions and the workflow engine (SES, WF) |
| `05-dashboards-and-widgets` | Frozen except stats metrics and the heatmap (DASH, STAT) |
| `06-cli` | CLI commands, output/exit codes, `cli.toml`, hooks, TUI (CLI) |
| `07-desktop-and-ux` | Dropped; design system and a11y apply to the shared review UI (DESK) |
| `08-architecture` | Stack, core modules, repo layout, multi-process, sync design |
| `09-data-model` | Entities, query language, archive format, paths |
| `10-non-functional-requirements` | Perf/reliability/privacy/security/a11y budgets (NFR-*) |
| `12-glossary` | Canonical terms (e.g. *fact* ≠ *note*, *source* ≠ `facts.source`) |
| `13-code-sources` | Sources, `@card` syntax, anchors, change detection, deck repos, code-native cards, problem practice, VS Code extension, MCP (SRC, CODE, PROB, EDIT) |
| `research/` | Naming study; Rust vs TypeScript; licensing options; market & viability; developer pivot and strategy |

## Conventions

- Git: follow `docs/GIT_WORKFLOW.md` (D-021): `<type>(<scope>): <summary>` commits, `<type>/<topic>` branches, squash-merge; PR title is the commit message.
- Requirement IDs `<MODULE>-NNN`, `NFR-<AREA>-NNN`; decisions `D-NNN`; open questions `OQ-NNN`; journeys `Jn`. Reference them in commits, PRs and code comments.
- Priorities: P0 = MVP, P1 = v1.x, P2 = later.
- Use glossary terms in code and UI.
- When changing the spec: decisions first, then affected files; bump *Last updated*; update the roadmap status table when a milestone moves.
- Keep spec files focused (< ~500 lines). Split rather than grow.
