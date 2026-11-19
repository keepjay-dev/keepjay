# 08 — Architecture & Technology

> **Status:** Draft (core stack **Accepted**: D-002 Rust core, D-003 Tauri + React, D-004 SQLite; supporting libraries in §3 to confirm at scaffolding; mobile approach D-012 still Proposed; see [research](research/core-language-rust-vs-typescript.md)) · **Version:** 0.2 · **Last updated:** 2026-11-19
> **Related:** [09-data-model](09-data-model.md) · [10-non-functional-requirements](10-non-functional-requirements.md) · [decisions](decisions.md)

## 1. Architectural goals

1. **One core, many shells.** All behaviour (scheduling, workflow runtime, metrics, queries, validation, import/export, customisation resolution) lives in one core library. Shells only render and collect input (D-002).
2. **Local-first.** A complete local database on every device. The network is optional (D-001).
3. **Sync-ready from day one.** Phase 1 has no sync, but the data model and write path are designed so sync can be added without migrating user data (D-009).
4. **Fast CLI, light desktop.** A native binary for the CLI, and a small desktop footprint.
5. **Testable.** The core is pure enough to be tested without any UI. Every shell is a thin adapter.

## 2. System overview

```
              ┌───────────────┐ ┌──────────────┐ ┌───────────────┐ ┌────────────────┐
  Shells      │  Desktop app  │ │     CLI      │ │    Web app    │ │  Mobile apps   │
              │ Tauri 2 + TS  │ │ Rust, clap + │ │ TS UI (shared)│ │ SwiftUI /      │
              │ React UI      │ │ ratatui TUI  │ │ + core (WASM) │ │ Jetpack Compose│
              └───────┬───────┘ └──────┬───────┘ └───────┬───────┘ └───────┬────────┘
                      │ Tauri IPC      │ direct calls    │ wasm-bindgen    │ UniFFI
                      ▼                ▼                 ▼                 ▼
              ┌───────────────────────────────────────────────────────────────────┐
  Core        │                    core-api  (commands · queries · events)        │
  (Rust)      ├───────────────────────────────────────────────────────────────────┤
              │ domain · srs (FSRS…) · workflow runtime · query engine · metrics  │
              │ customisation (documents, presets, settings registry) · io        │
              ├───────────────────────────────────────────────────────────────────┤
              │ storage: SQLite (WAL, FTS5) · migrations · oplog · media store    │
              │ sync-client (Phase 2)                                             │
              └──────────────────────────────────┬────────────────────────────────┘
                                                 │ HTTPS (Phase 2, optional; account needed)
                          ┌──────────────────────▼───────────────────────┐
                          │ Cloud services (§15): accounts & billing ·   │
                          │ entitlements · sync (E2EE) · AI gateway      │
                          └──────────────────────────────────────────────┘
              Phase 1 = Desktop + CLI + Core.   Phase 2 = Web + Sync.   Phase 3 = Mobile.
```

## 3. Proposed technology stack

