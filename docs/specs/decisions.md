# Decision Log

> **Status:** Draft · **Last updated:** 2026-11-19
> **Related:** [open-questions](open-questions.md) · [08-architecture](08-architecture.md)

A lightweight ADR log. Each decision has a stable ID. To change a decision, add a **new** entry and mark the old one `Superseded by D-NNN`. Never edit history.

**Statuses:** `Proposed` (awaiting product-owner review) · `Accepted` · `Superseded` · `Rejected`

## Summary

| ID | Decision | Status | Date |
|---|---|---|---|
| D-001 | Local-first architecture; sync is optional | Accepted | 2026-11-19 |
| D-002 | One shared Rust core for every platform | Accepted | 2026-11-19 |
| D-003 | Desktop shell: Tauri 2 + TypeScript/React UI | Accepted | 2026-11-19 |
| D-004 | SQLite everywhere (WAL, FTS5; SQLite-WASM on web) | Accepted | 2026-11-19 |
| D-005 | FSRS as the default scheduler behind a pluggable interface | Accepted | 2026-11-19 |
| D-006 | Customisation is data: versioned, schema-validated documents | Accepted | 2026-11-19 |
| D-007 | Bounded, declarative customisation; no user code in v1 | Accepted | 2026-11-19 |
| D-008 | Built-in presets/base versions use the same format as user documents | Accepted | 2026-11-19 |
| D-009 | Sync-ready data model from day one (UUIDv7, HLC, tombstones, oplog) | Accepted | 2026-11-19 |
| D-010 | Desktop and CLI open the DB directly; no background daemon in v1 | Accepted | 2026-11-19 |
| D-011 | Phase 1 build order: core → CLI → desktop | Accepted | 2026-11-19 |
| D-012 | Mobile: native UI (SwiftUI/Compose) over the core via UniFFI | Proposed (pending OQ-006) | 2026-11-19 |
| D-013 | Content rendering is sanitised; no scripts in card templates or notes | Accepted | 2026-11-19 |
| D-014 | Monorepo: Cargo workspace + pnpm workspace | Accepted | 2026-11-19 |
| D-015 | `docs/specs/` is the source of truth, with stable requirement/decision IDs | Accepted | 2026-11-19 |
| D-016 | *(private)* | Private | — |
| D-017 | Local features are free forever; paid features are gated by entitlements checked in the core, offline-tolerant | Accepted | 2026-11-19 |
| D-018 | Audience path: university students (MVP) → postgraduates → lifelong academics | Accepted | 2026-11-19 |
| D-019 | Product name: **Addisco** (provisional; open to change) | Accepted (provisional) | 2026-11-19 |
| D-022 | MVP narrowed to the wedge: SRS (FSRS, lossless Anki import with history) + sessions/workflows + exam planning + customisable Home dashboard + CLI | Accepted | 2026-11-19 |
| D-023 | *(private)* | Private | — |
| D-024 | *(private)* | Private | — |
| D-021 | Git workflow: Conventional Commits with fixed scopes, squash-merge only, protected `main`, PR title checked in CI | Accepted | 2026-11-19 |
| D-020 | Licence: AGPL-3.0-or-later core and apps; closed hosted services; CLA, trademark, permissive formats | Accepted | 2026-11-19 |

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

### D-012 — Native mobile over the shared core (pending OQ-006)
- **Context:** The brief asks for native mobile apps. Mobile needs widgets, notifications and platform feel.
- **Decision (proposed):** SwiftUI and Jetpack Compose UIs over the Rust core via UniFFI.
- **Alternatives:** Tauri 2 mobile or React Native (more UI reuse, less native).
- **Consequences:** Two additional UI codebases in Phase 3. The core must stay FFI-friendly (no async runtime leaks across the boundary, simple types).

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

### D-023 — AI as complement; what Pro sells
- *Private: kept in the business notes (D-039).*

### D-024 — Validation gate before M1
- *Private: kept in the business notes (D-039).*

