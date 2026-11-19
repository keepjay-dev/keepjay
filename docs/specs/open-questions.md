# Open Questions

> **Status:** Draft · **Last updated:** 2026-11-19
> **Related:** [decisions](decisions.md) · [11-roadmap](11-roadmap.md)

Questions that need a product-owner answer. Each has a **proposed default**, which the spec currently assumes, so work can proceed. 🔴 = blocking the next milestone (M0.2/M0.3). When answered, move the question to **Resolved** with the date and the resulting decision ID.

## Open

| ID | Question | Options | Proposed default (assumed in spec) | Affects | Needed by |
|---|---|---|---|---|---|
| 🔴 OQ-001 | Product name, CLI binary name, URL scheme? | — | Working titles: **StudyApp**, `study`, `studyapp://` | Package/bundle ids, repo layout, docs | M0.3 |
| 🔴 OQ-002 | Primary target persona for the MVP? | University student · exam candidate · language learner · self-learner · all equally | **University student (PER-1)**, with terminal power users (PER-5) as secondary | Preset priority, onboarding, default dashboard, marketing | M0.2 |
| 🔴 OQ-003 | Who is building this, with what skills and time budget? | Solo · small team; Rust/TS/Swift/Kotlin experience | Assume a small team comfortable learning Rust + TypeScript | D-002, D-003, D-012, roadmap dates | M0.2 |
| 🔴 OQ-004 | Business model and licence? | Fully open source (MIT/Apache-2.0 or GPL/AGPL) · open core · freemium (paid sync/hosting) · one-time purchase | **Open-source core and apps; optional paid hosted sync later**; licence TBD | LICENSE file, contribution model, telemetry policy, sync hosting | M0.3 |
| OQ-005 | Sync approach and hosting (Phase 2)? | Custom oplog + server (08 §9) · cr-sqlite · Automerge for all · third-party sync service; hosted · self-hosted · both | Custom oplog, self-hostable server, plus optional hosted service | 08 §9, SYNC-* | Phase 2 start |
| OQ-006 | Mobile approach? | Native SwiftUI/Compose + UniFFI · Tauri 2 mobile · React Native + core | **Native** (matches "native mobile apps" in the brief; D-012) | Phase 3 effort, D-012 | Before Phase 3 |
| OQ-007 | AI features: in scope, and when? | None · Phase 4 opt-in · earlier; BYO key · local models · hosted | Phase 4, opt-in, BYO key + local models (AI-005) | AI-*, privacy NFRs | Phase 2 planning |
| OQ-008 | Anki import fidelity at MVP? | Content only · content + scheduling state · + full review history | Content + media at MVP; history in v1.0 (IO-001) | M2 scope | M1 |
| OQ-009 | Will there ever be an extension/plugin API? | Never · sandboxed WASM extensions (Phase 4) · scripting | Not in v1; evaluate sandboxed WASM in Phase 4 | D-007, 03 §9 | Phase 4 |
| OQ-010 | Collaboration and educator features? | None · shared decks/presets · study groups · classrooms | Share files in v1.x; gallery and groups in Phase 4 | SHARE-*, PER-6 | Phase 2 planning |
| OQ-011 | Which presets ship at the MVP? | Any subset of the 6 in 03 §4 | Essentials, Minimal, University at MVP; others in v1.0 | 03 §4, onboarding | M4 |
| OQ-012 | How much gamification? | Goals + forgiving streaks only · XP/levels/badges · leaderboards | **Goals + forgiving streaks only**, all toggleable (principle 6) | GOAL-*, UX tone | M3 |
| OQ-013 | Which integrations matter most? | Google/Outlook calendar · Canvas/Moodle/Google Classroom · Notion/Obsidian · Quizlet | iCal export (v1.0); others Phase 4 | PLAN-009/010, IO-007 | Phase 2 planning |
| OQ-014 | Multiple local profiles per OS user? | No · yes (profile switcher) | No in v1 (one profile per data dir; `--data-dir` for power users) | 09 `profile` | M1 |
| OQ-015 | Notes editor style? | Markdown source + live preview (CodeMirror) · WYSIWYG (TipTap) · both modes | Source + live preview | NOTE-001, 07 §3 | M3 |
| OQ-016 | CLI distribution? | Bundled with desktop only · standalone only · both | **Both** (desktop installer offers PATH install; standalone package managers) | 06 §8, 08 §14 | M5 |
| OQ-017 | Allow user-defined CLI hooks (shell commands on events)? | No · yes, local-only, opt-in | Yes, P2, local-only and opt-in, CLI only | CLI-016, D-007 | v1.x |
| OQ-018 | Minimum OS versions? | See NFR-COMP-001 | As in NFR-COMP-001 | Build matrix | M0.3 |
| OQ-019 | Opt-in telemetry during beta to measure success metrics? | None · opt-in anonymous product metrics · opt-in crash reports only | Opt-in crash reports + opt-in anonymous product metrics | 00 §6, NFR-PRIV-002 | M5 |

## Resolved

| ID | Question | Answer | Date | Decision |
|---|---|---|---|---|
| — | — | — | — | — |
