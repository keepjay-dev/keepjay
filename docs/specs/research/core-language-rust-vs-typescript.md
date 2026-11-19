# Research — Core Language: Rust vs TypeScript

> **Status:** Decided: Rust core + TypeScript UI accepted (D-002); M0.4 spike validates velocity · **Last updated:** 2026-11-19
> **Related:** [08-architecture](../08-architecture.md) · [decisions D-002, D-003, D-012](../decisions.md) · OQ-003

## Question

The team knows TypeScript better but is drawn to Rust for long-term performance. Which language should the **shared core** (domain, storage, scheduler, workflow engine, queries, metrics, import/export) be written in?

Note that the **UI is TypeScript/React either way** (desktop now, web later). The choice is only about the core and the CLI, roughly 40–50% of the code.

## The two realistic stacks

| | **A. Rust core** (current proposal) | **B. TypeScript everywhere** |
|---|---|---|
| Core | Rust library (`study-core`) | TS package (`@app/core`) |
| Desktop | Tauri 2: Rust backend links the core; React UI | Electron: core in Node main process (or Tauri with the core in the webview + SQLite via plugin) |
| CLI | Native Rust binary (clap + ratatui) | Node/Bun binary (commander + Ink) |
| Web | Core → WASM in a worker + SQLite-WASM | Core runs natively in the browser + SQLite-WASM |
| Mobile | SwiftUI / Compose + core via UniFFI (truly native UI) | React Native + core in Hermes (native views, not SwiftUI/Compose) |
| SQLite | `rusqlite` (bundled) | `better-sqlite3` / `node:sqlite` / `bun:sqlite`; `op-sqlite` on RN |
| FSRS | `fsrs` crate (reference implementation, includes the optimiser) | `ts-fsrs` (scheduler); optimiser via WASM build of `fsrs` |

## Measured: CLI startup (this container, 2026-11-19)

Simulated `addisco status`: open a 100k-card SQLite DB, then run one count and one grouped query. 30 runs each.

| Runtime | Median | p95 | Distributable size |
|---|---|---|---|
| Process spawn floor (`/bin/true`) | 3 ms | 5 ms | — |
| **Rust** release binary (`rusqlite`) | **38 ms** | 50 ms | **2.3 MB** |
| Bun (`bun:sqlite`), compiled single binary | 47 ms | 62 ms | 104 MB |
| Node 22 (`node:sqlite`) | 70 ms | 84 ms | needs Node installed (or ~90 MB+ bundled) |
| Node 22, empty script | 32 ms | 47 ms | — |

**What this tells us:**
- Most of the time is **SQLite doing the query**, which is native C in every option. Runtime overhead is about 0 ms for Rust, ~10 ms for Bun and ~35 ms for Node.
- All three fit the < 100 ms budget (NFR-PERF-001) for this case. Raw speed **isn't the decisive factor** for the CLI. **Binary size** (2 MB vs ~100 MB) and **no runtime dependency** matter more for a CLI people install through Homebrew/winget.
- "Optimise later" in TypeScript is possible for specific hot paths (FSRS optimiser, big Anki imports, rollups) by calling Rust via WASM or napi. So performance alone wouldn't force Rust.

## Comparison for this project

Weights reflect this project's stated goals (native mobile, first-class CLI, local-first, long life).

| Criterion | Weight | Rust core | TS everywhere | Notes |
|---|---|---|---|---|
| **One core on every platform incl. *native* mobile** | 5 | ✅✅ | ⚠️ | Rust → UniFFI gives Swift/Kotlin a native library. A TS core on native SwiftUI/Compose needs an embedded JS engine and bridge, which is awkward and slow to start. TS realistically means React Native instead. |
| CLI quality (single small binary, no runtime) | 4 | ✅✅ | ⚠️ | Measured above: 2.3 MB vs ~104 MB. Startup is fine in both. |
| Desktop footprint | 3 | ✅✅ | ⚠️ | Tauri + Rust core: ~10–20 MB installer. Electron: ~80–150 MB. Tauri with a TS core is possible but splits the core across webview and plugins. |
| Correctness of tricky logic (workflow state machine, scheduler replay, sync merge, migrations) | 4 | ✅✅ | ✅ | Both have tagged unions. Rust adds exhaustiveness everywhere, no `any`/`undefined` holes, ownership, and fearless threading for imports and rollups. |
| Long-term performance headroom (500k cards, 5M logs, optimiser, imports) | 3 | ✅✅ | ✅ | TS can offload hot paths to WASM, but then you maintain two languages anyway. |
| **Speed to MVP for this team** | 5 | ⚠️ | ✅✅ | The main argument for TS. Expect the Rust core to take noticeably longer in the first months while the team learns ownership/borrowing; this narrows after that. |
| Hiring / contributors (open-source community) | 2 | ✅ | ✅✅ | TS has the larger pool. Rust has a strong enthusiast community for tools like this. |
| Web app | 2 | ✅ | ✅✅ | TS runs natively. Rust needs WASM (adds ~1–3 MB to load; fine for an installable PWA). |
| Ecosystem fit (SQLite, FSRS, TUI, CLI parsing) | 2 | ✅✅ | ✅ | `rusqlite`, `fsrs` (reference), `clap` and `ratatui` are best-in-class. TS has `ts-fsrs` and Ink (React-style TUI, familiar). |
| Cost of changing our minds later | 4 | ✅ | ⚠️ | Moving a TS core to Rust later is a full rewrite of the hardest code, with parity risk. Going Rust now avoids that. The language-neutral document formats (JSON Schemas) survive either way. |

✅✅ strong · ✅ good · ⚠️ workable with notable cost

**Weighted result:** Rust core wins clearly on the platform-reach, CLI, footprint and change-cost criteria. TypeScript wins on speed to MVP and familiarity. Because the brief commits to **native mobile apps** and a **first-class CLI**, and the UI stays in TypeScript anyway, the Rust core is the better long-term fit.

## Recommendation

**Keep D-002: a Rust core with a TypeScript/React UI (hybrid).** You keep using your strongest language for the UI (about half the code) and put Rust only where it pays off.

Mitigations for the Rust learning curve:

1. **Keep the core synchronous.** Use `rusqlite` with no async runtime in the core, and threads for background work. This removes the hardest part of Rust for newcomers and keeps UniFFI/WASM bindings simple.
2. **Simple ownership style:** owned data in view models, `Arc` where sharing is needed, no lifetime-heavy APIs across the facade.
3. **Generated TS types** (`specta`/`ts-rs`) so the UI side stays in familiar territory and the boundary is type-checked.
4. **Grow the core through the CLI first** (D-011): small, testable increments with fast feedback.
5. **Spike first (M0.4):** build one vertical slice (add card → review with FSRS → `addisco status`) in Rust before committing to the M1 plan, and record the actual velocity.

**Fallback:** if the spike shows Rust velocity is unacceptable, switch to stack B *before* M1. The data model, document formats and specs are language-neutral, so nothing written so far is lost.
