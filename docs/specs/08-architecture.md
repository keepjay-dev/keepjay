# 08 — Architecture & Technology

> **Status:** Draft (core stack **Accepted**: D-002 Rust core, D-004 SQLite; shells per D-034: CLI, VS Code extension, Tauri 2 mobile after a spike, web; supporting libraries in §3 to confirm when first used; see [research](research/core-language-rust-vs-typescript.md)) · **Version:** 0.3 · **Last updated:** 2026-11-23
> **Related:** [09-data-model](09-data-model.md) · [10-non-functional-requirements](10-non-functional-requirements.md) · [decisions](decisions.md)

## 1. Architectural goals

1. **One core, many shells.** All behaviour (scheduling, workflow runtime, metrics, queries, validation, import/export, customisation resolution) lives in one core library. Shells only render and collect input (D-002).
2. **Local-first.** A complete local database on every device. The network is optional (D-001).
3. **Sync-ready from day one.** Phase 1 has no sync, but the data model and write path are designed so sync can be added without migrating user data (D-009).
4. **Fast CLI, thin shells.** A native binary for the CLI. The VS Code extension, the mobile app and the web app reuse one React review UI (D-034).
5. **Testable.** The core is pure enough to be tested without any UI. Every shell is a thin adapter.

## 2. System overview

```
              ┌───────────────┐ ┌───────────────┐ ┌───────────────┐ ┌───────────────┐
  Shells      │      CLI      │ │ VS Code ext.  │ │  Mobile app   │ │    Web app    │
              │ Rust, clap +  │ │ TS + shared   │ │ Tauri 2 +     │ │ shared React  │
              │ ratatui TUI + │ │ React review  │ │ shared React  │ │ review UI     │
              │ MCP server    │ │ UI (webview)  │ │ review UI     │ │               │
              └───────┬───────┘ └───────┬───────┘ └───────┬───────┘ └───────┬───────┘
                      │ direct calls    │ bundled CLI,    │ Tauri IPC       │ wasm-bindgen
                      │                 │ --json          │                 │
                      ▼                 ▼                 ▼                 ▼
              ┌───────────────────────────────────────────────────────────────────┐
  Core        │                    core-api  (commands · queries · events)        │
  (Rust)      ├───────────────────────────────────────────────────────────────────┤
              │ domain · srs (FSRS…) · sources (scan, anchors, change detection)  │
              │ code (tree-sitter cloze, highlighting) · query engine · metrics   │
              │ io (Anki, Markdown cards, archive, NDJSON log)                    │
              ├───────────────────────────────────────────────────────────────────┤
              │ storage: SQLite (WAL, FTS5) · migrations · oplog · media store    │
              │ sync-client (Phase 2)                                             │
              └──────────────────────────────────┬────────────────────────────────┘
                                                 │ HTTPS (Phase 2, optional; account needed)
                          ┌──────────────────────▼───────────────────────┐
                          │ Services (§15): accounts & billing ·         │
                          │ entitlements · sync (AGPL, self-hostable) ·  │
                          │ AI gateway (hosted drafting)                 │
                          └──────────────────────────────────────────────┘
       Phase 1 = CLI + VS Code + Core.  Phase 2 = Mobile + Sync (+ browser extension).  Phase 3 = Web, JetBrains.
```

## 3. Proposed technology stack