| Layer | Proposal | Why | Alternatives considered |
|---|---|---|---|
| Core | **Rust** | Native speed for the CLI; memory safety; compiles to native (desktop/CLI), WASM (web), and iOS/Android libraries (UniFFI). One implementation everywhere. | TypeScript core (simpler hiring, but slower CLI start and weaker native-mobile story); Kotlin Multiplatform (strong mobile, weaker CLI/web) |
| Desktop shell | **Tauri 2** | Rust backend = direct core access; small binaries (~10–20 MB); system webview; web UI reusable for the web app | Electron (heavier, Node backend, duplicates logic); Flutter (UI not shareable with web as easily; Dart) |
| UI | **TypeScript + React + Vite** | Largest ecosystem for editors, grids and charts; same package powers desktop and web | Svelte, Solid (smaller ecosystems for the components we need) |
| UI data | TanStack Query (core queries, invalidated by change events) + a small store (e.g. Zustand) for UI state | Cache + invalidation maps cleanly onto core change events | Redux |
| CLI | **Rust**: `clap` (args), `ratatui` + `crossterm` (TUI) | Same core, single static binary, fast start | — |
| Storage | **SQLite** (bundled, WAL, FTS5) via `rusqlite` | Proven, embedded, multi-process safe, full-text search; available on every platform including WASM | sled/redb (no SQL, no FTS); IndexedDB-only on web |
| Web storage | SQLite-WASM with OPFS | Same schema and queries in the browser | IndexedDB adapter |
| Scheduler | **FSRS** via the `fsrs` Rust crate | State of the art, open, used by Anki; supports parameter optimisation | SM-2 only |
| Types across boundary | `serde` + `schemars` (JSON Schema) + `specta` or `ts-rs` (TS types) | Single source of truth for documents, CLI JSON and UI types | Hand-written types |
| Mobile (Phase 3) | **SwiftUI + Jetpack Compose** over the core via **UniFFI** | Truly native UX, widgets and notifications; shared logic | Tauri mobile or React Native (more UI reuse, less native) — OQ-006 |
| Sync server (Phase 2) | Rust (`axum`) + SQLite (self-host) / Postgres (hosted) | Same language; can reuse domain types | Managed backends (Firebase, Supabase): lock-in, conflicts with E2EE |
| Repo / build | Cargo workspace + pnpm workspace; GitHub Actions; `tauri-action` for desktop releases; `cargo-dist` for the CLI | Standard, cross-platform | Nx, Bazel (overkill now) |

### Supporting libraries (proposed, confirm at scaffolding)

| Need | Candidate |
|---|---|
| Markdown editor | CodeMirror 6 (source + live-preview decorations); TipTap if WYSIWYG is preferred (OQ-015) |
| Markdown rendering / sanitising | `pulldown-cmark` + `ammonia` in core (one renderer for every surface); KaTeX for math |
| Dashboard grid | `react-grid-layout` or `gridstack` (spike: keyboard accessibility) |
| Charts | uPlot (time series) or ECharts (spike: bundle size vs features) |
| Time & time zones | `jiff` (or `chrono` + `chrono-tz`) |
| IDs | `uuid` v7 |
| Notifications (CLI) | `notify-rust` |
| Paths | `directories` |
| Logging | `tracing` + rolling file appender |
| i18n | Fluent (`fluent-rs` / `@fluent/bundle`) or ICU MessageFormat |

## 4. Core module breakdown

Start as **one `study-core` crate with internal modules**, and split into crates when boundaries stabilise or compile times demand it.

| Module | Responsibility |
|---|---|
| `domain` | Entity types, IDs (UUIDv7), HLC timestamps, validation rules, errors |
| `storage` | SQLite connection management, migrations, repositories, oplog writer, media store, backups |
| `srs` | Scheduler trait; FSRS, SM-2, Leitner adapters; queue building; log replay |
| `query` | Query-language parser → AST → SQL; shared by search, views, widgets, steps |
| `workflow` | Document types, validation, runtime state machine, step-type implementations, outcomes |
| `metrics` | Metric definitions, `daily_stats` rollups, widget data functions |
| `custom` | Documents store (fork/reset/revisions), presets (apply/re-apply/package), settings registry and layered resolution |
| `io` | Import (Anki, CSV, archive), export (archive, CSV, Markdown), backup/restore |
| `api` | **Facade**: commands, queries and event subscriptions; the only surface shells use |
| `sync` | (Phase 2) oplog exchange, merge, encryption |

## 5. Repository layout (proposed)

