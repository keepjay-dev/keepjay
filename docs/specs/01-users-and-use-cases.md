# 01 — Users & Use Cases

> **Status:** Draft · **Version:** 0.2 · **Last updated:** 2026-11-19
> **Related:** [00-vision](00-vision.md) · [02-features](02-features.md) · [03-customization-model](03-customization-model.md)

## 1. Personas

Personas are fictional composites used to test design decisions.

**Audience path (D-018):** PER-1 University student (MVP primary) → PER-7 Postgraduate researcher → PER-8 Lifelong academic. PER-5 (terminal power user) is cross-cutting. PER-2…PER-4 are served through presets but aren't primary design targets.

| Stage | Persona | Phase | Key additions |
|---|---|---|---|
| 1 | PER-1 University student | 1 (MVP) | Courses, deadlines, exams, flashcards, sessions, dashboards |
| 2 | PER-7 Postgraduate researcher | 2 (v1.x / freemium) | Projects & milestones, reference library, literature notes, writing goals, sync across lab/home machines |
| 3 | PER-8 Lifelong academic | 2–4 | Keeping up with literature, long-term knowledge base, teaching prep, sharing with students |

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

### PER-5 — Terminal power user (cross-cutting)
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
| JTBD-1 | sit down to study | be told what to do next and start immediately | I don't spend willpower on planning |
| JTBD-2 | learn something | capture it as a note or card in seconds | it isn't forgotten |
| JTBD-3 | study regularly | have reviews scheduled optimally | I remember more in less time |
| JTBD-4 | have an exam or deadline | see whether I'm on track and what to prioritise | I avoid last-minute cramming |
| JTBD-5 | notice my routine isn't working | adjust the routine and how I see my progress | the tool fits me, not the reverse |
| JTBD-6 | switch devices or tools | take all my data with me | years of study history aren't lost |

## 3. Key user journeys

Journeys are the backbone of acceptance testing. Each one lists the surfaces it must work on in Phase 1.

### J1 — First run and quick start (Desktop, CLI)
1. Install and launch.
2. Choose a preset (e.g. *University*) from a gallery with previews, or skip (applies *Essentials*).
3. Optionally import an Anki deck or CSV, or add the sample deck.
4. Optionally create a first subject and exam date.
5. Land on the preset's Home dashboard, with a highlighted "Start your first session" action.
6. Start a session. **Target: ≤ 2 minutes from launch to step 6.**

CLI equivalent: `study init --preset university` → `study import anki deck.apkg` → `study session start`.

### J2 — Daily study loop (Desktop)
1. Open the app. The Home dashboard shows due cards, today's tasks, goal progress and the exam countdown.
2. Click *Daily Routine* in the Quick Start widget.
3. The workflow runs: checklist → reviews → new cards → 25-min focus on the top task → break → reflection.
4. The session summary shows time, cards and tasks done. Goals and the heatmap update.

### J3 — Daily study loop (CLI)
1. `study` shows today's summary (due cards, tasks, goals).
2. `study review` opens the TUI review until done or a limit is hit.
3. `study session start pomodoro` runs a TUI timer with the step list. An OS notification fires on each step change.
4. `study stats --range 7d` prints a table; add `--json` for scripts.

### J4 — Customise a workflow (Desktop; CLI via file)
1. Open Workflows → *Pomodoro* (built-in) → Edit. The app forks it (copy-on-write) and marks it "Modified from base".
2. Change focus to 40 min, add a `review` step filtered to `tag:biology`, and add a `reflect` step.
3. Validation runs live. Save.
4. The workflow appears in the Quick Start widget, the command palette and `study workflow list`.
5. Later, "Reset to base" restores the original.

CLI: `study workflow edit pomodoro` opens TOML in `$EDITOR`. The saved file is validated before it is stored.

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
1. `study add card "What produces ATP?" "Mitochondria" --deck Biology`.
2. The open desktop app shows the new card within ~1 s, without a restart (DESK-007).

### J8 — Back up, export, move machines (Desktop, CLI)
1. Automatic local backups run daily.
2. `study export archive ~/study-2026.studyarchive`, or Settings → Data → Export.
3. On a new machine: install → import archive → identical state, including review history and customisations.

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
