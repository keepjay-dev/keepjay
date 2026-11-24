// SPDX-License-Identifier: AGPL-3.0-or-later
//! `keepjay review`: full-screen TUI (CLI-005) and `--plain` line mode for screen readers,
//! dumb terminals and scripted input (06 §6.2, §9). `--quick` reviews a few cards and exits
//! (CLI-019); `keepjay while` reuses the TUI while a command runs (CLI-020). `u` undoes the
//! last answer, then the one before it, back to the start of the session (SRS-005, D-041).

use std::fmt;
use std::io::{BufRead, IsTerminal, Write};
use std::process::Child;
use std::time::{Duration, Instant};

use clap::Args;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Padding, Paragraph, Wrap};
use ratatui::{DefaultTerminal, Frame};
use serde::Serialize;
use study_core::{Core, Id, IntervalPreview, QueuedCard, Rating};

use super::Ctx;
use super::card_text::{CardText, Look};
use crate::output::CliError;

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
    /// Review a few due cards, then exit; prints nothing when nothing is due (CLI-019)
    #[arg(long)]
    pub quick: bool,
    /// With --quick: how many cards [default: 5]
    #[arg(
        short = 'n',
        requires = "quick",
        conflicts_with = "limit",
        value_name = "N"
    )]
    count: Option<u32>,
    /// With --quick: stop after this long, e.g. 90s, 2m, 1m30s
    #[arg(long, requires = "quick", value_name = "DURATION", value_parser = parse_duration)]
    time: Option<Duration>,
}

/// Cards per `review --quick` (06 §7 `quick_count`).
const QUICK_COUNT: u32 = 5;

/// `90s`, `2m`, `1m30s`, `1h`; a bare number is minutes.
pub fn parse_duration(s: &str) -> Result<Duration, String> {
    let s = s.trim();
    if let Ok(m) = s.parse::<u64>() {
        return Ok(Duration::from_secs(m * 60));
    }
    let bad = || format!("'{s}' isn't a duration; try 90s, 2m or 1m30s");
    let mut total = 0u64;
    let mut num = String::new();
    for c in s.chars() {
        if c.is_ascii_digit() {
            num.push(c);
            continue;
        }
        let n: u64 = num.parse().map_err(|_| bad())?;
        num.clear();
        total += n * match c {
            's' => 1,
            'm' => 60,
            'h' => 3600,
            _ => return Err(bad()),
        };
    }
    if !num.is_empty() || total == 0 {
        return Err(bad());
    }
    Ok(Duration::from_secs(total))
}

/// A command being watched by `keepjay while`.
pub struct Watch<'a> {
    pub label: String,
    pub child: &'a mut Child,
    pub started: Instant,
}

/// One review session, from `review` flags or from `while`.
pub struct Session<'a> {
    pub deck: Option<String>,
    pub limit: Option<u32>,
    pub deadline: Option<Instant>,
    /// Wait on screen for learning cards due soon (not in quick reviews).
    pub wait_learning: bool,
    pub watch: Option<Watch<'a>>,
}

impl Session<'_> {
    fn from_args(args: &ReviewArgs) -> Self {
        let limit = if args.quick {
            Some(args.count.or(args.limit).unwrap_or(QUICK_COUNT))
        } else {
            args.limit
        };
        Session {
            deck: args.deck.clone(),
            limit,
            deadline: args.time.map(|d| Instant::now() + d),
            wait_learning: !args.quick,
            watch: None,
        }
    }

    fn done(&self, summary: &Summary) -> bool {
        self.limit.is_some_and(|l| summary.reviewed >= l)
            || self.deadline.is_some_and(|d| Instant::now() >= d)
    }
}

/// Why a TUI session ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum End {
    /// No more cards, or the limit or time ran out.
    Done,
    /// `q` or Esc.
    Quit,
    /// Ctrl+C.
    Interrupted,
    /// The watched command finished.
    CommandExited,
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
        *self.count(r) += 1;
    }

    fn unrecord(&mut self, r: Rating) {
        self.reviewed = self.reviewed.saturating_sub(1);
        let n = self.count(r);
        *n = n.saturating_sub(1);
    }

    fn count(&mut self, r: Rating) -> &mut u32 {
        match r {
            Rating::Again => &mut self.again,
            Rating::Hard => &mut self.hard,
            Rating::Good => &mut self.good,
            Rating::Easy => &mut self.easy,
        }
    }
}

