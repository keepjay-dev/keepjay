// SPDX-License-Identifier: AGPL-3.0-or-later
//! Markdown card files (SRC-003; docs/specs/13-code-sources.md §4).
//!
//! ```text
//! ---
//! deck: Rust::Async
//! tags: [lang::rust, async]
//! ---
//!
//! Q: What does `.await` do when the future isn't ready?
//! A: Yields to the executor.
//! <!-- id: k22k1m8e -->
//!
//! C: In Tokio, {{c1::spawn_blocking}} runs blocking code on its own thread pool.
//! ```
//!
//! `Q:`/`A:` make a basic card and `C:` a cloze (an `A:` after it fills *Extra*). A card ends
//! at the next `Q:` or `C:`, a heading, a `---` line, or its id comment. Fenced code blocks
//! may appear anywhere in a card; markers inside them are ignored. Text before the first
//! card is ordinary prose.

use std::collections::BTreeMap;

use super::parse::{CardStyle, ParseWarning, Parsed, ParsedCard, has_cloze, is_short_id};
use crate::cards::CardType;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Question,
    Cloze,
}

struct Open<'a> {
    kind: Kind,
    start: usize,
    prompt: Vec<&'a str>,
    answer: Option<Vec<&'a str>>,
    /// Last line with content, 0-based.
    last: usize,
}

#[derive(Default)]
struct FrontMatter {
    deck: Option<String>,
    tags: Vec<String>,
}

/// Find the cards in a Markdown card file.
pub fn parse(path: &str, text: &str) -> Parsed {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Parsed::default();
    let (front, body_start) = front_matter(&lines);
    let mut open: Option<Open> = None;
    let mut in_fence = false;

    let mut i = body_start;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
        }
        if in_fence || trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            if let Some(card) = open.as_mut() {
                card.push(line, i);
            }
            i += 1;
            continue;
        }

        if let Some(rest) = line.strip_prefix("Q:") {
            finish(&mut out, path, &front, open.take(), None);
            open = Some(Open::new(Kind::Question, i, rest));
        } else if let Some(rest) = line.strip_prefix("C:") {
            finish(&mut out, path, &front, open.take(), None);
            open = Some(Open::new(Kind::Cloze, i, rest));
        } else if let Some(rest) = line.strip_prefix("A:") {
            match open.as_mut() {
                Some(card) if card.answer.is_none() => {
                    card.answer = Some(vec![rest.trim_start()]);
                    card.last = i;
                }
                _ => out.warnings.push(warning(
                    path,
                    i,
                    "`A:` without a `Q:` or `C:` before it".into(),
                )),
            }
        } else if let Some(id) = id_comment(trimmed) {
            match open.take() {
                Some(card) if is_short_id(id) => {
                    finish(&mut out, path, &front, Some(card), Some((id, i)))
                }
                Some(card) => {
                    out.warnings.push(warning(
                        path,
                        i,
                        format!("'{id}' isn't a card id; remove the comment and scan again to get a new one"),
                    ));
                    finish(&mut out, path, &front, Some(card), None);
                }
                None => {}
            }
        } else if is_heading(line) || trimmed == "---" {
            finish(&mut out, path, &front, open.take(), None);
        } else if let Some(card) = open.as_mut() {
            card.push(line, i);
        }
        i += 1;
    }
    finish(&mut out, path, &front, open.take(), None);
    out
}

impl<'a> Open<'a> {
    fn new(kind: Kind, start: usize, rest: &'a str) -> Self {
        Open {
            kind,
            start,
            prompt: vec![rest.trim_start()],
            answer: None,
            last: start,
        }
    }

    fn push(&mut self, line: &'a str, i: usize) {
        match self.answer.as_mut() {
            Some(a) => a.push(line),
            None => self.prompt.push(line),
        }
        if !line.trim().is_empty() {
            self.last = i;
        }
    }
}

