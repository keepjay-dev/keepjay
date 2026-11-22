# Decision Log

> **Status:** Draft · **Last updated:** 2026-11-22
> **Related:** [open-questions](open-questions.md) · [08-architecture](08-architecture.md)

A lightweight ADR log. Each decision has a stable ID. To change a decision, add a **new** entry and mark the old one `Superseded by D-NNN`. Never edit history.

**Statuses:** `Proposed` (awaiting product-owner review) · `Accepted` · `Superseded` · `Rejected`

## Summary

| ID | Decision | Status | Date |
|---|---|---|---|
| D-001 | Local-first architecture; sync is optional | Accepted | 2026-11-19 |
| D-002 | One shared Rust core for every platform | Accepted | 2026-11-19 |
| D-003 | Desktop shell: Tauri 2 + TypeScript/React UI | Superseded by D-034 (desktop app dropped; Tauri becomes the mobile shell) | 2026-11-19 |
| D-004 | SQLite everywhere (WAL, FTS5; SQLite-WASM on web) | Accepted | 2026-11-19 |
| D-005 | FSRS as the default scheduler behind a pluggable interface | Accepted | 2026-11-19 |
| D-006 | Customisation is data: versioned, schema-validated documents | Accepted | 2026-11-19 |
| D-007 | Bounded, declarative customisation; no user code in v1 | Accepted (amended by D-030: opt-in local shell hooks) | 2026-11-19 |
| D-008 | Built-in presets/base versions use the same format as user documents | Accepted | 2026-11-19 |
| D-009 | Sync-ready data model from day one (UUIDv7, HLC, tombstones, oplog) | Accepted | 2026-11-19 |
| D-010 | Desktop and CLI open the DB directly; no background daemon in v1 | Accepted | 2026-11-19 |
| D-011 | Phase 1 build order: core → CLI → desktop | Superseded by D-031 | 2026-11-19 |
| D-012 | Mobile: native UI (SwiftUI/Compose) over the core via UniFFI | Superseded by D-034 (Tauri 2 mobile after a spike; native only if needed) | 2026-11-19 |
| D-013 | Content rendering is sanitised; no scripts in card templates or notes | Accepted | 2026-11-19 |
| D-014 | Monorepo: Cargo workspace + pnpm workspace | Accepted | 2026-11-19 |
| D-015 | `docs/specs/` is the source of truth, with stable requirement/decision IDs | Accepted | 2026-11-19 |
| D-016 | *(private)* | Private | — |
| D-017 | Local features are free forever; paid features are gated by entitlements checked in the core, offline-tolerant | Accepted | 2026-11-19 |
| D-018 | Audience path: university students (MVP) → postgraduates → lifelong academics | Superseded by D-027 | 2026-11-19 |
| D-019 | Product name: **Addisco** (provisional; open to change) | Superseded by D-035 | 2026-11-19 |
| D-022 | MVP narrowed to the wedge: SRS (FSRS, lossless Anki import with history) + sessions/workflows + exam planning + customisable Home dashboard + CLI | Superseded by D-028 | 2026-11-19 |
| D-023 | *(private)* | Private | — |
| D-024 | *(private)* | Private | — |
| D-026 | The content-addressed media store is P0 core infrastructure (Anki media, IO-001); attachments and the resource library (RES-001/002) stay P1. Fields refer to media by name | Accepted | 2026-11-19 |
| D-021 | Git workflow: Conventional Commits with fixed scopes, squash-merge only, protected `main`, PR title checked in CI | Accepted | 2026-11-19 |
| D-020 | Licence: AGPL-3.0-or-later core and apps; closed hosted services; CLA, trademark, permissive formats | Accepted (amended by D-033: the sync server becomes AGPL and self-hostable) | 2026-11-19 |
| D-027 | Audience: developers, CS students and technical learners, in three rings; "Keep what you learn while you code" | Accepted | 2026-11-22 |
| D-028 | MVP wedge: code-anchored, text-native spaced repetition inside the developer workflow | Accepted | 2026-11-22 |
| D-029 | Text files own the content of cards from sources; SQLite owns scheduling state and history | Accepted | 2026-11-22 |
| D-030 | Extensibility: opt-in local shell hooks and JSON events; no embedded scripting | Accepted | 2026-11-22 |
| D-031 | Build order: core → CLI → VS Code extension → mobile review app | Accepted | 2026-11-22 |
| D-032 | *(private)* | Private | — |
| D-033 | *(private)* | Private | — |
| D-034 | Platforms: CLI → VS Code → mobile (Tauri 2) → browser extension → web → JetBrains; one shared review UI; no desktop app | Accepted | 2026-11-22 |
| D-035 | Product name: **Scrubjay** (CLI `scrubjay`, alias `sj`), replacing Addisco | Accepted (pending trademark search) | 2026-11-22 |

