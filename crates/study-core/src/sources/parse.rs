// SPDX-License-Identifier: AGPL-3.0-or-later
//! What the parsers produce: cards found in a file, before they are matched against the
//! database (docs/specs/13-code-sources.md §3–§5). Pure data; no I/O.

use std::collections::BTreeMap;

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::cards::CardType;

/// Where a card was written, which decides how its id is written back (SRC-005).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "style")]
pub enum CardStyle {
    /// An `@card` line comment. `marker` is the exact comment marker used (`//`, `///`, `#`).
    Comment { marker: String },
    /// A `Q:`/`A:` or `C:` card in a Markdown card file.
    Markdown,
}

/// One card found in a source file.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ParsedCard {
    /// The stable id written in the file (`@card(k22k1m8e)`, `<!-- id: k22k1m8e -->`).
    pub id: Option<String>,
    /// `Basic` (question and answer) or `Cloze` (`{{c1::…}}`).
    #[serde(serialize_with = "card_type_id")]
    pub card_type: CardType,
    /// Field values for the card type: Front/Back or Text/Extra.
    pub fields: BTreeMap<String, String>,
    /// Per-card tags (`tags:` line), in addition to the source's tags.
    pub tags: Vec<String>,
    /// Deck from the file's front matter, overriding the source's deck.
    pub deck: Option<String>,
    /// Path relative to the source root, with `/` separators.
    pub path: String,
    /// The card and its anchored code, 1-based and inclusive.
    pub line_start: u32,
    pub line_end: u32,
    /// The line holding the id: the `@card` line, or the line after a Markdown card.
    pub id_line: u32,
    /// The last line of the card's own text (its comment block, or its Markdown lines and
    /// id comment), 1-based. `line_start..=text_end` is what `stale retire` removes.
    #[serde(skip)]
    pub text_end: u32,
    /// SHA-256 of the anchored code (comment cards only), normalised (see [`code_hash`]).
    pub code_hash: Option<String>,
    pub style: CardStyle,
}

fn card_type_id<S: serde::Serializer>(t: &CardType, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(t.id())
}

/// A problem found while parsing; the card is skipped and the scan goes on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ParseWarning {
    pub path: String,
    pub line: u32,
    pub message: String,
}

/// Cards and warnings from one file.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Parsed {
    pub cards: Vec<ParsedCard>,
    pub warnings: Vec<ParseWarning>,
}

/// Hash of anchored code, insensitive to line endings and trailing whitespace so that a
/// checkout on another OS or an editor's whitespace cleanup doesn't mark cards stale.
// @card(vy6xf4vh) Why does `code_hash` hash `line.trim_end()` rather than the raw line?
// So a `\r\n` checkout on Windows, or an editor stripping trailing spaces, doesn't flag
// cards as stale: `trim_end` removes the `\r` as well as spaces and tabs.
pub fn code_hash(lines: &[&str]) -> String {
    let mut h = Sha256::new();
    for line in lines {
        h.update(line.trim_end().as_bytes());
        h.update(b"\n");
    }
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// Whether text contains a cloze deletion such as `{{c1::answer}}`.
pub fn has_cloze(text: &str) -> bool {
    let mut rest = text;
    while let Some(i) = rest.find("{{c") {
        let after = &rest[i + 3..];
        let digits = after.bytes().take_while(u8::is_ascii_digit).count();
        if digits > 0 && after[digits..].starts_with("::") {
            return true;
        }
        rest = after;
    }
    false
}

/// `@card:off` / `@card:on` (SRC-012): `Some(false)` stops finding cards from this line,
/// `Some(true)` starts again. Text after the switch (a reason) is allowed.
pub fn card_switch(text: &str) -> Option<bool> {
    let rest = text.trim().strip_prefix("@card:")?;
    if rest.starts_with(char::is_whitespace) {
        return None;
    }
    match rest.split_whitespace().next()? {
        "off" => Some(false),
        "on" => Some(true),
        _ => None,
    }
}

/// The warning for an `@card:` word that isn't `off` or `on`.
pub fn bad_switch(text: &str) -> Option<String> {
    let t = text.trim();
    (t.starts_with("@card:") && card_switch(t).is_none()).then(|| {
        let word = t.split_whitespace().next().unwrap_or(t);
        format!("`{word}` isn't a scan switch; write `@card:off` or `@card:on`")
    })
}

/// A short id as written in files (CLI-007): 8 lowercase Crockford base32 characters.
pub fn is_short_id(s: &str) -> bool {
    s.len() == 8
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b.is_ascii_lowercase() && !b"ilou".contains(&b)))
}

/// Card fields for a question and answer, or a cloze text and extra.
pub fn fields_for(question: &str, answer: &str) -> (CardType, BTreeMap<String, String>) {
    let mut fields = BTreeMap::new();
    if has_cloze(question) {
        fields.insert("Text".into(), question.to_string());
        if !answer.is_empty() {
            fields.insert("Extra".into(), answer.to_string());
        }
        (CardType::Cloze, fields)
    } else {
        fields.insert("Front".into(), question.to_string());
        fields.insert("Back".into(), answer.to_string());
        (CardType::Basic, fields)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_ignores_line_endings_and_trailing_space() {
        assert_eq!(
            code_hash(&["fn a() {", "}"]),
            code_hash(&["fn a() {  \r", "}"])
        );
        assert_ne!(code_hash(&["fn a() {}"]), code_hash(&["fn b() {}"]));
        assert_ne!(code_hash(&["  x"]), code_hash(&["x"]), "indentation counts");
    }

    #[test]
    fn cloze_detection() {
        assert!(has_cloze("Use {{c1::Arc}} here"));
        assert!(has_cloze("{{c12::x}}"));
        assert!(!has_cloze("{{c::x}} {{cx::y}} {c1::z}"));
    }

    #[test]
    fn short_ids() {
        assert!(is_short_id("k22k1m8e"));
        assert!(!is_short_id("K22K1M8E"));
        assert!(!is_short_id("k22k1m8"));
        assert!(
            !is_short_id("k22k1m8i"),
            "Crockford base32 has no i, l, o or u"
        );
    }
}
