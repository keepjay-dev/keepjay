// SPDX-License-Identifier: AGPL-3.0-or-later
//! Built-in card types and their card-generation rules (SRS-002; docs/specs/09 §4).
//!
//! Card types are customisation documents (D-006). User-editable templates arrive with
//! SRS-013 (P1); until then the five built-ins live here, identified by their document ids.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum CardType {
    Basic,
    BasicReverse,
    OptionalReverse,
    Cloze,
    TypeIn,
}

impl CardType {
    pub const ALL: [CardType; 5] = [
        CardType::Basic,
        CardType::BasicReverse,
        CardType::OptionalReverse,
        CardType::Cloze,
        CardType::TypeIn,
    ];

    /// Document id, as stored in `facts.card_type`.
    pub fn id(self) -> &'static str {
        match self {
            CardType::Basic => "builtin:basic",
            CardType::BasicReverse => "builtin:basic-reverse",
            CardType::OptionalReverse => "builtin:optional-reverse",
            CardType::Cloze => "builtin:cloze",
            CardType::TypeIn => "builtin:type-in",
        }
    }

    /// Accepts the document id or its short slug (`basic`, `cloze`, …).
    pub fn parse(s: &str) -> Result<Self> {
        let slug = s.strip_prefix("builtin:").unwrap_or(s);
        CardType::ALL
            .into_iter()
            .find(|t| &t.id()["builtin:".len()..] == slug)
            .ok_or_else(|| {
                let names: Vec<_> = CardType::ALL
                    .iter()
                    .map(|t| &t.id()["builtin:".len()..])
                    .collect();
                Error::validation(format!(
                    "unknown card type '{s}'; built-in types are: {}",
                    names.join(", ")
                ))
            })
    }

    /// Field names in order. The first field is the sort/duplicate-check field.
    pub fn fields(self) -> &'static [&'static str] {
        match self {
            CardType::Basic | CardType::BasicReverse | CardType::TypeIn => &["Front", "Back"],
            CardType::OptionalReverse => &["Front", "Back", "Add Reverse"],
            CardType::Cloze => &["Text", "Extra"],
        }
    }

    /// Validate field values and return the card ordinals this fact generates.
    pub fn card_ords(self, fields: &BTreeMap<String, String>) -> Result<Vec<u32>> {
        for name in fields.keys() {
            if !self.fields().contains(&name.as_str()) {
                return Err(Error::validation(format!(
                    "card type '{}' has no field '{name}' (fields: {})",
                    self.id(),
                    self.fields().join(", ")
                )));
            }
        }
        let get = |name: &str| fields.get(name).map(|v| v.trim()).unwrap_or("");
        let first = self.fields()[0];
        if get(first).is_empty() {
            return Err(Error::validation(format!(
                "field '{first}' must not be empty"
            )));
        }
        Ok(match self {
            CardType::Basic | CardType::TypeIn => vec![0],
            CardType::BasicReverse => vec![0, 1],
            CardType::OptionalReverse if get("Add Reverse").is_empty() => vec![0],
            CardType::OptionalReverse => vec![0, 1],
            CardType::Cloze => {
                let numbers = cloze_numbers(get("Text"));
                if numbers.is_empty() {
                    return Err(Error::validation(
                        "cloze text needs at least one deletion, e.g. {{c1::mitochondria}}",
                    ));
                }
                // Card ordinal = cloze number − 1 (c1 → 0), like Anki.
                numbers.into_iter().map(|n| n - 1).collect()
            }
        })
    }
}

/// Distinct cloze numbers (`{{c1::…}}`, `{{c2::…::hint}}`) in ascending order.
pub fn cloze_numbers(text: &str) -> Vec<u32> {
    let mut found = std::collections::BTreeSet::new();
    let mut rest = text;
    while let Some(start) = rest.find("{{c") {
        let after = &rest[start + 3..];
        let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !digits.is_empty()
            && after[digits.len()..].starts_with("::")
            && let Ok(n) = digits.parse::<u32>()
            && n > 0
        {
            found.insert(n);
        }
        rest = after;
    }
    found.into_iter().collect()
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
    fn generation_rules_per_type() {
        let qa = f(&[("Front", "Q"), ("Back", "A")]);
        assert_eq!(CardType::Basic.card_ords(&qa).unwrap(), vec![0]);
        assert_eq!(CardType::BasicReverse.card_ords(&qa).unwrap(), vec![0, 1]);
        assert_eq!(CardType::TypeIn.card_ords(&qa).unwrap(), vec![0]);
        assert_eq!(CardType::OptionalReverse.card_ords(&qa).unwrap(), vec![0]);
        let with_reverse = f(&[("Front", "Q"), ("Back", "A"), ("Add Reverse", "y")]);
        assert_eq!(
            CardType::OptionalReverse.card_ords(&with_reverse).unwrap(),
            vec![0, 1]
        );
    }

    #[test]
    fn cloze_generates_one_card_per_number() {
        let text = f(&[(
            "Text",
            "{{c1::Mitochondria}} make {{c2::ATP::energy}}; {{c1::they}} have DNA",
        )]);
        assert_eq!(CardType::Cloze.card_ords(&text).unwrap(), vec![0, 1]);
        assert_eq!(
            cloze_numbers("{{c10::x}} {{c3::y}} {{cx::z}} {{c0::w}}"),
            vec![3, 10]
        );
        assert!(
            CardType::Cloze
                .card_ords(&f(&[("Text", "no deletions")]))
                .is_err()
        );
    }

    #[test]
    fn rejects_unknown_fields_and_empty_first_field() {
        assert!(CardType::Basic.card_ords(&f(&[("Nope", "x")])).is_err());
        assert!(
            CardType::Basic
                .card_ords(&f(&[("Front", "  "), ("Back", "A")]))
                .is_err()
        );
    }

    #[test]
    fn parses_ids_and_slugs() {
        assert_eq!(CardType::parse("cloze").unwrap(), CardType::Cloze);
        assert_eq!(
            CardType::parse("builtin:type-in").unwrap(),
            CardType::TypeIn
        );
        let err = CardType::parse("image-occlusion").unwrap_err().to_string();
        assert!(err.contains("basic-reverse"), "{err}");
    }
}