---

### D-001 — Local-first architecture; sync is optional
- **Context:** Users need offline study, data ownership and privacy (G5). Desktop + CLI ship before any server exists.
- **Decision:** Every device holds a complete local database and works fully offline. Accounts and sync (Phase 2) are optional add-ons.
- **Alternatives:** Cloud-first with offline cache (simpler multi-device, but needs a server for MVP and weakens privacy).
- **Consequences:** Must design for later merge (D-009). Web app needs in-browser storage (D-004).

### D-002 — One shared Rust core for every platform
- **Context:** Six surfaces over time (desktop, CLI, web, iOS, Android, sync server) must behave identically (G6).
- **Decision:** Implement all domain behaviour in a Rust core exposed through one facade (`core-api`). Shells are thin.
- **Alternatives:** TypeScript core (one language with the UI, but slower CLI start, harder native mobile); Kotlin Multiplatform (strong mobile, weaker CLI/desktop/web); per-platform logic (drift).
- **Consequences:** Team needs Rust skills (OQ-003: the team knows TypeScript better). FFI/WASM boundaries need generated types. Fast native CLI.
- **Status:** Accepted by the product owner (2026-11-19). The UI stays TypeScript/React (D-003, still proposed). The M0.4 Rust vertical-slice spike remains as a velocity check. If it shows Rust velocity is unacceptable, a superseding decision (TypeScript everywhere) must be made **before M1**.
- **Analysis:** [research/core-language-rust-vs-typescript.md](research/core-language-rust-vs-typescript.md) compares both stacks for this project, with measured CLI startup times. It recommends keeping this decision (Rust core + TS UI), validated by a Rust vertical-slice spike in M0.4, with a TypeScript-everywhere fallback decided before M1.

### D-003 — Desktop shell: Tauri 2 + TypeScript/React UI
- **Context:** Need cross-platform desktop with a rich, customisable UI that can be reused for the web.
- **Decision:** Tauri 2 (Rust backend links the core directly) with a React + TypeScript + Vite frontend in a shared `packages/ui`.
- **Alternatives:** Electron (heavier, Node backend); Flutter (Dart, weaker web reuse); native per-OS (3× UI work).
- **Consequences:** Webview differences across OSes (WebView2 / WKWebView / WebKitGTK) need testing. Small binaries. UI reusable on web.
- **Superseded by D-034 (2026-11-22):** the standalone desktop app is dropped. Tauri 2 becomes the mobile shell, and the React UI becomes the shared review UI.

### D-004 — SQLite everywhere
- **Context:** Need embedded, reliable, multi-process-safe storage with full-text search on all platforms.
- **Decision:** SQLite (bundled) in WAL mode with FTS5 on desktop, CLI and mobile; SQLite-WASM with OPFS on web.
- **Alternatives:** Key-value stores (no SQL/FTS); IndexedDB on web (different query layer).
- **Consequences:** One schema and one query compiler for all platforms. Migrations must be rigorous.

### D-005 — FSRS default scheduler, pluggable interface
- **Context:** Scheduling quality is core value. FSRS is state of the art and open.
- **Decision:** FSRS by default via the `fsrs` crate. A `Scheduler` trait allows SM-2 and Leitner. Review logs store scheduler + version.
- **Consequences:** Card state must be replayable from logs. Parameter optimisation available in P1.

