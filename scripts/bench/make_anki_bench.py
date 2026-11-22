#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Generate a large, realistic Anki package to time the importer (NFR-PERF-008).

Uses the official `anki` Python package, like scripts/fixtures/make_anki_fixtures.py.

    python3 -m venv .venv && .venv/bin/pip install anki
    .venv/bin/python scripts/bench/make_anki_bench.py bench.apkg --cards 50000 --media 2000
    cargo build --release -p scrubjay-cli
    SCRUBJAY_DATA_DIR=$(mktemp -d) target/release/scrubjay init
    time SCRUBJAY_DATA_DIR=... target/release/scrubjay import anki bench.apkg

Content: Basic notes in 20 nested decks, 1–3 hierarchical tags each from a pool of 300,
about 60% of cards reviewed with ~8 answers each (~240k revlog rows at 50k cards), and
`--media` distinct ~30 KB images, each referenced by several notes. Deterministic (seeded).
"""

import argparse
import random
import tempfile
import time
from pathlib import Path

from anki.collection import AddNoteRequest, Collection
from anki.import_export_pb2 import ExportAnkiPackageOptions

DAY_MS = 86_400_000


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("out", type=Path)
    ap.add_argument("--cards", type=int, default=50_000)
    ap.add_argument("--media", type=int, default=0, help="number of distinct media files")
    ap.add_argument("--legacy", action="store_true", help="schema 11 instead of the latest")
    args = ap.parse_args()
    rng = random.Random(42)

    with tempfile.TemporaryDirectory() as tmp:
        col = Collection(str(Path(tmp) / "collection.anki2"))
        basic = col.models.by_name("Basic")
        decks = [
            col.decks.id(f"Subject {s}::Topic {t}") for s in range(5) for t in range(4)
        ]
        tags = [f"area{a}::topic{t}" for a in range(30) for t in range(10)]
        media = [
            col.media.write_data(f"figure-{i}.png", b"\x89PNG" + rng.randbytes(30_000))
            for i in range(args.media)
        ]

        requests = []
        for i in range(args.cards):
            note = col.new_note(basic)
            note.fields[0] = f"Question {i}: what is <b>term {i}</b>?"
            back = f"Answer {i}"
            if media:
                back += f'<br><img src="{media[i % len(media)]}">'
            note.fields[1] = back
            note.tags = rng.sample(tags, rng.randint(1, 3))
            requests.append(AddNoteRequest(note=note, deck_id=rng.choice(decks)))
        col.add_notes(requests)

        now = int(time.time() * 1000)
        rows = []
        for cid in col.find_cards(""):
            if rng.random() > 0.6:
                continue
            t = now - rng.randint(100, 400) * DAY_MS
            for _ in range(rng.randint(4, 12)):
                t += rng.randint(1, 30) * DAY_MS
                if t >= now:
                    break
                ease = rng.choices([1, 2, 3, 4], weights=[10, 10, 70, 10])[0]
                rows.append((t + len(rows), cid, ease, rng.randint(2_000, 20_000)))
        col.db.executemany(
            "insert into revlog (id, cid, usn, ease, ivl, lastIvl, factor, time, type)"
            " values (?, ?, -1, ?, 1, 0, 2500, ?, 1)",
            rows,
        )
        col.db.execute("update cards set type = 2, queue = 2, ivl = 10, due = 0"
                       " where id in (select distinct cid from revlog)")
        col.export_anki_package(
            out_path=str(args.out),
            options=ExportAnkiPackageOptions(
                with_scheduling=True, with_deck_configs=True, with_media=True,
                legacy=args.legacy,
            ),
            limit=None,
        )
        col.close()
    print(f"wrote {args.out}: {args.cards} cards, {len(rows)} reviews, {len(media)} media files")


if __name__ == "__main__":
    main()