fn finish(
    out: &mut Parsed,
    path: &str,
    front: &FrontMatter,
    card: Option<Open>,
    id: Option<(&str, usize)>,
) {
    let Some(card) = card else { return };
    let prompt = join(&card.prompt);
    let answer = card.answer.as_deref().map(join).unwrap_or_default();
    let line = card.start;
    let mut fields = BTreeMap::new();
    let card_type = match card.kind {
        Kind::Question => {
            if prompt.is_empty() || answer.is_empty() {
                out.warnings.push(warning(
                    path,
                    line,
                    "a `Q:` card needs both a question and an `A:` answer".into(),
                ));
                return;
            }
            fields.insert("Front".to_string(), prompt);
            fields.insert("Back".to_string(), answer);
            CardType::Basic
        }
        Kind::Cloze => {
            if !has_cloze(&prompt) {
                out.warnings.push(warning(
                    path,
                    line,
                    "a `C:` card needs a deletion such as {{c1::answer}}".into(),
                ));
                return;
            }
            fields.insert("Text".to_string(), prompt);
            if !answer.is_empty() {
                fields.insert("Extra".to_string(), answer);
            }
            CardType::Cloze
        }
    };
    let (id, id_line, line_end) = match id {
        Some((id, at)) => (Some(id.to_string()), at + 1, at + 1),
        None => (None, card.last + 2, card.last + 1),
    };
    out.cards.push(ParsedCard {
        id,
        card_type,
        fields,
        tags: front.tags.clone(),
        deck: front.deck.clone(),
        path: path.to_string(),
        line_start: (card.start + 1) as u32,
        line_end: line_end as u32,
        id_line: id_line as u32,
        code_hash: None,
        style: CardStyle::Markdown,
    });
}

/// Lines joined, with surrounding blank lines removed and the text trimmed at its ends.
fn join(lines: &[&str]) -> String {
    lines.join("\n").trim().to_string()
}

fn warning(path: &str, line: usize, message: String) -> ParseWarning {
    ParseWarning {
        path: path.to_string(),
        line: (line + 1) as u32,
        message,
    }
}

fn is_heading(line: &str) -> bool {
    let hashes = line.bytes().take_while(|&b| b == b'#').count();
    (1..=6).contains(&hashes) && line[hashes..].starts_with([' ', '\t'])
}

/// `<!-- id: k22k1m8e -->` → `k22k1m8e`.
fn id_comment(trimmed: &str) -> Option<&str> {
    let inner = trimmed.strip_prefix("<!--")?.strip_suffix("-->")?.trim();
    Some(inner.strip_prefix("id:")?.trim())
}

/// Front matter between `---` lines at the top: `deck:` and `tags:` (a `[a, b]` list or
/// space-separated). Other keys are ignored. Returns where the body starts.
fn front_matter(lines: &[&str]) -> (FrontMatter, usize) {
    let mut fm = FrontMatter::default();
    if lines.first().map(|l| l.trim()) != Some("---") {
        return (fm, 0);
    }
    let Some(end) = lines.iter().skip(1).position(|l| l.trim() == "---") else {
        return (fm, 0);
    };
    for line in &lines[1..=end] {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim();
        match key.trim() {
            "deck" => {
                let deck = unquote(value);
                if !deck.is_empty() {
                    fm.deck = Some(deck.to_string());
                }
            }
            "tags" => {
                let list = value
                    .strip_prefix('[')
                    .and_then(|v| v.strip_suffix(']'))
                    .unwrap_or(value);
                fm.tags = list
                    .split(|c: char| c == ',' || c.is_whitespace())
                    .map(unquote)
                    .filter(|t| !t.is_empty())
                    .map(str::to_string)
                    .collect();
            }
            _ => {}
        }
    }
    (fm, end + 2)
}

