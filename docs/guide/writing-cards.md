# Writing cards

Cards live in your files, next to what they're about. Keepjay reads them when you run
`keepjay scan`; edit the file to edit the card.

## Cards in code comments

Start a line comment with `@card`, put the question on the same line, and the answer on the
comment lines below it:

```python
# @card Why doesn't `asyncio.gather` cancel the other tasks when one fails?
# It raises the first exception but lets the others keep running;
# use a TaskGroup (3.11+) for structured cancellation.
# tags: lang::python async
async def fetch_all(urls):
```

- The answer is the comment lines directly below the question, with the same prefix and
  indentation. It ends at a blank comment line, a line of code, or the next `@card`.
- A last answer line `tags: a b` adds tags to that card.
- The card is **anchored** to the code just below it, up to the first blank line. When that
  code changes, the card is flagged as stale (see [Reviewing](reviewing.md#stale-cards)).
- After the first scan, the first line reads `@card(<id>)`. Keep the id: it's how the card
  keeps its review history when you edit or move it.

### Languages

| Comment | Languages |
|---|---|
| `//` | Rust, C, C++, C#, Go, Java, JavaScript, TypeScript, Kotlin, Swift, Scala, Dart, PHP, Zig |
| `#` | Python, Ruby, shell, YAML, TOML, R, Elixir, Perl, Nix, Julia |
| `--` | SQL, Haskell, Lua, Ada, Elm |
| `;` | Lisp, Emacs Lisp, Clojure, Scheme, assembly |
| `%` | LaTeX, Erlang |

Doc comments (`///`, `//!`) work too. Block comments (`/* … */`) aren't supported yet.

## Formatting

Card text is Markdown. In a review, code blocks are shown with their language behind a `│`
bar and syntax-coloured, inline `code` is coloured, and `**bold**`, `*italic*`, lists and
links are drawn as such. Line breaks are kept as you wrote them. Set `NO_COLOR` to turn
colour off.

Name the language after the opening fence (```` ```rust ````, ```` ```py ````) to get colours.
About 75 languages are recognised, including Rust, Python, JavaScript (also used for
TypeScript), Go, C, C++, C#, Java, shell, SQL, Ruby, Haskell, Lua, YAML and JSON. Others are
shown uncoloured. Colours come from your terminal's own palette, so they follow its theme.

Code blocks need a Markdown card file (see below); in a code comment, use inline `code`.

## Cloze cards

Put `{{c1::…}}` around the part to hide. A cloze card needs no answer lines:

```go
// @card A nil map in Go can be {{c1::read}} but not {{c2::written}}.
```

Each `cN` becomes its own card.

## Markdown card files

For cards that don't belong next to code, use a file ending in `.cards.md`, or any `.md`
file inside `.keepjay/cards/`:

````markdown
---
deck: Rust::Async
tags: [lang::rust, async]
---

Q: What does `.await` do when the future isn't ready?
A: Yields to the executor; the task is woken when it can make progress.

C: In Tokio, {{c1::spawn_blocking}} runs blocking code on a separate thread pool.

Q: What's wrong with this?
```rust
let guard = mutex.lock().unwrap();
some_async_call().await;
```
A: A `std` mutex guard is held across `.await`, which can block the executor.
````

- `Q:` and `A:` make a question-and-answer card; `C:` makes a cloze card.
- A card ends at the next `Q:`, `C:`, heading or `---`.
- The optional front matter sets the deck and tags for the whole file.
- After the first scan, each card gets an `<!-- id: … -->` line. Keep it.

## Decks and tags

Each source puts its cards in one deck, by default named after the folder:

```sh
keepjay source add ~/code/api --deck "Work::API" --tag work
```

Tags from the source, the file's front matter and the card itself are combined.

## Skipping parts of a file

A comment `@card:off` stops Keepjay finding cards until a comment `@card:on`, or the end of
the file. That's useful in tests or docs full of `@card` examples. In Markdown, use
`<!-- @card:off -->` and `<!-- @card:on -->`.

To skip whole files or folders, list them in a `.keepjayignore` file, which works like
`.gitignore`. Keepjay already skips whatever your `.gitignore` does, hidden folders, binary
files and files over 1 MiB.

## Cards without a file

You can also add a card that lives only in Keepjay's database, with no file behind it:

```sh
keepjay deck create Rust          # once
keepjay add card --deck Rust "What does Box<dyn Error> erase?" "The concrete error type"
```
