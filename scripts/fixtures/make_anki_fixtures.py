#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Generate real Anki packages for the importer tests (IO-001).

Uses the official `anki` Python package (Anki's own Rust engine), so the fixtures are
genuine exports, not hand-built imitations.

    python3 -m venv .venv && .venv/bin/pip install anki
    .venv/bin/python scripts/fixtures/make_anki_fixtures.py crates/study-core/tests/fixtures/anki

Writes `sample-legacy.apkg` (collection.anki21, schema 11) and `sample-latest.apkg`
(collection.anki21b, zstd + schema 18), plus `expected.json` describing the content.
"""

import json
import sys
import tempfile
import time
from pathlib import Path

from anki.collection import Collection
from anki.import_export_pb2 import ExportAnkiPackageOptions
from anki.scheduler_pb2 import CardAnswer

DAY_MS = 86_400_000


def main(out_dir: Path) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory() as tmp:
        col = Collection(str(Path(tmp) / "collection.anki2"))
        models = col.models

        def deck(name: str) -> int:
            return col.decks.id(name)

        def add(model_name: str, fields: list[str], deck_name: str, tags: str = "") -> int:
            note = col.new_note(models.by_name(model_name))
            for i, value in enumerate(fields):
                note.fields[i] = value
            if tags:
                note.tags = tags.split()
            col.add_note(note, deck(deck_name))
            return note.id

        # Custom note type: 3 fields, 1 template with a conditional section.
        vocab = models.new("Vocab (custom)")
        for name in ["Word", "Meaning", "Example"]:
            models.add_field(vocab, models.new_field(name))
        tmpl = models.new_template("Card 1")
        tmpl["qfmt"] = "<div class=word>{{Word}}</div>"
        tmpl["afmt"] = (
            "{{FrontSide}}<hr id=answer>{{Meaning}}"
            "{{#Example}}<br><i>{{Example}}</i>{{/Example}}"
        )
        models.add_template(vocab, tmpl)
        models.add(vocab)

        media_name = col.media.write_data("cell.png", b"\x89PNG fixture")

        basic = add(
            "Basic",
            ["What organelle produces most of the cell&#x27;s <b>ATP</b>?",
             f"Mitochondria<br><img src=\"{media_name}\">"],
            "Biology::Cells",
            "bio cells",
        )
        add("Basic (and reversed card)", ["Cell", "Zelle"], "Languages::German", "de")
        add("Basic (optional reversed card)", ["Nucleus", "Zellkern", "y"], "Languages::German")
        add("Basic (optional reversed card)", ["Ribosome", "Ribosom", ""], "Languages::German")
        add("Basic (type in the answer)", ["Capital of France?", "Paris"], "Geography")
        cloze = add(
            "Cloze",
            ["{{c1::Ribosomes}} build {{c2::proteins::macromolecules}}", "Translation&nbsp;step"],
            "Biology::Cells",
        )
        add("Vocab (custom)", ["Hund", "dog", "Der Hund bellt."], "Languages::German")
        add("Vocab (custom)", ["Katze", "cat", ""], "Languages::German")

        # A real scheduler answer (revlog written by Anki itself).
        card = col.get_note(basic).cards()[0]
        card.start_timer()
        states = col._backend.get_scheduling_states(card.id)
        answer = col.sched.build_answer(card=card, states=states, rating=CardAnswer.GOOD)
        col.sched.answer_card(answer)

        # Historical reviews for the first cloze card: injected the way Anki stores them.
        cloze_card = col.get_note(cloze).cards()[0]
        now = int(time.time() * 1000)
        history = [  # (days ago, ease, answer time ms)
            (20, 3, 6_000),
            (16, 3, 4_000),
            (9, 1, 9_000),
            (8, 3, 5_000),
        ]
        for days_ago, ease, ms in history:
            col.db.execute(
                "insert into revlog (id, cid, usn, ease, ivl, lastIvl, factor, time, type)"
                " values (?, ?, -1, ?, 1, 0, 2500, ?, 1)",
                now - days_ago * DAY_MS, cloze_card.id, ease, ms,
            )
        # A manual reschedule entry (ease 0) that the importer must ignore.
        col.db.execute(
            "insert into revlog (id, cid, usn, ease, ivl, lastIvl, factor, time, type)"
            " values (?, ?, -1, 0, 5, 0, 2500, 0, 4)",
            now - 1 * DAY_MS, cloze_card.id,
        )
        col.db.execute(
            "update cards set type = 2, queue = 2, ivl = 7, due = ?, reps = 4, lapses = 1"
            " where id = ?",
            col.sched.today + 3, cloze_card.id,
        )

        # Suspend the second cloze card; put the reversed German card in a filtered deck.
        col.sched.suspend_cards([col.get_note(cloze).cards()[1].id])
        fid = col.decks.new_filtered("Cram")
        filtered = col.decks.get(fid)
        filtered["terms"][0][0] = '"deck:Languages::German" card:2'  # [search, limit, order]
        col.decks.save(filtered)
        col.sched.rebuild_filtered_deck(fid)

        expected = {
            "notes": 8,
            "cards": 11,
            "decks": ["Biology::Cells", "Geography", "Languages::German"],
            "media": [media_name],
            "reviewed_cloze_answers": [3, 3, 1, 3],
        }
        for legacy, name in [(True, "sample-legacy.apkg"), (False, "sample-latest.apkg")]:
            col.export_anki_package(
                out_path=str(out_dir / name),
                options=ExportAnkiPackageOptions(
                    with_scheduling=True, with_deck_configs=True, with_media=True, legacy=legacy
                ),
                limit=None,
            )
        (out_dir / "expected.json").write_text(json.dumps(expected, indent=2) + "\n")
        col.close()
    print(f"wrote fixtures to {out_dir}")


if __name__ == "__main__":
    main(Path(sys.argv[1] if len(sys.argv) > 1 else "."))
