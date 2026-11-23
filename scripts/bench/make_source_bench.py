#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Generate a large source tree to time `keepjay scan` (NFR-PERF-009, SRC-009).

    python3 scripts/bench/make_source_bench.py /tmp/bench-src --files 10000
    cargo build --release -p keepjay-cli
    export KEEPJAY_DATA_DIR=$(mktemp -d)
    target/release/keepjay init && target/release/keepjay source add /tmp/bench-src
    time target/release/keepjay scan     # first scan: budget 5 s
    time target/release/keepjay scan     # re-scan, warm cache: budget 1 s

Content: source files in nested packages (Rust, Python, TypeScript, Go, SQL, shell) of
40–400 lines each, about 8% of them with 1–3 `@card` comments (some cloze), plus
`*.cards.md` files with 5–20 cards each. It also writes an ignored `target/` folder and a
few binary files, to check that the walker skips them cheaply. Deterministic (seeded).
"""

import argparse
import random
from pathlib import Path

LANGS = [
    # extension, comment prefix, line of code
    (".rs", "//", "    let v{i} = compute({i}).unwrap_or_default();"),
    (".py", "#", "    v{i} = compute({i}) or None"),
    (".ts", "//", "  const v{i} = compute({i}) ?? null;"),
    (".go", "//", "\tv{i} := compute({i})"),
    (".sql", "--", "SELECT id, name FROM t{i} WHERE deleted_at IS NULL;"),
    (".sh", "#", 'echo "step {i}"'),
]


def card(rng: random.Random, prefix: str, n: int) -> list[str]:
    if rng.random() < 0.25:
        return [f"{prefix} @card Step {n} uses {{{{c1::compute}}}} before {{{{c2::commit}}}}."]
    return [
        f"{prefix} @card Why does step {n} call compute() first?",
        f"{prefix} It loads the cached value, so the commit sees a consistent state.",
    ]


def source_file(rng: random.Random, ext: str, prefix: str, code: str, n: int) -> str:
    lines = [f"{prefix} generated benchmark file {n}"]
    cards = rng.randint(1, 3) if rng.random() < 0.08 else 0
    total = rng.randint(40, 400)
    card_at = sorted(rng.sample(range(5, total), cards)) if cards else []
    for i in range(total):
        if card_at and i == card_at[0]:
            card_at.pop(0)
            lines.append("")
            lines.extend(card(rng, prefix, n * 1000 + i))
        lines.append(code.format(i=i))
    return "\n".join(lines) + "\n"


def cards_md(rng: random.Random, n: int) -> str:
    out = [f"# Bench notes {n}", ""]
    for i in range(rng.randint(5, 20)):
        if rng.random() < 0.3:
            out += [f"C: Topic {n}.{i}: {{{{c1::answer {i}}}}} is the default.", ""]
        else:
            out += [f"Q: What does topic {n}.{i} configure?", f"A: Setting number {i}.", ""]
    return "\n".join(out)


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("out", type=Path)
    ap.add_argument("--files", type=int, default=10_000, help="source files (default 10000)")
    ap.add_argument("--md", type=int, default=200, help="*.cards.md files (default 200)")
    ap.add_argument("--seed", type=int, default=7)
    args = ap.parse_args()
    rng = random.Random(args.seed)
    root: Path = args.out
    if root.exists() and any(root.iterdir()):
        raise SystemExit(f"{root} exists and isn't empty")
    root.mkdir(parents=True, exist_ok=True)

    for n in range(args.files):
        ext, prefix, code = LANGS[n % len(LANGS)]
        d = root / f"pkg{n % 50:02d}" / f"mod{n % 400 // 50}"
        d.mkdir(parents=True, exist_ok=True)
        (d / f"file{n:05d}{ext}").write_text(source_file(rng, ext, prefix, code, n))
    notes = root / "notes"
    notes.mkdir(exist_ok=True)
    for n in range(args.md):
        (notes / f"topic{n:03d}.cards.md").write_text(cards_md(rng, n))

    # Things the walker must skip: an ignored build folder and binary files.
    (root / ".gitignore").write_text("target/\n")
    target = root / "target" / "debug"
    target.mkdir(parents=True)
    for n in range(500):
        (target / f"junk{n}.rs").write_text("// @card should never be found\n// no\n")
    for n in range(20):
        (root / f"blob{n}.bin").write_bytes(bytes(rng.randrange(256) for _ in range(4096)))
    print(f"wrote {args.files} source files and {args.md} card files under {root}")


if __name__ == "__main__":
    main()
