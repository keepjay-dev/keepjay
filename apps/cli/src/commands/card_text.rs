// SPDX-License-Identifier: AGPL-3.0-or-later
//! Card text drawn for the review TUI (SRS-008, CODE-001), from the core's
//! [`study_core::cards::rich`] model and [`study_core::cards::highlight`] tokens.
//!
//! - Prose keeps its line breaks and wraps to the column; bold and italic are real.
//! - Inline code is coloured, and keeps dimmed backticks so it isn't shown by colour alone
//!   (NFR-A11Y-004).
//! - Code blocks sit behind a `│` bar under a dim language label, with syntax colours from
//!   the terminal's own palette, so they suit light and dark themes. They're wrapped here,
//!   not by the paragraph, so every piece of a long line keeps the bar.
//! - `--no-color` or `NO_COLOR` turns the colour off; comments stay dim (NFR-A11Y-006).

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use study_core::cards::highlight::{self, Kind, Token};
use study_core::cards::rich::{self, Block};

const BAR: &str = "│ ";
const BAR_WIDTH: usize = 2;

/// How to draw one card's text.
#[derive(Debug, Clone, Copy)]
pub struct Look {
    /// Style for prose (the question is bold).
    pub prose: Style,
    /// Whether colour may be used.
    pub color: bool,
}

impl Look {
    pub fn new(prose: Style) -> Self {
        Look {
            prose,
            color: crate::output::color_enabled(),
        }
    }

    fn inline_code(&self) -> Style {
        self.token(Kind::Type)
    }

    fn token(&self, kind: Kind) -> Style {
        let style = Style::default();
        if kind == Kind::Comment {
            return style.add_modifier(Modifier::DIM);
        }
        if !self.color {
            return style;
        }
        match kind {
            Kind::Plain | Kind::Comment => style,
            Kind::Keyword => style.fg(Color::Magenta),
            Kind::String => style.fg(Color::Green),
            Kind::Constant => style.fg(Color::Yellow),
            Kind::Type => style.fg(Color::Cyan),
            Kind::Function => style.fg(Color::Blue),
        }
    }
}

/// A card's question or answer, parsed and highlighted once; drawn on every frame.
pub struct CardText {
    blocks: Vec<Block>,
    /// Tokens for each code block, in order; `None` for an unknown language.
    tokens: Vec<Option<Vec<Vec<Token>>>>,
}

impl CardText {
    pub fn new(text: &str) -> Self {
        let blocks = rich::parse(text);
        let tokens = blocks
            .iter()
            .filter_map(|b| match b {
                Block::Code { lang, lines } => Some(
                    lang.as_deref()
                        .and_then(|lang| highlight::highlight(lang, lines)),
                ),
                Block::Text(_) => None,
            })
            .collect();
        CardText { blocks, tokens }
    }

    /// The text as lines that fit `width` columns once wrapped.
    pub fn lines(&self, width: usize, look: Look) -> Vec<Line<'static>> {
        let dim = Style::default().add_modifier(Modifier::DIM);
        let mut out = Vec::new();
        let mut tokens = self.tokens.iter();
        for (i, block) in self.blocks.iter().enumerate() {
            if i > 0 {
                out.push(Line::from(""));
            }
            match block {
                Block::Text(text_lines) => {
                    for spans in text_lines {
                        let mut line = Vec::new();
                        for s in spans {
                            push_span(&mut line, s, look);
                        }
                        out.push(Line::from(line));
                    }
                }
                Block::Code { lang, lines } => {
                    if let Some(lang) = lang {
                        out.push(Line::from(Span::styled(lang.clone(), dim)));
                    }
                    let highlighted = tokens.next().and_then(Option::as_ref);
                    let room = width.saturating_sub(BAR_WIDTH).max(1);
                    for (n, code) in lines.iter().enumerate() {
                        let runs: Vec<(String, Style)> = match highlighted.and_then(|h| h.get(n)) {
                            Some(line) => line
                                .iter()
                                .map(|t| (t.text.clone(), look.token(t.kind)))
                                .collect(),
                            None => vec![(code.clone(), look.token(Kind::Plain))],
                        };
                        wrap_code(&mut out, &runs, room, dim);
                    }
                }
            }
        }
        out
    }
}

/// One line of code as bar-prefixed lines at most `room` characters wide.
fn wrap_code(out: &mut Vec<Line<'static>>, runs: &[(String, Style)], room: usize, dim: Style) {
    let mut line = vec![Span::styled(BAR, dim)];
    let mut used = 0;
    for (text, style) in runs {
        let mut piece = String::new();
        for c in text.replace('\t', "    ").chars() {
            if used == room {
                if !piece.is_empty() {
                    line.push(Span::styled(std::mem::take(&mut piece), *style));
                }
                out.push(Line::from(std::mem::replace(
                    &mut line,
                    vec![Span::styled(BAR, dim)],
                )));
                used = 0;
            }
            piece.push(c);
            used += 1;
        }
        if !piece.is_empty() {
            line.push(Span::styled(piece, *style));
        }
    }
    out.push(Line::from(line));
}

