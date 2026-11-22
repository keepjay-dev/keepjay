// SPDX-License-Identifier: AGPL-3.0-or-later
//! `scrubjay review`: full-screen TUI (CLI-005) and `--plain` line mode for screen readers,
//! dumb terminals and scripted input (06 §6.2, §9).

use std::fmt;
use std::io::{BufRead, IsTerminal, Write};
use std::time::Instant;

use clap::Args;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::{DefaultTerminal, Frame};
use serde::Serialize;
use study_core::{Core, Id, IntervalPreview, QueuedCard, Rating};

use crate::CliError;

#[derive(Args)]
pub struct ReviewArgs {
    /// Only this deck and its subdecks
    #[arg(short, long)]
    deck: Option<String>,
    /// Stop after this many cards
    #[arg(long)]
    limit: Option<u32>,
    /// Line mode: no full-screen UI; reads answers from stdin
    #[arg(long)]
    plain: bool,
}

#[derive(Debug, Default, Serialize)]
pub struct Summary {
    /// When the next learning card is due, if one is waiting (UTC ms).
    pub next_learning_due: Option<i64>,
    pub reviewed: u32,
    pub again: u32,
    pub hard: u32,
    pub good: u32,
    pub easy: u32,
}

impl Summary {
    fn record(&mut self, r: Rating) {
        self.reviewed += 1;
        match r {
            Rating::Again => self.again += 1,
            Rating::Hard => self.hard += 1,
            Rating::Good => self.good += 1,
            Rating::Easy => self.easy += 1,
        }
    }
}

impl fmt::Display for Summary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.reviewed == 0 {
            write!(f, "Nothing to review right now.")?;
        } else {
            write!(
                f,
                "Reviewed {} card{}: {} again, {} hard, {} good, {} easy.",
                self.reviewed,
                if self.reviewed == 1 { "" } else { "s" },
                self.again,
                self.hard,
                self.good,
                self.easy
            )?;
        }
        if let Some(due) = self.next_learning_due {
            let wait = (due - now_ms()).max(0) as f64 / 86_400_000.0;
            write!(f, " Next learning card due in {}.", fmt_interval(wait))?;
        }
        Ok(())
    }
}

pub fn run(core: &mut Core, ws: Id, args: &ReviewArgs) -> Result<Summary, CliError> {
    let tty = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
    if args.plain {
        plain(core, ws, args)
    } else if tty && std::env::var("TERM").as_deref() != Ok("dumb") {
        tui(core, ws, args)
    } else {
        Err(CliError::Usage(
            "`scrubjay review` needs an interactive terminal; use --plain to answer via stdin"
                .into(),
        ))
    }
}

pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Compact interval label: 1m, 6h, 4d, 2.5mo, 1.2y.
pub fn fmt_interval(days: f64) -> String {
    let minutes = days * 1440.0;
    if minutes < 60.0 {
        format!("{}m", minutes.round().max(1.0))
    } else if days < 1.0 {
        format!("{}h", (minutes / 60.0).round())
    } else if days < 30.0 {
        format!("{}d", days.round())
    } else if days < 365.0 {
        format!("{:.1}mo", days / 30.0)
    } else {
        format!("{:.1}y", days / 365.0)
    }
}

fn buttons(previews: &[IntervalPreview]) -> String {
    previews
        .iter()
        .map(|p| {
            format!(
                "[{}] {:?} {}",
                p.rating as u8,
                p.rating,
                fmt_interval(p.interval_days)
            )
        })
        .collect::<Vec<_>>()
        .join("   ")
}

fn limit_reached(args: &ReviewArgs, summary: &Summary) -> bool {
    args.limit.is_some_and(|l| summary.reviewed >= l)
}

// ---------------------------------------------------------------- plain mode

fn plain(core: &mut Core, ws: Id, args: &ReviewArgs) -> Result<Summary, CliError> {
    let stdin = std::io::stdin();
    let mut lines = stdin.lock().lines();
    let mut out = std::io::stdout().lock();
    let mut summary = Summary::default();
    while !limit_reached(args, &summary) {
        let Some(item) = core.next_card(ws, args.deck.as_deref(), now_ms())? else {
            summary.next_learning_due = core.next_learning_due(ws, args.deck.as_deref())?;
            break;
        };
        let shown = Instant::now();
        writeln!(out, "\n[{}] Q: {}", item.deck_path, item.question)?;
        write!(out, "(enter to show answer, q to quit) ")?;
        out.flush()?;
        match lines.next().transpose()? {
            None => break,
            Some(l) if l.trim().eq_ignore_ascii_case("q") => break,
            Some(_) => {}
        }
        writeln!(out, "A: {}", item.answer)?;
        let card: Id = item.card.id.parse()?;
        let previews = core.preview_intervals(card, now_ms())?;
        let rating = loop {
            write!(out, "{}   q quit\n> ", buttons(&previews))?;
            out.flush()?;
            let Some(line) = lines.next() else {
                return Ok(summary);
            };
            let line = line?;
            if line.trim().eq_ignore_ascii_case("q") {
                return Ok(summary);
            }
            match Rating::parse(&line) {
                Ok(r) => break r,
                Err(_) => writeln!(out, "Type 1–4 (again, hard, good, easy).")?,
            }
        };
        let elapsed = shown.elapsed().as_millis() as i64;
        let outcome = core.answer_card(card, rating, elapsed, now_ms())?;
        summary.record(rating);
        writeln!(
            out,
            "→ {:?}: next in {}",
            rating,
            fmt_interval(outcome.interval_days)
        )?;
    }
    Ok(summary)
}

