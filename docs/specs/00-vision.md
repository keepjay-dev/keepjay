# 00 — Vision & Goals

> **Status:** Draft · **Version:** 0.3 · **Last updated:** 2026-11-22
> **Related:** [01-users-and-use-cases](01-users-and-use-cases.md) · [11-roadmap](11-roadmap.md) · [decisions](decisions.md) · [research/developer-strategy](research/developer-strategy.md)

## 1. Problem

Engineers and CS students learn constantly: new languages and frameworks, an unfamiliar codebase, data structures and algorithms for exams and interviews, lessons from debugging, answers from AI assistants. Most of it fades within weeks.

- **AI makes forgetting easier.** In a 2026 randomised trial, developers who learned a new library with AI help scored 50% on a follow-up quiz, against 67% for those who coded by hand ([Anthropic](https://www.anthropic.com/research/AI-assistance-coding-skills)). The gap was biggest on debugging.
- **Spaced repetition works, but it lives outside the workflow.** Anki and its peers need a separate app, cards written by hand, and time set aside. Writing cards is the main chore, and a growing backlog is a common reason people quit.
- **Nothing knows about code.** Cards about code go silently wrong when the code changes. Problem practice (LeetCode and similar) is tracked in browser extensions that know nothing about the concepts behind it.
- **Lock-in.** Study history, often years of review logs, is trapped in proprietary clouds or binary databases.

## 2. Vision

> **Keep what you learn while you code.** AI writes more of the code; you still have to understand it. Scrubjay captures what you learn (from your code, your coding agent and your problem practice) and brings it back in the gaps of your day until it sticks.

**Audience (D-027):** three rings, in this order:

1. **Enthusiast engineers** (PER-9): the free launch community.
2. **Interview candidates and CS students** (PER-10, PER-11): the volume and the first revenue.
3. **Engineers learning on the job** (PER-12): the expansion.

Teams (PER-13) come later, and only on demand. The general study audience (PER-1…8, D-018) is frozen, not abandoned: the core stays domain-neutral.

**Positioning:** spaced repetition that lives where developers already work (terminal, editor, Git, coding agent, phone) and works with AI tools rather than competing with them (D-023).

## 3. Goals

| # | Goal | Measured by |
|---|---|---|
| G1 | **Capture where learning happens:** code comments, Markdown files, the terminal, the editor, coding agents and solutions repos | Median time to create a card from code < 10 s; ≥ 50% of new cards come from sources or agents rather than being typed in |
| G2 | **Review in the gaps:** build and test waits, the editor, the phone | Share of reviews done through `while`, `--quick`, the editor or mobile (tracked) |
| G3 | **Cards that stay correct:** anchored to code and flagged when it changes | Median time to resolve a stale card < 7 days |
| G4 | **Ready on the day:** exams and interviews by a target date | Predicted retention on the target date matches measured retention |
| G5 | **Local-first, plain text, open:** works offline; content in text files; history exportable | All local features work offline; lossless export; zero data-loss incidents |
| G6 | **One core, every surface:** the CLI, VS Code, mobile and the web share the Rust core | Scheduling and anchors behave identically on every surface |

## 4. Non-goals (for now)

- **A general study app for every learner.** Frozen, along with specs 03–05 (D-027, D-028).
- **A notes app or PKM tool.** Notes live in the user's editor and repos. We read Markdown; we don't replace Obsidian or Notion.
- **Workflow engine, dashboards, planner.** Frozen.
- **A standalone desktop app.** Dropped (D-034); VS Code, the TUI and later the web app cover it.
- **Executable cards and embedded scripting.** Cards never run code (D-013, D-030; CODE-004 is P2).
- **Copying problem content** from LeetCode or similar sites. Only links, tags and the user's own solutions.
- **Reporting individuals' learning to managers.** Teams, if they ever come, see aggregate progress only.
- **Real-time collaborative editing.** Collaboration happens through Git (deck repositories).

## 5. Product principles

1. **Meet developers where they work.** Terminal, editor, Git, coding agent and phone, before any new app.
2. **Text is the source; the database is state.** Card content from sources lives in plain files the user controls. Scheduling and history live in SQLite (D-029).
3. **One core, many shells.** Behaviour is implemented once in the Rust core; shells are thin (D-002).
4. **Local-first.** The network is an enhancement, never a requirement (D-001).
5. **Fast and keyboard-first.** Commands answer in under 100 ms; every action has a key.
6. **Respect attention.** No dark patterns or guilt mechanics. Heatmaps and streaks are optional and forgiving.
7. **Evidence-based by default.** FSRS, active recall and retrieval practice. Cards about concepts and reasoning, not trivia an AI can autocomplete.
8. **Never lose study history.** Review logs are append-only, backed up automatically, exportable, and survive card edits and moves (D-029).
9. **AI drafts, you decide.** AI proposes cards; nothing is saved or sent without the user's approval (AI-005).

## 6. Success metrics (initial)

| Area | Metric | Target (beta) |
|---|---|---|
| Activation | Median time from install to the first review of cards from your own repo or a deck (J15) | < 2 min |
| Activation | % of new users who finish a first review on day 1 | ≥ 60% |
| Engagement | D7 / D30 retention | ≥ 40% / ≥ 25% |
| Capture | % of new cards that come from sources, agents or solutions repos | ≥ 50% |
| Gaps | % of reviews done through `while`, `--quick`, the editor or mobile | Track only |
| Quality | Crash-free sessions | ≥ 99.5% |
| Trust | Data-loss incidents | 0 |

> Product metrics depend on opt-in telemetry (NFR-PRIV-002, OQ-019). Targets are provisional.

## 7. Scope by phase (summary)

| Phase | Surfaces | Headline |
|---|---|---|
| 0 | CLI | Foundations (done); the developer slice (M0.5); the validation gate (M0.6, D-032) |
| 1 | CLI + VS Code | **Free developer MVP:** sources and anchors, code-native cards, composability and MCP, interview mode and ready-by-date → public beta |
| 2 | + mobile review app (+ browser extension if needed) | **Pro launch:** accounts, review-log sync, hosted backups, hosted AI drafting |
| 3 | + web, JetBrains | Creator decks, Teams on demand, the general study audience (unfreeze) |

Details: [11-roadmap.md](11-roadmap.md).

## 8. Business model (D-033, D-017)

- **Free forever:** everything that runs on the user's device, on every surface. That includes interview mode, ready-by-date, bring-your-own-key and local-model AI, and self-hosted sync. No account needed.
- **Supporter:** GitHub Sponsors from the 0.2 alpha; a one-time supporter licence later. It adds no features.
- **Pro** (Phase 2; prices to test in OQ-024: monthly and yearly plans, a student discount, a 3-month interview-season pass): hosted sync (the easy way to review on a phone), hosted backups, and hosted AI drafting within a quota.
- **Later:** creator decks with a revenue share; Teams, only on demand.
- **Promise:** your study data is always yours. Local features never need an account, and stopping payment never locks or deletes your data (ACCT-004).
- **Licence (D-020, amended by D-033):** the core, the apps and the sync server are AGPL-3.0-or-later. Accounts and billing, the entitlement issuer and the AI gateway are proprietary. Formats and presets are permissive. "Scrubjay" is a trademark.
