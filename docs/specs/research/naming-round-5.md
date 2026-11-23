# Research — Product Name, Round 5 (After the Scrubjay Conflicts)

> **Status:** Decided: the product owner chose **Keepjay** on 2026-11-23 (D-037, superseding D-035) · **Last updated:** 2026-11-23
> **Related:** [naming-round-4.md](naming-round-4.md) (rubric, earlier angles) · [naming.md](naming.md) (rounds 1–3, CLI conventions §6, pre-release checklist §7) · [decisions D-035](../decisions.md)

## 1. Question

When reserving names for the M0.6 landing page, the product owner found **`scrubjay.com` taken**. Looking further turned up worse:

| Where | What it is | Risk |
|---|---|---|
| `scrubjay.app` | **Scrubjay**, a paid, "privacy-first" desktop app for finding clips in video. It runs locally on Mac, Windows and Linux, with a free trial. | **High:** live, paid software in trademark class 9, sold on local-first privacy like ours |
| `github.com/henba1/scrubjay` (18★) | Memory and sync for AI coding agents, tagline **"Recall what · where · when"** | **High for confusion:** a developer tool telling the same scrub-jay story as D-035 |
| `scrubjay.dev` | **ScrubJay**, commercial face and number-plate search across surveillance video | Medium: another market, an unwelcome association |
| `scrubjay.com` | Registered since 1997; no site | Low as a product; the domain is unavailable |

D-035 chose Scrubjay over Muista partly because its `.com` "looked free". Round 5 finds a name that passes the checks round 4 missed.

## 2. Short answer

