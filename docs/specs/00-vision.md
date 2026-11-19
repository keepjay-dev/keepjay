# 00 — Vision & Goals

> **Status:** Draft · **Version:** 0.2 · **Last updated:** 2026-11-19
> **Related:** [01-users-and-use-cases](01-users-and-use-cases.md) · [11-roadmap](11-roadmap.md) · [decisions](decisions.md)

## 1. Problem

Serious learners stitch together five or more tools: a flashcard app (spaced repetition), a notes app, a Pomodoro timer, a planner or calendar for deadlines and exams, and a spreadsheet or habit tracker for progress. The result:

- **Fragmentation.** Constant context switching, duplicated data, and no single picture of progress.
- **Rigid tools or blank canvases.** Dedicated study apps impose one way of studying. General tools (Notion-style) can do anything, but they take hours to set up and maintain, and they know nothing about learning: no scheduler, no session tracking.
- **Terminal users left out.** Developers and power users who live in the terminal have no first-class study tooling.
- **Lock-in and privacy.** Study history (often years of review logs) is trapped in proprietary clouds.

## 2. Vision

> A study environment that adapts to how each person learns. Start from a proven base setup in minutes, then shape every workflow, dashboard and habit to fit, on any device, from a GUI or the terminal, with data you own.

**Audience path (D-018):** university students first (MVP), then postgraduate researchers, then lifelong academics. The app should grow with a person from first-year undergrad to professor.

**Positioning (D-023):** *Your learning, for decades, on your machine.* Addisco is the memory and plan layer that works with the AI tools students already use: they help you understand, Addisco makes it stick.

## 3. Goals

| # | Goal | Measured by |
|---|---|---|
| G1 | **All-in-one study workspace**: spaced-repetition flashcards, notes, timed sessions, planning, goals and analytics in one place | Core study loop needs no external tool |
| G2 | **Quick start** via base versions and presets | Median time from first launch to first started session < 2 min |
| G3 | **Deep but bounded customisation** of workflows, dashboards, card types and settings, with no code | ≥ 40% of week-1 retained users modify or create ≥ 1 workflow or dashboard |
| G4 | **First-class CLI** with parity on the core study loop | Add → review → session → stats fully doable from the CLI |
| G5 | **Local-first and private**: fully functional offline; data exportable in open formats | All features except sync/updates/AI work offline; lossless round-trip export |
| G6 | **One core, every platform**: desktop + CLI → web → native mobile with identical behaviour | Scheduling, workflows and metrics run from the shared core on every platform |

## 4. Non-goals (for now)

- **General PKM replacement.** Notes exist to serve studying. We won't chase Obsidian/Notion feature parity.
- **Teacher/classroom management or LMS replacement.** Integrations may come later (OQ-013).
- **Social network or public feeds.**
- **User-authored code** (scripts, custom widgets, plugins) in v1. Customisation is declarative and bounded (D-007, OQ-009).
- **Real-time multi-user collaborative editing** in v1 (OQ-010).

## 5. Product principles

1. **Opinionated defaults, generous remix.** Every install starts with a working, evidence-based setup. Every part of it can be changed using the provided building blocks.
2. **Customisation is data.** Anything a user can customise is a versioned, validated, portable document. It can be exported, shared, diffed and reset to its base version (D-006).
3. **One core, many shells.** Behaviour (scheduling, workflow execution, metrics, validation) is implemented once in the core. UIs are thin (D-002).
4. **Local-first.** The network is an enhancement, never a requirement (D-001).
5. **Keyboard-first, mouse-friendly, accessible.** Every action is reachable from the keyboard and the command palette. Target WCAG 2.2 AA.
6. **Respect attention.** No dark patterns or guilt mechanics. Streaks and gamification are optional and forgiving.
7. **Evidence-based learning by default.** Spaced repetition (FSRS), active recall, interleaving and retrieval practice are the defaults, not add-ons.
8. **Never lose study history.** Review logs and session events are append-only, backed up automatically, and exportable.

## 6. Success metrics (initial)

| Area | Metric | Target (beta) |
|---|---|---|
| Activation | Median time to first session | < 2 min |
| Activation | % of new users completing a first session on day 1 | ≥ 60% |
| Engagement | D7 / D30 retention | ≥ 40% / ≥ 25% |
| Customisation | % of active users with ≥ 1 customised workflow or dashboard | ≥ 40% |
| CLI | % of weekly active users who use the CLI | Track only (no target yet) |
| Quality | Crash-free sessions | ≥ 99.5% |
| Trust | Data-loss incidents | 0 |

> Product metrics depend on opt-in telemetry (NFR-PRIV-002). Targets are provisional.

## 7. Scope by phase (summary)

| Phase | Platforms | Headline |
|---|---|---|
| 0 | — | Specification, decisions, repo and core skeleton |
| 1 | Desktop (Windows/macOS/Linux) + CLI | **Free.** Full local study loop, presets, workflow and dashboard customisation |
| 2 | + Web | **Freemium launch:** optional accounts, paid sync (E2EE) and AI study chat (usage-limited), free web app; academic-track features |
| 3 | + iOS, Android | Native mobile apps: quick review, widgets, reminders, capture |
| 4 | All | Ecosystem: sharing gallery, collaboration, extensions, more AI and paid services |

Details: [11-roadmap.md](11-roadmap.md).

## 8. Business model (D-016, D-017)

- **Phase 1:** desktop app and CLI are **free**.
- **Phase 2 onwards: freemium.** *Free* includes everything that runs on the user's device, on every platform. *Pro* (paid) covers hosted services and academic features: multi-device sync, hosted backups, and academic features such as the reference library and projects, with a student discount (D-023). A hosted AI chat may come later as an optional add-on.
- **Promise:** your study data is always yours. Local features never require an account, and stopping payment never locks or deletes your data (ACCT-004).
- **Licence (D-020):** the core and apps are open source (AGPL-3.0-or-later); hosted services are proprietary; formats and presets are permissive (MIT/CC0). The "Addisco" name is a trademark.
- Open: self-hosted sync (OQ-021), AI bring-your-own-key (OQ-022), pricing (OQ-024).
