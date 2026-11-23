// SPDX-License-Identifier: AGPL-3.0-or-later
//! `@card` line comments (SRC-002; docs/specs/13-code-sources.md §3).
//!
//! ```text
//! // @card(k22k1m8e) What does Core::open do before running migrations?
//! // Creates the data dir and loads or creates the device id.
//! // tags: lang::rust
//! pub fn open(paths: Paths) -> Result<Self> {
//! ```
//!
//! The question follows `@card` on the same line. The answer is the following comment
//! lines with the same indentation and marker, up to a blank comment line, a code line or
//! another `@card`. The anchored code is the lines after the comment block, up to the first
//! blank line (at most [`MAX_ANCHORED_LINES`]).
//!
//! A comment `@card:off` stops finding cards until a comment `@card:on`, or to the end of
//! the file (SRC-012), e.g. before test code whose strings contain `@card` examples.

use super::parse::{
    CardStyle, ParseWarning, Parsed, ParsedCard, bad_switch, card_switch, code_hash, fields_for,
    is_short_id,
};

/// Upper bound on anchored code lines per card, until syntax-tree anchors (CODE-002).
pub const MAX_ANCHORED_LINES: usize = 30;

/// A comment line split into its parts.
struct CommentLine<'a> {
    indent: &'a str,
    /// The marker as written: the prefix plus repeats (`///`, `##`, `;;`) or `//!`.
    marker: &'a str,
    /// The text after the marker and one optional space.
    text: &'a str,
}

fn split_comment<'a>(line: &'a str, prefix: &str) -> Option<CommentLine<'a>> {
    let trimmed = line.trim_start();
    let indent = &line[..line.len() - trimmed.len()];
    let rest = trimmed.strip_prefix(prefix)?;
    let last = prefix.chars().last()?;
    let repeats = rest.len() - rest.trim_start_matches(last).len();
    let mut marker_len = prefix.len() + repeats;
    if prefix == "//" && trimmed[marker_len..].starts_with('!') {
        marker_len += 1; // `//!` inner doc comments
    }
    let marker = &trimmed[..marker_len];
    let after = &trimmed[marker_len..];
    let text = after.strip_prefix(' ').unwrap_or(after);
    Some(CommentLine {
        indent,
        marker,
        text,
    })
}

/// `@card`, `@card(<id>)`, followed by the question. Returns (id, question) or `None`
/// when the comment isn't a card. `Err` carries a warning for a malformed card.
fn split_card(text: &str) -> Option<Result<(Option<String>, String), String>> {
    let rest = text.trim_start().strip_prefix("@card")?;
    let (id, question) = if let Some(after) = rest.strip_prefix('(') {
        let Some(close) = after.find(')') else {
            return Some(Err("unclosed `@card(` id".into()));
        };
        let id = after[..close].trim();
        if !is_short_id(id) {
            return Some(Err(format!(
                "'{id}' isn't a card id (8 characters such as `k22k1m8e`); remove it and scan again to get a new one"
            )));
        }
        (Some(id.to_string()), &after[close + 1..])
    } else {
        (None, rest)
    };
    // `@cards`, `@cardinal`… are not cards.
    if !question.is_empty() && !question.starts_with(char::is_whitespace) {
        return None;
    }
    Some(Ok((id, question.trim().to_string())))
}