```
custom-study-app/
├── crates/
│   ├── study-core/          # the core (modules above); split later if needed
│   ├── study-wasm/          # Phase 2: wasm-bindgen wrapper for web
│   └── study-ffi/           # Phase 3: UniFFI bindings for iOS/Android
├── apps/
│   ├── cli/                 # `addisco` binary (clap + ratatui)
│   ├── desktop/             # Tauri app: src-tauri/ (Rust) + frontend entry
│   ├── web/                 # Phase 2
│   ├── ios/                 # Phase 3
│   └── android/             # Phase 3
├── packages/
│   ├── ui/                  # shared React components & screens (desktop + web)
│   ├── design-tokens/       # tokens → CSS vars (+ Swift/Kotlin later)
│   └── api-types/           # TS types generated from core
├── presets/                 # built-in presets and base documents (TOML), embedded at build
├── schemas/                 # generated JSON Schemas (documents, CLI JSON)
├── docs/
│   └── specs/               # this specification
├── Cargo.toml               # workspace
├── package.json             # pnpm workspace root
└── pnpm-workspace.yaml
```

## 6. Core API design

All shells talk to the core through one facade with three primitives:

```rust
// Illustrative, not final
let core = Core::open(Paths::default())?;

// Commands mutate state, are validated, run in one transaction, append to the oplog, emit events
core.execute(Command::AnswerCard { card_id, rating: Rating::Good, elapsed_ms: 5400 })?;
core.execute(Command::StartSession { workflow: "builtin:pomodoro".into(), params })?;

// Queries are read-only and return serialisable view models
let today: TodaySummary = core.query(Query::TodaySummary { workspace_id })?;
let data: WidgetData   = core.query(Query::WidgetData { dashboard_id, widget_id })?;

// Subscriptions deliver change events (entity kinds + ids) for cache invalidation
let sub = core.subscribe(|ev: ChangeEvent| ui.invalidate(ev.affects()));
```

- **Desktop:** each command/query is exposed as a Tauri command (JSON over IPC). Change events are forwarded as Tauri events. The UI uses generated TS types.
- **CLI:** calls the facade directly, in-process.
- **Web (Phase 2):** the same facade compiled to WASM, running in a worker.
- **Mobile (Phase 3):** the same facade via UniFFI-generated Swift/Kotlin.

## 7. Storage & data directory

