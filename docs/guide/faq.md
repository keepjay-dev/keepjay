# FAQ

**Is it free?**
Yes. Everything that runs on your own machines is free, forever (D-017). A paid plan later
will cover only things hosted for you, such as syncing to a phone.

**How is it different from Anki?**
Keepjay uses the same scheduling algorithm (FSRS), and imports Anki decks with their review
history. The difference is where cards live: in your code and your Git history, next to what
they explain, and Keepjay notices when that code changes.

**Does it change my files?**
Only to write a short id into each card on the first scan, so the card keeps its history
when you move it (turn that off per source with `--no-write-ids`), and when you run
`keepjay stale retire`, which asks first. See [PRIVACY.md](../../PRIVACY.md).

**Does it send anything over the network?**
No. See [PRIVACY.md](../../PRIVACY.md).

**Should I commit the ids it writes?**
Yes. They're how a card keeps its review history across edits, moves and clones.

**What happens if I delete a card from my file?**
The next scan retires it. Its review history is kept, and if you put the card back (with its
id), it's restored.

**Can teammates share cards?**
Cards in a repository travel with it: a teammate who registers the same repository gets the
same cards, with their own review history. Shared deck repositories are on the
[roadmap](../../ROADMAP.md).

**Which editors are supported?**
Any: Keepjay is a command-line tool and reads plain files. A VS Code extension is planned.

**Is it stable?**
It's an alpha. Your data is kept in SQLite with backups before every database upgrade, but
expect rough edges, and report them as
[issues](https://github.com/keepjay-dev/keepjay/issues).

**Why "Keepjay"?**
Food-caching jays remember what they hid, where and when. Keepjay keeps what you learned,
where it lives in your code, and notices when it changes.
