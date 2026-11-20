// SPDX-License-Identifier: AGPL-3.0-or-later
//! Plain-text question/answer rendering for each built-in card type (SRS-002).
//!
//! The desktop app will render Markdown/HTML through sanitised templates (SRS-013, D-013).
//! This plain form serves the CLI and screen readers.

use std::collections::BTreeMap;

use super::CardType;

/// Question and answer text for one card of a fact.
pub fn render(
    card_type: CardType,
    fields: &BTreeMap<String, String>,
    ord: u32,
) -> (String, String) {
    let get = |name: &str| fields.get(name).map(|v| v.trim()).unwrap_or("").to_string();
    match card_type {
        CardType::Basic | CardType::TypeIn => (get("Front"), get("Back")),
        CardType::BasicReverse | CardType::OptionalReverse if ord == 1 => {
            (get("Back"), get("Front"))
        }
        CardType::BasicReverse | CardType::OptionalReverse => (get("Front"), get("Back")),
        CardType::Cloze => {
            let text = get("Text");
            let number = ord + 1;
            let question = render_cloze(&text, number, false);
            let mut answer = render_cloze(&text, number, true);
            let extra = get("Extra");
            if !extra.is_empty() {
                answer.push_str("\n\n");
                answer.push_str(&extra);
            }
            (question, answer)
        }
    }
}

/// Replace cloze markers: the active deletion becomes `[...]` (or `[hint]`) in the question
/// and `[answer]` when revealed; other deletions show their text.
pub fn render_cloze(text: &str, active: u32, reveal: bool) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("{{c") {
        let after = &rest[start + 3..];
        let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
        let body_start = digits.len() + 2;
        let parsed = (!digits.is_empty() && after[digits.len()..].starts_with("::"))
            .then(|| after[body_start..].find("}}"))
            .flatten();
        let Some(end) = parsed else {
            // Not a cloze marker: keep the literal text and continue after it.
            out.push_str(&rest[..start + 3]);
            rest = after;
            continue;
        };
        out.push_str(&rest[..start]);
        let inner = &after[body_start..body_start + end];
        let (answer, hint) = match inner.split_once("::") {
            Some((a, h)) => (a, Some(h)),
            None => (inner, None),
        };
        if digits.parse::<u32>().ok() == Some(active) {
            if reveal {
                out.push('[');
                out.push_str(answer);
                out.push(']');
            } else {
                out.push('[');
                out.push_str(hint.unwrap_or("..."));
                out.push(']');
            }
        } else {
            out.push_str(answer);
        }
        rest = &after[body_start + end + 2..];
    }
    out.push_str(rest);
    out
}

/// Convert stored HTML (e.g. from Anki) to readable plain text: line breaks for block tags,
/// `[image: …]` / `[audio: …]` placeholders, entities decoded, other tags removed. Text
/// without tags passes through unchanged (`a < b` is not a tag).
pub fn html_to_text(html: &str) -> String {
    if !html.contains('<') && !html.contains('&') && !html.contains("[sound:") {
        return html.to_string();
    }
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(start) = rest.find('<') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let is_tag = after
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '/' || c == '!');
        let Some(end) = after.find('>').filter(|_| is_tag) else {
            out.push('<');
            rest = after;
            continue;
        };
        let tag = &after[..end];
        let name: String = tag
            .trim_start_matches('/')
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
            .to_ascii_lowercase();
        rest = &after[end + 1..];
        match name.as_str() {
            "br" | "div" | "p" | "tr" | "h1" | "h2" | "h3" | "h4" | "hr" => {
                if !out.ends_with('\n') && !out.is_empty() {
                    out.push('\n');
                }
            }
            "li" if !tag.starts_with('/') => out.push_str("\n• "),
            "img" => {
                let src = attr(tag, "src").unwrap_or_default();
                out.push_str(&format!("[image: {src}]"));
            }
            "style" | "script" if !tag.starts_with('/') => {
                let close = format!("</{name}>");
                rest = rest
                    .to_ascii_lowercase()
                    .find(&close)
                    .map(|i| &rest[i + close.len()..])
                    .unwrap_or("");
            }
            _ => {}
        }
    }
    out.push_str(rest);
    let text = decode_entities(&out).replace("[sound:", "[audio: ");
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let mut joined = lines.join("\n");
    while joined.contains("\n\n\n") {
        joined = joined.replace("\n\n\n", "\n\n");
    }
    joined.trim().to_string()
}