### D-006 — Customisation is data
- **Context:** "Fully customise within the range of functionality provided" needs to be safe, portable, shareable and reversible.
- **Decision:** Every customisable object is a document (`kind`, `schema`, `id`, `revision`, `based_on`) stored as JSON, exported as TOML/JSON, validated against generated JSON Schemas, with forward migrations per schema version.
- **Consequences:** Generic tooling (fork, reset, history, import/export, sharing) works for every kind. Schema versioning discipline is required.

### D-007 — Bounded, declarative customisation; no user code in v1
- **Context:** User code would break cross-platform parity, make sharing unsafe and raise support costs.
- **Decision:** Users compose built-in step types, widget types, metrics, query operators and logic-less templates. The expression language for conditions is side-effect free and not Turing-complete.
- **Alternatives:** JS/Lua scripting; plugin API now.
- **Consequences:** New capabilities ship as new built-in blocks. A sandboxed extension API stays an open question (OQ-009).
- **Amended by D-030 (2026-11-22):** opt-in, local shell-command hooks on events are allowed in the CLI. There's still no embedded scripting and no user code inside the app.

### D-008 — Built-ins use the same format as user documents
- **Context:** Base versions and presets should be inspectable, diffable and resettable.
- **Decision:** Built-in presets and base documents live as TOML files in `presets/`, embedded at build time, in exactly the format users export.
- **Consequences:** Presets dogfood the document format. "Reset to base" and "diff with base" are trivial.

### D-009 — Sync-ready data model from day one
- **Context:** Retrofitting sync onto a local schema usually forces painful migrations.
- **Decision:** UUIDv7 ids generated on device; `created_at/updated_at/deleted_at/hlc/origin_device` on syncable rows; append-only logs for reviews and session events; an `oplog` of every change; device-scoped settings stored separately.
- **Consequences:** Slight storage overhead. Hard deletes are avoided. The oplog also powers live refresh (D-010).

### D-010 — No daemon in v1
- **Context:** Desktop and CLI must share data and see each other's changes.
- **Decision:** Both open the SQLite DB directly. Long-running processes detect changes via `PRAGMA data_version` + `oplog`. Active-session control uses optimistic concurrency.
- **Alternatives:** Background daemon with IPC (always-on reminders, but more install and failure complexity).
- **Consequences:** Background reminders require the desktop tray app (launch at login). Revisit in Phase 2.

### D-011 — Build order: core → CLI → desktop
- **Context:** The core is the riskiest, most reused part. The CLI is the cheapest shell.
- **Decision:** Build and dogfood the core through the CLI first, then the desktop app.
- **Consequences:** Early internal use is terminal-based. Desktop milestones start on a stable core API.
- **Superseded by D-031 (2026-11-22).**

### D-012 — Native mobile over the shared core (pending OQ-006)
- **Context:** The brief asks for native mobile apps. Mobile needs widgets, notifications and platform feel.
- **Decision (proposed):** SwiftUI and Jetpack Compose UIs over the Rust core via UniFFI.
- **Alternatives:** Tauri 2 mobile or React Native (more UI reuse, less native).
- **Consequences:** Two additional UI codebases in Phase 3. The core must stay FFI-friendly (no async runtime leaks across the boundary, simple types).
- **Superseded by D-034 (2026-11-22):** Tauri 2 mobile after a spike; native UIs only if widgets or notifications need them.

### D-013 — Sanitised rendering, no scripts
- **Context:** Card templates and notes are user and third-party content (shared decks, Anki imports).
- **Decision:** Render Markdown/templates in core, sanitise HTML, display in a sandboxed, script-free frame. Templates are logic-less.
- **Consequences:** Some Anki decks that rely on JS templates won't behave identically; the importer warns about them.

### D-014 — Monorepo with Cargo + pnpm workspaces
- **Decision:** One repository holding the core, all shells, shared UI packages, presets, schemas and docs.
- **Consequences:** Atomic cross-cutting changes. CI must be path-filtered to stay fast.

