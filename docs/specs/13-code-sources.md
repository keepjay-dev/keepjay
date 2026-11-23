# 13 — Code Sources, Code-Native Cards, Problem Practice & Editor Integrations

> **Status:** Draft · **Version:** 0.1 · **Last updated:** 2026-11-23
> **Related:** [02-features](02-features.md) · [06-cli](06-cli.md) · [09-data-model](09-data-model.md) · decisions D-028, D-029, D-030, D-034 · [research/developer-pivot](research/developer-pivot.md) · [research/developer-strategy](research/developer-strategy.md)

This file defines four modules at the heart of the developer MVP (D-028):

- **SRC:** how cards come from code and text files;
- **CODE:** how code is shown and turned into cards;
- **PROB:** how algorithm problems are practised;
- **EDIT:** the editor and browser integrations.

## 1. Principles

1. **Text is the source; the database is state** (D-029). Card content from sources lives in the user's files. SQLite keeps scheduling, history and anchors.
2. **Never execute code.** The scanner only reads, and no card type runs anything (D-013).
3. **Keep history through change.** Card edits, moves and upstream deck updates keep review history (principle 8).
4. **Read only what the user registered.** No crawling of the home directory and no network access during scans.
5. **One implementation.** Scanning, anchoring and change detection live in the core. The CLI, the VS Code extension and the MCP server call the same functions.

## 2. Requirements

### Sources (SRC)

| ID | Requirement | Pri |
|---|---|---|
| SRC-001 | Register folders and Git repositories as **sources** (`keepjay source add <path>`), with a deck, tags and a kind (`repo`, `folder`, `problems`, `deck_repo`). Scan on demand (`keepjay scan`), and optionally from Git hooks installed by `keepjay source hooks install`. Sources have a name, unique per workspace, used to address them. Removing a source keeps the cards already created from it and their history; registering the folder again reconnects them through the ids in the files. *Status (M0.5):* `source add|list|rm` implemented (schema v7). The kind (`repo` or `folder`) is detected with the system `git`, and a repository's identity is its root commit. `--kind problems` comes with PROB-005; hooks come with M1. | P0 |
| SRC-002 | Find `@card` cards in line comments using a table of comment prefixes per file extension (§3), in Q/A and `{{c1::…}}` cloze forms. Block comments: P1. *Status (M0.5):* parser implemented (`sources/comments.rs`). It also accepts doc-comment markers (`///`, `//!`, `##`, `;;`); a card's answer must use the same marker and indentation. The anchored code stops at a blank line or the next card. Malformed cards (no question, no answer, a bad id) become warnings with file and line; the scan goes on. | P0 |
| SRC-003 | Read **Markdown card files** (`*.cards.md`, and any `.md` under `.keepjay/cards/`) with Anki-compatible cloze syntax (§4; IO-008). *Status (M0.5):* parser implemented (`sources/markdown.rs`). An `A:` after a `C:` fills *Extra*. Text before the first card is prose. | P0 |
| SRC-004 | Each source card stores an **anchor**: repository identity (root commit), relative path, line range and a hash of the anchored code (§5), plus the symbol path once CODE-002 lands. *Status (M0.5):* stored in `anchors` on every scan (schema v7); line numbers follow moved cards. | P0 |
| SRC-005 | **Stable identity:** on first scan, write the card's short id into the source (`@card(k22k1m8e)`), opt-out per source (OQ-028). Without ids, match by anchor and text similarity, and ask when unsure. Edits and moves keep review history. *Status (M0.5):* implemented. Ids are written after the database commit, in place for comments and as an added line for Markdown. Writes are atomic and keep line endings and permissions. An id found in a file but unknown here (a teammate's clone) is kept: the fact gets that short id. A repeated id (a copied card) gets a new id with a warning. Without an id, cards are matched by file and first field; fuzzy matching comes in M1. | P0 |
| SRC-006 | **Change detection:** when anchored code changes, the card is flagged stale and listed by `keepjay stale` (keep, edit or retire). Cards removed from the source are retired, and their history is kept. *Status (M0.5):* implemented. Editing the card's text also clears the flag. A retired card that reappears is restored with its history. `keepjay stale` offers `list`, `keep` and `retire`. Because the file holds the card (D-029), `retire` removes the card's own text from the file (its comment block, or its Markdown lines with the id comment and one blank line), never the code under it, and then scans the source: the card retires with its history and the other cards' lines update. It shows the lines and asks first on a terminal; scripts need `--yes`, and `--dry-run` shows the lines without changing anything. If the card is no longer in the file, it says so and edits nothing. `stale edit` arrives with SRC-007 in M1. | P0 |
| SRC-007 | `keepjay edit <card>` and `keepjay open <card>` open `$EDITOR` at the card's file and line. | P0 |
| SRC-008 | The scanner reads only registered sources. It respects `.gitignore` and `.keepjayignore` (and, from before the rename, `.scrubjayignore` and `.scrubjay/cards/`; D-037), skips binary files and files over 1 MiB, doesn't follow symlinks out of the source root, and never executes code. *Status (M0.5):* walker implemented with the `ignore` crate. It also skips hidden files and folders other than `.keepjay/` (so `.env` is never read), and honours `.git/info/exclude` and the global Git ignore file. Symlinks aren't followed at all. A file that can't be read is reported as a warning; the rest of the source is still scanned. | P0 (constraint) |
| SRC-009 | Re-scanning a 10,000-file source takes under 1 s with a warm file cache (NFR-PERF-009). *Status (M0.5):* met. On a generated 10,000-file tree with about 4,000 cards (`scripts/bench/make_source_bench.py`), on a 2016 MacBook Pro, Intel i7-6567U: a re-scan takes 0.25–0.37 s and a first scan that writes ids into 966 files takes 1.6 s. Files are read in parallel. Unchanged cards write nothing, so a re-scan adds nothing to the change log. | P0 |
| SRC-010 | **Sidecar cards** anchored to `path#symbol`, for repositories where teammates don't want card comments in the code. | P1 |
| SRC-011 | **Deck repositories:** `keepjay deck subscribe <git-url>` clones a deck repo; `keepjay deck pull` updates it, keeping review history through stable ids (SRC-005). Local edits can be offered upstream as a patch (`keepjay deck diff`). | P0 |
| SRC-012 | **Scan switches:** a comment `@card:off` stops finding cards until a comment `@card:on`, or to the end of the file; `<!-- @card:off -->` and `<!-- @card:on -->` do the same in Markdown card files. Text after the switch is allowed (a reason). An unknown `@card:` word is a warning. For files whose strings or examples contain `@card`, such as a parser's tests, without leaving them out entirely with `.keepjayignore`. *Status (M0.5):* implemented. | P0 |

