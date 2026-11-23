# 01 — Users & Use Cases

> **Status:** Draft · **Version:** 0.3 · **Last updated:** 2026-11-23
> **Related:** [00-vision](00-vision.md) · [02-features](02-features.md) · [13-code-sources](13-code-sources.md) · research/developer-strategy *(private notes)*

## 1. Personas

Personas are fictional composites used to test design decisions.

**Audience (D-027):** three rings, in order. They share one engine: capture → schedule → review in the gaps.

| Ring | Persona | Role | Hook |
|---|---|---|---|
| 1 | PER-9 Enthusiast engineer | Free launch community | Cards in your code that notice when the code changes, reviewed while the build runs |
| 2 | PER-10 CS student, PER-11 Interview candidate | Volume and first revenue | Your solutions repo becomes a review plan, ready by interview or exam day, on your phone |
| 3 | PER-12 Engineer learning on the job | Expansion | Your coding agent drafts cards from today's session; you approve them in one keystroke |
| Later | PER-13 Engineering lead | Buyer for Teams, only on demand | Onboarding decks anchored to the team's repos, aggregate progress only |

The general study personas (PER-1…PER-8, D-018) are **frozen** (§1.2). PER-5 merges into PER-9.

### 1.1 Developer personas

| ID | Persona | Works in | Wants to retain | Reviews when | Would pay for |
|---|---|---|---|---|---|
| PER-9 | **Enthusiast engineer** | Terminal, Neovim/Emacs or VS Code; dotfiles in Git | Language internals, their codebase, books and papers | Build and test waits; terminal | Supporting the project; maybe sync |
| PER-10 | **CS student** | VS Code, IntelliJ in Java courses, a phone; often Windows | Course concepts, data structures and algorithms, complexity, maths | Commutes and queues (phone); before exams | Little: student-priced Pro, free offers |
| PER-11 | **Interview candidate** (student or job switcher) | LeetCode in the browser, a solutions repo (often synced by a LeetHub-style extension), VS Code | Problem patterns, their own solutions, system design | Daily during a prep season; phone | **Yes:** already pays for LeetCode Premium or NeetCode Pro |
| PER-12 | **Engineer learning on the job** | VS Code, Cursor or JetBrains; coding agents | New language and framework concepts, debugging lessons, domain knowledge, certification content | Build waits; editor | Pro, especially if an employer's learning budget covers it |
| PER-13 | **Engineering lead** (later buyer) | — | The team's ramp-up on its codebase | — | Per-seat Teams |

**Success, by persona:**

- **PER-9:** reviews cards from their own repos while tests run, and their cards never silently go stale.
- **PER-10:** is ready for the exam on the day, without cramming.
- **PER-11:** re-solves a problem pattern in the interview that they first met weeks ago.
- **PER-12:** keeps the concepts their coding agent explained, not just the code it wrote.

### 1.2 Frozen: general study personas (D-018, superseded by D-027)

Kept for the general study audience, which can return by unfreezing the specs. They aren't design targets for the MVP.

### PER-1 — University student
- Takes 4–6 courses per semester, with weekly assignments, midterms and finals.
- **Needs:** one place for deadlines, lecture notes, per-course flashcards, and a clear answer to "am I on track?".
- **Today:** a planner app, Anki, Notion and a phone timer, none of them connected.
- **Success:** opens the app, sees what's due today across courses, runs a study block, and sees progress per course.

### PER-2 — High-stakes exam candidate
- Preparing for one big exam on a fixed date (medical boards, bar, CFA, professional certifications, language proficiency tests).
- Very high card volumes (10k–50k) and question banks, with strict daily targets.
- **Needs:** an exam countdown, daily targets derived from the exam date, fast review at volume, practice tests, and retention analytics.

### PER-3 — Language learner
- A daily habit over months or years: reviews vocabulary, reads, listens, writes.
- **Needs:** reverse and type-in cards, audio, solid CJK/RTL text input (IME), streaks with rest days, and quick 10-minute sessions (on mobile later).

### PER-4 — Self-directed learner
- Learning programming, an instrument, or an online course on the side.
- **Needs:** goals and projects, notes linked to cards, deep-work sessions, and light structure without a semester calendar.

### PER-5 — Terminal power user (cross-cutting; merged into PER-9)
- A developer or researcher who lives in the terminal and keeps dotfiles in git. Overlaps with any persona above.
- **Needs:** a fast CLI, scriptable JSON output, config as files, `$EDITOR` integration, TUI review, and Vim keybindings.

### PER-6 — Educator / content creator (later)
- Creates decks and presets for students. Phase 4 (OQ-010). Overlaps with PER-8.

### PER-7 — Postgraduate researcher (Master's / PhD)
- Spends months or years on one dissertation or thesis, alongside coursework, qualifying exams, teaching duties and papers.
- **Needs:** projects with milestones (chapters, submissions), a reference library with citations (Zotero/BibTeX), literature notes linked to references, reading lists, writing goals, deep-work sessions, supervisor meeting notes, and data that syncs between lab and home machines.
- **Today:** Zotero + Word/LaTeX + Notion/Obsidian + a to-do app + occasional Anki for qualifying exams.
- **Success:** sees thesis progress at a glance, runs a "writing sprint" workflow daily, and turns reading into notes and cards without losing the source.

