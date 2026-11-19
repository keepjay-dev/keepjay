// SPDX-License-Identifier: AGPL-3.0-or-later
//! Mapping Anki note types onto Addisco's built-in card types (IO-001, SRS-002).
//!
//! Detection is structural (field and template shape), not by name: stock note type names
//! are localised. Anything that doesn't match a built-in exactly is imported card by card,
//! rendered through its own Anki template into a Basic card, so nothing studyable is lost
//! until editable card types arrive (SRS-013).

use std::collections::{BTreeMap, HashMap};

use super::collection::NoteType;
use crate::cards::CardType;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Mapping {
    /// Fields map onto a built-in card type; cloze keeps its text field index.
    Builtin {
        card_type: CardType,
        cloze_text: usize,
    },
    /// Each Anki card becomes its own Basic fact from the rendered template.
    Rendered,
    Skip(&'static str),
}

impl Mapping {
    pub fn label(&self) -> String {
        match self {
            Mapping::Builtin { card_type, .. } => {
                card_type.id().trim_start_matches("builtin:").to_string()
            }
            Mapping::Rendered => "basic (rendered from the Anki template)".into(),
            Mapping::Skip(why) => format!("skipped: {why}"),
        }
    }
}

/// Does the template use `field` (plain or through a filter like `type:` or `text:`)?
fn uses(template: &str, field: &str) -> bool {
    template.contains(&format!("{{{{{field}}}}}")) || template.contains(&format!(":{field}}}}}"))
}

pub(crate) fn map(t: &NoteType) -> Mapping {
    let all = t.templates.iter().flat_map(|(q, a)| [q, a]);
    if all.clone().any(|s| s.contains("image-occlusion"))
        || (t.cloze && t.fields.first().map(String::as_str) == Some("Occlusion"))
    {
        return Mapping::Skip("image occlusion isn't supported yet (SRS-015)");
    }
    if t.fields.is_empty() || t.templates.is_empty() {
        return Mapping::Skip("note type has no fields or templates");
    }
    if t.cloze {
        let q = &t.templates[0].0;
        let text = t
            .fields
            .iter()
            .position(|f| q.contains(&format!("{{{{cloze:{f}}}}}")))
            .unwrap_or(0);
        return Mapping::Builtin {
            card_type: CardType::Cloze,
            cloze_text: text,
        };
    }
    let f = &t.fields;
    let tpl = &t.templates;
    let card_type = match (f.len(), tpl.len()) {
        (2, 1)
            if tpl[0].0.contains(&format!("{{{{type:{}}}}}", f[1])) && uses(&tpl[0].0, &f[0]) =>
        {
            Some(CardType::TypeIn)
        }
        (2, 1) if uses(&tpl[0].0, &f[0]) && !uses(&tpl[0].0, &f[1]) && uses(&tpl[0].1, &f[1]) => {
            Some(CardType::Basic)
        }
        (2, 2)
            if uses(&tpl[0].0, &f[0])
                && !uses(&tpl[0].0, &f[1])
                && uses(&tpl[1].0, &f[1])
                && !uses(&tpl[1].0, &f[0]) =>
        {
            Some(CardType::BasicReverse)
        }
        (3, 2)
            if uses(&tpl[0].0, &f[0])
                && uses(&tpl[1].0, &f[1])
                && tpl[1].0.contains(&format!("{{{{#{}}}}}", f[2])) =>
        {
            Some(CardType::OptionalReverse)
        }
        _ => None,
    };
    match card_type {
        Some(card_type) => Mapping::Builtin {
            card_type,
            cloze_text: 0,
        },
        None => Mapping::Rendered,
    }
}

/// Field values for a built-in card type. HTML is kept as Anki stored it (lossless).
pub(crate) fn builtin_fields(
    card_type: CardType,
    cloze_text: usize,
    values: &[String],
) -> BTreeMap<String, String> {
    let get = |i: usize| values.get(i).cloned().unwrap_or_default();
    let mut out = BTreeMap::new();
    match card_type {
        CardType::Cloze => {
            out.insert("Text".into(), get(cloze_text));
            let extra: Vec<String> = values
                .iter()
                .enumerate()
                .filter(|(i, v)| *i != cloze_text && !v.trim().is_empty())
                .map(|(_, v)| v.clone())
                .collect();
            if !extra.is_empty() {
                out.insert("Extra".into(), extra.join("<br><br>"));
            }
        }
        _ => {
            for (i, name) in card_type.fields().iter().enumerate() {
                let v = get(i);
                if !v.is_empty() {
                    out.insert((*name).to_string(), v);
                }
            }
        }
    }
    out
}

/// Render an Anki template: `{{#F}}…{{/F}}` and `{{^F}}…{{/F}}` sections, `{{F}}` with
/// optional filters (`text:F`, `type:F`, `hint:F`), and `{{FrontSide}}`. Unknown keys
/// (`{{Tags}}`, `{{Deck}}`, …) render empty.
pub(crate) fn render(template: &str, fields: &HashMap<&str, &str>, front_side: &str) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let Some(end) = after.find("}}") else {
            out.push_str(&rest[start..]);
            return out;
        };
        let tag = after[..end].trim();
        rest = &after[end + 2..];
        if let Some(name) = tag.strip_prefix('#').or_else(|| tag.strip_prefix('^')) {
            let close = format!("{{{{/{name}}}}}");
            let Some(close_at) = rest.find(&close) else {
                continue;
            };
            let inner = &rest[..close_at];
            rest = &rest[close_at + close.len()..];
            let present = fields.get(name).is_some_and(|v| !v.trim().is_empty());
            if present == tag.starts_with('#') {
                out.push_str(&render(inner, fields, front_side));
            }
        } else if tag == "FrontSide" {
            out.push_str(front_side);
        } else if !tag.starts_with('/') {
            let name = tag.rsplit(':').next().unwrap_or(tag);
            out.push_str(fields.get(name).copied().unwrap_or(""));
        }
    }
    out.push_str(rest);
    out
}