### Code-native cards (CODE)

| ID | Requirement | Pri |
|---|---|---|
| CODE-001 | Code fields keep whitespace exactly and carry a language. The TUI and the shared review UI highlight syntax. | P0 |
| CODE-002 | **Tree-sitter cloze:** `keepjay card from-code <path>:<symbol> --hide body\|signature\|return\|params` generates ordinary cloze facts from syntax-tree spans, anchored to the symbol. Four languages at the MVP (OQ-030). | P0 |
| CODE-003 | A `code-qa` card type with Code, Question, Answer and Complexity fields. | P1 |
| CODE-004 | **Checked answers:** a card may run a command the user configured (e.g. their own test runner). Opt-in, local cards only, never from imported or shared decks. Needs a D-013 amendment. | P2 |
| CODE-005 | Card types that need no execution: type the code and see a diff against the answer; predict the output; spot the bug (from a diff). | P1 |

### Problem practice (PROB)

| ID | Requirement | Pri |
|---|---|---|
| PROB-001 | **Problem cards** (`builtin:problem`): URL, title, difficulty, pattern tags, notes, and a link to the user's solution file (anchored through SRC). No copied problem statements. | P0 |
| PROB-002 | **Re-solve grading** (§8). Problem decks default to a lower desired retention (0.80). | P0 |
| PROB-003 | Stats by pattern tag (`keepjay stats --by tag --scope 'card_type:problem'`). | P0 |
| PROB-004 | Import problem lists from CSV or NDJSON files of URLs. | P0 |
| PROB-005 | Import a **solutions repository** in the layouts LeetHub-style sync extensions produce (one folder per problem) as problem cards with URL, title, difficulty and solution file (§8). | P0 |

### Editor and browser integrations (EDIT)