fn push_span(line: &mut Vec<Span<'static>>, s: &rich::Span, look: Look) {
    let mut style = look.prose;
    if s.style.bold {
        style = style.add_modifier(Modifier::BOLD);
    }
    if s.style.italic {
        style = style.add_modifier(Modifier::ITALIC);
    }
    if s.style.dim {
        style = style.add_modifier(Modifier::DIM);
    }
    if s.style.code {
        let tick = style.add_modifier(Modifier::DIM);
        line.push(Span::styled("`", tick));
        line.push(Span::styled(
            s.text.clone(),
            style.patch(look.inline_code()),
        ));
        line.push(Span::styled("`", tick));
    } else {
        line.push(Span::styled(s.text.clone(), style));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn look(color: bool) -> Look {
        Look {
            prose: Style::default(),
            color,
        }
    }

    fn lines(text: &str, width: usize, look: Look) -> Vec<Line<'static>> {
        CardText::new(text).lines(width, look)
    }

    fn plain(lines: &[Line]) -> Vec<String> {
        lines.iter().map(|l| l.to_string()).collect()
    }

    fn fg_of(line: &Line, text: &str) -> Option<Color> {
        line.spans
            .iter()
            .find(|s| s.content.contains(text))
            .unwrap_or_else(|| panic!("{text} not in {line:?}"))
            .style
            .fg
    }

    #[test]
    fn code_block_gets_a_label_and_a_bar_on_every_piece() {
        let text = "Why?\n```rust\nlet x = 1;\n\n0123456789abc\n```";
        let out = lines(text, 12, look(true));
        assert_eq!(
            plain(&out),
            vec![
                "Why?",
                "",
                "rust",
                "│ let x = 1;",
                "│ ",
                "│ 0123456789",
                "│ abc",
            ]
        );
        assert_eq!(out[3].spans[0].style.fg, None, "the bar isn't coloured");
        // No piece is wider than the column, so the paragraph never re-wraps code.
        assert!(out.iter().all(|l| l.width() <= 12));
    }

    #[test]
    fn known_languages_are_coloured_from_the_palette() {
        let out = lines("```rust\nfn f() -> u32 { 42 } // c\n```", 80, look(true));
        let code = &out[1];
        assert_eq!(fg_of(code, "fn"), Some(Color::Magenta));
        assert_eq!(fg_of(code, "42"), Some(Color::Yellow));
        let comment = code
            .spans
            .iter()
            .find(|s| s.content.contains("// c"))
            .unwrap();
        assert!(comment.style.add_modifier.contains(Modifier::DIM));
    }

    #[test]
    fn colours_survive_wrapping() {
        // "let" is split by a 2-column room; both halves keep the keyword colour.
        let out = lines("```rust\nlet\n```", 4, look(true));
        assert_eq!(plain(&out), vec!["rust", "│ le", "│ t"]);
        assert_eq!(fg_of(&out[1], "le"), Some(Color::Magenta));
        assert_eq!(fg_of(&out[2], "t"), Some(Color::Magenta));
    }

    #[test]
    fn unknown_or_missing_language_is_plain() {
        for text in ["```\nlet x\n```", "```klingon\nlet x\n```"] {
            let out = lines(text, 80, look(true));
            let code = out.last().unwrap();
            assert!(code.spans.iter().all(|s| s.style.fg.is_none()), "{text}");
        }
    }

    #[test]
    fn inline_code_keeps_dim_backticks() {
        let out = lines("Use `Arc::clone` here", 80, look(true));
        assert_eq!(plain(&out), vec!["Use `Arc::clone` here"]);
        let spans = &out[0].spans;
        assert!(spans[1].style.add_modifier.contains(Modifier::DIM));
        assert_eq!(spans[2].style.fg, Some(Color::Cyan));
    }

    #[test]
    fn no_color_draws_without_colour_but_comments_stay_dim() {
        let out = lines("`x`\n```rust\nfn y() {} // c\n```", 80, look(false));
        assert!(
            out.iter()
                .flat_map(|l| &l.spans)
                .all(|s| s.style.fg.is_none())
        );
        let comment = out
            .last()
            .unwrap()
            .spans
            .iter()
            .find(|s| s.content.contains("// c"));
        assert!(comment.unwrap().style.add_modifier.contains(Modifier::DIM));
    }

    #[test]
    fn question_style_applies_to_prose_not_code_blocks() {
        let bold = Look {
            prose: Style::default().add_modifier(Modifier::BOLD),
            color: true,
        };
        let out = lines("Question?\n```\ncode\n```", 80, bold);
        assert!(out[0].spans[0].style.add_modifier.contains(Modifier::BOLD));
        assert!(!out[2].spans[1].style.add_modifier.contains(Modifier::BOLD));
    }
}