/// Find `@card` comments in a file that uses `prefix` for line comments.
pub fn parse(path: &str, text: &str, prefix: &str) -> Parsed {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Parsed::default();
    let mut i = 0;
    let mut on = true;
    while i < lines.len() {
        let Some(head) = split_comment(lines[i], prefix) else {
            i += 1;
            continue;
        };
        if let Some(switch) = card_switch(head.text) {
            on = switch;
            i += 1;
            continue;
        }
        if !on {
            i += 1;
            continue;
        }
        if let Some(message) = bad_switch(head.text) {
            out.warnings.push(ParseWarning {
                path: path.to_string(),
                line: (i + 1) as u32,
                message,
            });
            i += 1;
            continue;
        }
        let Some(card) = split_card(head.text) else {
            i += 1;
            continue;
        };
        let start = i;
        let warn = |out: &mut Parsed, message: String| {
            out.warnings.push(ParseWarning {
                path: path.to_string(),
                line: (start + 1) as u32,
                message,
            })
        };
        let (id, question) = match card {
            Ok(c) => c,
            Err(message) => {
                warn(&mut out, message);
                i += 1;
                continue;
            }
        };

        // Answer: continuation lines with the same indentation and marker.
        let mut answer: Vec<&str> = Vec::new();
        let mut j = i + 1;
        while j < lines.len() {
            match split_comment(lines[j], prefix) {
                Some(c)
                    if c.indent == head.indent
                        && c.marker == head.marker
                        && !c.text.trim().is_empty()
                        && split_card(c.text).is_none() =>
                {
                    answer.push(c.text);
                    j += 1;
                }
                _ => break,
            }
        }
        let block_end = j; // exclusive

        // A final `tags:` line holds per-card tags.
        let mut tags = Vec::new();
        if let Some(last) = answer.last()
            && let Some(list) = last.trim().strip_prefix("tags:")
        {
            tags = list.split_whitespace().map(str::to_string).collect();
            answer.pop();
        }

        // Anchored code: following lines up to a blank line or the next card.
        let is_card = |line: &str| {
            split_comment(line, prefix)
                .is_some_and(|c| split_card(c.text).is_some() || card_switch(c.text).is_some())
        };
        let mut k = block_end;
        while k < lines.len()
            && k - block_end < MAX_ANCHORED_LINES
            && !lines[k].trim().is_empty()
            && !is_card(lines[k])
        {
            k += 1;
        }
        let code = &lines[block_end..k];

        i = block_end;
        if question.is_empty() {
            warn(&mut out, "`@card` has no question after it".into());
            continue;
        }
        let answer = answer.join("\n");
        let (card_type, fields) = fields_for(&question, &answer);
        if card_type == crate::cards::CardType::Basic && answer.is_empty() {
            warn(
                &mut out,
                "card has no answer: put it on the comment lines below the question, or use {{c1::…}} for a cloze".into(),
            );
            continue;
        }
        out.cards.push(ParsedCard {
            id,
            card_type,
            fields,
            tags,
            deck: None,
            path: path.to_string(),
            line_start: (start + 1) as u32,
            line_end: k.max(block_end) as u32,
            id_line: (start + 1) as u32,
            code_hash: Some(code_hash(code)),
            style: CardStyle::Comment {
                marker: head.marker.to_string(),
            },
        });
    }
    out
}

