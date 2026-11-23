# 02 — Feature Catalogue

> **Status:** Draft · **Version:** 0.3 · **Last updated:** 2026-11-23
> **Related:** [13-code-sources](13-code-sources.md) · [03-customization-model](03-customization-model.md) · [04-workflows](04-workflows.md) · [05-dashboards-and-widgets](05-dashboards-and-widgets.md) · [11-roadmap](11-roadmap.md)

This is the master list of functional requirements. Modules with their own detail spec keep their requirements there, so the two copies can't drift; this file only summarises them (§ "Modules specified elsewhere").

Priority legend: **P0** = MVP, **P1** = v1.x, **P2** = later (see [README](README.md#priorities)).

## Module map

| Code | Module | Requirements in | Status (D-028, D-034) |
|---|---|---|---|
| WS | Workspaces & organisation | this file | Active (subjects frozen) |
| SRS | Flashcards & spaced repetition | this file | **Active** |
| SRC / CODE / PROB / EDIT | Code sources, code-native cards, problem practice, editor integrations | [13-code-sources](13-code-sources.md) | **Active** |
| NOTE | Notes | this file | Frozen |
| SES / WF | Study sessions & workflows | [04-workflows](04-workflows.md) | Frozen |
| DASH / STAT | Dashboards, widgets & analytics | [05-dashboards-and-widgets](05-dashboards-and-widgets.md) | Frozen, except the metrics behind `keepjay stats` and STAT-008 |
| PLAN | Planner: tasks, assignments, exams | this file | Frozen (ready-by-date moved to SRS-016) |
| GOAL | Goals, habits & streaks | this file | Frozen |
| QUIZ | Practice & quizzes | this file | Frozen (problem practice is PROB) |
| RES | Resources & attachments | this file | Frozen (the media store stays, D-026) |
| NOTIF | Notifications & reminders | this file | Frozen; review reminders return with the mobile app (M8) |
| SRCH | Search, query language, command palette | this file (syntax in [09 §6](09-data-model.md#6-query-language)) | Active: search and core query operators |
| IO | Import, export & backup | this file (formats in [09](09-data-model.md)) | **Active** |
| CUST | Customisation & presets | [03-customization-model](03-customization-model.md) | Frozen, except configuration as files |
| CLI | Command-line tool | [06-cli](06-cli.md) | **Active** |
| DESK | Desktop app | [07-desktop-and-ux](07-desktop-and-ux.md) | **Dropped** (D-034); its design-system and accessibility sections apply to the shared review UI |
| SYNC / WEB / MOB | Sync, web app, mobile apps | this file (design in [08](08-architecture.md)) | Phase 2 (sync, mobile); Phase 3 (web) |
| SHARE | Sharing & community | this file | Decks shared as Git repositories (SRC-011); the rest later |
| AI | AI assistance (complement, D-023) | this file | Active: MCP server (P0); hosted drafting in Pro (Phase 2) |
| ACCT | Accounts, plans & entitlements (freemium, Phase 2) | this file (design in [08 §15](08-architecture.md#15-cloud-services-accounts-and-entitlements-phase-2)) | Phase 2 |

**Frozen** requirements keep their IDs and text, but they aren't scheduled (D-028). They come back only through a new decision.

---

## Workspaces & organisation (WS)

| ID | Requirement | Pri |
|---|---|---|
| WS-001 | Users MUST be able to create, rename, archive and delete multiple **workspaces**. Each workspace isolates its content, customisations and applied preset. | P0 |
| WS-002 | A workspace contains a tree of **subjects** (e.g. *Biology › Cell Biology*). Depth is unlimited in data; the UI is optimised for up to 4 levels. *Frozen (D-028): decks, tags and sources organise developer content.* | P0 |
| WS-003 | **Tags** (hierarchical with `::`, e.g. `bio::cells`) MUST be applicable to facts/cards, notes, tasks, exams and resources. | P0 |
| WS-004 | The active workspace MUST be switchable from a switcher (desktop) and via `--workspace` or a configured default (CLI). | P0 |
| WS-005 | Deleted items go to a **trash** with restore. Retention defaults to 30 days (configurable). | P0 |
| WS-006 | Content MAY be moved between workspaces, keeping its tags and links. | P1 |
| WS-007 | **Custom properties**: user-defined typed fields (text, number, date, select, checkbox) on tasks, notes and exams. | P1 |
| WS-008 | An "All workspaces" scope aggregates dashboards and stats across workspaces. | P1 |

## Flashcards & spaced repetition (SRS)

| ID | Requirement | Pri |
|---|---|---|
| SRS-001 | **Decks** form a tree per workspace. A deck MAY be associated with a subject. | P0 |
| SRS-002 | **Card types** define named fields and one or more templates. Each **fact** (a set of field values) generates one **card** per applicable template. Built-in types: Basic, Basic + reverse, Optional reverse, Cloze, Type-in answer. | P0 |
| SRS-003 | **FSRS** MUST be the default scheduler (D-005). The scheduler interface MUST be pluggable. SM-2 and Leitner SHOULD be selectable in deck options. Implemented with the `fsrs` crate 7.x (FSRS-7 default parameters) behind a `Scheduler` trait. | P0 (FSRS) / P1 (others) |
| SRS-004 | **Deck options** (a customisation document): daily new/review limits, desired retention, learning/relearning steps, maximum interval, sibling burying, new-card order. *Note (M0.4):* with FSRS-7, learning and relearning steps aren't needed, because the algorithm models short-term memory itself (intervals under a day, minimum 1 minute). Steps remain an option for SM-2 (P1). *Queue rule (M0.4):* learning cards are **never shown before they are due** (no Anki-style learn-ahead). FSRS-7 uses real elapsed time, so early reviews add almost nothing and a forgotten card looped in testing. Sessions instead wait for a learning card due within 20 minutes (TUI countdown) or end with "next card due in Xm". Daily new-card limit: 20 by default. | P0 |
| SRS-005 | **Review screen**: prompt → reveal → grade (Again / Hard / Good / Easy) with next-interval previews. Supports undo of the last answer, in-place edit, flag, suspend, bury, and fully keyboard-driven operation. *Status (M0.4):* CLI review implemented (reveal, grade 1–4 with interval previews, quit). Undo, edit in place, flag, suspend and bury are still to do. For cards from sources, *edit* opens the source file at the card's line (SRC-007, D-029). | P0 |
| SRS-006 | Every answer MUST be written to an append-only **review log** (card, rating, timestamp, time taken, previous state, scheduler + version). Card scheduling state MUST be reproducible by replaying the log. | P0 |
| SRS-007 | **Duplicate detection** on add (same first field, same card type, same deck), with a warning and an override option. | P0 |
| SRS-008 | Card content supports Markdown, images, LaTeX math (KaTeX) and code highlighting. Code highlighting is P0 in the TUI and the shared review UI (CODE-001). Audio playback is P1. | P0 / P1 |
| SRS-009 | **Card browser**: search with the query language, sort, multi-select, bulk operations (move, tag, suspend, reschedule, change card type), preview. | P0 |
| SRS-010 | **Filtered review**: an ad-hoc queue from any query (e.g. failed last week, or `review --query 'tag:algo::dp'`), with an optional *cram* mode that doesn't affect scheduling. | P0 |
| SRS-011 | **FSRS parameter optimisation** from the user's own review history, per deck options. | P1 |
| SRS-012 | **Leech** detection (lapses ≥ threshold), with a configurable action (tag or suspend). | P1 |
| SRS-013 | **Template editing** using a logic-less, Mustache-like syntax (`{{Field}}`, `{{#Field}}…{{/Field}}`, `{{cloze:Text}}`) and scoped CSS. No scripting (D-013). | P1 |
| SRS-014 | Due-date load balancing ("fuzz") to avoid review spikes. | P1 |
| SRS-015 | **Image occlusion** card type. | P2 |
| SRS-016 | **Ready by a date:** a **target** gives a scope (deck or tag query) a date and a desired retention, e.g. an exam or interview. Scheduling aims for that retention on the date, then returns to normal spacing. `status`, the VS Code status bar and the mobile app show predicted retention for the date. Replaces the frozen exam countdown (PLAN-003). | P0 |
| SRS-017 | **Forgiving backlog:** after missed days, overdue reviews are spread over a configurable number of days (default 3), lowest retrievability first, instead of arriving all at once. | P0 |

## Notes (NOTE)

> **Frozen (D-028).** Notes live in the user's editor and repositories. Markdown card files are covered by SRC-003.

| ID | Requirement | Pri |
|---|---|---|
| NOTE-001 | Markdown notes organised by subject and tags, with title and body. **MVP (D-022): plain notes** for reflections and quick notes; live preview, pin/favourite and the rich editor are P1. Notes are well served by other tools, so interoperate (IO-008) rather than out-build them. | P0 (plain) / P1 (rich) |
| NOTE-002 | Math, code blocks, tables, checklists, and images pasted or dropped into managed media. | P1 |
| NOTE-003 | Export notes as plain `.md` files with front-matter, individually or in bulk. | P0 |
| NOTE-004 | `[[wiki links]]` between notes and to cards, tasks and exams, with a backlinks panel. | P1 |
| NOTE-005 | **Cards from notes**: select text → create a basic or cloze card. The card keeps a `generated_from` link to the note. | P1 |
| NOTE-006 | Note templates (Lecture, Cornell, Summary, Book chapter) as customisation documents. | P1 |
| NOTE-007 | Revision history for note bodies (last N saves). | P1 |
| NOTE-008 | Optional two-way mirror of notes to a folder of Markdown files (Obsidian-compatible). | P2 |
| NOTE-009 | **Citations** in notes (`[@key]`) linked to the reference library (RES-005), with a bibliography rendered on export. Academic track. | P2 |
| NOTE-010 | **Literature notes**: a note template bound to a reference (summary, key claims, methods, quotes with page numbers). Academic track. | P2 |

## Planner: tasks, assignments, exams (PLAN)

> **Frozen (D-028).** Ready-by-date scheduling moved to SRS-016.

| ID | Requirement | Pri |
|---|---|---|
| PLAN-001 | **Tasks**: title, description, subject, tags, status (todo / doing / done), priority, due date/time, estimate, checklist. | P0 |
| PLAN-002 | **Assignments** (a task kind with a submission date and optional grade) and **exams** (date/time, location, linked subjects/decks, target score). | P0 (exams) / P1 (assignments) |
| PLAN-003 | Exam **countdown** (P0) and **on-track indicator** based on remaining new cards, due load and days left (P1). | P0 |
| PLAN-004 | **Study plan generator**: given an exam and its scope, propose daily targets (new cards/day, minutes/day), create a goal, and recalculate as progress changes. | P1 |
| PLAN-005 | Recurring tasks (daily, weekly, or a custom RRULE subset). | P1 |
| PLAN-006 | **Agenda and calendar** views (day / week / month) combining tasks, exams, scheduled workflows and review forecast. | P1 |
| PLAN-007 | **Saved views** of tasks (list / board / table) with filters and sort, as customisation documents. | P1 |
| PLAN-008 | Time-blocking: schedule a workflow at a date/time, which creates a reminder. | P1 |
| PLAN-009 | iCal (`.ics`) export of exams and deadlines. | P1 |
| PLAN-010 | Two-way sync with Google/Outlook calendars. | P2 |
| PLAN-011 | **Projects with milestones**: long-horizon work (dissertation, thesis chapters, papers, grant applications) with milestones, linked tasks, notes and references, and a progress timeline. Academic track. | P1 |
| PLAN-012 | Supervisor/meeting log: recurring meeting notes with action items that become tasks. Academic track. | P2 |

## Goals, habits & streaks (GOAL)

> **Frozen (D-028).** GOAL-004's module toggle still applies to the optional review heatmap (STAT-008).

| ID | Requirement | Pri |
|---|---|---|
| GOAL-001 | **Goals** = metric + target + period + scope. Examples: "90 min/day of study"; "300 reviews/week in `deck:Japanese`"; "finish all new cards in `deck:Biochem` by 2027-05-01". | P0 |
| GOAL-002 | Progress is computed automatically from metrics ([05 §5](05-dashboards-and-widgets.md#5-metrics-catalogue)). Manual habit goals use check-ins (P1). | P0 / P1 |
| GOAL-003 | Goal history (met/missed per period) is available to widgets and the CLI. | P0 |
| GOAL-004 | Streaks, gamification and celebratory animations can be turned off entirely (module toggle, CUST-006). | P0 |
| GOAL-005 | **Forgiving streaks** per goal: rest days per week, streak freezes, a "minimum viable day" threshold. | P1 |
| GOAL-006 | **Writing goals** (words written per day/week in notes or a project) for essays and theses. Academic track. | P2 |

## Practice & quizzes (QUIZ)

> **Frozen (D-028).** Algorithm problem practice is PROB ([13](13-code-sources.md)).

| ID | Requirement | Pri |
|---|---|---|
| QUIZ-001 | **Question items**: multiple choice (single/multi), true/false, short answer, with explanation and tags. Implemented as a card-type family so they can also participate in SRS. | P1 |
| QUIZ-002 | **Practice test**: N questions from a query, optional time limit, scoring, review of mistakes. | P1 |
| QUIZ-003 | **Exam simulation**: timed, no feedback until the end, results per subject/tag. | P1 |
| QUIZ-004 | Missed questions can be tagged or queued for SRS review automatically. | P1 |
| QUIZ-005 | Import question banks (CSV / documented JSON). | P2 |

## Resources & attachments (RES)

> **Frozen (D-028).** The content-addressed media store itself stays P0 core infrastructure (D-026).

| ID | Requirement | Pri |
|---|---|---|
| RES-001 | Attach files (any type) and links to subjects, notes, tasks and exams. Files live in managed, content-addressed media storage (the store itself is P0, D-026). | P1 |
| RES-002 | Resource library view: filter by subject, tag or type; open in the OS default app. | P1 |
| RES-003 | Built-in PDF viewer with highlights that can become notes or cards, linked back to the page. | P2 |
| RES-004 | Reading-progress tracking (pages/chapters) that feeds `read` workflow steps and goals. | P2 |
| RES-005 | **Reference library**: import/export BibTeX and CSL-JSON, sync with Zotero (OQ-013), and store metadata (authors, year, DOI, tags, PDF attachment). Academic track. | P2 |
| RES-006 | **Reading lists** per subject/project with status (to read, reading, read) and priority. Academic track. | P2 |

## Notifications & reminders (NOTIF)

> **Frozen (D-028).** Review reminders return with the mobile app (M8).

| ID | Requirement | Pri |
|---|---|---|
| NOTIF-001 | OS notifications for session step transitions (focus over, break over) while any surface is running. | P0 |
| NOTIF-002 | Reminders: daily review reminder, scheduled workflows, deadline/exam reminders with configurable lead times. | P1 |
| NOTIF-003 | Quiet hours and per-category toggles. | P1 |
| NOTIF-004 | Customisable sounds (bundled set plus user files) for step transitions. | P1 |
| NOTIF-005 | Actionable notifications where the OS allows ("Start review", "Snooze 10 min"). | P1 |

## Search, query language & command palette (SRCH)

| ID | Requirement | Pri |
|---|---|---|
| SRCH-001 | Global full-text search across facts/cards, notes, tasks, exams and resources (titles and content). | P0 |
| SRCH-002 | **Command palette** (Ctrl/Cmd+K) exposing every action, navigation target and workflow. *Dropped with the desktop app (D-034); the VS Code extension registers its commands in VS Code's own palette (EDIT-001).* | P0 |
| SRCH-003 | One shared **query language** ([09 §6](09-data-model.md#6-query-language)) used by search, the card browser, filtered review, workflow steps, widgets, goals and saved views. | P0 (core operators) / P1 (full) |
| SRCH-004 | Saved searches. | P1 |

## Import, export & backup (IO)

| ID | Requirement | Pri |
|---|---|---|
| IO-001 | **Anki import** (`.apkg` / `.colpkg`): note types → card types, notes → facts, cards, decks, tags, media, **and full review history with scheduling state** (all P0 per D-022, because switching costs are the main barrier). See OQ-008. *Status (M0.4+):* implemented for both package formats (legacy schema 11 and latest zstd schema 18), verified against real Anki 26.09 exports. Note types map to built-ins by structure, so localised names work; other types are rendered through their own templates. Field HTML is kept losslessly. History is replayed through FSRS. Re-imports are skipped by note guid. The import is one transaction. Tags are imported onto facts (WS-003). Media goes into the content-addressed store (D-026); fields keep referring to files by name, and a name already used for other content becomes `name-<hash8>.ext`. The summary reports added, already present, renamed, skipped, missing and unused files. 50k cards with 230k reviews, 100k tags and 2,000 media files import in 12–13 s (legacy format with 5,000 files: 16 s) (NFR-PERF-008; `scripts/bench/make_anki_bench.py`). | P0 |
| IO-002 | **CSV/TSV import and export** for facts/cards and tasks, with column mapping. NDJSON (CLI-011) and Markdown card files (SRC-003) cover the MVP. | P1 |
| IO-003 | **Full export archive** (`.keepjayarchive`, an open documented format) that round-trips everything, including customisations and history. *MVP (D-029):* content from sources stays in the user's files; the archive holds everything else, plus the NDJSON review log, which can also be exported on its own (`keepjay export log`). | P0 |
| IO-004 | **Automatic local backups** (default: daily, keep 7 daily + 4 weekly) plus manual backup/restore. | P0 |
| IO-005 | Export any customisation document as a standalone TOML/JSON file (see 03). | P0 |
| IO-006 | **Delete all my data**, with confirmation. | P0 |
| IO-008 | **Markdown interop**: import Markdown files and folders exported from Obsidian, Notion, AFFiNE or Logseq as linked source notes, and convert Q/A and cloze patterns into cards; export notes and cards back to Markdown (D-022). The Markdown card format is SRC-003 (D-028). | P0 |
| IO-007 | Import from Quizlet exports, Markdown folders (Obsidian/Logseq) and Notion exports. | P2 |

## Sync, web & mobile (SYNC, WEB, MOB)

Sync and the mobile review app are Phase 2, the Pro launch. The web app is Phase 3 (D-034). They're listed now because they constrain Phase 1 design (see [08 §9](08-architecture.md#9-sync-design-phase-2-constrains-phase-1)). Hosted sync is a **paid** feature, and the sync server is self-hostable (D-033). The web and mobile apps themselves are free (OQ-025).

| ID | Requirement | Pri |
|---|---|---|
| SYNC-001 | Sync is optional. The app MUST be fully usable without an account, on every platform that can store data locally. | P0 (constraint) |
| SYNC-002 | Multi-device sync of all content, history and customisations. Offline edits merge automatically. The first release is **review-log-first** (D-034): review logs merge as a union, and the phone only reads cards from Git sources. | P2 |
| SYNC-003 | End-to-end encryption option: keys never leave devices in plaintext. | P2 |
| SYNC-004 | Self-hostable sync server (single binary), AGPL-3.0-or-later (D-033, resolves OQ-021). | P2 |
| SYNC-007 | **Hosted backups** (Pro): encrypted, versioned off-device backups with restore to any device (D-023). | P2 |
| SYNC-005 | Device-scoped settings never sync (window state, CLI colours, per-device sounds). | P2 |
| SYNC-006 | Changes that can't merge automatically (e.g. one note body edited on two devices) produce a conflict copy and a notification. | P2 |
| WEB-001 | Browser app reusing the shared review UI (`packages/ui`); installable as a PWA. Phase 3. | P2 |
| WEB-002 | Local-first in the browser (core via WASM, SQLite in OPFS), with sync for cross-device persistence. | P2 |
| MOB-001 | iOS and Android **review app** over the shared core and review UI: Tauri 2 mobile after a spike, native only if widgets or notifications need it (D-034, OQ-006). Phase 2, with Pro. | P2 |
| MOB-002 | Mobile-first flows: quick review, quick capture (share sheet), today view, ready-by-date countdown (SRS-016). | P2 |
| MOB-003 | Home-screen widgets (due count, next target) and actionable review reminders. | P2 |
| MOB-004 | Dashboards render with `narrow` layouts; editing is supported in a simplified form. *Frozen with dashboards (D-028).* | P2 |

## Sharing & community (SHARE)

| ID | Requirement | Pri |
|---|---|---|
| SHARE-001 | Share decks, workflows, dashboards and presets as files. *For the MVP:* decks are shared as Git repositories (SRC-011). | P1 |
| SHARE-002 | Public gallery of presets and decks (requires accounts). | P2 |
| SHARE-003 | Subscribe to shared decks and receive updates without losing personal scheduling. | P2 |
| SHARE-004 | Shared workspaces / study groups / supervisor sharing (OQ-010). | P2 |

## AI assistance (AI) — complement, not competitor (D-023, D-033)

AI assistants and coding agents (ChatGPT, Claude, Claude Code, Cursor, NotebookLM) already explain and generate for free ([research/market.md](research/market.md)). Keepjay is **the memory layer that works with them**:

- the user's own AI tools read and write Keepjay data through an MCP server and the CLI (AI-010, AI-011);
- bring-your-own-key and local models are supported and free (AI-009);
- **hosted AI drafting** (AI-012) is part of Pro.

The hosted AI chat (AI-006…008) is **cut** (D-033).

| ID | Requirement | Pri |
|---|---|---|
| AI-010 | **MCP server** (`keepjay mcp`) exposing core tools to the user's own AI assistant or coding agent: search cards, list due and stale cards, read stats and target readiness, add cards anchored to code ([13 §10](13-code-sources.md#10-mcp-server-ai-010)). Read-only by default; writes need explicit user approval in the client. | P0 |
| AI-011 | **Import from AI tools**: paste or import flashcards and Q&A produced by ChatGPT, Claude or NotebookLM (Markdown, CSV, JSON), with provenance and duplicate detection. | P1 |
| AI-005 | AI features MUST be opt-in and clearly labelled. The user sees **what content will be sent** (e.g. "this note + 12 cards") before it leaves the device. Nothing is sent in the background. | P2 (constraint) |
| AI-006 | **AI study chat**: a chat panel (desktop; later web/mobile; `keepjay ask` in the CLI) that can use selected context: notes, cards, a deck, session history, a reference. Example uses: "explain", "quiz me on this", "summarise this lecture", "what should I review today?". | Cut (D-033) |
| AI-007 | **Usage limits**: per-plan quotas metered by the gateway. Remaining quota is visible in the chat and in Settings → Plan. A clear message appears at the limit; nothing else in the app degrades. *The quota mechanism moves to AI-012.* | Cut (D-033) |
| AI-008 | Chat answers can be turned into cards or notes in one step, with a provenance link (`generated_from` → chat). | Cut (D-033) |
| AI-012 | **Hosted AI drafting (Pro):** slides, PDFs, diffs and coding-agent sessions become draft cards that the user reviews before saving (AI-005). Metered by the AI gateway against a monthly quota; the remaining quota is visible, and nothing else degrades at the limit. Bring-your-own-key and local models stay free (AI-009). | P2 (Pro launch) |
| AI-001 | Generate cards from a note, selection or PDF page. The user reviews them before saving. | P2 |
| AI-002 | Explain an answer or give a hint during review. | P2 |
| AI-003 | Grade free-text answers (type-in / short answer). | P2 |
| AI-004 | Suggest study-plan or workflow tweaks from analytics. | P2 |
| AI-009 | Bring-your-own-key and local models for in-app AI features (OQ-022 resolved by D-023). | P1 |

## Accounts, plans & entitlements (ACCT) — Phase 2 (D-016, D-017)

| ID | Requirement | Pri |
|---|---|---|
| ACCT-001 | Accounts are **optional** and only needed for paid/hosted features. All local features work without one (SYNC-001, D-017). | P2 (constraint) |
| ACCT-002 | Plans (D-033): **Free** (everything local, on every platform, including self-hosted sync); **Supporter** (one-time, no features); **Pro** (hosted sync, hosted backups, hosted AI drafting with a quota), with a **student discount** (ACCT-007) and a 3-month interview-season pass. Creator decks and Teams later. Plan names and pricing in OQ-024. | P2 |
| ACCT-003 | **Entitlements are enforced in the core** from a signed, locally cached entitlement token. Paid features keep working offline for a grace period (e.g. 14 days) before needing a refresh. | P2 |
| ACCT-004 | Downgrade or lapse **never deletes, locks or hides local data**. Sync stops; each device keeps its full copy, which remains usable and exportable. | P2 (constraint) |
| ACCT-005 | Billing via a web checkout (merchant of record handles tax/VAT). Mobile purchases use app-store billing where required. One account and entitlement covers all platforms. | P2 |
| ACCT-006 | Account management: sign up/in (email + passkey/OAuth), devices list, plan, usage, invoices, data export, account deletion. | P2 |
| ACCT-007 | Student/academic discount verification (e.g. institutional email). | P2 |
| ACCT-008 | The CLI supports `keepjay account login|status|logout` via a device-code flow (no password typed into the terminal). | P2 |

---

## Modules specified elsewhere (summary)

| Module | MVP (P0) scope in one line | Spec |
|---|---|---|
| SRC / CODE / PROB / EDIT | `@card` comments and Markdown card files, anchors with change detection, deck repositories, tree-sitter cloze, problem practice from the user's solutions repo, the VS Code extension. | [13](13-code-sources.md) |
| CLI | `keepjay` binary sharing the core and data, TUI review, `review --quick`, `while`, `status --format`, JSON output, stable exit codes, hooks, `cli.toml` with aliases, `$EDITOR` round-trips, MCP server. | [06](06-cli.md) |
| STAT | The metrics engine behind `keepjay stats` (STAT-001, 003, 004). Dashboards are frozen. | [05](05-dashboards-and-widgets.md) |
| CUST | Frozen, except configuration as files (`cli.toml`, deck options as TOML). | [03](03-customization-model.md) |
| SES / WF | Frozen (D-028). | [04](04-workflows.md) |
| DESK | Dropped (D-034). | [07](07-desktop-and-ux.md) |

## MVP at a glance

MVP wedge (D-028): **code-anchored, text-native spaced repetition inside the developer workflow**, local-first and reliable. A Phase 1 public beta user can:

1. Install the CLI or the VS Code extension, scan a repository, and review their first cards in under 2 minutes (J15).
2. Write cards as `@card` comments or Markdown card files, import Anki decks **with full review history**, or subscribe to deck repositories that update without losing their schedule.
3. See cards flagged when the code they're anchored to changes, and keep, edit or retire them (J17).
4. Review in the gaps: `keepjay while -- cargo test`, `review --quick`, the VS Code status bar, a prompt segment (J16).
5. Turn code into cards with tree-sitter cloze, and let their coding agent draft anchored cards through MCP, with approval (J18).
6. Practise algorithm problems from their own solutions repo, by pattern, ready by an interview or exam date (J19, J20).
7. Script everything (JSON, NDJSON, hooks), back up automatically, and export their review history.
