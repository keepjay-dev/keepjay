# Open Questions

> **Status:** Draft · **Last updated:** 2026-11-19
> **Related:** [decisions](decisions.md) · [11-roadmap](11-roadmap.md)

Questions that need a product-owner answer. Each has a **proposed default**, which the spec currently assumes, so work can proceed. 🔴 = blocking the next milestone (M0.2/M0.3). When answered, move the question to **Resolved** with the date and the resulting decision ID.

## Open

| ID | Question | Options | Proposed default (assumed in spec) | Affects | Needed by |
|---|---|---|---|---|---|
| 🔴 OQ-020 | Licence? (Split out of OQ-004.) | Open source permissive (MIT/Apache-2.0) · open source copyleft (AGPL/GPL) · source-available · proprietary/closed | **Apps open source (AGPL-3.0 or GPL-3.0), hosted services closed**, so forks can't simply resell the paid cloud. Alternative: closed source like many freemium study apps. | LICENSE file, contributions, OQ-021 | M0.3 |
| OQ-005 | Sync protocol (Phase 2)? | Custom oplog + server (08 §9) · cr-sqlite · Automerge for all · third-party sync service | Custom oplog with our own hosted server | 08 §9, SYNC-* | Phase 2 start |
| OQ-006 | Mobile approach? | Native SwiftUI/Compose + UniFFI · Tauri 2 mobile · React Native + core | **Native** (matches "native mobile apps" in the brief; D-012) | Phase 3 effort, D-012 | Before Phase 3 |
| OQ-008 | Anki import fidelity at MVP? | Content only · content + scheduling state · + full review history | Content + media at MVP; history in v1.0 (IO-001) | M2 scope | M1 |
| OQ-009 | Will there ever be an extension/plugin API? | Never · sandboxed WASM extensions (Phase 4) · scripting | Not in v1; evaluate sandboxed WASM in Phase 4 | D-007, 03 §9 | Phase 4 |
| OQ-010 | Collaboration and educator features? | None · shared decks/presets · study groups · classrooms / supervisor sharing | Share files in v1.x; gallery and groups in Phase 4 (possible paid features) | SHARE-*, PER-6, PER-7 | Phase 2 planning |
| OQ-011 | Which presets ship at the MVP? | Any subset of 03 §4 | Essentials, Minimal, University at MVP; `postgrad` with the academic-track features | 03 §4, onboarding | M4 |
| OQ-012 | How much gamification? | Goals + forgiving streaks only · XP/levels/badges · leaderboards | **Goals + forgiving streaks only**, all toggleable (principle 6) | GOAL-*, UX tone | M3 |
| OQ-013 | Which integrations matter most? | Zotero/BibTeX · Google/Outlook calendar · Canvas/Moodle · Notion/Obsidian · Quizlet | **Zotero/BibTeX** (academic track), then iCal export; others Phase 4 | RES-005, PLAN-009/010, IO-007 | Phase 2 planning |
| OQ-014 | Multiple local profiles per OS user? | No · yes (profile switcher) | No in v1 (one profile per data dir; `--data-dir` for power users) | 09 `profile` | M1 |
| OQ-015 | Notes editor style? | Markdown source + live preview (CodeMirror) · WYSIWYG (TipTap) · both modes | Source + live preview | NOTE-001, 07 §3 | M3 |
| OQ-016 | CLI distribution? | Bundled with desktop only · standalone only · both | **Both** (desktop installer offers PATH install; standalone package managers) | 06 §8, 08 §14 | M5 |
| OQ-017 | Allow user-defined CLI hooks (shell commands on events)? | No · yes, local-only, opt-in | Yes, P2, local-only and opt-in, CLI only | CLI-016, D-007 | v1.x |
| OQ-018 | Minimum OS versions? | See NFR-COMP-001 | As in NFR-COMP-001 | Build matrix | M0.3 |
| OQ-019 | Opt-in telemetry during beta to measure success metrics? | None · opt-in anonymous product metrics · opt-in crash reports only | Opt-in crash reports + opt-in anonymous product metrics | 00 §6, NFR-PRIV-002 | M5 |
| OQ-021 | With paid sync, can users still self-host sync for free? | Yes (self-host free, hosted paid) · no self-host · self-host as a paid licence | **Paid hosted sync only at launch**; revisit self-hosting later (it suits the academic audience but competes with revenue). SYNC-004 deferred. | SYNC-004, OQ-020 | Phase 2 planning |
| OQ-022 | AI in the paid tier: our hosted models only, or also bring-your-own-key / local models? | Hosted only · hosted + BYO key (free or discounted) · + local models | **Hosted, usage-limited AI in the paid plan; BYO key / local models as a later option** | AI-005…008, pricing | Phase 2 planning |
| OQ-023 | Which features are paid beyond sync and AI ("and more")? | Hosted backups · larger AI quotas · gallery/sharing extras · collaboration · premium presets/themes · advanced analytics | Only **hosted services** are paid (D-017); local features (incl. analytics, themes, presets) stay free | ACCT-002, D-017 | Phase 2 planning |
| OQ-024 | *(private)* | | | | |
| OQ-025 | Is the web app free or paid? | Free (local in browser) · free with sync paid · paid only | **Free** as a local-first app; cross-device persistence via paid sync | WEB-*, ACCT-002 | Phase 2 planning |

## Resolved

| ID | Question | Answer | Date | Decision |
|---|---|---|---|---|
| OQ-001 | Product name, CLI binary, URL scheme? | **Addisco** (`addisco` / `addi`, `addisco://`), provisional and open to change; pre-release trademark/domain checks pending | 2026-11-19 | D-019 |
| OQ-002 | Primary target persona for the MVP? | University students first, then postgraduates, then lifelong academics | 2026-11-19 | D-018 |
| OQ-003 | Team skills and language? | More familiar with TypeScript; prefers Rust for long-term optimisation. Analysis recommends Rust core + TS UI, confirmed by an M0.4 spike ([research](research/core-language-rust-vs-typescript.md)) | 2026-11-19 | D-002 (pending acceptance) |
| OQ-004 | Business model? | Free desktop + CLI first; then freemium with paid sync, AI chat with usage limits, and more. Licence split out into OQ-020. | 2026-11-19 | D-016 |
| OQ-007 | AI features: in scope, and when? | Yes: an AI study chat with usage limits as a paid feature of the freemium phase (Phase 2) | 2026-11-19 | D-016 |
