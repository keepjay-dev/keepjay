# 02 — Feature Catalogue

> **Status:** Draft · **Version:** 0.2 · **Last updated:** 2026-11-19
> **Related:** [03-customization-model](03-customization-model.md) · [04-workflows](04-workflows.md) · [05-dashboards-and-widgets](05-dashboards-and-widgets.md) · [11-roadmap](11-roadmap.md)

This is the master list of functional requirements. Modules with their own detail spec keep their requirements there, so the two copies can't drift; this file only summarises them (§ "Modules specified elsewhere").

Priority legend: **P0** = MVP, **P1** = v1.x, **P2** = later (see [README](README.md#priorities)).

## Module map

| Code | Module | Requirements in |
|---|---|---|
| WS | Workspaces & organisation | this file |
| SRS | Flashcards & spaced repetition | this file |
| NOTE | Notes | this file |
| SES / WF | Study sessions & workflows | [04-workflows](04-workflows.md) |
| DASH / STAT | Dashboards, widgets & analytics | [05-dashboards-and-widgets](05-dashboards-and-widgets.md) |
| PLAN | Planner: tasks, assignments, exams | this file |
| GOAL | Goals, habits & streaks | this file |
| QUIZ | Practice & quizzes | this file |
| RES | Resources & attachments | this file |
| NOTIF | Notifications & reminders | this file |
| SRCH | Search, query language, command palette | this file (syntax in [09 §6](09-data-model.md#6-query-language)) |
| IO | Import, export & backup | this file (formats in [09](09-data-model.md)) |
| CUST | Customisation & presets | [03-customization-model](03-customization-model.md) |
| CLI | Command-line tool | [06-cli](06-cli.md) |
| DESK | Desktop app | [07-desktop-and-ux](07-desktop-and-ux.md) |
| SYNC / WEB / MOB | Sync, web app, mobile apps | this file (design in [08](08-architecture.md)) |
| SHARE | Sharing & community | this file |
| AI | AI study assistance (paid, usage-limited) | this file |
| ACCT | Accounts, plans & entitlements (freemium, Phase 2) | this file (design in [08 §15](08-architecture.md#15-cloud-services-accounts-and-entitlements-phase-2)) |

---

## Workspaces & organisation (WS)

| ID | Requirement | Pri |
|---|---|---|
| WS-001 | Users MUST be able to create, rename, archive and delete multiple **workspaces**. Each workspace isolates its content, customisations and applied preset. | P0 |
| WS-002 | A workspace contains a tree of **subjects** (e.g. *Biology › Cell Biology*). Depth is unlimited in data; the UI is optimised for up to 4 levels. | P0 |
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
| SRS-003 | **FSRS** MUST be the default scheduler (D-005). The scheduler interface MUST be pluggable. SM-2 and Leitner SHOULD be selectable in deck options. | P0 (FSRS) / P1 (others) |
| SRS-004 | **Deck options** (a customisation document): daily new/review limits, desired retention, learning/relearning steps, maximum interval, sibling burying, new-card order. | P0 |
| SRS-005 | **Review screen**: prompt → reveal → grade (Again / Hard / Good / Easy) with next-interval previews. Supports undo of the last answer, in-place edit, flag, suspend, bury, and fully keyboard-driven operation. | P0 |
| SRS-006 | Every answer MUST be written to an append-only **review log** (card, rating, timestamp, time taken, previous state, scheduler + version). Card scheduling state MUST be reproducible by replaying the log. | P0 |
| SRS-007 | **Duplicate detection** on add (same first field, same card type, same deck), with a warning and an override option. | P0 |
| SRS-008 | Card content supports Markdown, images, LaTeX math (KaTeX) and code highlighting. Audio playback is P1. | P0 / P1 |
| SRS-009 | **Card browser**: search with the query language, sort, multi-select, bulk operations (move, tag, suspend, reschedule, change card type), preview. | P0 |
| SRS-010 | **Filtered review**: an ad-hoc queue from any query (e.g. failed last week), with an optional *cram* mode that doesn't affect scheduling. | P1 |
| SRS-011 | **FSRS parameter optimisation** from the user's own review history, per deck options. | P1 |
| SRS-012 | **Leech** detection (lapses ≥ threshold), with a configurable action (tag or suspend). | P1 |
| SRS-013 | **Template editing** using a logic-less, Mustache-like syntax (`{{Field}}`, `{{#Field}}…{{/Field}}`, `{{cloze:Text}}`) and scoped CSS. No scripting (D-013). | P1 |
| SRS-014 | Due-date load balancing ("fuzz") to avoid review spikes. | P1 |
| SRS-015 | **Image occlusion** card type. | P2 |

## Notes (NOTE)

| ID | Requirement | Pri |
|---|---|---|
| NOTE-001 | Markdown notes with live preview, organised by subject and tags, with title, body, pin/favourite. | P0 |
| NOTE-002 | Math, code blocks, tables, checklists, and images pasted or dropped into managed media. | P0 |
| NOTE-003 | Export notes as plain `.md` files with front-matter, individually or in bulk. | P0 |
| NOTE-004 | `[[wiki links]]` between notes and to cards, tasks and exams, with a backlinks panel. | P1 |
| NOTE-005 | **Cards from notes**: select text → create a basic or cloze card. The card keeps a `generated_from` link to the note. | P1 |
| NOTE-006 | Note templates (Lecture, Cornell, Summary, Book chapter) as customisation documents. | P1 |
| NOTE-007 | Revision history for note bodies (last N saves). | P1 |
| NOTE-008 | Optional two-way mirror of notes to a folder of Markdown files (Obsidian-compatible). | P2 |
| NOTE-009 | **Citations** in notes (`[@key]`) linked to the reference library (RES-005), with a bibliography rendered on export. Academic track. | P2 |
| NOTE-010 | **Literature notes**: a note template bound to a reference (summary, key claims, methods, quotes with page numbers). Academic track. | P2 |

## Planner: tasks, assignments, exams (PLAN)

| ID | Requirement | Pri |
|---|---|---|
| PLAN-001 | **Tasks**: title, description, subject, tags, status (todo / doing / done), priority, due date/time, estimate, checklist. | P0 |
| PLAN-002 | **Assignments** (a task kind with a submission date and optional grade) and **exams** (date/time, location, linked subjects/decks, target score). | P0 |
| PLAN-003 | Exam **countdown** (P0) and **on-track indicator** based on remaining new cards, due load and days left (P1). | P0 / P1 |
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

| ID | Requirement | Pri |
|---|---|---|
| GOAL-001 | **Goals** = metric + target + period + scope. Examples: "90 min/day of study"; "300 reviews/week in `deck:Japanese`"; "finish all new cards in `deck:Biochem` by 2027-05-01". | P0 |
| GOAL-002 | Progress is computed automatically from metrics ([05 §5](05-dashboards-and-widgets.md#5-metrics-catalogue)). Manual habit goals use check-ins (P1). | P0 / P1 |
| GOAL-003 | Goal history (met/missed per period) is available to widgets and the CLI. | P0 |
| GOAL-004 | Streaks, gamification and celebratory animations can be turned off entirely (module toggle, CUST-006). | P0 |
| GOAL-005 | **Forgiving streaks** per goal: rest days per week, streak freezes, a "minimum viable day" threshold. | P1 |
| GOAL-006 | **Writing goals** (words written per day/week in notes or a project) for essays and theses. Academic track. | P2 |

## Practice & quizzes (QUIZ)

| ID | Requirement | Pri |
|---|---|---|
| QUIZ-001 | **Question items**: multiple choice (single/multi), true/false, short answer, with explanation and tags. Implemented as a card-type family so they can also participate in SRS. | P1 |
| QUIZ-002 | **Practice test**: N questions from a query, optional time limit, scoring, review of mistakes. | P1 |
| QUIZ-003 | **Exam simulation**: timed, no feedback until the end, results per subject/tag. | P1 |
| QUIZ-004 | Missed questions can be tagged or queued for SRS review automatically. | P1 |
| QUIZ-005 | Import question banks (CSV / documented JSON). | P2 |

## Resources & attachments (RES)

| ID | Requirement | Pri |
|---|---|---|
| RES-001 | Attach files (any type) and links to subjects, notes, tasks and exams. Files live in managed, content-addressed media storage. | P0 |
| RES-002 | Resource library view: filter by subject, tag or type; open in the OS default app. | P1 |
| RES-003 | Built-in PDF viewer with highlights that can become notes or cards, linked back to the page. | P2 |
| RES-004 | Reading-progress tracking (pages/chapters) that feeds `read` workflow steps and goals. | P2 |
| RES-005 | **Reference library**: import/export BibTeX and CSL-JSON, sync with Zotero (OQ-013), and store metadata (authors, year, DOI, tags, PDF attachment). Academic track. | P2 |
| RES-006 | **Reading lists** per subject/project with status (to read, reading, read) and priority. Academic track. | P2 |

## Notifications & reminders (NOTIF)

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
| SRCH-002 | **Command palette** (Ctrl/Cmd+K) exposing every action, navigation target and workflow. | P0 |
| SRCH-003 | One shared **query language** ([09 §6](09-data-model.md#6-query-language)) used by search, the card browser, filtered review, workflow steps, widgets, goals and saved views. | P0 (core operators) / P1 (full) |
| SRCH-004 | Saved searches. | P1 |

## Import, export & backup (IO)

| ID | Requirement | Pri |
|---|---|---|
| IO-001 | **Anki import** (`.apkg` / `.colpkg`): note types → card types, notes → facts, cards, decks, tags and media (P0). Review history and scheduling state (P1). See OQ-008. | P0 / P1 |
| IO-002 | **CSV/TSV import and export** for facts/cards and tasks, with column mapping. | P0 |
| IO-003 | **Full export archive** (`.studyarchive`, an open documented format) that round-trips everything, including customisations and history. | P0 |
| IO-004 | **Automatic local backups** (default: daily, keep 7 daily + 4 weekly) plus manual backup/restore. | P0 |
| IO-005 | Export any customisation document as a standalone TOML/JSON file (see 03). | P0 |
| IO-006 | **Delete all my data**, with confirmation. | P0 |
| IO-007 | Import from Quizlet exports, Markdown folders (Obsidian/Logseq) and Notion exports. | P2 |

## Sync, web & mobile (SYNC, WEB, MOB)

Phase 2–3 features. They are listed now because they constrain Phase 1 design (see [08 §9](08-architecture.md#9-sync-design-phase-2-constrains-phase-1)). Hosted sync is a **paid** feature (D-016). The web and mobile apps themselves are free (OQ-025).

| ID | Requirement | Pri |
|---|---|---|
| SYNC-001 | Sync is optional. The app MUST be fully usable without an account, on every platform that can store data locally. | P0 (constraint) |
| SYNC-002 | Multi-device sync of all content, history and customisations. Offline edits merge automatically. | P2 |
| SYNC-003 | End-to-end encryption option: keys never leave devices in plaintext. | P2 |
| SYNC-004 | Self-hostable sync server (single binary). **Deferred** pending OQ-021 (conflicts with paid hosted sync). | P2 |
| SYNC-005 | Device-scoped settings never sync (window state, CLI colours, per-device sounds). | P2 |
| SYNC-006 | Changes that can't merge automatically (e.g. one note body edited on two devices) produce a conflict copy and a notification. | P2 |
| WEB-001 | Browser app reusing the desktop UI package; installable as a PWA. | P2 |
| WEB-002 | Local-first in the browser (core via WASM, SQLite in OPFS), with sync for cross-device persistence. | P2 |
| MOB-001 | Native iOS and Android apps over the shared core (D-012, OQ-006). | P2 |
| MOB-002 | Mobile-first flows: quick review, run workflows, quick capture (share sheet), today view. | P2 |
| MOB-003 | Home-screen widgets (due count, streak, next exam) and actionable notifications. | P2 |
| MOB-004 | Dashboards render with `narrow` layouts; editing is supported in a simplified form. | P2 |

## Sharing & community (SHARE)

| ID | Requirement | Pri |
|---|---|---|
| SHARE-001 | Share decks, workflows, dashboards and presets as files. | P1 |
| SHARE-002 | Public gallery of presets and decks (requires accounts). | P2 |
| SHARE-003 | Subscribe to shared decks and receive updates without losing personal scheduling. | P2 |
| SHARE-004 | Shared workspaces / study groups / supervisor sharing (OQ-010). | P2 |

## AI study assistance (AI) — paid, usage-limited (D-016)

AI arrives with the freemium phase (Phase 2) as a paid feature, served through our AI gateway with per-plan usage limits ([08 §15](08-architecture.md#15-cloud-services-accounts-and-entitlements-phase-2)). The provider is abstracted behind the gateway and chosen at Phase 2 planning.

| ID | Requirement | Pri |
|---|---|---|
| AI-005 | AI features MUST be opt-in and clearly labelled. The user sees **what content will be sent** (e.g. "this note + 12 cards") before it leaves the device. Nothing is sent in the background. | P2 (constraint) |
| AI-006 | **AI study chat**: a chat panel (desktop; later web/mobile; `study ask` in the CLI) that can use selected context: notes, cards, a deck, session history, a reference. Example uses: "explain", "quiz me on this", "summarise this lecture", "what should I review today?". | P2 (first AI feature) |
| AI-007 | **Usage limits**: per-plan quotas metered by the gateway. Remaining quota is visible in the chat and in Settings → Plan. A clear message appears at the limit; nothing else in the app degrades. | P2 |
| AI-008 | Chat answers can be turned into cards or notes in one step, with a provenance link (`generated_from` → chat). | P2 |
| AI-001 | Generate cards from a note, selection or PDF page. The user reviews them before saving. | P2 |
| AI-002 | Explain an answer or give a hint during review. | P2 |
| AI-003 | Grade free-text answers (type-in / short answer). | P2 |
| AI-004 | Suggest study-plan or workflow tweaks from analytics. | P2 |
| AI-009 | Bring-your-own-key and local models (OQ-022). | P2 (open) |

## Accounts, plans & entitlements (ACCT) — Phase 2 (D-016, D-017)

| ID | Requirement | Pri |
|---|---|---|
| ACCT-001 | Accounts are **optional** and only needed for paid/hosted features. All local features work without one (SYNC-001, D-017). | P2 (constraint) |
| ACCT-002 | Plans: **Free** (everything local, on every platform) and **Pro** (hosted sync, AI chat with a usage quota, and later hosted extras; OQ-023). Plan names and pricing in OQ-024. | P2 |
| ACCT-003 | **Entitlements are enforced in the core** from a signed, locally cached entitlement token. Paid features keep working offline for a grace period (e.g. 14 days) before needing a refresh. | P2 |
| ACCT-004 | Downgrade or lapse **never deletes, locks or hides local data**. Sync stops; each device keeps its full copy, which remains usable and exportable. | P2 (constraint) |
| ACCT-005 | Billing via a web checkout (merchant of record handles tax/VAT). Mobile purchases use app-store billing where required. One account and entitlement covers all platforms. | P2 |
| ACCT-006 | Account management: sign up/in (email + passkey/OAuth), devices list, plan, usage, invoices, data export, account deletion. | P2 |
| ACCT-007 | Student/academic discount verification (e.g. institutional email). | P2 |
| ACCT-008 | The CLI supports `study account login|status|logout` via a device-code flow (no password typed into the terminal). | P2 |

---

## Modules specified elsewhere (summary)

| Module | MVP (P0) scope in one line | Spec |
|---|---|---|
| SES / WF | All sessions run through the workflow engine. Step types: focus, break, review, learn, checklist, reflect, message, repeat. Base workflows, list editor, parameters, TOML import/export, crash-safe resume, shared control from desktop and CLI. | [04](04-workflows.md) |
| DASH / STAT | Multiple dashboards per workspace, grid edit mode, 12 P0 widgets, metrics engine in core, stats screen, `study stats`. | [05](05-dashboards-and-widgets.md) |
| CUST | Onboarding presets, documents with fork/reset-to-base, layered settings, module toggles, themes, re-apply preset, import/export. | [03](03-customization-model.md) |
| CLI | `study` binary sharing the core and data, TUI review/session, JSON output, stable exit codes, `cli.toml` with aliases, `$EDITOR` round-trips. | [06](06-cli.md) |
| DESK | Win/macOS/Linux, sidebar + palette navigation, onboarding < 2 min, keyboard-first, live refresh from the CLI, signed auto-updates, CLI install. | [07](07-desktop-and-ux.md) |

## MVP at a glance

A Phase 1 public beta user can:

1. Install the desktop app (with the CLI), pick a preset, and start a session in under 2 minutes.
2. Create workspaces, subjects, decks, tags; add, import (Anki, CSV) and review cards with FSRS.
3. Write Markdown notes; manage tasks, assignments and exams with countdowns.
4. Run built-in or custom workflows from the desktop or the CLI, with crash-safe sessions.
5. Customise dashboards (12 widget types) and workflows, theme the app, hide unused modules, and reset anything to base.
6. Track goals and see stats on the desktop, in the CLI, or as JSON.
7. Back up, export everything to an open format, and restore on another machine.