- One SQLite database per profile (`study.db`), WAL mode, `foreign_keys = ON`, `busy_timeout = 5000`.
- **Migrations** are forward-only and numbered. `PRAGMA user_version` holds the schema version. A backup is always taken before migrating (NFR-REL-004).
- **Media** is content-addressed (`media/ab/cd/<sha256>.<ext>`) and immutable. It's garbage-collected only after a retention window longer than the oldest backup.
- **Backups** use the SQLite online backup API, compressed (`backups/*.db.zst`), plus a media manifest.
- Layout and default paths: [09 §8](09-data-model.md#8-on-disk-layout).

## 8. Multi-process access (desktop + CLI)

No daemon in v1 (D-010). Both processes open the database directly.

- **Writes:** short `BEGIN IMMEDIATE` transactions. Never hold a transaction across user think-time.
- **Change detection:** every committed command appends rows to `oplog` (monotonic `seq`). Long-running processes (desktop, CLI TUI) check `PRAGMA data_version` every ~500 ms and on window focus. If it changed, they read `oplog` since their last seen `seq` and emit `ChangeEvent`s → UI invalidation. Target: ≤ 1 s end-to-end (DESK-007).
- **Active session coordination:** the session row carries a `revision`. Commands use optimistic concurrency (`expected_revision`), so desktop and CLI can both control a session safely ([04 §5.2](04-workflows.md#52-rules)).
- **Version skew:** each binary knows the max schema version it supports. Newer DB → read-only with an "update required" message (CLI exit `6`). Older DB → migrate under an exclusive lock after a backup. Other open processes detect the schema change and prompt a restart.
- **Why not a daemon?** It would add install/permissions complexity and failure modes. Background reminders are covered by the desktop tray app with launch-at-login (DESK-008). Revisit if Phase 2 sync needs always-on background work.

## 9. Sync design (Phase 2, constrains Phase 1)

**Phase 1 obligations (D-009),** so sync can be added without data migration:

1. Every syncable row has `id` (UUIDv7, generated on device), `created_at`, `updated_at`, `deleted_at` (tombstone, never hard-delete synced rows), `hlc` (hybrid logical clock) and `origin_device`.
2. Every local change is recorded in `oplog` with entity, id, changed fields, HLC and device.
3. Review logs and session events are **append-only**.
4. Card scheduling state is **derivable** by replaying review logs (SRS-006).
5. Device-scoped settings are stored separately from synced settings.

**Proposed Phase 2 merge semantics:**

| Data | Merge strategy |
|---|---|
| Most entities (cards' content, tasks, decks, settings, documents) | Per-field last-writer-wins by HLC |
| Review logs, session events, goal check-ins | Append-only set union (no conflicts) |
| Card scheduling state | Recomputed by replaying the merged review log (deterministic) |
| Note bodies (long text) | v1: LWW per document + conflict copy (SYNC-006). Later: text CRDT (e.g. Automerge / Yrs) |
| Customisation documents | LWW per document + revision history, so the "losing" version stays recoverable |

**Transport:** devices push and pull batches of oplog entries to a sync server. The server stores opaque, optionally end-to-end encrypted batches (SYNC-003), with keys derived from a user passphrase and held in the OS keychain. Hosted sync is a paid feature (D-016); self-hosting is deferred (SYNC-004, OQ-021). Final choice between this custom oplog design and off-the-shelf options (cr-sqlite, Automerge-everything, ElectricSQL/PowerSync-style) will be a Phase 2 decision (OQ-005).

## 10. Platform strategies

### Web (Phase 2)
- Reuse `packages/ui` with a responsive layout. The core runs as WASM in a Web Worker, with SQLite-WASM persisted to OPFS for local-first in the browser.
- Multi-tab coordination via Web Locks / a SharedWorker. Installable PWA.
- Sync gives cross-device persistence; without sign-in, data lives in that browser only (clearly communicated).

### Mobile (Phase 3)
- Native UI (SwiftUI / Compose) over the core via UniFFI. SQLite bundled.
- Platform features: home-screen widgets, actionable notifications scheduled ahead from the core's forecast, share-sheet capture, offline-first.
- Design tokens are exported to Swift/Kotlin for visual consistency.
- Alternative under consideration: Tauri 2 mobile (maximum UI reuse, less native feel). Decide before Phase 3 (OQ-006).

## 11. Security & privacy architecture

- **No network by default** except an update check, which can be disabled. All endpoints are documented (NFR-PRIV-003).
- **Content rendering:** card templates and notes are rendered to HTML in the core and sanitised (`ammonia`). On desktop they're displayed in a sandboxed frame with a CSP that forbids scripts (`script-src 'none'`). No user-supplied JS anywhere (D-013).
- **Tauri hardening:** capability-scoped IPC commands, strict CSP, no remote content in the main webview, external links open in the OS browser.
- **Secrets** (Phase 2 sync keys, optional AI API keys) go in the OS keychain, never in the DB or config files.
- **Imports are untrusted:** Anki packages, archives and presets are parsed defensively (size limits, path-traversal checks in zips, schema validation).
- Optional database encryption at rest (SQLCipher) is P2.

## 12. Observability

- Structured local logs (`tracing`) with rotation; no content in logs by default.
- *Export diagnostics* (desktop) / `addisco doctor --report` (CLI) produce a redacted bundle for bug reports.
- Crash reporting and usage telemetry are **opt-in**, anonymous and documented (NFR-PRIV-002).

## 13. Testing strategy

| Level | What | Tools |
|---|---|---|
| Core unit & property tests | Scheduler, query parser, workflow state machine, settings resolution, merge (later) | `cargo test`, `proptest` |
| Golden tests | Import/export round-trips, document migrations, CLI output snapshots | `insta` |
| Integration | Core API against real SQLite; multi-process (desktop + CLI) concurrency | test harness spawning processes |
| UI components | Widgets, editors, review screen | Vitest + Testing Library |
| End-to-end | Journeys J1–J9 on the built desktop app | Playwright / WebDriver (`tauri-driver`) |
| Performance | NFR-PERF budgets on a seeded 100k-card fixture | Criterion benches in CI (regression alerts) |
| Accessibility | Automated checks + manual screen-reader pass per release | axe-core |

## 14. Build, release, distribution

- **CI matrix:** Windows x64/arm64, macOS universal, Linux x64/arm64.
- **Desktop:** signed and notarised builds (Apple), Authenticode (Windows). Linux as AppImage, `.deb`, `.rpm` and Flatpak. Tauri updater with signed manifests; channels `stable` / `beta` (nightly internal).
- **CLI:** bundled with desktop installers (optional PATH install) and released standalone via `cargo-dist` → Homebrew, winget, Scoop, `.deb`/`.rpm`, shell installer, `cargo install`.
- **Versioning:** SemVer. Desktop and CLI share a version number when released together. DB schema version is separate and monotonic.

## 15. Cloud services, accounts and entitlements (Phase 2)

The freemium model (D-016) adds a small set of hosted services. None of them are on the path of any free, local feature (D-017).

| Service | Responsibility | Notes |
|---|---|---|
| **Account service** | Sign-up/in (email + passkey, OAuth), devices, account deletion | Device-code flow for the CLI (ACCT-008) |
| **Billing** | Plans, checkout, invoices, student discount | Merchant of record for web/desktop; app-store billing on mobile (OQ-024) |
| **Entitlement issuer** | Issues short-lived **signed entitlement tokens** (plan, features, quotas, expiry) | Verified offline in the core with an embedded public key |
| **Sync server** | Oplog exchange (§9), E2EE blobs | Paid; self-hosting deferred (OQ-021) |
| **AI gateway** (deferred by D-023: built only if the optional hosted AI chat goes ahead) | Proxies AI requests to the model provider(s), meters usage per account, enforces quotas, strips identifiers | Provider abstracted; provider keys never ship in clients |

### Entitlements in the core

```rust
// Illustrative
core.entitlements().check(Feature::HostedSync)?;  // Ok / Err(NotEntitled { upgrade_hint })
core.entitlements().quota(Quota::AiMessages)       // { used, limit, resets_at }
```

- Gating happens **only in the core**, so desktop, CLI, web and mobile behave identically, and shells just render the `NotEntitled` state.
- Tokens are cached in the device store and refreshed opportunistically. Offline grace period (e.g. 14 days) before paid features pause (ACCT-003).
- Lapse or downgrade turns paid features off. **Local data and all free features are untouched** (ACCT-004).
- The free/local build has no dependency on these services. If the services are unreachable, only paid features are affected (NFR-OFF-002).

### AI gateway data flow

```
User selects context ──► core builds request + shows preview (AI-005) ──► user confirms
      ──► HTTPS to AI gateway (auth token) ──► quota check ──► provider ──► streamed reply
      ──► core stores chat locally (syncs like any other data if sync is on)
```

- Only the previewed context is sent. Chats are stored locally as data owned by the user.
- The gateway keeps no conversation content beyond what's needed for abuse prevention and billing metering. Retention is documented (NFR-PRIV-007).

## 16. Licensing and repository boundaries (D-020)

```
github.com/<org>/addisco            (public, AGPL-3.0-or-later unless noted)
├── crates/                         AGPL-3.0-or-later
├── apps/cli, apps/desktop, apps/web, apps/ios, apps/android
├── packages/                       AGPL-3.0-or-later
├── schemas/                        MIT       (document & CLI JSON schemas)
├── presets/                        CC0-1.0   (built-in presets, sample content)
├── docs/                           CC BY 4.0 (incl. sync protocol & token format specs)
├── LICENSE · CONTRIBUTING.md (CLA) · TRADEMARKS.md · SECURITY.md

<private repo>/addisco-services     (proprietary)
├── sync-server/  ai-gateway/  accounts-billing/  entitlement-issuer/
```

- `apps/sync-server` moves from the public layout in §5 to the private services repo.
- Mobile store builds are published under our own licence grant, which is possible because the CLA keeps copyright with us. Source for the same version stays available under AGPL.
- The public repo never depends on the private one. The free/local build compiles and runs fully from public sources (D-017).
