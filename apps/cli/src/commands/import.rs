// SPDX-License-Identifier: AGPL-3.0-or-later
//! `keepjay import anki` (IO-001).

use std::path::PathBuf;

use clap::Subcommand;

use super::Ctx;
use crate::output::CliError;

#[derive(Subcommand)]
pub enum ImportCmd {
    /// Import an Anki package (.apkg / .colpkg) with its review history
    Anki {
        /// Path to the .apkg or .colpkg file
        file: PathBuf,
        /// Put imported decks under this deck, e.g. --into "From Anki"
        #[arg(long, value_name = "DECK")]
        into: Option<String>,
    },
}

pub fn run(ctx: &mut Ctx, cmd: ImportCmd) -> Result<(), CliError> {
    let ImportCmd::Anki { file, into } = cmd;
    let ws = ctx.workspace_for_new()?;
    let options = study_core::AnkiImportOptions { deck_prefix: into };
    let s = ctx.core.import_anki(ws, &file, &options)?;
    ctx.out.emit(&s, || report(&s))
}

fn report(s: &study_core::AnkiImportSummary) -> String {
    let mut lines = vec![format!(
        "Imported {} notes → {} cards with {} past reviews ({} Anki format).",
        s.notes_imported, s.cards_imported, s.review_logs_imported, s.format
    )];
    if s.notes_already_imported > 0 {
        lines.push(format!(
            "  Skipped {} notes already imported earlier.",
            s.notes_already_imported
        ));
    }
    if !s.decks_created.is_empty() {
        lines.push(format!("  Decks: {}", s.decks_created.join(", ")));
    }
    for t in &s.note_types {
        let plural = if t.notes == 1 { "" } else { "s" };
        lines.push(format!(
            "  {} ({} note{plural}) → {}",
            t.name, t.notes, t.imported_as
        ));
    }
    if s.suspended > 0 {
        lines.push(format!("  {} cards stay suspended.", s.suspended));
    }
    for (reason, n) in &s.skipped_reasons {
        lines.push(format!("  Skipped {n} notes: {reason}."));
    }
    if s.cards_reset_without_history > 0 {
        lines.push(format!(
            "  {} studied cards had no review history and start as new.",
            s.cards_reset_without_history
        ));
    }
    if s.notes_tagged > 0 {
        lines.push(format!(
            "  Tags on {} notes ({} new tags).",
            s.notes_tagged, s.tags_created
        ));
    }
    if s.tags_skipped > 0 {
        lines.push(format!(
            "  Skipped {} tags that aren't valid here.",
            s.tags_skipped
        ));
    }
    if s.media_files > 0 {
        let plural = if s.media_added == 1 { "" } else { "s" };
        let mut media = format!("  Media: {} file{plural} added", s.media_added);
        if s.media_already_present > 0 {
            media += &format!(", {} already here", s.media_already_present);
        }
        if s.media_renamed > 0 {
            media += &format!(
                ", {} renamed because the name was taken by another file",
                s.media_renamed
            );
        }
        lines.push(media + ".");
    }
    if s.media_skipped > 0 {
        lines.push(format!(
            "  Skipped {} media files (unsafe name, too large or damaged).",
            s.media_skipped
        ));
    }
    if s.media_missing > 0 {
        lines.push(format!(
            "  {} files the cards refer to aren't in the package (Anki: Tools → Check Media).",
            s.media_missing
        ));
    }
    lines.push("Next: keepjay review".into());
    lines.join("\n")
}