### D-015 — Spec as source of truth
- **Decision:** `docs/specs/` defines the product. Requirements, decisions and open questions have stable IDs that are referenced from code, commits and PRs. Changes follow the process in [README](README.md#changing-the-spec).

### D-016 — Business model: free first, then freemium
- *Private: kept in the business notes (D-039).*

### D-017 — Free local features forever; entitlements in core
- **Context:** Freemium must not undermine local-first trust (D-001, principle 4).
- **Decision (accepted 2026-11-19):** Anything that runs purely on the user's device stays free and never needs an account. Paid features are things that cost money to run (hosted sync, AI usage, hosted backups, gallery extras…). Entitlements are checked **in the core** (one rule for every surface) using signed, locally cached entitlement tokens, with an offline grace period. Downgrading or lapsing never deletes, locks or hides local data. Synced data stays on each device and remains exportable.
- **Consequences:** No feature-gating code in shells. Plan changes are data-only. Clear, honest pricing message: "your study data is always yours and always free to use locally".

### D-018 — Audience path
- **Context:** Product owner decision (2026-11-19), resolving OQ-002.
- **Decision:** The MVP targets **university students** (PER-1), then expands to **postgraduate researchers** (PER-7) and **lifelong academics** (PER-8). Exam candidates, language learners and self-learners (PER-2…4) are served by presets but aren't primary design targets. Terminal power users (PER-5) stay a cross-cutting secondary persona.
- **Consequences:** Academic-track features join the roadmap: projects/milestones (thesis), reference management and citations, reading/literature workflows, writing goals. Preset priority changes: `university` is the default showcase, and a `postgrad` preset is added.
- **Superseded by D-027 (2026-11-22).** The academic track is cut from the roadmap; the general study audience is frozen, not deleted.

### D-019 — Product name: Addisco (provisional)
- **Context:** Product owner choice (2026-11-19) after the 153-name study ([research/naming.md](research/naming.md)), resolving OQ-001 for now. The owner is open to changing it later. Shortlisted alternatives: Edisco, Lampas, Prokope.
- **Decision:** The product is **Addisco**, from Latin *addisco*, "I learn further, I keep learning". CLI binary `addisco`, short alias `addi`, URL scheme `addisco://`.
- **Keep renames cheap:** only **user-facing** identifiers carry the brand. Internal code identifiers stay neutral (`study-core` crate, `study.db` file, internal module names), so a rename doesn't touch code structure or user databases.

| Brand-dependent identifier | Current value | Where |
|---|---|---|
| Product / app display name | Addisco | UI, docs, installers |
| CLI binary / short alias | `addisco` / `addi` | 06, CLI-018 |
| URL scheme | `addisco://` | 06 §8, DESK-011 |
| Environment variables | `ADDISCO_*` | 06 §3, 09 §8 |
| Data / config directory names | `addisco` (Linux), `Addisco` (macOS/Windows) | 09 §8 |
| File extensions | `.addiscoarchive`, `.addiscopreset` | 03 §8, 09 §7 |
| CLI JSON schema id | `addisco.cli/1` | 06 §6 |
| Published package names (to reserve) | crates.io `addisco`, npm scope `@addisco`, Homebrew/winget/Scoop `addisco`, bundle id TBD | 08 §14 |

- **Before public release:** run the pre-release checks in [research/naming.md §7](research/naming.md#7-before-committing-to-a-name) (trademark, domains, app stores, native-speaker check) and reserve the package names. If the name changes before public release, update this table and the spec with a find/replace; after release, data dirs, env vars and file extensions need a migration path (read old names, write new ones).
- **Superseded by D-035 (2026-11-22):** the product is now Scrubjay.

### D-020 — Licence: open-source apps, closed services, with safeguards
- **Context:** Product owner decision (2026-11-19), resolving OQ-020, based on [research/licensing.md](research/licensing.md) (Option A).
- **Decision:**

| Part | Licence | Notes |
|---|---|---|
| Core (`crates/`), desktop, CLI, web and mobile apps (`apps/` except services), shared UI packages | **AGPL-3.0-or-later** | OSI open source; anyone who distributes or hosts a modified version must publish their changes |
| Hosted services: sync server, AI gateway, account/billing, entitlement issuer | **Proprietary** | Separate private repository; never in the public repo |
| Document schemas, archive format spec, built-in presets and sample content (`schemas/`, `presets/`, format docs) | **MIT** (code-like files) / **CC0-1.0** (presets, sample content) | So other tools can read and write Addisco data, and presets can be shared freely |
| Documentation (`docs/`) | **CC BY 4.0** | |
| Name and logo | **Trademark**, not licensed | Forks may use the code, not the name or logo |

- **Safeguards (all required before the first external contribution or public release):**
  1. **Contributor License Agreement** for every external contribution, enforced by a CLA bot on pull requests. It grants us the right to relicense, which we need to publish the iOS app under our own terms (AGPL conflicts with App Store terms) and to offer commercial licences.
  2. **Trademark:** file for "Addisco" once the name is final (D-019 pre-release checks), and publish a trademark policy (`TRADEMARKS.md`).
  3. **Repository boundary:** services live in a private repo. The sync protocol and entitlement-token format are documented publicly, because open clients must implement them.
  4. **Dependency licence policy:** an allowlist of AGPL-compatible licences, enforced in CI (`cargo deny`, JS licence checker). No GPL-incompatible or unknown-licence dependencies.
  5. **SPDX headers** on source files (`SPDX-License-Identifier: AGPL-3.0-or-later`) and a top-level `LICENSE` plus per-directory licence files for the MIT/CC0 parts.
- **Consistent with:** D-001 (local-first), D-016 (paid = hosted services), D-017 (local features free forever; entitlements in core). Entitlement checks live in open code, so they could be removed in a fork. That's acceptable: they only gate hosted services, which need our servers anyway.
- **Revisit if:** the post-launch signals in [research/licensing.md §7](research/licensing.md#7-what-to-measure-after-launch) show heavy leakage, for example a fork with free sync taking a large share of active users. Any change must be *more* open, or must apply only to new code we own outright, to avoid the HashiCorp-style backlash.
- **Amended by D-033 (2026-11-22):** the sync server moves to the public repository under AGPL-3.0-or-later and is self-hostable. Accounts and billing, the entitlement issuer and the AI gateway stay proprietary. This is a change toward *more* openness, as the rule above requires.

### D-021 — Git workflow
- **Context:** Commits had been consistent only by habit. Nothing defined branching, merging, PR requirements or releases, and nothing enforced the format. Product owner approved the recommendation (2026-11-19).
- **Decision:** [docs/GIT_WORKFLOW.md](../GIT_WORKFLOW.md) is the standard:
  - Conventional Commits with a fixed list of types and scopes, citing spec IDs.
  - Protected `main`; short-lived `<type>/<topic>` branches.
  - **Squash-merge only.** The PR title becomes the commit message and is checked in CI (`.github/workflows/pr-title.yml`).
  - SemVer `vX.Y.Z` tags with changelogs generated from commit titles.
  - Signed commits optional now, required once external contributors are accepted.
- **Alternatives:** Rebase-merge (keeps every commit, but needs every commit to be clean and makes changelogs noisier); merge commits (non-linear history).
- **Consequences:** One commit per PR on `main`, which makes history and changelogs readable. Branch protection and merge settings must be applied in GitHub settings by a maintainer (GIT_WORKFLOW §7).

### D-022 — Narrow the MVP to the wedge
- **Context:** The market review ([research/market.md](research/market.md)) found the original P0 scope too wide for a small team. All-in-one competitors (RemNote, AFFiNE) suffer maturity complaints, and notes are well served elsewhere. Product owner approved (2026-11-19).
- **Decision:** The MVP is **Anki-grade spaced repetition (FSRS; lossless Anki import including review history) + study sessions and workflows + exam planning (countdown, on-track) + a customisable Home dashboard (core widgets) + the CLI**, local-first and reliable.
  - Moved to P1: rich notes editor (NOTE-001 rich, NOTE-002), assignments, attachments (RES-001), `metric_chart` and `task_list` widgets.
  - Added: Markdown interop with other note tools (IO-008, P1).
- **Consequences:** M1–M4 scope shrinks; reliability (NFR-REL) stays P0 and is a selling point.
- **Superseded by D-028 (2026-11-22).**

### D-023 — AI as complement; what Pro sells
- *Private: kept in the business notes (D-039).*

### D-024 — Validation gate before M1
- *Private: kept in the business notes (D-039).*

### D-026 — Media store at P0, attachments at P1
- **Context:** D-022 moved attachments (RES-001) to P1, but lossless Anki import (IO-001, P0 per D-022 and OQ-008) needs card images and sounds. Both need the same storage. Product owner approved (2026-11-19).
- **Decision:**
  1. The **content-addressed media store** (08 §7: `media/ab/cd/<sha256>.<ext>`, immutable, deduplicated) and the `media` table are **P0 core infrastructure**, built with the Anki media import.
  2. **Fields refer to media by name**, as in Anki (`<img src="cell.png">`, `[sound:x.mp3]`). A per-workspace `media_names` table maps names to content. Fields are stored unchanged, so a future Anki export stays lossless. If an imported name already means other content, the file is stored as `name-<hash8>.ext` and that import's fields are rewritten to match, which is what Anki does.
  3. **Attachments, links and the resource library** (RES-001, RES-002) stay **P1** and will build on the same store.
- **Safety:** paths on disk come only from the hash and a sanitised extension, never from names in archives; names with path separators or control characters are refused. Files are synced to disk and renamed into place. Per-file (1 GiB) and per-package (64 GiB declared) limits guard against zip bombs.
- **Alternatives:** rewrite every reference to a hash (simpler lookups, but lossy for export and unreadable in raw fields); keep media inside SQLite (simpler backups, but a 20 GB library, NFR-SCALE-002, would bloat the database and slow backups).
- **Consequences:** media files are written before the import transaction so the database isn't locked during large copies. If the import then fails, the files stay as unreferenced content until garbage collection (08 §7, not built yet). Rendering media in review (audio is P1), archive export of media, backups' media manifest and garbage collection are separate work.

### D-027 — Audience: developers, CS students and technical learners
- **Context:** The student market is crowded (Anki, Quizlet, Knowt, RemNote, free AI study modes) and the original scope was too wide ([research/market.md](research/market.md)). What's built (a fast Rust CLI with FSRS and Anki import) fits developers better than undergraduates, who do much of their reviewing on phones. Developers who learn with AI help retain less (an Anthropic trial in 2026: 50% vs 67% on a follow-up quiz). Evaluation: [research/developer-pivot.md](research/developer-pivot.md) and [research/developer-strategy.md](research/developer-strategy.md). Product owner approved (2026-11-22).
- **Decision:** Positioning: **"Keep what you learn while you code."** The MVP serves three rings, in order:
  1. enthusiast engineers (PER-9), the free launch community;
  2. interview candidates and CS students (PER-10, PER-11), the volume and first revenue;
  3. engineers learning on the job (PER-12), the expansion.

  Engineering leads buying for teams (PER-13) are a later option, only on demand. The general study audience (PER-1…8) is frozen, not deleted.
- **Alternatives:** Keep the university-student wedge (D-022); start B2B with team onboarding; interview prep only.
- **Consequences:** Supersedes D-018. The academic track is cut from the roadmap. Specs 03, 04 and 05 are frozen and 07 is dropped (D-028, D-034). The core stays domain-neutral, so the general audience can return by unfreezing.

### D-028 — MVP wedge: code-anchored, text-native spaced repetition inside the developer workflow
- **Decision:** The MVP is:
  - FSRS with lossless Anki import (built);
  - cards from `@card` comments and Markdown card files, anchored to code with change detection;
  - deck repositories that update without losing the user's schedule;
  - terminal micro-reviews (`review --quick`, `addisco while`);
  - a composable CLI (JSON, hooks, MCP server);
  - the VS Code extension;
  - code-native cards (tree-sitter cloze);
  - interview mode (problems from the user's solutions repo) with ready-by-date scheduling and a forgiving backlog.

  Requirements: [13-code-sources](13-code-sources.md), SRS-016 and SRS-017, and the priority changes in [02](02-features.md).
- **Out of the MVP (frozen):** workflows and sessions, dashboards, presets and settings layers, notes, planner, goals and quizzes. **Cut:** the standalone desktop app (D-034), the academic track, hosted AI chat, executable cards (CODE-004 stays P2) and embedded scripting (D-030).
- **Consequences:** Supersedes D-022. Reliability (NFR-REL) stays P0. Frozen requirements keep their IDs and return only through a new decision.

### D-029 — Text files own the content of cards from sources; SQLite owns state
- **Context:** Developers want cards in plain text under Git. A database can't usefully live in Git. Identifying cards by a hash of their text, as hashcards does, resets progress on every edit.
- **Decision:** For cards from sources (`@card` comments, Markdown card files, deck repositories, solutions repos), the **text files are the source of truth for content**. SQLite keeps scheduling state, review history and anchors; D-004 and D-009 are unchanged. Each card has a stable id, written into the source on first scan unless the user opts out (OQ-028). Editing an anchored card opens its source file. Review history can be exported as NDJSON for users who want it in Git.
- **Consequences:** Re-scans are idempotent through `facts.source`, as Anki re-imports already are. Facts from sources are edited only through their files, so a scan never fights an in-app edit. The mobile app only reads them.

### D-030 — Extensibility: opt-in local shell hooks and JSON events; no embedded scripting
- **Context:** The pivot proposal included Lua hooks. Embedding a runtime conflicts with D-007 and creates an API we'd have to keep stable.
- **Decision:** At the MVP, users extend Addisco through:
  - shell commands run on events (CLI-016: local, opt-in, CLI only);
  - the CLI's JSON output and NDJSON input;
  - the MCP server.

  Editor plugins (e.g. Neovim, written in Lua) live outside the core. There's no embedded scripting. A sandboxed extension API stays OQ-009.
- **Consequences:** Amends D-007. Resolves OQ-017. Executable cards stay out: D-013 is unchanged, and CODE-004 (P2) would need its own decision.

### D-031 — Build order: core → CLI → VS Code extension → mobile review app
- **Decision:**
  1. Keep building and dogfooding through the CLI.
  2. Ship the VS Code extension in Phase 1 (76% of developers use VS Code, Stack Overflow 2025).
  3. Ship the mobile review app with Pro in Phase 2.

  The standalone desktop app is dropped. Its scaffold stays frozen, and building in CI, until it's removed.
- **Consequences:** Supersedes D-011. The shared React review UI (`packages/ui`) is built for the VS Code webview first, then reused on mobile and the web.

### D-032 — Validation gate for the developer pivot
- *Private: kept in the business notes (D-039).*

### D-033 — Business model for developers
- *Private: kept in the business notes (D-039).*

### D-034 — Platforms
- **Decision:** Surfaces, in order:
  1. CLI and TUI (built);
  2. VS Code extension (Phase 1; also on Open VSX for VS Code forks);
  3. mobile review app (Phase 2, with Pro);
  4. light browser extension (Phase 2, only if solutions-repo import isn't enough);
  5. web app (Phase 3);
  6. JetBrains plugin (Phase 3).

  A Neovim plugin stays thin and community-led. One React review UI (`packages/ui`) serves VS Code, mobile and the web. The mobile app uses **Tauri 2 mobile**, reusing that UI and linking the core directly, after a spike. Native SwiftUI/Compose comes in only if widgets or notifications need it. The standalone desktop app is dropped.
- **Alternatives:** keep the Tauri desktop app (redundant with VS Code, the TUI and the web app); native mobile from the start (two more UI codebases).
- **Consequences:**
  - Supersedes D-003 and D-012. The spike becomes the deciding step for OQ-006.
  - Mobile moves from Phase 3 to Phase 2.
  - The first sync release is review-log-first: review logs merge as a union, and the phone only reads cards from Git sources ([08 §9](08-architecture.md#9-sync-design-phase-2-constrains-phase-1)).

### D-035 — Product name: Scrubjay
- **Context:**
  - The product owner didn't like *Addisco*.
  - The pivot to developers (D-027) also changed who judges the name.
  - Round 3 ([research/naming.md §15](research/naming.md#15-round-3-developer-brief-2026-11-22)) re-checked the name against a developer brief.
  - Round 4 ([research/naming-round-4.md](research/naming-round-4.md)) searched 58 new names from nine angles. It scored them on uniqueness, CLI fit, meaning, say-and-spell, brand feel and breadth.

  The finalists were Muista (4.33), Cofio (4.20) and Scrubjay (4.17), with Addisco at 3.95. Product owner chose Scrubjay (2026-11-22).
- **Decision:** the product is **Scrubjay**.
  - **Story:** it's named after the scrub jay, a bird that remembers *what* it cached, *where* and *when* (Clayton & Dickinson, *Nature* 1998). That mirrors the product: cards (what), anchors in code (where), and stale detection when the code changes (when).
  - **CLI:** binary `scrubjay`. The short alias is `sj`, the initials of *scrub jay*, an exception to CLI-018's "first letters" rule for a compound name.
- **Why over the higher-scoring Muista:**
  - it's an English word with a story rather than another unfamiliar foreign word, which was the complaint about Addisco;
  - it gives a natural mascot and logo;
  - it's the only finalist whose `.com` looked free.
- **Identifiers** (replacing the D-019 table; internal code identifiers stay neutral, as before: `study-core`, `study.db`):

| Brand-dependent identifier | Value | Where |
|---|---|---|
| Product / app display name | Scrubjay | UI, docs, installers |
| CLI binary / short alias | `scrubjay` / `sj` | 06, CLI-018 |
| URL scheme | `scrubjay://` | 06 |
| Environment variables | `SCRUBJAY_*` (the old `ADDISCO_DATA_DIR` / `ADDISCO_CONFIG` still work) | 06 §3, 09 §8 |
| Data / config directory names | `scrubjay` (Linux), `Scrubjay` (macOS/Windows) | 09 §8 |
| Source-tree files | `.scrubjayignore`, `.scrubjay/cards/` | 13 |
| File extensions | `.scrubjayarchive`, `.scrubjaypreset` | 03 §8, 09 §7 |
| CLI JSON schema id | `scrubjay.cli/1` | 06 §6 |
| Crate / package names | `scrubjay-cli` (binary `scrubjay`), `scrubjay-desktop` (frozen), npm scope `@scrubjay` | Cargo, pnpm |
| Published names (to reserve) | crates.io `scrubjay`, npm `@scrubjay`, Homebrew `scrubjay`, GitHub org (the `scrubjay` handle is taken; e.g. `scrubjay-dev`), domain `scrubjay.com` | 08 §14 |

- **Migration for 0.1 alpha testers** (who already have `Addisco` folders):
  - on first run, an existing `Addisco`/`addisco` data or config folder is renamed to the new name. If that fails, it's used where it is;
  - `ADDISCO_DATA_DIR` and `ADDISCO_CONFIG` are still read.

  The CLI JSON schema id changes from `addisco.cli/1` to `scrubjay.cli/1`, a breaking change for scripts that check it (noted in the changelog).
- **Unchanged history:** the decision log, research and dated change-log entries keep the old name.
- **Before public release:**
  - register `scrubjay.com`;
  - reserve the crate, the npm scope and a GitHub org;
  - run the trademark search in classes 9, 41 and 42;
  - do a say-aloud check with developers in the M0.6 interviews.
- **Consequences:** supersedes D-019 and resolves OQ-001 again. The repository was renamed from `custom-study-app` to `mo-sajaad/scrubjay` on 2026-11-22.