| Layer | Proposal | Why | Alternatives considered |
|---|---|---|---|
| Core | **Rust** | Native speed for the CLI; memory safety; compiles to native (CLI, Tauri mobile) and WASM (web). One implementation everywhere. | TypeScript core (simpler hiring, but slower CLI start and weaker native-mobile story); Kotlin Multiplatform (strong mobile, weaker CLI/web) |
| Editor shell | **VS Code extension** (TypeScript), calling the bundled CLI with `--json`; review in a webview | 76% of developers use VS Code (Stack Overflow 2025); no logic outside the core; also runs in VS Code forks through Open VSX | A long-lived language-server-style process (only if profiling shows the need) |
| Mobile shell (Phase 2) | **Tauri 2 mobile**, decided by a spike (D-034, OQ-006) | Reuses the React review UI and links the Rust core directly, with no binding layer | Native SwiftUI/Compose over UniFFI (better widgets and notifications; two more UI codebases); React Native |
| UI | **TypeScript + React + Vite** | Largest ecosystem; one review UI package powers the VS Code webview, mobile and web | Svelte, Solid (smaller ecosystems for the components we need) |
| UI data | TanStack Query (core queries, invalidated by change events) + a small store (e.g. Zustand) for UI state | Cache + invalidation maps cleanly onto core change events | Redux |
| CLI | **Rust**: `clap` (args), `ratatui` + `crossterm` (TUI) | Same core, single static binary, fast start | — |
| Storage | **SQLite** (bundled, WAL, FTS5) via `rusqlite` | Proven, embedded, multi-process safe, full-text search; available on every platform including WASM | sled/redb (no SQL, no FTS); IndexedDB-only on web |
| Web storage | SQLite-WASM with OPFS | Same schema and queries in the browser | IndexedDB adapter |
| Scheduler | **FSRS** via the `fsrs` Rust crate | State of the art, open, used by Anki; supports parameter optimisation | SM-2 only |
| Types across boundary | `serde` + `schemars` (JSON Schema) + `specta` or `ts-rs` (TS types) | Single source of truth for documents, CLI JSON and UI types | Hand-written types |
| Source scanning | `ignore` (ripgrep's walker: respects `.gitignore`) + `memchr` prefilter for `@card`; tree-sitter grammars, four at the MVP (OQ-030) | Fast and proven; grammars load only when needed (NFR-PERF-009) | Regex-only parsing (no symbol anchors) |
| Git | The system `git` binary for deck repositories and hook install | Respects the user's credentials and configuration | libgit2, gitoxide |
| Sync server (Phase 2) | Rust (`axum`) + SQLite (self-host) / Postgres (hosted); **AGPL, self-hostable** (D-033) | Same language; can reuse domain types | Managed backends (Firebase, Supabase): lock-in, conflicts with E2EE |
| Repo / build | Cargo workspace + pnpm workspace; GitHub Actions; the CLI release workflow (static binaries); `vsce`/`ovsx` for the extension; Tauri mobile builds in Phase 2 | Standard, cross-platform | Nx, Bazel (overkill now) |

### Supporting libraries (proposed, confirm at scaffolding)

| Need | Candidate |
|---|---|
| Markdown rendering / sanitising | `pulldown-cmark` + `ammonia` in core (one renderer for every surface); KaTeX for math |
| Syntax highlighting | `tree-sitter-highlight` (shares the grammars) or `syntect`, in core, producing terminal styles and HTML |
| Markdown editor, dashboard grid, charts | Frozen with notes and dashboards (D-028) |
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
| `srs` | Scheduler trait; FSRS, SM-2, Leitner adapters; queue building; log replay; targets (SRS-016); backlog smoothing (SRS-017) |
| `sources` | Source registry, scanning (`@card` comments, Markdown card files), anchors, change detection, id write-back, deck repositories, solutions-repo import ([13](13-code-sources.md)) |
| `code` | Tree-sitter parsing, symbol anchors, tree-sitter cloze, syntax highlighting |
| `query` | Query-language parser → AST → SQL; shared by search, filtered review, targets, stats |
| `metrics` | Metric definitions and `daily_stats` rollups behind `scrubjay stats` |
| `io` | Import (Anki, archive; CSV P1), export (archive, NDJSON review log, Markdown), backup/restore |
| `workflow`, `custom` | Frozen (D-028): workflow runtime, presets and layered settings. Configuration files only. |
| `api` | **Facade**: commands, queries and event subscriptions; the only surface shells use |
| `sync` | (Phase 2) oplog exchange, merge, encryption |

## 5. Repository layout (proposed)

```
scrubjay/
├── crates/
│   ├── study-core/          # the core (modules above); split later if needed
│   └── study-wasm/          # Phase 3: wasm-bindgen wrapper for web
├── apps/
│   ├── cli/                 # `scrubjay` binary (clap + ratatui) + MCP server
│   ├── vscode/              # Phase 1 (M3): VS Code extension (EDIT-001)
│   ├── mobile/              # Phase 2: Tauri 2 mobile review app (D-034)
│   ├── sync-server/         # Phase 2: AGPL, self-hostable (D-033)
│   ├── browser/             # Phase 2, only if needed (EDIT-004)
│   ├── web/                 # Phase 3
├── packages/
│   ├── ui/                  # shared React review UI (VS Code webview, mobile, web)
│   ├── design-tokens/       # tokens → CSS vars (+ Swift/Kotlin later)
│   └── api-types/           # TS types generated from core
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

- **CLI and MCP server:** call the facade directly, in-process.
- **VS Code extension:** calls the bundled CLI with `--json` (one-shot commands stay under 100 ms, NFR-PERF-001). The webview uses generated TS types.
- **Mobile (Phase 2):** each command/query is exposed as a Tauri command (JSON over IPC). Change events are forwarded as Tauri events.
- **Web (Phase 3):** the same facade compiled to WASM, running in a worker.

## 7. Storage & data directory

- One SQLite database per profile (`study.db`), WAL mode, `foreign_keys = ON`, `busy_timeout = 5000`.
- **Migrations** are forward-only and numbered. `PRAGMA user_version` holds the schema version. A backup is always taken before migrating (NFR-REL-004).
- **Media** is content-addressed (`media/ab/cd/<sha256>.<ext>`) and immutable. It's garbage-collected only after a retention window longer than the oldest backup.
- **Backups** use the SQLite online backup API, compressed (`backups/*.db.zst`), plus a media manifest.
- Layout and default paths: [09 §8](09-data-model.md#8-on-disk-layout).

## 8. Multi-process access (CLI, VS Code, MCP)

No daemon in v1 (D-010). Every process (the terminal CLI, the CLI bundled in VS Code, the MCP server) opens the database directly. The desktop app named in this section's title was dropped (D-034); the rules apply to all local processes.

- **Writes:** short `BEGIN IMMEDIATE` transactions. Never hold a transaction across user think-time.
- **Change detection:** every committed command appends rows to `oplog` (monotonic `seq`). Long-running processes (desktop, CLI TUI) check `PRAGMA data_version` every ~500 ms and on window focus. If it changed, they read `oplog` since their last seen `seq` and emit `ChangeEvent`s → UI invalidation. Target: ≤ 1 s end-to-end (DESK-007).
- **Active session coordination:** the session row carries a `revision`. Commands use optimistic concurrency (`expected_revision`), so desktop and CLI can both control a session safely ([04 §5.2](04-workflows.md#52-rules)).
- **Version skew:** each binary knows the max schema version it supports. Newer DB → read-only with an "update required" message (CLI exit `6`). Older DB → migrate under an exclusive lock after a backup. Other open processes detect the schema change and prompt a restart.
- **Why not a daemon?** It would add install/permissions complexity and failure modes. Review reminders come with the mobile app (M8); on the computer, the prompt segment (CLI-021) and the VS Code status bar show what's due. Revisit if Phase 2 sync needs always-on background work.

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

**Transport:** devices push and pull batches of oplog entries to a sync server. The server stores opaque, optionally end-to-end encrypted batches (SYNC-003), with keys derived from a user passphrase and held in the OS keychain. Hosted sync is a Pro feature; the server is AGPL and self-hostable (SYNC-004, D-033). Final choice between this custom oplog design and off-the-shelf options (cr-sqlite, Automerge-everything, ElectricSQL/PowerSync-style) will be a Phase 2 decision (OQ-005).

**First release: review-log-first (D-034).** The mobile review app needs far less than general sync:

- Review logs are append-only, so merging them is a set union.
- Cards from Git sources are authored on the computer (D-029); the phone receives them read-only.
- Quick capture on the phone only creates new facts.
- Card state is recomputed from the merged log.

So the first release has almost no merge conflicts. Per-field LWW for other entities comes after it.

## 10. Platform strategies

### VS Code extension (Phase 1)
- A per-platform VSIX bundles the CLI. Data calls go through `--json`, and review runs in a webview with the shared UI. Details: [13 §9](13-code-sources.md#9-vs-code-extension-edit-001).
- Published on the VS Code Marketplace and Open VSX, so VS Code forks can install it.

### Mobile (Phase 2)
- A review app: quick review, quick capture, ready-by-date countdown, home-screen widget, review reminders (MOB-001…003).
- **Tauri 2 mobile**, reusing `packages/ui` and linking the core directly, after a spike at the start of M8 (D-034, OQ-006). Fall back to native SwiftUI/Compose over UniFFI only if widgets or notifications need it.
- Needs sync (§9), which is what Pro sells (D-033).

### Web (Phase 3)
- Reuse `packages/ui` with a responsive layout. The core runs as WASM in a Web Worker, with SQLite-WASM persisted to OPFS for local-first in the browser.
- Multi-tab coordination via Web Locks / a SharedWorker. Installable PWA.
- Sync gives cross-device persistence; without sign-in, data lives in that browser only (clearly communicated).

## 11. Security & privacy architecture

- **No network by default** except an update check, which can be disabled. All endpoints are documented (NFR-PRIV-003).
- **Content rendering:** card templates and notes are rendered to HTML in the core and sanitised (`ammonia`). In the VS Code webview, the mobile app and the web app they're displayed with a CSP that forbids scripts (`script-src 'none'` for card content). No user-supplied JS anywhere (D-013).
- **Scanning:** the scanner reads registered sources only and never executes code (SRC-008). Deck repositories are untrusted content.
- **Tauri hardening (mobile):** capability-scoped IPC commands, strict CSP, no remote content in the main webview, external links open in the OS browser.
- **Secrets** (Phase 2 sync keys, optional AI API keys) go in the OS keychain, never in the DB or config files.
- **Imports are untrusted:** Anki packages, archives and presets are parsed defensively (size limits, path-traversal checks in zips, schema validation).
- Optional database encryption at rest (SQLCipher) is P2.

## 12. Observability

- Structured local logs (`tracing`) with rotation; no content in logs by default.
- `scrubjay doctor --report` produces a redacted bundle for bug reports.
- Crash reporting and usage telemetry are **opt-in**, anonymous and documented (NFR-PRIV-002).

## 13. Testing strategy

| Level | What | Tools |
|---|---|---|
| Core unit & property tests | Scheduler, query parser, workflow state machine, settings resolution, merge (later) | `cargo test`, `proptest` |
| Golden tests | Import/export round-trips, document migrations, CLI output snapshots | `insta` |
| Integration | Core API against real SQLite; multi-process (desktop + CLI) concurrency | test harness spawning processes |
| UI components | Widgets, editors, review screen | Vitest + Testing Library |
| End-to-end | Journeys J15–J20, J3 and J8 through the CLI and the VS Code extension | CLI integration tests; `@vscode/test-electron` |
| Performance | NFR-PERF budgets on a seeded 100k-card fixture | Criterion benches in CI (regression alerts) |
| Accessibility | Automated checks + manual screen-reader pass per release | axe-core |

## 14. Build, release, distribution

- **CI matrix:** Windows x64/arm64, macOS universal, Linux x64/arm64.
- **CLI:** static binaries from the release workflow (built), then Homebrew, Scoop/winget, AUR, Nix, `.deb`/`.rpm`, a shell installer and `cargo install` (OQ-016). Signed binaries by M4/M5.
- **VS Code extension:** per-platform VSIX bundling the CLI, published to the Marketplace and Open VSX.
- **Mobile (Phase 2):** store builds through Tauri's mobile tooling; licence grant per the CLA (§16).
- **Versioning:** SemVer. The CLI and the CLI bundled in the extension share a version. DB schema version is separate and monotonic.

## 15. Cloud services, accounts and entitlements (Phase 2)

The freemium model (D-016) adds a small set of hosted services. None of them are on the path of any free, local feature (D-017).

| Service | Responsibility | Notes |
|---|---|---|
| **Account service** | Sign-up/in (email + passkey, OAuth), devices, account deletion | Device-code flow for the CLI (ACCT-008) |
| **Billing** | Plans, checkout, invoices, student discount | Merchant of record for web/desktop; app-store billing on mobile (OQ-024) |
| **Entitlement issuer** | Issues short-lived **signed entitlement tokens** (plan, features, quotas, expiry) | Verified offline in the core with an embedded public key |
| **Sync server** | Oplog exchange (§9), E2EE blobs | Hosted = Pro; the server is AGPL, self-hostable, and lives in the public repo (D-033) |
| **AI gateway** (Phase 2, for hosted AI drafting, AI-012) | Proxies AI requests to the model provider(s), meters usage per account, enforces quotas, strips identifiers | Provider abstracted; provider keys never ship in clients |

### Entitlements in the core

```rust
// Illustrative
core.entitlements().check(Feature::HostedSync)?;  // Ok / Err(NotEntitled { upgrade_hint })
core.entitlements().quota(Quota::AiDrafts)         // { used, limit, resets_at }
```

- Gating happens **only in the core**, so the CLI, VS Code, mobile and web behave identically, and shells just render the `NotEntitled` state.
- Tokens are cached in the device store and refreshed opportunistically. Offline grace period (e.g. 14 days) before paid features pause (ACCT-003).
- Lapse or downgrade turns paid features off. **Local data and all free features are untouched** (ACCT-004).
- The free/local build has no dependency on these services. If the services are unreachable, only paid features are affected (NFR-OFF-002).

### AI gateway data flow

```
User selects context ──► core builds request + shows preview (AI-005) ──► user confirms
      ──► HTTPS to AI gateway (auth token) ──► quota check ──► provider ──► streamed reply
      ──► draft cards shown for approval ──► approved cards saved locally (and to the source file)
```

- Only the previewed context is sent. Drafts become cards only after the user approves them (AI-005).
- The gateway keeps no conversation content beyond what's needed for abuse prevention and billing metering. Retention is documented (NFR-PRIV-007).

## 16. Licensing and repository boundaries (D-020)

```
github.com/<org>/scrubjay            (public, AGPL-3.0-or-later unless noted)
├── crates/                         AGPL-3.0-or-later
├── apps/cli, apps/vscode, apps/mobile, apps/web, apps/browser
├── apps/sync-server                AGPL-3.0-or-later, self-hostable (D-033)
├── packages/                       AGPL-3.0-or-later
├── schemas/                        MIT       (document & CLI JSON schemas)
├── docs/                           CC BY 4.0 (incl. sync protocol & token format specs)
├── LICENSE · CONTRIBUTING.md (CLA) · TRADEMARKS.md · SECURITY.md

<private repo>/scrubjay-services     (proprietary)
├── ai-gateway/  accounts-billing/  entitlement-issuer/
```

- `apps/sync-server` stays in the public repo under the AGPL (D-033, amending D-020).
- Mobile store builds are published under our own licence grant, which is possible because the CLA keeps copyright with us. Source for the same version stays available under AGPL.
- The public repo never depends on the private one. The free/local build compiles and runs fully from public sources (D-017).