### PER-8 — Lifelong academic (postdoc, lecturer, professor, independent scholar)
- Keeps up with the literature in their field over decades, maintains expertise, prepares teaching, learns adjacent fields.
- **Needs:** a durable, exportable long-term knowledge base, reading pipelines, spaced review of key concepts and papers, light planning, and sharing material with students.
- **Success:** decades of notes and review history stay intact and portable (principle 8); the setup grows with them, not against them.

## 2. Jobs to be done

| # | When I… | I want to… | So that… |
|---|---|---|---|
| JTBD-7 | learn something while coding (from code, docs or an AI assistant) | capture it as a card in seconds, where I already am | it isn't forgotten |
| JTBD-8 | wait on a build or a test run, or sit on a train | review a few cards | reviewing costs me no extra time |
| JTBD-9 | change code I wrote cards about | be told which cards may now be wrong | my cards stay true |
| JTBD-10 | solved a problem weeks ago | re-solve it at the right time, by pattern | I can solve it in the interview |
| JTBD-11 | have an exam or interview on a date | know I'll be ready on the day | I avoid last-minute cramming |
| JTBD-12 | rely on an AI assistant to explain things | keep the concepts it taught me | I don't depend on it for them |
| JTBD-3 | study regularly | have reviews scheduled optimally | I remember more in less time |
| JTBD-6 | switch devices or tools | take all my content and history with me | years of review history aren't lost |

Frozen with the general study audience: JTBD-1 (be told what to do next), JTBD-2 (capture as a note), JTBD-4 (on-track planning) and JTBD-5 (adjust routines).

## 3. Key user journeys

Journeys are the backbone of acceptance testing. Each one lists the surfaces it must work on.

**Active journeys (D-028):** J15–J20, plus J3 (CLI daily loop) and J8 (back up, export, move machines). J15 replaces J1. The rest (J1, J2, J4–J7, J9–J14) are frozen with their specs.

### J15 — First scan (CLI; VS Code from M3)
1. Install the CLI, or the VS Code extension, which bundles it.
2. In a repository: `keepjay source add .`, then `keepjay scan`. Cards come from `@card` comments and `*.cards.md` files, and their ids are written back into the files (OQ-028).
3. `keepjay review` (or *Review* in VS Code). **Target: ≤ 2 minutes from install to the first answered card.**

Other ways in: `keepjay import anki deck.apkg` (with review history), or `keepjay deck subscribe <git-url>`.

*Checked 2026-11-23 (CLI, 0.2.0-alpha.1 release build):* passes. Download and checksum to a running binary took 2 s, and `source add .` to an answered card took 3 s with scripted keys. Fixed after the check: `source add` on a fresh install failed with "run `keepjay init` first"; it now creates the workspace. A teammate's clone keeps every card id. The VS Code steps wait for M3.

### J16 — Review while waiting (CLI, VS Code)
1. `keepjay while -- cargo test` starts the tests and opens a review.
2. When the tests finish, the review stops, the end of the test output is shown, and the command returns the tests' exit code.
3. In VS Code, the status bar shows "5 due". Clicking it opens the review panel while a task runs.

*Checked 2026-11-23 (CLI, 0.2.0-alpha.1 release build, in a pseudo-terminal):* passes. With passing tests, a card answered during the run, the output tail shown, exit 0. With a failing test, exit 101 and the panic in the tail. `q` streams the output and still returns 101. Ctrl+C stops `cargo test` and the test binary it started, and exits 130, with nothing left running. The VS Code steps wait for M3.

### J17 — Code changed (CLI, VS Code)
1. A teammate rewrites a function that a card is anchored to.
2. The next scan (manual, from a Git hook, or by VS Code on save) flags the card as stale.
3. In `keepjay stale`, or from the gutter mark in VS Code, the user keeps, edits (opens the file at the line) or retires the card. Review history is kept in every case.

### J18 — Agent-made cards (any MCP client)
1. After a debugging session, the user asks their coding agent to make cards from what they learned.
2. The agent calls the MCP server with draft cards anchored to the changed code, and the client asks for approval.
3. Approved cards are written to a Markdown card file (or as `@card` comments) and scheduled.

### J19 — Problem practice (CLI, VS Code)
1. A LeetHub-style extension already pushes the user's accepted solutions to a GitHub repo.
2. `keepjay source add ~/leetcode --kind problems` imports each solution as a problem card (URL, title, difficulty, pattern tags, solution file).
3. When a card is due, the user re-solves the problem and grades the outcome (failed, with hints, solved, solved fast).
4. `keepjay stats --by tag --scope 'card_type:problem'` shows weak patterns.