// @card:off the tests below are full of `@card` examples (SRC-012).
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::CardType;

    fn one(text: &str, prefix: &str) -> ParsedCard {
        let p = parse("src/lib.rs", text, prefix);
        assert!(p.warnings.is_empty(), "{:?}", p.warnings);
        assert_eq!(p.cards.len(), 1, "{:?}", p.cards);
        p.cards.into_iter().next().unwrap()
    }

    #[test]
    fn question_answer_and_anchored_code() {
        let src = "\
use x;

// @card What does Core::open do before running migrations?
// Creates the data dir and loads or creates the device id.
pub fn open(paths: Paths) -> Result<Self> {
    todo!()
}

fn other() {}
";
        let c = one(src, "//");
        assert_eq!(c.card_type, CardType::Basic);
        assert_eq!(
            c.fields["Front"],
            "What does Core::open do before running migrations?"
        );
        assert_eq!(
            c.fields["Back"],
            "Creates the data dir and loads or creates the device id."
        );
        assert_eq!((c.line_start, c.line_end, c.id_line), (3, 7, 3));
        assert_eq!(
            c.code_hash.as_deref(),
            Some(
                code_hash(&[
                    "pub fn open(paths: Paths) -> Result<Self> {",
                    "    todo!()",
                    "}"
                ])
                .as_str()
            )
        );
        assert_eq!(c.id, None);
    }

    #[test]
    fn editing_the_card_text_doesnt_change_the_code_hash() {
        let a = one("// @card Q?\n// A.\nfn f() {}\n", "//");
        let b = one(
            "// @card(k22k1m8e) Better Q?\n// Better A.\nfn f() {}\n",
            "//",
        );
        assert_eq!(a.code_hash, b.code_hash);
        assert_eq!(b.id.as_deref(), Some("k22k1m8e"));
        let c = one("// @card Q?\n// A.\nfn g() {}\n", "//");
        assert_ne!(a.code_hash, c.code_hash);
    }

    #[test]
    fn python_with_id_tags_and_multiline_answer() {
        let src = "\
def run():
    # @card(q8w2m4za) Why does `asyncio.gather` not cancel siblings?
    # It propagates the first exception but lets the other tasks run;
    # use a TaskGroup (3.11+) for structured cancellation.
    # tags: lang::python async
    return gather(a(), b())
";
        let c = one(src, "#");
        assert_eq!(c.id.as_deref(), Some("q8w2m4za"));
        assert_eq!(
            c.fields["Back"],
            "It propagates the first exception but lets the other tasks run;\nuse a TaskGroup (3.11+) for structured cancellation."
        );
        assert_eq!(c.tags, ["lang::python", "async"]);
        assert_eq!((c.line_start, c.line_end), (2, 6));
    }

    #[test]
    fn cloze_needs_no_answer() {
        let c = one(
            "-- @card {{c1::VACUUM INTO}} copies a live database\nSELECT 1;\n",
            "--",
        );
        assert_eq!(c.card_type, CardType::Cloze);
        assert_eq!(
            c.fields["Text"],
            "{{c1::VACUUM INTO}} copies a live database"
        );
        assert!(!c.fields.contains_key("Extra"));
    }

    #[test]
    fn doc_comment_markers_and_answer_boundaries() {
        let src = "\
/// @card What does the answer end at?
/// A blank comment line.
///
/// Not part of the answer.
fn f() {}
";
        let c = one(src, "//");
        assert_eq!(c.fields["Back"], "A blank comment line.");
        assert_eq!(
            c.style,
            CardStyle::Comment {
                marker: "///".into()
            }
        );
        // The anchored code starts after the comment block, at the blank comment line.
        assert_eq!(c.line_end, 5);

        // A different marker or indentation ends the answer.
        let c = one("// @card Q?\n// A.\n    // not answer\nx\n", "//");
        assert_eq!(c.fields["Back"], "A.");
        let c = one("// @card Q?\n// A.\n//! not answer\n", "//");
        assert_eq!(c.fields["Back"], "A.");
    }

    #[test]
    fn consecutive_cards_and_non_cards() {
        let src = "\
// @cards are not cards, nor is @cardinal
// @card First?
// One.
// @card Second?
// Two.
let x = 1;
";
        let p = parse("a.rs", src, "//");
        assert!(p.warnings.is_empty());
        let fronts: Vec<_> = p.cards.iter().map(|c| c.fields["Front"].as_str()).collect();
        assert_eq!(fronts, ["First?", "Second?"]);
        assert_eq!(
            p.cards[0].line_end, 3,
            "no code directly after the first card"
        );
        assert_eq!(p.cards[1].line_end, 6);
    }

    #[test]
    fn anchored_code_is_capped() {
        let mut src = String::from("# @card Q?\n# A.\n");
        for n in 0..50 {
            src.push_str(&format!("x{n} = {n}\n"));
        }
        let c = one(&src, "#");
        assert_eq!(c.line_end, 2 + MAX_ANCHORED_LINES as u32);
    }

    #[test]
    fn malformed_cards_become_warnings() {
        let src = "\
// @card
// @card Question without an answer?
fn f() {}
// @card(NOT-AN-ID) Q?
// A.
// @card(k22k1m8e Q?
";
        let p = parse("a.rs", src, "//");
        assert!(p.cards.is_empty(), "{:?}", p.cards);
        let lines: Vec<_> = p.warnings.iter().map(|w| w.line).collect();
        assert_eq!(lines, [1, 2, 4, 6], "{:?}", p.warnings);
    }

    #[test]
    fn crlf_files() {
        let c = one("// @card Q?\r\n// A.\r\nfn f() {}\r\n", "//");
        assert_eq!(c.fields["Back"], "A.");
        assert_eq!(
            c.code_hash,
            one("// @card Q?\n// A.\nfn f() {}\n", "//").code_hash
        );
    }

    #[test]
    fn card_off_and_on_skip_a_region() {
        let text = concat!(
            "// @card First?\n// One.\nfn a() {}\n\n",
            "// @card:off examples below aren't cards\n",
            "// @card Not a card?\n// No.\n",
            "// @card:on\n",
            "# @card wrong prefix, ignored anyway\n\n",
            "// @card Second?\n// Two.\nfn b() {}\n",
            "// @card:off to the end of the file\n",
            "const S: &str = \"x\";\n// @card Also not?\n// No.\n",
        );
        let p = parse("src/lib.rs", text, "//");
        assert!(p.warnings.is_empty(), "{:?}", p.warnings);
        let questions: Vec<&str> = p.cards.iter().map(|c| c.fields["Front"].as_str()).collect();
        assert_eq!(questions, ["First?", "Second?"]);
        // The anchored code stops at the switch line.
        assert_eq!(p.cards[1].line_end, 13);
        assert_eq!(card_switch("@card:offline"), None);
        let p = parse("a.rs", "// @card:off: oops\n// @card Q?\n// A.\n", "//");
        assert_eq!(p.cards.len(), 1, "a misspelt switch changes nothing");
        assert!(
            p.warnings[0].message.contains("@card:off:"),
            "{:?}",
            p.warnings
        );
        assert_eq!(card_switch("@card: off"), None);
    }
}
