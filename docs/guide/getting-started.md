# Getting started

Keepjay turns notes in your code into flashcards and schedules them so you remember them.
This page takes about five minutes.

## Install

1. Download the archive for your platform from the
   [releases page](https://github.com/keepjay-dev/keepjay/releases): macOS (Apple silicon or
   Intel), Linux (x86-64) or Windows (x86-64).
2. Extract it and put `keepjay` (`keepjay.exe` on Windows) in a folder on your `PATH`.
3. Alpha builds aren't code-signed yet, so the first run needs one extra step:
   - **macOS:** run `xattr -d com.apple.quarantine keepjay` once, or allow it in
     System Settings → Privacy & Security.
   - **Windows:** in the SmartScreen prompt, choose **More info → Run anyway**.
4. Check it works: `keepjay --version`.

Optional: `keepjay alias install` adds the short command `kj`, unless another `kj` is already
on your `PATH`.

## Your first cards

In any repository, add a comment above some code you want to remember:

```rust
// @card What does `Arc::clone` actually copy?
// Only the pointer: it increments the reference count; the data isn't cloned.
let shared = Arc::clone(&config);
```

Then register the repository and scan it:

```sh
keepjay source add .      # once per repository
keepjay scan              # finds @card comments and card files
```

The scan writes a short id into each card's comment (`@card(k22k1m8e)`), so the card keeps
its history if you move or edit it. Commit those ids with your code.

## Review

```sh
keepjay review            # full-screen review: space shows the answer, 1–4 grades it
keepjay review --quick    # just a few cards, then back to work
keepjay while -- cargo test   # review while your tests run
```

Run `keepjay` on its own to see what's due today.

## Next

- [Writing cards](writing-cards.md): comments in every language, Markdown card files, cloze
  cards and tags.
- [Reviewing](reviewing.md): quick reviews, reviewing during builds, your shell prompt, and
  stale cards.
- [Commands](commands.md): every command on one page.
- [FAQ](faq.md).
