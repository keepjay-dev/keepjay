# Open Questions

> **Status:** Draft · **Last updated:** 2026-11-22
> **Related:** [decisions](decisions.md) · [11-roadmap](11-roadmap.md)

Questions that need a product-owner answer. Each has a **proposed default**, which the spec currently assumes, so work can proceed. 🔴 = blocking the next milestone. When answered, move the question to **Resolved** with the date and the resulting decision ID.

## Open

| ID | Question | Options | Proposed default (assumed in spec) | Affects | Needed by |
|---|---|---|---|---|---|
| OQ-005 | Sync protocol (Phase 2)? | Custom oplog + server (08 §9) · cr-sqlite · Automerge for all · third-party sync service | Custom oplog with our own server, **review-log-first** (D-034); the server is AGPL and self-hostable (D-033) | 08 §9, SYNC-* | M7 |
| OQ-006 | Mobile approach? | Tauri 2 mobile · native SwiftUI/Compose + UniFFI · React Native + core | **Tauri 2 mobile**, decided by a spike at the start of M8 (D-034); native only if widgets or notifications need it | M8 effort, D-034 | M8 |
| OQ-009 | Will there ever be an extension/plugin API? | Never · sandboxed WASM extensions (Phase 4) · scripting | Not in v1; shell hooks and JSON cover the MVP (D-030); evaluate sandboxed WASM later | D-007, D-030, 03 §9 | Phase 3 |
| OQ-010 | Collaboration and educator features? | None · deck repositories · creator decks · Teams | Deck repositories (SRC-011) in v1; creator decks and Teams later, Teams only on demand (D-033) | SRC-011, SHARE-*, PER-13 | Phase 2 planning |
| OQ-011 | Which presets ship at the MVP? | Any subset of 03 §4 · none | **None:** presets are frozen (D-028). The CLI creates a base `cli.toml`, and starter deck repositories take the place of presets. | 03 §4, onboarding | M5 |
| OQ-012 | How much gamification? | Heatmap + forgiving streaks only · XP/levels/badges · leaderboards | **Optional review heatmap (STAT-008) and forgiving streaks only**, all toggleable (principle 6) | STAT-008, GOAL-* | v1.0 |
| OQ-013 | Which integrations matter most? | LeetHub-style solutions repos · coding agents (MCP) · VS Code · JetBrains · browser · Zotero · calendars | Solutions repos, MCP and VS Code at the MVP (D-028, D-034); browser extension in Phase 2 if needed; JetBrains in Phase 3; Zotero and calendars frozen | PROB-005, AI-010, EDIT-* | M3 |
| OQ-014 | Multiple local profiles per OS user? | No · yes (profile switcher) | No in v1 (one profile per data dir; `--data-dir` for power users) | 09 `profile` | M1 |
| OQ-015 | Notes editor style? | Markdown source + live preview (CodeMirror) · WYSIWYG (TipTap) · both modes | Frozen with notes (D-028): notes live in the user's own editor | NOTE-001 | If notes are unfrozen |
| OQ-016 | CLI distribution? | Standalone only · bundled in the VS Code extension · both | **Both:** standalone (GitHub releases, Homebrew, Scoop/winget, AUR, Nix, `cargo install`) and bundled per platform in the VS Code extension. No desktop installer (D-034). | 06 §8, 08 §14 | M5 |
| OQ-018 | Minimum OS versions? | See NFR-COMP-001 | As in NFR-COMP-001 | Build matrix | M5 |
| OQ-019 | Opt-in telemetry during the alpha and beta to measure success metrics? | None · opt-in anonymous product metrics · opt-in crash reports only | **Opt-in anonymous weekly ping from the 0.2 alpha** (needed for the D-032 retention gate), plus opt-in crash reports | 00 §6, NFR-PRIV-002, D-032 | M0.6 |
| OQ-024 | *(private)* | | | | |
| OQ-025 | Is the web app free or paid? | Free (local in browser) · free with sync paid · paid only | **Free** as a local-first app; cross-device persistence via paid or self-hosted sync. The web app moves to Phase 3 (D-034). | WEB-*, ACCT-002 | Phase 3 planning |
| OQ-027 | Keep AGPL for the CLI if teams become the market? | AGPL · MPL-2.0 · Apache-2.0 | Keep AGPL unless Teams goes ahead. Some companies restrict AGPL software internally; D-020 allows changes that are more open. | D-020, PER-13 | Before Teams |
| OQ-028 | Write card ids into source comments and files? | Always · opt-out per source · never | **Opt-out per source** (`--no-write-ids`). Without ids, match by anchor and text similarity, and ask when unsure. | SRC-005, D-029 | M0.5 |
| OQ-030 | Which four tree-sitter languages at the MVP? | Any four | **Python, TypeScript/JavaScript, Rust, Go**; revisit if the M0.6 interviews favour Java or C++, which are common in CS courses | CODE-002 | M2 |

