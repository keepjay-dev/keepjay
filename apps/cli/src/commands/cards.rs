// SPDX-License-Identifier: AGPL-3.0-or-later
//! `keepjay deck list|create`, `keepjay add card` and `keepjay tag list`.

use std::collections::BTreeMap;

use clap::{Args, Subcommand};
use study_core::{CardType, NewFact};

use super::Ctx;
use crate::output::CliError;

#[derive(Subcommand)]
pub enum DeckCmd {
    /// List decks with their card counts
    List,
    /// Create a deck; `Parent::Child` creates missing parents too
    Create { path: String },
}

#[derive(Subcommand)]
pub enum AddCmd {
    /// Add a card (a fact and the cards its card type generates)
    Card(AddCardArgs),
}

#[derive(Subcommand)]
pub enum TagCmd {
    /// List tags with how many facts carry each
    List,
}

#[derive(Args)]
pub struct AddCardArgs {
    /// First field (Front, or Text for cloze)
    first: Option<String>,
    /// Second field (Back, or Extra for cloze)
    second: Option<String>,
    /// Deck path, e.g. "Biology::Cells"
    #[arg(short, long)]
    deck: String,
    /// Card type: basic, basic-reverse, optional-reverse, cloze, type-in
    #[arg(short = 't', long = "type", default_value = "basic")]
    card_type: String,
    /// Set any field by name, e.g. --field "Add Reverse=y" (repeatable)
    #[arg(long = "field", value_name = "NAME=VALUE")]
    fields: Vec<String>,
    /// Add even if an identical first field exists in the deck
    #[arg(long)]
    allow_duplicate: bool,
    /// Tag the card, e.g. --tag bio::cells (repeatable)
    #[arg(long = "tag", value_name = "TAG")]
    tags: Vec<String>,
}

pub fn deck(ctx: &mut Ctx, cmd: DeckCmd) -> Result<(), CliError> {
    match cmd {
        DeckCmd::List => {
            let ws = ctx.workspace()?;
            let decks = ctx.core.list_decks(ws)?;
            ctx.out.emit(&decks, || {
                if decks.is_empty() {
                    return "No decks yet. Create one with `keepjay deck create <name>`."
                        .to_string();
                }
                decks
                    .iter()
                    .map(|d| {
                        let depth = d.path.matches("::").count();
                        format!(
                            "{}  {}{}  ({} cards)",
                            d.short_id,
                            "  ".repeat(depth),
                            d.name,
                            d.cards
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            })
        }
        DeckCmd::Create { path } => {
            let ws = ctx.workspace_for_new()?;
            let deck = ctx.core.create_deck(ws, &path)?;
            ctx.out.emit(&deck, || {
                format!("Created deck '{}' ({})", deck.path, deck.short_id)
            })
        }
    }
}

pub fn add(ctx: &mut Ctx, cmd: AddCmd) -> Result<(), CliError> {
    let AddCmd::Card(args) = cmd;
    let ws = ctx.workspace_for_new()?;
    let new = new_fact(args)?;
    let deck = new.deck.clone();
    let fact = ctx.core.add_fact(ws, new)?;
    ctx.out.emit(&fact, || {
        let n = fact.cards.len();
        let ids: Vec<_> = fact.cards.iter().map(|c| c.short_id.as_str()).collect();
        let tags = if fact.tags.is_empty() {
            String::new()
        } else {
            format!(" (tags: {})", fact.tags.join(", "))
        };
        format!(
            "Added {n} card{} to {deck}: {}{tags}",
            if n == 1 { "" } else { "s" },
            ids.join(", ")
        )
    })
}

pub fn tag(ctx: &mut Ctx, cmd: TagCmd) -> Result<(), CliError> {
    let TagCmd::List = cmd;
    let ws = ctx.workspace()?;
    let tags = ctx.core.list_tags(ws)?;
    ctx.out.emit(&tags, || {
        if tags.is_empty() {
            return "No tags yet. Add one with `keepjay add card … --tag <tag>`.".to_string();
        }
        let width = tags
            .iter()
            .map(|t| t.path.chars().count())
            .max()
            .unwrap_or(0);
        tags.iter()
            .map(|t| {
                let plural = if t.facts == 1 { "" } else { "s" };
                format!("{:width$}  {} fact{plural}", t.path, t.facts)
            })
            .collect::<Vec<_>>()
            .join("\n")
    })
}

/// Map positional values and `--field` flags onto the card type's fields.
fn new_fact(args: AddCardArgs) -> Result<NewFact, CliError> {
    let card_type = CardType::parse(&args.card_type)?;
    let names = card_type.fields();
    let mut fields = BTreeMap::new();
    for (name, value) in names.iter().zip([args.first, args.second]) {
        if let Some(v) = value {
            fields.insert(name.to_string(), v);
        }
    }
    for pair in args.fields {
        let (name, value) = pair
            .split_once('=')
            .ok_or_else(|| CliError::Usage(format!("--field expects NAME=VALUE, got '{pair}'")))?;
        fields.insert(name.trim().to_string(), value.to_string());
    }
    Ok(NewFact {
        deck: args.deck,
        card_type,
        fields,
        allow_duplicate: args.allow_duplicate,
        tags: args.tags,
    })
}