// ---------------------------------------------------------------- TUI

struct Screen {
    item: QueuedCard,
    previews: Vec<IntervalPreview>,
    revealed: bool,
    shown: Instant,
    counts: study_core::DueCounts,
}

fn tui(core: &mut Core, ws: Id, args: &ReviewArgs) -> Result<Summary, CliError> {
    // ratatui::init installs a panic hook that restores the terminal.
    let mut terminal = ratatui::init();
    let result = tui_loop(&mut terminal, core, ws, args);
    ratatui::restore();
    result
}

fn tui_loop(
    terminal: &mut DefaultTerminal,
    core: &mut Core,
    ws: Id,
    args: &ReviewArgs,
) -> Result<Summary, CliError> {
    let mut summary = Summary::default();
    let deck = args.deck.as_deref();
    'cards: while !limit_reached(args, &summary) {
        let Some(item) = core.next_card(ws, deck, now_ms())? else {
            // Wait for a learning card due soon instead of ending the session.
            match core.next_learning_due(ws, deck)? {
                Some(due) if due - now_ms() <= study_core::srs::LEARNING_WAIT_MS => {
                    if wait_for(terminal, due)? {
                        continue 'cards;
                    }
                    break;
                }
                other => {
                    summary.next_learning_due = other;
                    break;
                }
            }
        };
        let card: Id = item.card.id.parse()?;
        let mut screen = Screen {
            previews: core.preview_intervals(card, now_ms())?,
            counts: core.queue_counts(ws, deck, now_ms())?,
            item,
            revealed: false,
            shown: Instant::now(),
        };
        loop {
            terminal.draw(|f| draw(f, &screen, deck, &summary))?;
            let Event::Key(key) = event::read()? else {
                continue;
            };
            if key.kind != KeyEventKind::Press {
                continue; // Windows reports releases too
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break 'cards,
                KeyCode::Char(' ') | KeyCode::Enter if !screen.revealed => screen.revealed = true,
                KeyCode::Char(c @ '1'..='4') if screen.revealed => {
                    let rating = Rating::from_number(c as u8 - b'0')?;
                    let elapsed = screen.shown.elapsed().as_millis() as i64;
                    core.answer_card(card, rating, elapsed, now_ms())?;
                    summary.record(rating);
                    continue 'cards;
                }
                _ => {}
            }
        }
    }
    Ok(summary)
}

/// Show a countdown until `due`. Returns `false` if the user quits.
fn wait_for(terminal: &mut DefaultTerminal, due: i64) -> Result<bool, CliError> {
    loop {
        let left = due - now_ms();
        if left <= 0 {
            return Ok(true);
        }
        terminal.draw(|f| {
            let msg = format!(
                "Next card in {}s. It's a card you're relearning: waiting improves recall.",
                (left + 999) / 1000
            );
            f.render_widget(
                Paragraph::new(vec![Line::from(msg), Line::from("q quit").dim()])
                    .wrap(Wrap { trim: true })
                    .block(Block::default().borders(Borders::ALL)),
                f.area(),
            );
        })?;
        if event::poll(std::time::Duration::from_millis(250))?
            && let Event::Key(k) = event::read()?
            && k.kind == KeyEventKind::Press
            && matches!(k.code, KeyCode::Char('q') | KeyCode::Esc)
        {
            return Ok(false);
        }
    }
}

fn draw(f: &mut Frame, s: &Screen, deck: Option<&str>, summary: &Summary) {
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(3),
        Constraint::Length(2),
    ])
    .areas(f.area());

    let title = format!(" Review · {} ", deck.unwrap_or("All decks"));
    let counts = format!(
        "{} learning · {} due · {} new · {} done ",
        s.counts.learning, s.counts.review, s.counts.new, summary.reviewed
    );
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::from(title).bold(),
            Span::from(
                " ".repeat(
                    (header.width as usize)
                        .saturating_sub(deck.unwrap_or("All decks").len() + counts.len() + 11),
                ),
            ),
            Span::from(counts).dim(),
        ])),
        header,
    );

    let mut text = vec![
        Line::from(Span::from(s.item.deck_path.clone()).dim()),
        Line::from(""),
        Line::from(s.item.question.clone()).bold(),
    ];
    if s.revealed {
        text.push(Line::from(""));
        text.push(Line::from("─".repeat(body.width.saturating_sub(4) as usize)).dim());
        text.push(Line::from(""));
        text.extend(s.item.answer.lines().map(|l| Line::from(l.to_string())));
    }
    f.render_widget(
        Paragraph::new(text)
            .wrap(Wrap { trim: false })
            .block(Block::default().borders(Borders::ALL)),
        body,
    );

    let hint = if s.revealed {
        Line::from(buttons(&s.previews)).style(Style::default().add_modifier(Modifier::BOLD))
    } else {
        Line::from("space reveal")
    };
    f.render_widget(
        Paragraph::new(vec![hint, Line::from("q quit").dim()]),
        footer,
    );
}

#[cfg(test)]
mod tests {
    use super::fmt_interval;

    #[test]
    fn interval_labels() {
        assert_eq!(fmt_interval(0.0001), "1m");
        assert_eq!(fmt_interval(10.0 / 1440.0), "10m");
        assert_eq!(fmt_interval(0.25), "6h");
        assert_eq!(fmt_interval(4.4), "4d");
        assert_eq!(fmt_interval(75.0), "2.5mo");
        assert_eq!(fmt_interval(730.0), "2.0y");
    }
}