/// Question and answer for one card of a rendered note type. The answer drops the
/// repeated front side and Anki's `<hr id=answer>` divider: Addisco shows them separately.
pub(crate) fn render_card(t: &NoteType, ord: u32, values: &[String]) -> Option<(String, String)> {
    let (q, a) = t.templates.get(ord as usize)?;
    let fields: HashMap<&str, &str> = t
        .fields
        .iter()
        .map(String::as_str)
        .zip(values.iter().map(String::as_str))
        .collect();
    let question = render(q, &fields, "");
    let mut answer = render(a, &fields, "");
    for divider in ["<hr id=answer>", "<hr id=\"answer\">"] {
        answer = answer.replace(divider, "");
    }
    Some((question.trim().to_string(), answer.trim().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nt(cloze: bool, fields: &[&str], templates: &[(&str, &str)]) -> NoteType {
        NoteType {
            name: "t".into(),
            cloze,
            fields: fields.iter().map(|s| s.to_string()).collect(),
            templates: templates
                .iter()
                .map(|(q, a)| (q.to_string(), a.to_string()))
                .collect(),
        }
    }

    const ANS: &str = "{{FrontSide}}\n\n<hr id=answer>\n\n{{Back}}";

    #[test]
    fn detects_stock_types_by_shape() {
        let basic = nt(false, &["Front", "Back"], &[("{{Front}}", ANS)]);
        assert_eq!(map(&basic).label(), "basic");
        let reverse = nt(
            false,
            &["Front", "Back"],
            &[
                ("{{Front}}", ANS),
                ("{{Back}}", "{{FrontSide}}<hr id=answer>{{Front}}"),
            ],
        );
        assert_eq!(map(&reverse).label(), "basic-reverse");
        let typed = nt(
            false,
            &["Front", "Back"],
            &[(
                "{{Front}}\n\n{{type:Back}}",
                "{{Front}}<hr id=answer>{{type:Back}}",
            )],
        );
        assert_eq!(map(&typed).label(), "type-in");
        let optional = nt(
            false,
            &["Front", "Back", "Add Reverse"],
            &[
                ("{{Front}}", ANS),
                ("{{#Add Reverse}}{{Back}}{{/Add Reverse}}", "x"),
            ],
        );
        assert_eq!(map(&optional).label(), "optional-reverse");
        let cloze = nt(
            true,
            &["Text", "Back Extra"],
            &[("{{cloze:Text}}", "{{cloze:Text}}")],
        );
        assert_eq!(map(&cloze).label(), "cloze");
    }

    #[test]
    fn localised_names_and_swapped_templates() {
        // German stock names map by shape...
        let einfach = nt(
            false,
            &["Vorderseite", "Rückseite"],
            &[("{{Vorderseite}}", "{{Rückseite}}")],
        );
        assert_eq!(map(&einfach).label(), "basic");
        // ...but a template that shows the second field first is rendered, not mis-mapped.
        let swapped = nt(false, &["Front", "Back"], &[("{{Back}}", "{{Front}}")]);
        assert_eq!(map(&swapped), Mapping::Rendered);
        let io = nt(
            true,
            &["Occlusion", "Image"],
            &[("{{cloze:Occlusion}}", "x")],
        );
        assert!(matches!(map(&io), Mapping::Skip(_)));
    }

    #[test]
    fn renders_sections_filters_and_front_side() {
        let t = nt(
            false,
            &["Word", "Meaning", "Example"],
            &[(
                "<div>{{Word}}</div>",
                "{{FrontSide}}<hr id=answer>{{Meaning}}{{#Example}}<br><i>{{Example}}</i>{{/Example}}{{^Example}}(no example){{/Example}}",
            )],
        );
        let full = render_card(
            &t,
            0,
            &["Hund".into(), "dog".into(), "Der Hund bellt.".into()],
        )
        .unwrap();
        assert_eq!(full.0, "<div>Hund</div>");
        assert_eq!(full.1, "dog<br><i>Der Hund bellt.</i>");
        let sparse = render_card(&t, 0, &["Katze".into(), "cat".into(), String::new()]).unwrap();
        assert_eq!(sparse.1, "cat(no example)");
        assert!(render_card(&t, 3, &[]).is_none());
    }

    #[test]
    fn cloze_extra_collects_other_fields() {
        let f = builtin_fields(
            CardType::Cloze,
            0,
            &[
                "{{c1::a}}".into(),
                "extra".into(),
                String::new(),
                "more".into(),
            ],
        );
        assert_eq!(f["Text"], "{{c1::a}}");
        assert_eq!(f["Extra"], "extra<br><br>more");
    }
}