fn unquote(s: &str) -> &str {
    let s = s.trim();
    s.strip_prefix('"')
        .and_then(|v| v.strip_suffix('"'))
        .or_else(|| s.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
        .unwrap_or(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
---
deck: Rust::Async
tags: [lang::rust, async]
---

Some prose before the cards is fine.

Q: What does `.await` do when the future isn't ready?
A: Yields to the executor; the task is woken when the future can make progress.
<!-- id: k22k1m8e -->

C: In Tokio, {{c1::spawn_blocking}} runs blocking code on a dedicated thread pool.

Q: What's wrong with this?
```rust
let guard = mutex.lock().unwrap();
// Q: not a card, it's inside a fence
some_async_call().await;
```
A: A `std` mutex guard is held across `.await`.

## Next section
";

    #[test]
    fn sample_file() {
        let p = parse("notes/async.cards.md", SAMPLE);
        assert!(p.warnings.is_empty(), "{:?}", p.warnings);
        assert_eq!(p.cards.len(), 3);

        let q = &p.cards[0];
        assert_eq!(q.card_type, CardType::Basic);
        assert_eq!(q.id.as_deref(), Some("k22k1m8e"));
        assert_eq!(q.deck.as_deref(), Some("Rust::Async"));
        assert_eq!(q.tags, ["lang::rust", "async"]);
        assert_eq!((q.line_start, q.line_end, q.id_line), (8, 10, 10));
        assert!(q.fields["Back"].starts_with("Yields"));

        let c = &p.cards[1];
        assert_eq!(c.card_type, CardType::Cloze);
        assert_eq!(c.id, None);
        assert_eq!((c.line_start, c.line_end, c.id_line), (12, 12, 13));

        let fenced = &p.cards[2];
        assert_eq!(
            fenced.fields["Front"],
            "What's wrong with this?\n```rust\nlet guard = mutex.lock().unwrap();\n// Q: not a card, it's inside a fence\nsome_async_call().await;\n```"
        );
        assert_eq!(
            fenced.fields["Back"],
            "A `std` mutex guard is held across `.await`."
        );
        assert_eq!(
            fenced.line_end, 20,
            "ends before the blank line and heading"
        );
    }

    #[test]
    fn multi_line_answers_and_extra_on_cloze() {
        let p = parse(
            "a.cards.md",
            "Q: Two lines?\nA: First.\nSecond.\n\nC: {{c1::x}} is x\nA: Extra info\n",
        );
        assert!(p.warnings.is_empty(), "{:?}", p.warnings);
        assert_eq!(p.cards[0].fields["Back"], "First.\nSecond.");
        assert_eq!(p.cards[1].fields["Extra"], "Extra info");
        assert!(p.cards[0].deck.is_none() && p.cards[0].tags.is_empty());
    }

    #[test]
    fn problems_become_warnings() {
        let p = parse(
            "a.cards.md",
            "A: orphan\nQ: no answer\n\nC: no deletion\n# Heading\nQ: fine?\nA: yes\n<!-- id: BAD -->\n",
        );
        assert_eq!(p.cards.len(), 1, "the last card is kept without an id");
        assert_eq!(p.cards[0].id, None);
        let lines: Vec<_> = p.warnings.iter().map(|w| w.line).collect();
        assert_eq!(lines, [1, 2, 4, 8], "{:?}", p.warnings);
    }

    #[test]
    fn front_matter_variants() {
        let p = parse(
            "a.cards.md",
            "---\ndeck: \"Algo::DP\"\ntags: dp 'pattern::knapsack'\nauthor: me\n---\nQ: q\nA: a\n",
        );
        assert_eq!(p.cards[0].deck.as_deref(), Some("Algo::DP"));
        assert_eq!(p.cards[0].tags, ["dp", "pattern::knapsack"]);
        // An unterminated block isn't front matter.
        let p = parse("a.cards.md", "---\ndeck: X\nQ: q\nA: a\n");
        assert_eq!(p.cards[0].deck, None);
    }
}