| ID | Requirement | Pri |
|---|---|---|
| EDIT-001 | **VS Code extension** (§9): due count in the status bar, review panel, card from selection, gutter marks on anchored and stale cards. Built on the bundled CLI's JSON and the shared React review UI. Published on the VS Code Marketplace and Open VSX. | P0 |
| EDIT-002 | **Neovim plugin:** due count, review in a floating window, jump to anchored cards. Thin Lua over the CLI's JSON; community-led. | P1 |
| EDIT-003 | **JetBrains plugin** with the features of EDIT-001. | P2 |
| EDIT-004 | **Browser extension:** after an accepted submission on a problem site, offer "schedule this problem" (URL, title, difficulty, and the user's code, saved to their solutions repo or locally); clip a docs passage into a card. Talks to the local CLI through native messaging, or to sync. Built only if PROB-005 isn't enough (D-034). | P2 |

## 3. `@card` comments (SRC-002)

```rust
// @card What does Core::open do before running migrations?
// Creates the data dir and loads or creates the device id.
pub fn open(paths: Paths) -> Result<Self> {
```

```python
# @card(q8w2m4za) Why does `asyncio.gather` not cancel siblings when one task fails?
# By default it propagates the first exception but lets the other tasks run;
# use a TaskGroup (3.11+) for structured cancellation.
# tags: lang::python async
```

Rules:

- A card starts with a line comment whose text begins with `@card`. The question follows on the same line.
- The answer is the following consecutive comment lines (same prefix, same indentation). It ends at a blank comment line, a non-comment line or another `@card`.
- A question containing `{{c1::…}}` makes a cloze card, which needs no answer lines.
- `@card(<id>)` carries the stable id written on first scan (SRC-005).
- A comment `@card:off` stops finding cards until `@card:on`, or to the end of the file (SRC-012), e.g. before a test module full of `@card` examples.
- The deck and tags come from the source's settings. A last answer line of the form `tags: a b` adds per-card tags.
- **The anchored code**, until CODE-002 lands, is the comment block plus the following lines, up to the first blank line (30 lines at most). From M2 it's the next syntax node (function, method, type or statement).

Comment prefixes (extendable in `cli.toml`):

| Prefix | Languages (by extension) |
|---|---|
| `//` | Rust, C, C++, C#, Go, Java, JavaScript, TypeScript, Kotlin, Swift, Scala, Dart, PHP |
| `#` | Python, Ruby, shell, YAML, TOML, R, Elixir, Perl, Nix |
| `--` | SQL, Haskell, Lua, Ada, Elm |
| `;` | Lisp, Clojure, Scheme, assembly |
| `%` | LaTeX, Erlang, MATLAB |

## 4. Markdown card files (SRC-003)

````markdown
---
deck: Rust::Async
tags: [lang::rust, async]
---

Q: What does `.await` do when the future isn't ready?
A: Yields to the executor; the task is woken when the future can make progress.
<!-- id: k22k1m8e -->

C: In Tokio, {{c1::spawn_blocking}} runs blocking code on a dedicated thread pool.

Q: What's wrong with this?
```rust
let guard = mutex.lock().unwrap();
some_async_call().await;
```
A: A `std` mutex guard is held across `.await`, which can deadlock the executor.
````

- `Q:` and `A:` make a question/answer card; `C:` makes a cloze card. A card ends at the next `Q:`, `C:`, heading or `---`.
- Fenced code blocks are allowed in any field and keep their language (CODE-001).
- The optional front matter sets `deck` and `tags` for the file.
- The id is written as an HTML comment after the card on first scan (SRC-005).

## 5. Anchors and change detection (SRC-004, SRC-006)

| Anchor field | Meaning |
|---|---|
| `repo` | Root commit hash of the Git repository (stable across clones and remotes), or the source's id for plain folders |
| `path` | Path relative to the source root |
| `line_start`, `line_end` | Lines of the anchored code |
| `symbol` | Symbol path from tree-sitter, e.g. `Core::open` (from M2) |
| `code_hash` | SHA-256 of the anchored code, with trailing whitespace and line endings normalised |
| `stale_since` | Set when `code_hash` changes; cleared by *keep* or *edit* |

What a re-scan does:

| Found | Action |
|---|---|
| Same id, same text, same code | Nothing |
| Same id, card text changed | Update the fields; keep history |
| Same id, anchored code changed | Mark stale (`stale_since`) and list it in `keepjay stale`; the hook `scan.stale_found` fires (CLI-016) |
| Same id, moved (other path or lines) with the same code | Update the anchor; not stale |
| Id no longer in the source | Retire (soft delete), keeping history; restore it if it reappears |
| New card without an id | Create it; write the id back unless the source opted out |
| No id, source opted out | Match by anchor and text similarity; ask when unsure (`--yes` creates new cards) |

In the stale queue the user can:

- **keep** the card (re-baseline the hash);
- **edit** it (opens the file at the line);
- **retire** it.

## 6. Storage (D-029)

- **Entities:** `source` and `anchor` ([09 §3](09-data-model.md#3-entities)). Facts from a source have `facts.source = src:<source id>`.
- **The written id** is the fact's short id (CLI-007). The core checks that it's unique within the workspace.
- **Facts from sources are edited only through their files.** `keepjay card edit`, the VS Code panel and the MCP server open the file or write to it, so a scan never fights an in-app edit. The mobile app only reads them.
- **History in Git:** `keepjay export log <file>` writes the review log as NDJSON.

## 7. Deck repositories (SRC-011)

A deck repository is a Git repository of Markdown card files with ids. Keepjay uses the system `git` binary, which respects the user's credentials and configuration.

| Command | Effect |
|---|---|
| `keepjay deck subscribe <url> [--name N] [--path P]` | Clone (default `<data_dir>/decks/<name>`) and register it as a `deck_repo` source; scan |
| `keepjay deck pull [NAME]` | `git pull --ff-only`, then scan. Changed cards update in place with history kept; new cards are added; removed cards retire. |
| `keepjay deck diff [NAME]` | Show local edits as a patch to send upstream |

Deck repositories are untrusted content: they're rendered through the sanitiser, and no card type can execute anything (D-013).

## 8. Problem practice (PROB)

`builtin:problem` fields: URL, Title, Difficulty, Pattern (tags such as `pattern::sliding-window`), Notes, and Solution (an anchored file).

| Outcome | Rating |
|---|---|
| Couldn't solve it | Again |
| Solved with hints, or over time | Hard |
| Solved | Good |
| Solved quickly and cleanly | Easy |

- **Default desired retention is 0.80** for problem decks, against 0.90 for flashcards, because a re-solve costs minutes, not seconds. FSRS optimisation (SRS-011) runs per problem deck once there's enough history.
- **Solutions repo import (PROB-005)** recognises the layouts of LeetHub, LeetHub 2.0, LeetHub-Neo and similar tools: one folder per problem, holding the solution file and a README with the problem's title and URL. It maps difficulty when present and never fetches problem statements.
- **Pattern stats (PROB-003)** show, per pattern tag: problems, lapses, average re-solve outcome and current retrievability.

## 9. VS Code extension (EDIT-001)

- **Status bar:** "N due", and the next target's countdown (SRS-016). Click to open the review panel.
- **Review panel:** a webview running the shared React review UI (`packages/ui`), rendering the core's sanitised HTML with a CSP that forbids scripts (D-013). Grades go through the CLI.
- **Commands:**
  - *Card from selection* (Q/A or tree-sitter cloze);
  - *Review*;
  - *Quick review*;
  - *Scan workspace*;
  - *Show stale cards*.
- **Editor decorations:** gutter marks on `@card` lines, a warning mark on stale cards, and a code lens "Review this card".
- **Data:** one-shot calls to the bundled CLI with `--json`, which stay under 100 ms (NFR-PERF-001). A long-lived mode is added only if profiling shows the need.
- **Packaging:** per-platform VSIX bundling the CLI, on the VS Code Marketplace and Open VSX. It shares the data dir with the terminal CLI (D-010).

## 10. MCP server (AI-010)

`keepjay mcp` (stdio) exposes:

| Tool | Access |
|---|---|
| `search_cards` | Read |
| `due_summary` | Read |
| `stale_cards` | Read |
| `stats` | Read |
| `add_cards` | Write, approved in the client (AI-005, AI-010) |

`add_cards` writes cards with anchors to a Markdown card file (default `.keepjay/cards/agent.cards.md`) or as `@card` comments. The cards carry provenance (`generated_from` → `agent:<client name>`).

## 11. Git hooks

`keepjay source hooks install` adds `post-commit`, `post-merge` and `post-checkout` hooks that run `keepjay scan --quiet` in the background. They chain any existing hooks, and `keepjay source hooks uninstall` removes them. Nothing is installed without that command.
