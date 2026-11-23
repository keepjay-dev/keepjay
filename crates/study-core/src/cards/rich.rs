// SPDX-License-Identifier: AGPL-3.0-or-later
//! Card text as styled blocks for review screens (SRS-008, CODE-001).
//!
//! Card fields are Markdown. [`parse`] turns rendered question or answer text into a small
//! model that any surface can draw: prose lines made of styled spans, and code blocks that
//! keep their whitespace exactly and carry their language. The TUI draws it today; the
//! VS Code and mobile review will share it (08 §4).
//!
//! Line breaks inside a paragraph are kept, unlike in standard Markdown: answers written as
//! comment lines, and text imported from Anki, rely on them.

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

/// How a run of prose is styled.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Style {
    pub bold: bool,
    pub italic: bool,
    /// Inline code (`like_this`), without its backticks.
    pub code: bool,
    /// Secondary text added for the reader, such as a link's address.
    pub dim: bool,
}

/// A run of text in one style.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub text: String,
    pub style: Style,
}

/// A part of the card, drawn with a blank line between parts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    /// Prose: lines of styled spans, line breaks as written.
    Text(Vec<Vec<Span>>),
    /// A code block, one string per line, whitespace exactly as written (CODE-001).
    Code {
        /// The first word of a fence's info string (```` ```rust ````), if any.
        lang: Option<String>,
        lines: Vec<String>,
    },
}

/// Parse card text (CommonMark, without extensions) into blocks.
pub fn parse(text: &str) -> Vec<Block> {
    let mut b = Builder::default();
    for event in Parser::new_ext(text, Options::empty()) {
        b.event(event);
    }
    b.end_block();
    b.blocks
}

#[derive(Default)]
struct Builder {
    blocks: Vec<Block>,
    lines: Vec<Vec<Span>>,
    line: Vec<Span>,
    bold: u32,
    italic: u32,
    quote: u32,
    /// One entry per open list: the next item's number, or `None` for bullets.
    lists: Vec<Option<u64>>,
    /// Address and text so far of the open link.
    link: Option<(String, String)>,
    /// Inside an image, whose alt text is replaced by a placeholder.
    image: bool,
    code: Option<(Option<String>, String)>,
}

impl Builder {
    fn event(&mut self, event: Event) {
        if let Some((_, buf)) = &mut self.code {
            match event {
                Event::Text(t) => buf.push_str(&t),
                Event::End(TagEnd::CodeBlock) => {
                    let (lang, buf) = self.code.take().expect("inside a code block");
                    let lines = buf.lines().map(str::to_string).collect();
                    self.blocks.push(Block::Code { lang, lines });
                }
                _ => {}
            }
            return;
        }
        match event {
            Event::Start(tag) => self.start(tag),
            Event::End(tag) => self.end(tag),
            Event::Text(t) if self.image => drop(t),
            Event::Text(t) => {
                if let Some((_, seen)) = &mut self.link {
                    seen.push_str(&t);
                }
                self.push(&t, self.style());
            }
            Event::Code(t) => {
                let style = Style {
                    code: true,
                    ..self.style()
                };
                self.push(&t, style);
            }
            Event::Html(t) | Event::InlineHtml(t) => {
                // Shown as written: card text may hold `Vec<T>` or Anki leftovers.
                let mut parts = t.split('\n').peekable();
                while let Some(part) = parts.next() {
                    self.push(part, self.style());
                    if parts.peek().is_some() {
                        self.end_line();
                    }
                }
            }
            Event::SoftBreak | Event::HardBreak => self.end_line(),
            Event::Rule => {
                self.end_block();
                self.push(
                    "───",
                    Style {
                        dim: true,
                        ..Style::default()
                    },
                );
                self.end_block();
            }
            _ => {}
        }
    }

    fn start(&mut self, tag: Tag) {
        match tag {
            Tag::Heading { .. } => {
                self.end_block();
                self.bold += 1;
            }
            Tag::BlockQuote(_) => {
                self.end_block();
                self.quote += 1;
            }
            Tag::CodeBlock(kind) => {
                self.end_block();
                let lang = match kind {
                    CodeBlockKind::Fenced(info) => info
                        .split_whitespace()
                        .next()
                        .map(|l| l.trim_matches(|c| c == '{' || c == '}').to_string())
                        .filter(|l| !l.is_empty()),
                    CodeBlockKind::Indented => None,
                };
                self.code = Some((lang, String::new()));
            }
            Tag::List(start) => {
                if self.lists.is_empty() {
                    self.end_block();
                } else {
                    self.end_line();
                }
                self.lists.push(start);
            }
            Tag::Item => {
                self.end_line();
                let depth = self.lists.len().saturating_sub(1);
                let marker = match self.lists.last_mut() {
                    Some(Some(n)) => {
                        *n += 1;
                        format!("{}. ", *n - 1)
                    }
                    _ => "• ".to_string(),
                };
                let text = format!("{}{marker}", "  ".repeat(depth));
                self.push(&text, Style::default());
            }
            Tag::Emphasis => self.italic += 1,
            Tag::Strong => self.bold += 1,
            Tag::Link { dest_url, .. } => self.link = Some((dest_url.to_string(), String::new())),
            Tag::Image { dest_url, .. } => {
                self.image = true;
                let text = format!("[image: {dest_url}]");
                self.push(
                    &text,
                    Style {
                        dim: true,
                        ..Style::default()
                    },
                );
            }
            _ => {}
        }
    }

