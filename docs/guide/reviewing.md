# Reviewing

Keepjay schedules each card with FSRS, the spaced-repetition algorithm Anki also uses: a card
comes back just before you'd forget it, so a few minutes a day is enough.

## A full review

```sh
keepjay review
```

Read the question and recall the answer, then press **space** (or Enter) to see it. Grade
yourself:

| Key | Grade | Meaning |
|---|---|---|
| `1` | Again | You didn't remember it |
| `2` | Hard | You remembered, with effort |
| `3` | Good | You remembered |
| `4` | Easy | You knew it instantly |

`q` or Esc stops; your progress is saved after every card. `--deck NAME` reviews one deck,
and `--limit N` stops after N cards.

For screen readers or scripts, `keepjay review --plain` asks line by line and reads grades
from standard input.

## Quick reviews

```sh
keepjay review --quick            # 5 due cards, then exit
keepjay review --quick -n 3       # 3 cards
keepjay review --quick --time 2m  # at most two minutes
```

When nothing is due, `--quick` prints nothing and exits straight away, so it's safe to put in
a script or an alias.

## Review while something runs

```sh
keepjay while -- cargo test
keepjay while -- npm run build
```

Keepjay runs the command and opens a review. When the command finishes, the review closes
and you see the end of its output. `keepjay while` exits with the command's own exit code,
so `keepjay while -- make && ./deploy` still stops on a failed build. Press `q` to stop
reviewing early; the command keeps running.

## In your shell prompt

`keepjay status --format` prints one line from a template:

```sh
keepjay status --format '{due} due, {stale} stale'
```

The fields are `{due}`, `{review}`, `{learning}`, `{new}`, `{stale}` and `{reviewed}` (cards
reviewed today). It's fast enough to run on every prompt.

## Stale cards

A card is **stale** when the code under it has changed since you wrote it, so its answer may
be out of date. `keepjay scan` reports new stale cards, and `keepjay stale` lists them:

```sh
keepjay stale                     # list stale cards
keepjay stale keep k22k1m8e       # still right: clear the flag
keepjay stale retire k22k1m8e     # no longer worth keeping
```

- **Still right?** `stale keep`.
- **Out of date?** Edit the card in its file and run `keepjay scan`. Its review history is
  kept.
- **Not needed any more?** `stale retire` removes the card's own lines from the file
  (never the code under it) and retires the card, keeping its history. It shows the lines
  and asks first; use `--dry-run` to only look, or `--yes` in scripts.

## Coming from Anki

```sh
keepjay import anki ~/Downloads/deck.apkg
```

Imports notes, cards, tags, media and your full review history, so scheduling carries on
where Anki left off. `--into "From Anki"` puts the imported decks under one parent deck.
