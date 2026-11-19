# Decision Log

> **Status:** Draft · **Last updated:** 2026-11-19
> **Related:** [open-questions](open-questions.md) · [08-architecture](08-architecture.md)

A lightweight ADR log. Each decision has a stable ID. To change a decision, add a **new** entry and mark the old one `Superseded by D-NNN`. Never edit history.

**Statuses:** `Proposed` (awaiting product-owner review) · `Accepted` · `Superseded` · `Rejected`

## Summary

| ID | Decision | Status | Date |
|---|---|---|---|
| D-001 | Local-first architecture; sync is optional | Proposed | 2026-11-19 |
| D-002 | One shared Rust core for every platform | Proposed | 2026-11-19 |
| D-003 | Desktop shell: Tauri 2 + TypeScript/React UI | Proposed | 2026-11-19 |
| D-004 | SQLite everywhere (WAL, FTS5; SQLite-WASM on web) | Proposed | 2026-11-19 |
| D-005 | FSRS as the default scheduler behind a pluggable interface | Proposed | 2026-11-19 |
| D-006 | Customisation is data: versioned, schema-validated documents | Proposed | 2026-11-19 |
| D-007 | Bounded, declarative customisation; no user code in v1 | Proposed | 2026-11-19 |
| D-008 | Built-in presets/base versions use the same format as user documents | Proposed | 2026-11-19 |
| D-009 | Sync-ready data model from day one (UUIDv7, HLC, tombstones, oplog) | Proposed | 2026-11-19 |
| D-010 | Desktop and CLI open the DB directly; no background daemon in v1 | Proposed | 2026-11-19 |
| D-011 | Phase 1 build order: core → CLI → desktop | Proposed | 2026-11-19 |
| D-012 | Mobile: native UI (SwiftUI/Compose) over the core via UniFFI | Proposed (pending OQ-006) | 2026-11-19 |
| D-013 | Content rendering is sanitised; no scripts in card templates or notes | Proposed | 2026-11-19 |
| D-014 | Monorepo: Cargo workspace + pnpm workspace | Proposed | 2026-11-19 |
| D-015 | `docs/specs/` is the source of truth, with stable requirement/decision IDs | Proposed | 2026-11-19 |

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
- **Consequences:** Team needs Rust skills (OQ-003). FFI/WASM boundaries need generated types. Fast native CLI.

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