fn attr(tag: &str, name: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let at = lower.find(&format!("{name}="))? + name.len() + 1;
    let value = &tag[at..];
    let (quote, value) = match value.chars().next()? {
        q @ ('"' | '\'') => (Some(q), &value[1..]),
        _ => (None, value),
    };
    let end = match quote {
        Some(q) => value.find(q)?,
        None => value
            .find(|c: char| c.is_whitespace() || c == '/')
            .unwrap_or(value.len()),
    };
    Some(value[..end].to_string())
}

pub(crate) fn decode_entities(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find('&') {
        out.push_str(&rest[..i]);
        let after = &rest[i + 1..];
        let Some(semi) = after.find(';').filter(|&j| j <= 10) else {
            out.push('&');
            rest = after;
            continue;
        };
        let entity = &after[..semi];
        let decoded = match entity {
            "nbsp" => Some(' '),
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            e if e.starts_with("#x") || e.starts_with("#X") => u32::from_str_radix(&e[2..], 16)
                .ok()
                .and_then(char::from_u32),
            e if e.starts_with('#') => e[1..].parse().ok().and_then(char::from_u32),
            _ => None,
        };
        match decoded {
            Some(c) => {
                out.push(c);
                rest = &after[semi + 1..];
            }
            None => {
                out.push('&');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn basic_and_reverse() {
        let qa = f(&[("Front", "Cell"), ("Back", "Zelle")]);
        assert_eq!(
            render(CardType::Basic, &qa, 0),
            ("Cell".into(), "Zelle".into())
        );
        assert_eq!(
            render(CardType::BasicReverse, &qa, 1),
            ("Zelle".into(), "Cell".into())
        );
    }

    #[test]
    fn cloze_hides_only_the_active_deletion() {
        let t = f(&[
            (
                "Text",
                "{{c1::Mitochondria}} make {{c2::ATP::energy molecule}}",
            ),
            ("Extra", "Krebs cycle"),
        ]);
        let (q1, a1) = render(CardType::Cloze, &t, 0);
        assert_eq!(q1, "[...] make ATP");
        assert_eq!(a1, "[Mitochondria] make ATP\n\nKrebs cycle");
        let (q2, _) = render(CardType::Cloze, &t, 1);
        assert_eq!(q2, "Mitochondria make [energy molecule]");
    }

    #[test]
    fn html_from_anki_becomes_readable_text() {
        assert_eq!(
            html_to_text("What produces the cell&#x27;s <b>ATP</b>?"),
            "What produces the cell's ATP?"
        );
        assert_eq!(
            html_to_text("Mitochondria<br><img src=\"cell.png\">"),
            "Mitochondria\n[image: cell.png]"
        );
        assert_eq!(html_to_text("<div>a</div><div>b&nbsp;c</div>"), "a\nb c");
        assert_eq!(
            html_to_text("say [sound:hund.mp3]"),
            "say [audio: hund.mp3]"
        );
        assert_eq!(html_to_text("<style>.x{}</style>Hi"), "Hi");
        assert_eq!(html_to_text("a < b && c"), "a < b && c");
    }

    #[test]
    fn malformed_markers_are_left_alone() {
        assert_eq!(
            render_cloze("{{cx::a}} and {{c1::b", 1, false),
            "{{cx::a}} and {{c1::b"
        );
    }
}