    fn end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph if !self.lists.is_empty() => self.end_line(),
            TagEnd::Paragraph => self.end_block(),
            TagEnd::Heading(_) => {
                self.bold = self.bold.saturating_sub(1);
                self.end_block();
            }
            TagEnd::BlockQuote(_) => {
                self.end_block();
                self.quote = self.quote.saturating_sub(1);
            }
            TagEnd::List(_) => {
                self.lists.pop();
                if self.lists.is_empty() {
                    self.end_block();
                }
            }
            TagEnd::Item => self.end_line(),
            TagEnd::Emphasis => self.italic = self.italic.saturating_sub(1),
            TagEnd::Strong => self.bold = self.bold.saturating_sub(1),
            TagEnd::Link => {
                if let Some((url, seen)) = self.link.take()
                    && !url.is_empty()
                    && url != seen
                {
                    let text = format!(" ({url})");
                    self.push(
                        &text,
                        Style {
                            dim: true,
                            ..Style::default()
                        },
                    );
                }
            }
            TagEnd::Image => self.image = false,
            _ => {}
        }
    }

    fn style(&self) -> Style {
        Style {
            bold: self.bold > 0,
            italic: self.italic > 0,
            ..Style::default()
        }
    }

    fn push(&mut self, text: &str, style: Style) {
        if text.is_empty() {
            return;
        }
        if self.line.is_empty() && self.quote > 0 {
            self.line.push(Span {
                text: "> ".repeat(self.quote as usize),
                style: Style {
                    dim: true,
                    ..Style::default()
                },
            });
        }
        match self.line.last_mut() {
            Some(last) if last.style == style => last.text.push_str(text),
            _ => self.line.push(Span {
                text: text.to_string(),
                style,
            }),
        }
    }

    fn end_line(&mut self) {
        if !self.line.is_empty() {
            self.lines.push(std::mem::take(&mut self.line));
        }
    }

    fn end_block(&mut self) {
        self.end_line();
        if !self.lines.is_empty() {
            self.blocks
                .push(Block::Text(std::mem::take(&mut self.lines)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain(text: &str) -> Span {
        Span {
            text: text.into(),
            style: Style::default(),
        }
    }

    fn styled(text: &str, f: impl FnOnce(&mut Style)) -> Span {
        let mut style = Style::default();
        f(&mut style);
        Span {
            text: text.into(),
            style,
        }
    }

    fn text(lines: &[&[Span]]) -> Block {
        Block::Text(lines.iter().map(|l| l.to_vec()).collect())
    }

    #[test]
    fn plain_text_keeps_its_line_breaks() {
        assert_eq!(
            parse("first line\nsecond line\n\nnew paragraph"),
            vec![
                text(&[&[plain("first line")], &[plain("second line")]]),
                text(&[&[plain("new paragraph")]]),
            ]
        );
    }

    #[test]
    fn fenced_code_keeps_whitespace_and_language() {
        let src = "What's wrong?\n```rust\nfn main() {\n    let x = 1;\n\n}\n```\nAfter.";
        assert_eq!(
            parse(src),
            vec![
                text(&[&[plain("What's wrong?")]]),
                Block::Code {
                    lang: Some("rust".into()),
                    lines: vec![
                        "fn main() {".into(),
                        "    let x = 1;".into(),
                        "".into(),
                        "}".into()
                    ],
                },
                text(&[&[plain("After.")]]),
            ]
        );
    }

    #[test]
    fn fence_without_language_and_indented_code() {
        assert_eq!(
            parse("```\nx\n```"),
            vec![Block::Code {
                lang: None,
                lines: vec!["x".into()]
            }]
        );
        assert_eq!(
            parse("Run:\n\n    cargo test\n"),
            vec![
                text(&[&[plain("Run:")]]),
                Block::Code {
                    lang: None,
                    lines: vec!["cargo test".into()]
                },
            ]
        );
    }

    #[test]
    fn inline_code_bold_and_italic() {
        assert_eq!(
            parse("Use `Arc::clone`, **not** *deep* copies"),
            vec![text(&[&[
                plain("Use "),
                styled("Arc::clone", |s| s.code = true),
                plain(", "),
                styled("not", |s| s.bold = true),
                plain(" "),
                styled("deep", |s| s.italic = true),
                plain(" copies"),
            ]])]
        );
    }

    #[test]
    fn snake_case_and_generics_stay_as_written() {
        assert_eq!(
            parse("a snake_case_name returns Vec<T> & 2 * 3"),
            vec![text(&[&[plain(
                "a snake_case_name returns Vec<T> & 2 * 3"
            )]])]
        );
    }

    #[test]
    fn cloze_placeholders_stay_as_written() {
        assert_eq!(
            parse("A nil map can be [...] but not written."),
            vec![text(&[&[plain("A nil map can be [...] but not written.")]])]
        );
    }

    #[test]
    fn lists_get_markers() {
        assert_eq!(
            parse("- one\n- two\n  - nested\n\n3. three\n4. four"),
            vec![
                text(&[&[plain("• one")], &[plain("• two")], &[plain("  • nested")],]),
                text(&[&[plain("3. three")], &[plain("4. four")]]),
            ]
        );
    }

    #[test]
    fn links_show_their_address_once() {
        assert_eq!(
            parse("See [the docs](https://example.com) or <https://example.org>"),
            vec![text(&[&[
                plain("See the docs"),
                styled(" (https://example.com)", |s| s.dim = true),
                plain(" or https://example.org"),
            ]])]
        );
    }

    #[test]
    fn headings_are_bold_and_quotes_marked() {
        assert_eq!(
            parse("# Title\n> quoted"),
            vec![
                text(&[&[styled("Title", |s| s.bold = true)]]),
                text(&[&[styled("> ", |s| s.dim = true), plain("quoted")]]),
            ]
        );
    }

    #[test]
    fn empty_text_has_no_blocks() {
        assert_eq!(parse(""), vec![]);
        assert_eq!(parse("\n\n"), vec![]);
    }
}