## Resolved

| ID | Question | Answer | Date | Decision |
|---|---|---|---|---|
| OQ-029 | Which editor integration first? | **VS Code** (76% of developers use it); a Neovim plugin stays thin and community-led | 2026-11-22 | D-034 |
| OQ-026 | What does Pro sell to developers? | Hosted sync (what makes phone review easy), hosted backups and hosted AI drafting; supporter licence; interview-season pass; creator decks and Teams later | 2026-11-22 | D-033 |
| OQ-021 | With paid sync, can users still self-host sync for free? | **Yes:** the sync server is AGPL and self-hostable; hosted sync is the paid convenience | 2026-11-22 | D-033 |
| OQ-017 | Allow user-defined CLI hooks (shell commands on events)? | **Yes, P0:** local, opt-in, CLI only (CLI-016) | 2026-11-22 | D-030 |
| OQ-001 | Product name, CLI binary, URL scheme? | **Scrubjay** (`scrubjay` / `sj`, `scrubjay://`) after naming rounds 3–4, replacing the provisional Addisco (2026-11-19, D-019); trademark and domain checks pending | 2026-11-22 | D-035 |
| OQ-008 | Anki import fidelity at MVP? | Content, media **and full review history** at MVP (switching costs are the main barrier) | 2026-11-19 | D-022 (still applies under D-028) |
| OQ-023 | Which features are paid beyond sync? | Pro = hosted sync + hosted backups + academic features, student discount; local features stay free; hosted AI chat optional later | 2026-11-19 | D-023 (Pro redefined by D-033) |
| OQ-022 | AI in the paid tier? | AI is a complement: MCP server, AI imports and bring-your-own-key supported; hosted AI chat is optional and later | 2026-11-19 | D-023 (hosted chat cut, hosted drafting added by D-033) |
| OQ-020 | Licence? | Option A: AGPL-3.0-or-later core and apps; proprietary hosted services; CLA, trademark, MIT/CC0 formats and presets, CC BY docs ([research](research/licensing.md)) | 2026-11-19 | D-020 (sync server made AGPL by D-033) |
| OQ-002 | Primary target persona for the MVP? | University students first, then postgraduates, then lifelong academics | 2026-11-19 | D-018 (superseded by D-027: developers and CS students) |
| OQ-003 | Team skills and language? | More familiar with TypeScript; prefers Rust for long-term optimisation. Analysis recommends Rust core + TS UI, confirmed by an M0.4 spike ([research](research/core-language-rust-vs-typescript.md)) | 2026-11-19 | D-002 (accepted) |
| OQ-004 | Business model? | Free desktop + CLI first; then freemium with paid sync, AI chat with usage limits, and more. Licence split out into OQ-020. | 2026-11-19 | D-016 (amended by D-023, D-033) |
| OQ-007 | AI features: in scope, and when? | Yes: an AI study chat with usage limits as a paid feature of the freemium phase (Phase 2) | 2026-11-19 | D-016, amended by D-023 and D-033 (hosted chat cut; hosted drafting in Pro) |