### J20 — Ready by interview day (CLI, VS Code; mobile in Phase 2)
1. `keepjay target set Interview --scope 'deck:Interview' --date 2027-02-15`.
2. Daily reviews adjust to reach the target retention on that day, and `status` shows the predicted retention for the date.
3. After the date, the deck returns to normal long-term spacing.

---

The journeys below come from the general study app. **J3 and J8 still apply** (J3 without its session step). The others are frozen.

### J1 — First run and quick start (Desktop, CLI) — frozen, replaced by J15
1. Install and launch.
2. Choose a preset (e.g. *University*) from a gallery with previews, or skip (applies *Essentials*).
3. Optionally import an Anki deck or CSV, or add the sample deck.
4. Optionally create a first subject and exam date.
5. Land on the preset's Home dashboard, with a highlighted "Start your first session" action.
6. Start a session. **Target: ≤ 2 minutes from launch to step 6.**

CLI equivalent: `keepjay init --preset university` → `keepjay import anki deck.apkg` → `keepjay session start`.

### J2 — Daily study loop (Desktop)
1. Open the app. The Home dashboard shows due cards, today's tasks, goal progress and the exam countdown.
2. Click *Daily Routine* in the Quick Start widget.
3. The workflow runs: checklist → reviews → new cards → 25-min focus on the top task → break → reflection.
4. The session summary shows time, cards and tasks done. Goals and the heatmap update.

### J3 — Daily study loop (CLI) — active
1. `keepjay` shows today's summary (due and stale cards, the next target).
2. `keepjay review` opens the TUI review until done or a limit is hit.
3. *(Frozen with sessions, D-028:)* `keepjay session start pomodoro` runs a TUI timer with the step list.
4. `keepjay stats --range 7d` prints a table; add `--json` for scripts.

### J4 — Customise a workflow (Desktop; CLI via file)
1. Open Workflows → *Pomodoro* (built-in) → Edit. The app forks it (copy-on-write) and marks it "Modified from base".
2. Change focus to 40 min, add a `review` step filtered to `tag:biology`, and add a `reflect` step.
3. Validation runs live. Save.
4. The workflow appears in the Quick Start widget, the command palette and `keepjay workflow list`.
5. Later, "Reset to base" restores the original.

CLI: `keepjay workflow edit pomodoro` opens TOML in `$EDITOR`. The saved file is validated before it is stored.

### J5 — Customise a dashboard (Desktop)
1. Home → *Edit layout*.
2. Add the *Heatmap* and *Countdown* widgets, resize *Due cards*, remove *Text*.
3. Configure the heatmap to show `subject:Physics` only.
4. Save. Create a second dashboard, "Exam week", from a template and set it as Home during the exam period.

### J6 — Plan for an exam (Desktop, CLI)
1. Add the exam "Biochem Final" with a date and link its decks and subjects.
2. The app proposes daily targets (cards/day, minutes/day) to finish new cards a configurable number of days before the exam (P1).
3. A countdown and an on-track indicator appear on the dashboard, and a goal is created.

### J7 — Quick capture across surfaces (CLI → Desktop)
1. `keepjay add card "What produces ATP?" "Mitochondria" --deck Biology`.
2. The open desktop app shows the new card within ~1 s, without a restart (DESK-007).

### J8 — Back up, export, move machines (CLI) — active
1. Automatic local backups run daily.
2. `keepjay export archive ~/study-2026.keepjayarchive`. Content from sources stays in the user's own files (D-029); `keepjay export log` writes the review history as NDJSON.
3. On a new machine: install → import archive → clone the same repositories → identical state, including review history.

### J9 — Recover from over-customisation (Desktop, CLI)
1. A user has heavily modified settings and dashboards and is unhappy with the result.
2. Per item: *Reset to base*. Per workspace: *Re-apply preset* (content is untouched).
3. Revision history (P1) can restore yesterday's dashboard.

### J10 — Share a setup (Desktop, CLI) — P1
1. *Save as preset* bundles selected workflows, dashboards, card types and settings into a file.
2. A friend imports the file and picks which parts to apply.

### J11 — Multi-device (Phase 2)
1. Opt in to sync (optional account, E2EE).
2. Review on the web at the library. The desktop app reflects those reviews and the session history.

### J12 — On the go (Phase 3)
1. A phone reminder: "12 cards due".
2. Quick review in the native mobile app. A home-screen widget shows the due count and streak.

### J13 — Thesis project (PER-7, Phase 2)
1. Create the project "PhD thesis" with chapter milestones and a submission date (PLAN-011).
2. Import references from Zotero (RES-005); write literature notes per paper (NOTE-010).
3. Run a daily *Writing Sprint* workflow; a writing goal tracks words per week (GOAL-006).
4. The project dashboard shows milestone progress, words written, reading-list status and the next supervisor meeting.

### J14 — Ask the AI study chat (Pro, Phase 2)
1. Select a lecture note and its deck → *Ask AI*. The app shows exactly what will be sent (AI-005).
2. "Quiz me on this" → answer in chat → convert useful Q&A into cards in one step (AI-008).
3. The remaining monthly quota is visible. At the limit, chat pauses with a clear message, and nothing else is affected (AI-007).
