// SPDX-License-Identifier: AGPL-3.0-or-later
//! Card text drawn for the review TUI (SRS-008, CODE-001), from the core's
//! [`study_core::cards::rich`] model.
//!
//! - Prose keeps its line breaks and wraps to the column; bold and italic are real.
//! - Inline code is coloured, and keeps dimmed backticks so it isn't shown by colour alone
//!   (NFR-A11Y-004).
//! - Code blocks sit behind a `│` bar under a dim language label. They're wrapped here,
//!   not by the paragraph, so every piece of a long line keeps the bar.
//! - `NO_COLOR` turns the colour off (NFR-A11Y-006).

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
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
        let color = std::env::var_os("NO_COLOR").is_none_or(|v| v.is_empty());
        Look { prose, color }
    }

    fn code(&self) -> Style {
        if self.color {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        }
    }
}

/// `text` as lines that fit `width` columns once wrapped.
pub fn lines(text: &str, width: usize, look: Look) -> Vec<Line<'static>> {
    let dim = Style::default().add_modifier(Modifier::DIM);
    let mut out = Vec::new();
    for (i, block) in rich::parse(text).into_iter().enumerate() {
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
                    out.push(Line::from(Span::styled(lang, dim)));
                }
                let room = width.saturating_sub(BAR_WIDTH).max(1);
                for code in lines {
                    let chars: Vec<char> = code.replace('\t', "    ").chars().collect();
                    if chars.is_empty() {
                        out.push(Line::from(Span::styled(BAR, dim)));
                    }
                    for piece in chars.chunks(room) {
                        out.push(Line::from(vec![
                            Span::styled(BAR, dim),
                            Span::styled(piece.iter().collect::<String>(), look.code()),
                        ]));
                    }
                }
            }
        }
    }
    out
}

fn push_span(line: &mut Vec<Span<'static>>, s: rich::Span, look: Look) {
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
        line.push(Span::styled(s.text, style.patch(look.code())));
        line.push(Span::styled("`", tick));
    } else {
        line.push(Span::styled(s.text, style));
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

    fn plain(lines: &[Line]) -> Vec<String> {
        lines.iter().map(|l| l.to_string()).collect()
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
        // The code is coloured; the bar is not.
        assert_eq!(out[3].spans[1].style.fg, Some(Color::Cyan));
        assert_eq!(out[3].spans[0].style.fg, None);
        // No piece is wider than the column, so the paragraph never re-wraps code.
        assert!(out.iter().all(|l| l.width() <= 12));
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
    fn no_color_draws_without_colour() {
        let out = lines("`x`\n```\ny\n```", 80, look(false));
        assert!(
            out.iter()
                .flat_map(|l| &l.spans)
                .all(|s| s.style.fg.is_none())
        );
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