| # | Name | Alias | Score | Domains free | Why | Main risk |
|---|---|---|---|---|---|---|
| 1 | **Keepjay** | `kj` | **4.60** | **`.com`** `.dev` `.app` `.io` `.sh` | Says the tagline ("**Keep** what you learn while you code") and keeps the jay: the mascot, the logo and the what/where/when story. Two everyday English words. Every check is clean. | The GitHub handle `KeepJay` is a dormant, empty account (use `keepjay-dev`). "Keep" brings Google Keep to mind, mildly. |
| 2 | Recalljay | `rj` | 4.25 | `.com` `.dev` `.app` `.io` | Says what it does, keeps the jay | 9 letters; "recall" is a crowded word in memory apps (Recall, Recallify) |
| 3 | Muista | `mui` | 4.20 | `.dev` `.app` `.io` | Round 4's top scorer: Finnish for "remember!"; still no products | A foreign word, the complaint about Addisco; `.com` parked since 2013 |
| 4 | Mindjay | `mj` | 4.18 | `.com` `.dev` `.app` `.io` | Clean checks | Vaguer meaning; near *Medjay* (Assassin's Creed) |
| – | Scrubjay | `sj` | 3.43 | `.io` only | The current name | The conflicts in §1 |

**Recommendation: Keepjay.**
- It's the only finalist that's clean on every check *and* says the product's promise in its own name.
- It also keeps nearly everything the owner liked about Scrubjay: an English name, the bird mascot and the story. Food-caching jays remember what they hid, where and when (Clayton & Dickinson, *Nature* 1998); Keepjay keeps what you learned, anchored where it lives, and notices when it changes.
- Renaming is cheap now, before anything public. The Addisco-to-Scrubjay migration code can be reused for anyone who already has a `scrubjay` data folder.

## 3. Method

### 3.1 What changed from round 4

Round 4 had two gaps, and both mattered:

1. **Domains were checked with DNS.** A domain with no address record looks free even when it's registered (`scrubjay.com` has been registered since 1997). Round 5 asks each registry directly over **RDAP**: Verisign for `.com`, Google Registry for `.dev` and `.app`, and Identity Digital for `.io` and `.sh`. It also checks the method on names with known answers first.
2. **Web checks looked for study apps, not same-name products.** Round 5 searches the bare name for **any live software product**, opens the site behind every registered `.com` or `.app`, and reads the top GitHub repositories with the name.

The other checks are unchanged: crates.io, npm, PyPI, Homebrew formula and cask, GitHub repository count and handle, and local command and alias collisions.

### 3.2 Candidates (145 names in three batches)

| Angle | Examples |
|---|---|
| Animals that cache food | Nuthatch, Chickadee, Nutcracker, Pinyon jay, Waxwing, Kea |
| Scatter-hoarding and caching (a developer pun) | Nutcache, Seedcache, Scatterhoard, Stashjay, Cachejay, Cachebird, Hoardbird |
| Marks you leave to remember | Dogear, Earmark, Cairn, Waymark, Marginalia, Breadcrumb, Fingerstring, Tally |
| Forgetting-curve and spacing science | Ebbinghaus, Leitner, Rerun, Reprise, Encore, Againly |
| Memory traces | Engram, Imprint, Etch, Ingrain, Myelin, Hippocamp, Seahorse |
| Short English words | Ken, Wit, Rote, Kept, Brushup, Unforget, Keepalive, Callback, Revisit |
| Coined compounds | Cardloom, Remembit, Knowcache, Kenmark, Gutterdeck, Knowstill, Memojay |
| The jay family (keeps the mascot) | Keepjay, Recalljay, Mindjay, Learnjay, Notejay, Jotjay, Kenjay, Jaywise |
| Earlier finalists, rechecked | Muista, Cofio, Knowkeep, Kumbuka, Edisco, Addisco, Memheap |

### 3.3 Scoring

The rubric is round 4's, so scores are comparable: uniqueness 25%, CLI fit 20%, meaning and story 15%, say and spell 15%, brand feel 15%, breadth 10%.

| Name | Uniq | CLI | Meaning | Say | Brand | Breadth | **Score** |
|---|---|---|---|---|---|---|---|
| **Keepjay** | 5.0 | 4.0 | 4.5 | 5.0 | 4.5 | 4.5 | **4.60** |
| Recalljay | 5.0 | 3.0 | 4.5 | 4.5 | 4.0 | 4.5 | 4.25 |
| Muista | 4.5 | 4.0 | 4.5 | 4.0 | 4.0 | 4.0 | 4.20 |
| Mindjay | 4.5 | 4.0 | 3.5 | 4.5 | 4.0 | 4.5 | 4.18 |
| Scrubjay (rescored) | 1.5 | 3.5 | 4.0 | 4.5 | 4.5 | 4.0 | 3.43 |

## 4. Results

### 4.1 Which domains are free

Almost every pronounceable name's `.com` is registered, usually parked. Of 145 names, only 13 had a free `.com`, and most of those are compounds: Keepjay, Recalljay, Mindjay, Learnjay, Jaymem, Knowstill, Memojay, Stashjay, Recallbird, Recallwren, Inlinecards, Memheap and Marshtit. Requiring a free `.com` therefore pushes towards two-word compounds. A free `.dev` (with `get<name>.com` as a fallback) is the realistic bar for most developer tools, but Keepjay meets both.

### 4.2 Ruled out by live products

These passed the registry checks but failed the product search, which is exactly the check round 4 lacked:

| Name | Conflict |
|---|---|
| **Cachejay** | `cachejay.com`, registered October 2026: "the human cache for ChatGPT, Claude and AI agents", with a Claude Code plugin |
| **Memojay** | An actively released desktop memo app (werty0411/memojay, v2.8.0 in November 2026) |
| Jaycache | An old Java geocaching tool on SourceForge |
| Cofio | Cofio Software (since 2006), as in round 4 |

### 4.3 Ruled out by registries or noise

Nearly every single English word (Cairn, Tally, Engram, Rerun, Keepalive, Recall, Kept, Ken) is taken on crates.io, npm and PyPI, with hundreds to tens of thousands of GitHub repositories. The most attractive bird names (Nuthatch, Chickadee, Nutcracker, Waxwing) are taken everywhere.

### 4.4 Finalist checks (2026-11-23)

| Check | Keepjay | Recalljay | Muista | Mindjay |
|---|---|---|---|---|
| `.com` · `.dev` · `.app` · `.io` (RDAP) | free · free · free · free | free · free · free · free | parked · free · free · free | free · free · free · free |
| crates · npm · PyPI · Homebrew | free | free | free | free |
| GitHub repositories with the name | 0 | 0 | 21 (Finnish hobby repos) | 1 |
| GitHub handle | taken (dormant, 0 repos, 2018) | free | taken | — |
| Products found on the web | none | none | none | none (*Medjay* is a near name) |
| Alias on `PATH` / Homebrew | `kj` free | `rj` free | `mui` free | `mj` free |

## 5. If Keepjay is chosen

1. **On the day:**
   - register `keepjay.com` and `keepjay.dev`;
   - reserve the crate and the npm name;
   - create the `keepjay-dev` GitHub org;
   - run the trademark search in classes 9, 41 and 42.
2. **Add a new decision** that supersedes D-035, carrying over its story and identifier table with the new values: `keepjay` / `kj`, `KEEPJAY_*`, `keepjay.cli/1`, and `.keepjayignore`.
3. **Rename** in one PR, as the Addisco rename was done. On first run, an existing Scrubjay (and Addisco) data or config folder is renamed to the new name, and `SCRUBJAY_*` variables are still read. The CLI schema id changes from `scrubjay.cli/1` to `keepjay.cli/1`, noted in the changelog.
4. **Hold the landing page (#30) and the survey** until the rename lands, since both use the name throughout.
5. **Rename the GitHub repository** (`mo-sajaad/scrubjay` → `mo-sajaad/keepjay`); GitHub redirects the old URLs.

## 6. Sources

- Scrubjay conflicts: [scrubjay.app](https://scrubjay.app) · [scrubjay.dev](https://scrubjay.dev) · [henba1/scrubjay](https://github.com/henba1/scrubjay)
- Ruled out: [cachejay.com](https://cachejay.com) · [werty0411/memojay releases](https://github.com/werty0411/memojay/releases/tag/v2.8.0) · [JayCache (SourceForge)](https://sourceforge.net/projects/jaycache/)
- Product searches for Keepjay, Recalljay, Mindjay, Muista, Knowstill, Hoardbird and Cachebird found no products. The nearest names were [Recallify](https://www.producthunt.com/posts/recallify), [MindJack](https://en.wikipedia.org/wiki/MindJack) and [Medjay](https://en.wikipedia.org/wiki/Medjay).
- Jay memory: [Clayton & Dickinson, *Nature* 1998](https://www.doi.org/10.1038/26216)
