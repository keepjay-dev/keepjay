# 12 — Glossary

> **Status:** Draft · **Version:** 0.3 · **Last updated:** 2026-11-23

Canonical terms. Use them consistently in UI copy, code identifiers and docs. When a UI label differs from the code name, both are given.

| Term | Code name | Definition |
|---|---|---|
| **`@card` comment** | — | A code comment that defines a card, scanned from a source (13 §3). |
| **Active session** | `session` (status running/paused) | The one session currently running for a profile (SES-004). Frozen with sessions. |
| **Anchor** | `anchor` | The link from a card to the code it's about: repository, path, lines, symbol and a hash of the code (SRC-004). |
| **Base setup** | — | What a workspace looks like with zero customisation: built-in defaults + its applied preset (03 §2). |
| **Base version** | `builtin:<slug>@<v>` | The built-in, unmodified definition of a document. |
| **Built-in** | `builtin:` prefix | Shipped with the app, read-only, embedded in the binary. |
| **AI gateway** | — | Our hosted proxy between clients and AI model providers, used for hosted AI drafting (AI-012); meters usage and enforces plan quotas. |
| **Keepjay** | — | The product name (D-037), after food-caching jays, birds that remember what they cached, where and when. Formerly *Scrubjay* (D-035) and the provisional *Addisco* (D-019). CLI `keepjay`, alias `kj`. |
| **CLA** | — | Contributor License Agreement: every external contributor grants us the rights needed to relicense their contribution (D-020). |
| **Card** | `card` | One reviewable item generated from a fact by a card template. Has its own scheduling state. |
| **Card type** | `card_type` (document) | Defines a fact's fields and the templates that turn it into cards. Anki calls this a "note type". |
| **Cloze** | — | A card type that hides marked parts of a text (`{{c1::…}}`). |
| **Command palette** | — | Ctrl/Cmd+K launcher for every action, item and workflow. |
| **Core** | `study-core` | The shared Rust library that implements all behaviour (08). |
| **Customisation document** | `document` | Any user-customisable object (workflow, dashboard, card type, deck options, theme, saved view, keymap, note template, preset) stored as versioned, validated data (03 §7). |
| **Dashboard** | `dashboard` (document) | A grid of configured widgets. Each workspace has one *Home* dashboard. |
| **Data dir** | `data_dir` | Folder holding the database, media, backups and logs (09 §8). |
| **Deck** | `deck` | A hierarchical container of cards with associated deck options. |
| **Deck options** | `deck_options` (document) | Scheduler choice and parameters, daily limits, steps. |
| **Deck repository** | `source` (kind `deck_repo`) | A Git repository of Markdown card files that users subscribe to and pull, keeping their schedule (SRC-011). |
| **Device** | `device` | One installation on one machine. Device-scoped settings never sync. |
| **Due** | — | A card whose next review date is today or earlier (by study day). |
| **Entitlement** | `entitlements` | A signed, locally cached statement of which paid features and quotas an account has; checked only in the core (D-017). |
| **Exam** | `exam` | A dated assessment linked to a scope (decks/subjects); drives countdowns and plans. |
| **Fact** | `fact` | The set of field values from which cards are generated. Anki calls this a "note"; renamed so it doesn't clash with *Notes*. |
| **Fork** | — | A user-modified copy of a built-in document that takes its place (copy-on-write). *Reset to base* removes it. |
| **FSRS** | `srs::fsrs` | Free Spaced Repetition Scheduler, the default scheduling algorithm. |
| **Goal** | `goal` | Metric + target + period + scope, with progress computed automatically. |
| **HLC** | `hlc` | Hybrid logical clock timestamp used to order changes across devices (sync). |
| **Layer** | — | One level in settings resolution: defaults → user → preset → workspace → device → invocation (03 §5). |
| **Lapse** | — | A review graded *Again* on a card in review state. |
| **Markdown card file** | `*.cards.md` | A Markdown file of `Q:`/`A:` and `C:` cards, read from a source (SRC-003). |
| **MCP server** | `keepjay mcp` | Lets the user's AI assistant or coding agent search, read and, with approval, add cards (AI-010). |
| **Micro-review** | — | A short review that fits a gap in the day: `review --quick`, `keepjay while`, the VS Code status bar (CLI-019, CLI-020). |
| **Leech** | — | A card that keeps lapsing (≥ threshold). |
| **Local-first** | — | Data lives and works on the device; the network only adds sync. |
| **Metric** | — | A named quantity computed by the metrics engine (05 §5). |
| **Module** | `modules.<name>` | A top-level feature area (Study, Notes, Planner, Goals, Quizzes…) that can be toggled per workspace. |
| **Note** | `note` | A Markdown document. Not the same as a *fact*. |
| **Oplog** | `oplog` | Append-only log of every local change; drives live refresh and future sync. |
| **Pattern** | `pattern::` tag | A problem-solving technique (sliding window, two pointers…) used to group problem cards (PROB-003). |
| **Plan** | — | *Free* (all local features), *Supporter* (no features) or *Pro* (hosted sync, hosted backups, hosted AI drafting quota) (D-033). Names provisional (OQ-024). |
| **Problem card** | `builtin:problem` | A card for an algorithm problem that the user re-solves when it's due, linked to their solution file (PROB-001). |
| **Preset** | `preset` | A bundle of documents and workspace settings applied to a workspace (03 §4). |
| **Profile** | `profile` | The local user identity owning a data dir; maps to a sync account later. |
| **Project** | `project` | A long-horizon piece of work (thesis, paper) with milestones, linked tasks, notes and references (PLAN-011). |
| **Query** | — | An expression in the shared query language (09 §6). |
| **Range** | — | A time window for metrics (`7d`, `this_week`, `until:next_exam`, …). |
| **Reflection** | `reflect` step → `note` | A prompted journal entry written during a session, saved as a note linked to it. |
| **Reference** | `reference` | A bibliographic record (paper, book) in the reference library, citable from notes (RES-005, NOTE-009). |
| **Review log** | `review_log` | Append-only record of one card answer; the source of truth for scheduling. |
| **Saved view** | `saved_view` (document) | A stored query plus layout (list/board/table) and columns. |
| **Scope** | — | A query restricting which items a widget, goal or step considers. |
| **Session** | `session` | One run of a workflow, including ad-hoc timers and manual logs. |
| **Session event** | `session_event` | Append-only record of a session state change. |
| **Shell** | — | A user-facing app (CLI, VS Code extension, mobile, web) built on the core. |
| **Solutions repository** | `source` (kind `problems`) | A folder or Git repository of the user's own problem solutions, often synced by a LeetHub-style extension (PROB-005). |
| **Source** | `source` | A registered folder or Git repository that Keepjay scans for cards (SRC-001). Not the same as `facts.source`, the provenance field. |
| **Stale card** | `anchor.stale_since` | A card whose anchored code changed since the card was last confirmed; listed by `keepjay stale` (SRC-006). |
| **Step / step type** | `step`, `step.type` | One unit of a workflow (`focus`, `review`, `break`…) of a built-in type (04 §3). |
| **Streak** | — | Consecutive periods in which a goal was met, with forgiving rules (GOAL-005). |
| **Study day** | `study_day` | Local date shifted by the day-start hour (default 04:00), so late-night study counts for the previous day. |
| **Subject** | `subject` | A hierarchical area of study (course, topic) used to organise everything. UI MAY call these "Courses" via terminology overrides (CUST-016). |
| **Tag** | `tag` | A hierarchical label (`bio::cells`) applicable to most items. |
| **Target** | `target` | A date and desired retention for a scope (deck or tag), e.g. an exam or interview (SRS-016). |
| **Task / assignment** | `task` (`kind`) | A to-do item; an assignment is a task with a submission date and optional grade. |
| **Template (card)** | `card_type.templates[]` | Logic-less markup producing a card's front and back from fact fields. |
| **Trigger** | `workflow.triggers` | How a workflow is started: manual, scheduled reminder (P1), event (P2). |
| **TUI** | — | The CLI's full-screen terminal interface (review, session, dashboard). |
| **Widget / widget type** | `widget`, `widget.type` | One configured block on a dashboard, of a built-in type (05 §4). |
| **Workflow** | `workflow` (document) | A customisable recipe for a study session: params, steps, triggers (04). |
| **Workspace** | `workspace` | A top-level container of content and customisation (e.g. "Uni", "Japanese"). |