/// The session's answers, newest last, so `u` can undo them in turn (D-041).
#[derive(Default)]
struct Answered(Vec<(Id, Id, Rating)>);

impl Answered {
    fn push(&mut self, log: &str, card: Id, rating: Rating) -> Result<(), CliError> {
        self.0.push((log.parse()?, card, rating));
        Ok(())
    }

    fn any(&self) -> bool {
        !self.0.is_empty()
    }

    /// Undo the newest answer; returns its card, ready to show again.
    fn undo(
        &mut self,
        core: &mut Core,
        summary: &mut Summary,
    ) -> Result<Option<QueuedCard>, CliError> {
        let Some((log, card, rating)) = self.0.pop() else {
            return Ok(None);
        };
        core.undo_answer(log)?;
        summary.unrecord(rating);
        Ok(Some(core.queued_card(card)?))
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

/// `keepjay review`.
pub fn command(ctx: &mut Ctx, args: ReviewArgs) -> Result<(), CliError> {
    let ws = ctx.workspace()?;
    let summary = run(ctx.core, ws, &args)?;
    if args.quick && summary.reviewed == 0 && !ctx.out.json {
        return Ok(()); // silent when nothing was due (CLI-019)
    }
    ctx.out.emit(&summary, || summary.to_string())
}

pub fn run(core: &mut Core, ws: Id, args: &ReviewArgs) -> Result<Summary, CliError> {
    let mut session = Session::from_args(args);
    // A quick review with nothing due is silent and fine anywhere, even without a
    // terminal, so it can sit in a shell's startup file (CLI-019).
    if args.quick
        && core
            .next_card(ws, session.deck.as_deref(), now_ms())?
            .is_none()
    {
        return Ok(Summary::default());
    }
    if args.plain {
        plain(core, ws, &session)
    } else if interactive() {
        tui(core, ws, &mut session).map(|(summary, _)| summary)
    } else {
        Err(CliError::Usage(
            "`keepjay review` needs an interactive terminal; use --plain to answer via stdin"
                .into(),
        ))
    }
}

/// Both ends are a terminal that can draw the TUI.
pub fn interactive() -> bool {
    std::io::stdin().is_terminal()
        && std::io::stdout().is_terminal()
        && std::env::var("TERM").as_deref() != Ok("dumb")
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

// ---------------------------------------------------------------- plain mode

fn plain(core: &mut Core, ws: Id, s: &Session) -> Result<Summary, CliError> {
    let stdin = std::io::stdin();
    let mut lines = stdin.lock().lines();
    let mut out = std::io::stdout().lock();
    let mut summary = Summary::default();
    let mut answered = Answered::default();
    let mut again: Option<QueuedCard> = None;
    while again.is_some() || !s.done(&summary) {
        let item = match again.take() {
            Some(item) => item,
            None => match core.next_card(ws, s.deck.as_deref(), now_ms())? {
                Some(item) => item,
                None => {
                    summary.next_learning_due = core.next_learning_due(ws, s.deck.as_deref())?;
                    break;
                }
            },
        };
        let shown = Instant::now();
        writeln!(out, "\n[{}] Q: {}", item.deck_path, item.question)?;
        let undo = if answered.any() {
            ", u to undo the last answer"
        } else {
            ""
        };
        write!(out, "(enter to show answer{undo}, q to quit) ")?;
        out.flush()?;
        match lines.next().transpose()? {
            None => break,
            Some(l) if l.trim().eq_ignore_ascii_case("q") => break,
            Some(l) if l.trim().eq_ignore_ascii_case("u") && answered.any() => {
                again = answered.undo(core, &mut summary)?;
                writeln!(out, "Undone.")?;
                continue;
            }
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
        answered.push(&outcome.review_log_id, card, rating)?;
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
    /// The question and answer, parsed and highlighted once per card.
    question: CardText,
    answer: CardText,
    previews: Vec<IntervalPreview>,
    revealed: bool,
    shown: Instant,
    counts: study_core::DueCounts,
    /// An answer from this session can be undone with `u`.
    can_undo: bool,
}

/// Run a TUI session. The caller's terminal is restored before this returns.
pub fn tui(core: &mut Core, ws: Id, s: &mut Session) -> Result<(Summary, End), CliError> {
    // ratatui::init installs a panic hook that restores the terminal.
    let mut terminal = ratatui::init();
    let result = tui_loop(&mut terminal, core, ws, s);
    ratatui::restore();
    result
}

enum Input {
    Key(KeyEvent),
    CommandExited,
    Tick,
}

/// The next key press, or a tick every 200 ms so the screen and the watched command
/// can be checked.
// @card(vjxwxz3v) Why does the review's `input` ignore key events unless `k.kind == KeyEventKind::Press`?
// On Windows, crossterm also reports key releases, so every key would act twice.
fn input(s: &mut Session) -> Result<Input, CliError> {
    if let Some(w) = &mut s.watch
        && w.child.try_wait()?.is_some()
    {
        return Ok(Input::CommandExited);
    }
    if event::poll(Duration::from_millis(200))?
        && let Event::Key(k) = event::read()?
        && k.kind == KeyEventKind::Press
    // Windows reports releases too
    {
        return Ok(Input::Key(k));
    }
    Ok(Input::Tick)
}

fn quit_key(k: &KeyEvent) -> Option<End> {
    match k.code {
        KeyCode::Char('c') if k.modifiers.contains(KeyModifiers::CONTROL) => Some(End::Interrupted),
        KeyCode::Char('q') | KeyCode::Esc => Some(End::Quit),
        _ => None,
    }
}

fn tui_loop(
    terminal: &mut DefaultTerminal,
    core: &mut Core,
    ws: Id,
    s: &mut Session,
) -> Result<(Summary, End), CliError> {
    let mut summary = Summary::default();
    let mut answered = Answered::default();
    let mut again: Option<QueuedCard> = None;
    let deck = s.deck.clone();
    let deck = deck.as_deref();
    'cards: loop {
        if again.is_none() && s.done(&summary) {
            return Ok((summary, End::Done));
        }
        let next = match again.take() {
            Some(item) => Some(item),
            None => core.next_card(ws, deck, now_ms())?,
        };
        let Some(item) = next else {
            let next = core.next_learning_due(ws, deck)?;
            let soon = next.filter(|due| due - now_ms() <= study_core::srs::LEARNING_WAIT_MS);
            if s.watch.is_some() {
                // Keep the screen up until the command finishes; a learning card may
                // come due in the meantime.
                let msg = match soon {
                    Some(_) => "Next card soon: it's one you're relearning.",
                    None => "Nothing more to review right now.",
                };
                match wait(terminal, s, &summary, soon, msg, answered.any())? {
                    Waited::Next => continue 'cards,
                    Waited::Undo => {
                        again = answered.undo(core, &mut summary)?;
                        continue 'cards;
                    }
                    Waited::End(end) => return Ok((summary, end)),
                }
            }
            match soon {
                Some(due) if s.wait_learning => {
                    let msg = "It's a card you're relearning: waiting improves recall.";
                    match wait(terminal, s, &summary, Some(due), msg, answered.any())? {
                        Waited::Next => continue 'cards,
                        Waited::Undo => {
                            again = answered.undo(core, &mut summary)?;
                            continue 'cards;
                        }
                        Waited::End(end) => return Ok((summary, end)),
                    }
                }
                _ => {
                    summary.next_learning_due = next;
                    return Ok((summary, End::Done));
                }
            }
        };
        let card: Id = item.card.id.parse()?;
        let mut screen = Screen {
            previews: core.preview_intervals(card, now_ms())?,
            counts: core.queue_counts(ws, deck, now_ms())?,
            question: CardText::new(&item.question),
            answer: CardText::new(&item.answer),
            item,
            revealed: false,
            shown: Instant::now(),
            can_undo: answered.any(),
        };
        loop {
            terminal.draw(|f| draw(f, &screen, s, &summary))?;
            let key = match input(s)? {
                Input::CommandExited => return Ok((summary, End::CommandExited)),
                Input::Tick => continue,
                Input::Key(k) => k,
            };
            if let Some(end) = quit_key(&key) {
                return Ok((summary, end));
            }
            match key.code {
                KeyCode::Char(' ') | KeyCode::Enter if !screen.revealed => screen.revealed = true,
                KeyCode::Char(c @ '1'..='4') if screen.revealed => {
                    let rating = Rating::from_number(c as u8 - b'0')?;
                    let elapsed = screen.shown.elapsed().as_millis() as i64;
                    let outcome = core.answer_card(card, rating, elapsed, now_ms())?;
                    answered.push(&outcome.review_log_id, card, rating)?;
                    summary.record(rating);
                    continue 'cards;
                }
                KeyCode::Char('u') if answered.any() => {
                    again = answered.undo(core, &mut summary)?;
                    continue 'cards;
                }
                _ => {}
            }
        }
    }
}

/// What happened while [`wait`]ing.
enum Waited {
    /// It's time for the next card.
    Next,
    /// `u`: undo the last answer.
    Undo,
    /// The session ends.
    End(End),
}

/// Show `msg` until `until` (UTC ms) passes, if given, or a key ends the wait.
fn wait(
    terminal: &mut DefaultTerminal,
    s: &mut Session,
    summary: &Summary,
    until: Option<i64>,
    msg: &str,
    can_undo: bool,
) -> Result<Waited, CliError> {
    loop {
        let mut lines = vec![Line::from(msg.to_string())];
        if let Some(at) = until {
            let left = at - now_ms();
            if left <= 0 {
                return Ok(Waited::Next);
            }
            lines.push(Line::from(format!(
                "Next card in {}s.",
                (left + 999) / 1000
            )));
        }
        terminal.draw(|f| {
            let [header, body, footer] = frame(f);
            draw_header(f, header, s, None, summary);
            f.render_widget(
                Paragraph::new(lines.clone())
                    .wrap(Wrap { trim: true })
                    .block(card_block()),
                body,
            );
            let hint = if can_undo { "u undo" } else { "" };
            draw_footer(f, footer, s, Line::from(hint));
        })?;
        match input(s)? {
            Input::CommandExited => return Ok(Waited::End(End::CommandExited)),
            Input::Key(k) => {
                if let Some(end) = quit_key(&k) {
                    return Ok(Waited::End(end));
                }
                if can_undo && k.code == KeyCode::Char('u') {
                    return Ok(Waited::Undo);
                }
            }
            Input::Tick => {}
        }
    }
}

/// The widest the review gets, borders included: 76 characters of text, so lines stay
/// readable on a wide terminal. Narrower terminals use their full width (CLI-005).
const MAX_WIDTH: u16 = 80;

fn frame(f: &Frame) -> [Rect; 3] {
    layout(f.area())
}

/// Header, card and footer in one column, centred when the terminal is wider than
/// [`MAX_WIDTH`]. Text inside stays left-aligned so code keeps its indentation.
fn layout(area: Rect) -> [Rect; 3] {
    let [column] = Layout::horizontal([Constraint::Max(MAX_WIDTH)])
        .flex(Flex::Center)
        .areas(area);
    Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(3),
        Constraint::Length(2),
    ])
    .areas(column)
}

fn card_block() -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .padding(Padding::horizontal(1))
}

/// `mm:ss`, or `h:mm:ss` past an hour.
pub fn fmt_clock(d: Duration) -> String {
    let t = d.as_secs();
    if t >= 3600 {
        format!("{}:{:02}:{:02}", t / 3600, t / 60 % 60, t % 60)
    } else {
        format!("{:02}:{:02}", t / 60, t % 60)
    }
}

fn draw_header(
    f: &mut Frame,
    area: Rect,
    s: &Session,
    counts: Option<&study_core::DueCounts>,
    summary: &Summary,
) {
    let title = match &s.watch {
        Some(w) => format!(" Review · while {} ", w.label),
        None => format!(" Review · {} ", s.deck.as_deref().unwrap_or("All decks")),
    };
    let mut right = match counts {
        Some(c) => format!(
            "{} learning · {} due · {} new · {} done",
            c.learning, c.review, c.new, summary.reviewed
        ),
        None => format!("{} done", summary.reviewed),
    };
    if let Some(w) = &s.watch {
        right.push_str(&format!(" · running {}", fmt_clock(w.started.elapsed())));
    }
    right.push(' ');
    let gap = (area.width as usize).saturating_sub(title.chars().count() + right.chars().count());
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::from(title).bold(),
            Span::from(" ".repeat(gap)),
            Span::from(right).dim(),
        ])),
        area,
    );
}

