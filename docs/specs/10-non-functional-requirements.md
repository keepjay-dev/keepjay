# 10 — Non-Functional Requirements

> **Status:** Draft · **Version:** 0.3 · **Last updated:** 2026-11-23
> **Related:** [08-architecture](08-architecture.md) · 07-desktop-and-ux *(private)*

Unless stated otherwise, budgets are measured on a **reference machine** (4-core mid-range laptop, 8 GB RAM, SSD) with the **reference dataset**: 100k cards, 1M review logs, 2k notes, 1k tasks, 5k sessions.

## Performance (PERF)

| ID | Requirement | Pri |
|---|---|---|
| NFR-PERF-001 | CLI: simple read commands (`keepjay status`, `keepjay deck list`) complete in < 100 ms p95, cold. *Measured 2026-11-23:* `status` median 20 ms, p95 29 ms, with 4,000 source cards of which 399 are stale (release build, 2016 MacBook Pro, Intel i7-6567U). | P0 |
| NFR-PERF-002 | Desktop: cold launch to interactive Home < 2 s; warm < 1 s. *Dropped with the desktop app (D-034).* | — |
| NFR-PERF-003 | Review: next card shown < 50 ms p95 after grading (TUI, VS Code review panel, mobile). | P0 |
| NFR-PERF-004 | Dashboard with 15 widgets renders < 300 ms; a single widget refresh after a change < 100 ms. *Frozen with dashboards (D-028).* | — |
| NFR-PERF-005 | Full-text search returns first results < 100 ms. | P0 |
| NFR-PERF-006 | Live refresh across processes (e.g. VS Code ↔ a running TUI) ≤ 1 s. | P0 |
| NFR-PERF-007 | Desktop idle memory < 250 MB; installer < 30 MB (excluding OS webview). *Dropped with the desktop app (D-034).* | — |
| NFR-PERF-008 | Anki import of a 50k-card package with media < 60 s. | P1 |
| NFR-PERF-009 | Scanning: re-scanning a 10,000-file source < 1 s with a warm file cache; first scan < 5 s (SRC-009). Parsers load only on scan and card-generation paths, so `status` and `review` stay within NFR-PERF-001. *Measured 2026-11-23* (10,000 files, about 4,000 cards, 2016 MacBook Pro, Intel i7-6567U): re-scan 0.25–0.37 s, first scan with id write-back 1.6 s. On macOS, write-back uses `fsync` rather than `F_FULLFSYNC`, as Git does. | P0 |
| NFR-PERF-010 | VS Code extension: due count in the status bar within 1 s of activation; the review panel opens in < 500 ms (EDIT-001). | P0 |

## Scale (SCALE)

| ID | Requirement | Pri |
|---|---|---|
| NFR-SCALE-001 | Supported without exceeding 2× the PERF budgets: 500k cards, 5M review logs, 20k notes, 100 workspaces per profile. | P1 |
| NFR-SCALE-002 | Media library up to 20 GB. | P1 |

## Reliability & data integrity (REL)

| ID | Requirement | Pri |
|---|---|---|
| NFR-REL-001 | No committed data is lost on crash or power loss (SQLite transactions, WAL, `synchronous = NORMAL` minimum with checkpointing). | P0 |
| NFR-REL-002 | Automatic backups (IO-004) with verified restore; integrity quick-check on startup; full check via `keepjay doctor`. | P0 |
| NFR-REL-003 | Sessions resume after crash (SES-005); an in-progress review answer is never double-counted. | P0 |
| NFR-REL-004 | Every schema migration is preceded by a backup and is atomic. | P0 |
| NFR-REL-005 | Export/import round-trip is lossless (property-tested on fixtures). | P0 |
| NFR-REL-007 | Paid features keep working offline for the entitlement grace period; service outages never affect free/local features (D-017). | P2 |
| NFR-REL-006 | Crash-free session rate ≥ 99.5% (beta), ≥ 99.9% (v1.0). | P0 |

## Offline (OFF)

| ID | Requirement | Pri |
|---|---|---|
| NFR-OFF-001 | 100% of functionality works offline, except sync, update checks, optional AI and online imports. | P0 |
| NFR-OFF-002 | No feature may block on network availability; network actions time out gracefully and are retryable. | P0 |

## Privacy (PRIV)

| ID | Requirement | Pri |
|---|---|---|
| NFR-PRIV-001 | No account required for any local functionality (SYNC-001). | P0 |
| NFR-PRIV-002 | Telemetry and crash reporting are **opt-in**, anonymous, and documented field by field; the app is fully functional with them off. | P0 |
| NFR-PRIV-003 | All network endpoints the app may contact are documented; the update check can be disabled. | P0 |
| NFR-PRIV-004 | Users can export all their data (IO-003) and delete all of it (IO-006). | P0 |
| NFR-PRIV-005 | Logs contain no study content by default. | P0 |
| NFR-PRIV-006 | AI features send nothing without explicit, per-request consent and a preview of the content sent (AI-005). | P2 |
| NFR-PRIV-007 | The AI gateway documents data handling: no training on user content, minimal retention for metering/abuse prevention, provider sub-processors listed. | P2 |
| NFR-PRIV-008 | Free/local use collects no personal data; an account holds only what billing and sync need. | P2 |