fn draw_footer(f: &mut Frame, area: Rect, s: &Session, hint: Line) {
    let quit = match &s.watch {
        Some(w) => format!(
            "q stop reviewing ({} keeps running) · ctrl+c stop both",
            w.label
        ),
        None => "q quit".to_string(),
    };
    f.render_widget(Paragraph::new(vec![hint, Line::from(quit).dim()]), area);
}

fn draw(f: &mut Frame, screen: &Screen, s: &Session, summary: &Summary) {
    let [header, body, footer] = frame(f);
    draw_header(f, header, s, Some(&screen.counts), summary);

    // Inside the border (2) and the padding (2).
    let width = body.width.saturating_sub(4) as usize;
    let bold = Style::default().add_modifier(Modifier::BOLD);
    let mut text = vec![
        Line::from(Span::from(screen.item.deck_path.clone()).dim()),
        Line::from(""),
    ];
    text.extend(screen.question.lines(width, Look::new(bold)));
    if screen.revealed {
        text.push(Line::from(""));
        text.push(Line::from("─".repeat(width)).dim());
        text.push(Line::from(""));
        let plain = Look::new(Style::default());
        text.extend(screen.answer.lines(width, plain));
    }
    f.render_widget(
        Paragraph::new(text)
            .wrap(Wrap { trim: false })
            .block(card_block()),
        body,
    );

    let undo = if screen.can_undo { "   u undo" } else { "" };
    let hint = if screen.revealed {
        Line::from(vec![
            Span::from(buttons(&screen.previews)).bold(),
            Span::from(undo).dim(),
        ])
    } else {
        Line::from(format!("space reveal{undo}"))
    };
    draw_footer(f, footer, s, hint);
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use ratatui::layout::Rect;

    use super::{MAX_WIDTH, fmt_clock, fmt_interval, layout, parse_duration};

    #[test]
    fn durations() {
        assert_eq!(parse_duration("90s"), Ok(Duration::from_secs(90)));
        assert_eq!(parse_duration("2m"), Ok(Duration::from_secs(120)));
        assert_eq!(parse_duration("1m30s"), Ok(Duration::from_secs(90)));
        assert_eq!(parse_duration("1h"), Ok(Duration::from_secs(3600)));
        assert_eq!(parse_duration("3"), Ok(Duration::from_secs(180)));
        for bad in ["", "0s", "m", "5x", "1m30"] {
            assert!(parse_duration(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn review_column_is_capped_and_centred() {
        // A wide terminal: an 80-column column in the middle.
        let [header, body, footer] = layout(Rect::new(0, 0, 200, 40));
        for r in [header, body, footer] {
            assert_eq!((r.x, r.width), (60, MAX_WIDTH));
        }
        assert_eq!((header.height, body.height, footer.height), (1, 37, 2));

        // A narrow terminal: the full width, as before.
        let [header, body, _] = layout(Rect::new(0, 0, 60, 20));
        assert_eq!((header.x, header.width, body.width), (0, 60, 60));
    }

    #[test]
    fn clock() {
        assert_eq!(fmt_clock(Duration::from_secs(41)), "00:41");
        assert_eq!(fmt_clock(Duration::from_secs(3725)), "1:02:05");
    }

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