## Security (SEC)

| ID | Requirement | Pri |
|---|---|---|
| NFR-SEC-001 | Card/note rendering is sanitised; no user-supplied script executes anywhere (D-013). | P0 |
| NFR-SEC-002 | Imported files (Anki, archives, presets) are treated as untrusted: size limits, zip path-traversal checks, schema validation. | P0 |
| NFR-SEC-003 | Releases are code-signed (Apple notarisation, Windows Authenticode); update manifests are signed. | P0 |
| NFR-SEC-004 | Dependency auditing in CI (`cargo audit`/`cargo deny`, `pnpm audit`); no known critical CVEs at release. | P0 |
| NFR-SEC-005 | Tauri IPC is capability-scoped; strict CSP; no remote content in the main webview. | P0 |
| NFR-SEC-006 | Secrets (sync keys, API keys) are stored only in the OS keychain. | P2 |
| NFR-SEC-007 | Optional encryption at rest for the local DB. | P2 |
| NFR-SEC-008 | Sync is end-to-end encryptable; the server never needs plaintext (SYNC-003). | P2 |

## Accessibility (A11Y)

| ID | Requirement | Pri |
|---|---|---|
| NFR-A11Y-001 | Desktop and web conform to **WCAG 2.2 AA**. | P0 |
| NFR-A11Y-002 | Every feature is operable by keyboard alone, including dashboard edit mode and the workflow editor. | P0 |
| NFR-A11Y-003 | Screen-reader support tested with NVDA (Windows), VoiceOver (macOS) and Orca (Linux) on the J1–J5 journeys. | P0 |
| NFR-A11Y-004 | Themes (including custom) meet contrast requirements; information is never conveyed by colour alone. | P0 |
| NFR-A11Y-005 | Respect OS reduced-motion, high-contrast and font-size settings. | P0 |
| NFR-A11Y-006 | The CLI offers `--plain` (no full-screen TUI, no box drawing) and honours `NO_COLOR`. | P0 |

## Internationalisation (I18N)

| ID | Requirement | Pri |
|---|---|---|
| NFR-I18N-001 | All UI strings are externalised; English at launch. | P0 |
| NFR-I18N-002 | Locale-aware dates, numbers, plurals; configurable week start. | P0 |
| NFR-I18N-003 | Full Unicode content support, including IME composition for CJK and correct RTL rendering in content. | P0 |
| NFR-I18N-004 | RTL UI layout. | P1 |
| NFR-I18N-005 | Additional UI languages (community translations). | P1 |

## Compatibility (COMP)

| ID | Requirement | Pri |
|---|---|---|
| NFR-COMP-001 | Desktop: Windows 10 (22H2)+ and 11 (x64, arm64); macOS 13+ (Apple silicon, Intel); Linux x64/arm64 with glibc ≥ 2.31 (Ubuntu 20.04+, Fedora 38+), as AppImage, `.deb`, `.rpm`, Flatpak. Minimum versions tracked in OQ-018. | P0 |
| NFR-COMP-002 | CLI: the same OS matrix; terminals: Windows Terminal, macOS Terminal/iTerm2, common Linux terminals; degrades gracefully on limited terminals. | P0 |
| NFR-COMP-003 | Web (Phase 2): the latest two versions of Chrome, Edge, Firefox and Safari. | P2 |
| NFR-COMP-004 | Mobile (Phase 3): iOS 17+, Android 10+ (to confirm). | P2 |
| NFR-COMP-005 | Backward compatibility: any archive or document exported by any released version imports into later versions. | P0 |

## Maintainability (MAINT)

| ID | Requirement | Pri |
|---|---|---|
| NFR-MAINT-001 | Core line coverage ≥ 80%; scheduler, workflow runtime, query parser and migrations ≥ 90%. | P0 |
| NFR-MAINT-002 | Public core API documented (rustdoc); generated TS types checked into CI (no drift). | P0 |
| NFR-MAINT-003 | Significant technical decisions recorded in `decisions.md`. | P0 |
| NFR-MAINT-004 | Conventional commits referencing requirement IDs where applicable. | P0 |
| NFR-MAINT-006 | Licence compliance (D-020): SPDX headers on all source files; dependency licences checked against an AGPL-compatible allowlist in CI; release builds fail on unknown or incompatible licences. | P0 |
| NFR-MAINT-007 | Every external pull request has a signed CLA before merge (CI check). | P0 |
| NFR-MAINT-005 | Lint and format enforced in CI (rustfmt, clippy, ESLint, Prettier). | P0 |
