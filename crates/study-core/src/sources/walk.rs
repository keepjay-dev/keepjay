// SPDX-License-Identifier: AGPL-3.0-or-later
//! Walking a source for cards (SRC-002, SRC-003, SRC-008; docs/specs/13-code-sources.md).
//!
//! Reads only files under the source root. Respects `.gitignore` (also outside Git work
//! trees), `.git/info/exclude`, the user's global Git ignore file and `.keepjayignore`
//! (or the pre-rename `.scrubjayignore`, D-037). Skips hidden files and folders except
//! `.keepjay/` (and `.scrubjay/`), binary files, files over
//! [`MAX_FILE_BYTES`] and anything that isn't UTF-8. Symlinks aren't followed. Nothing
//! is executed.

use std::path::Path;
use std::sync::Mutex;

use ignore::WalkState;

use serde::Serialize;

use super::parse::{ParseWarning, ParsedCard};
use super::{comments, markdown};
use crate::error::{Error, Result};

/// Files larger than this are skipped (SRC-008).
pub const MAX_FILE_BYTES: u64 = 1024 * 1024;

/// The folder (inside a source) whose Markdown files are all card files.
pub const CARDS_DIR: &str = ".keepjay/cards";
/// Names from before the rename to Keepjay (D-037), still read so that existing cards
/// aren't retired by a scan after upgrading.
const LEGACY_CARDS_DIR: &str = ".scrubjay/cards";
const IGNORE_FILES: [&str; 2] = [".keepjayignore", ".scrubjayignore"];
const TOOL_DIRS: [&str; 2] = [".keepjay", ".scrubjay"];

/// Line-comment prefix by file extension (13 §3).
pub fn comment_prefix(extension: &str) -> Option<&'static str> {
    Some(match extension.to_ascii_lowercase().as_str() {
        "rs" | "c" | "h" | "cc" | "cpp" | "cxx" | "hpp" | "hh" | "cs" | "go" | "java" | "js"
        | "mjs" | "cjs" | "jsx" | "ts" | "mts" | "cts" | "tsx" | "kt" | "kts" | "swift"
        | "scala" | "dart" | "php" | "zig" => "//",
        "py" | "rb" | "sh" | "bash" | "zsh" | "fish" | "yaml" | "yml" | "toml" | "r" | "ex"
        | "exs" | "pl" | "nix" | "jl" => "#",
        "sql" | "hs" | "lua" | "adb" | "ads" | "elm" => "--",
        "lisp" | "el" | "clj" | "cljs" | "cljc" | "scm" | "ss" | "asm" => ";",
        "tex" | "sty" | "erl" | "hrl" => "%",
        _ => return None,
    })
}

/// How a file is read for cards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileKind {
    Comments(&'static str),
    Markdown,
}

fn classify(rel: &str) -> Option<FileKind> {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    let in_cards_dir = [CARDS_DIR, LEGACY_CARDS_DIR]
        .iter()
        .any(|dir| rel.starts_with(&format!("{dir}/")));
    if name.ends_with(".cards.md") || (in_cards_dir && name.ends_with(".md")) {
        return Some(FileKind::Markdown);
    }
    let ext = name.rsplit_once('.')?.1;
    comment_prefix(ext).map(FileKind::Comments)
}

/// Everything found in one walk of a source.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Found {
    pub cards: Vec<ParsedCard>,
    pub warnings: Vec<ParseWarning>,
    /// Files with a known comment syntax, or Markdown card files.
    pub files_considered: u64,
    /// Files actually parsed (comment files that mention `@card`, and all card files).
    pub files_parsed: u64,
    pub skipped_large: u64,
    pub skipped_binary: u64,
}

/// What reading one file gave.
enum Outcome {
    Large,
    Binary,
    /// A comment file that doesn't mention `@card`.
    NoCards,
    Unreadable(String),
    Parsed(super::parse::Parsed),
}

/// Find all cards under `root`. Files are read in parallel; results are put in path
/// order, so a scan's output doesn't depend on thread timing.
pub fn find_cards(root: &Path) -> Result<Found> {
    let mut builder = ignore::WalkBuilder::new(root);
    builder
        .hidden(false)
        .git_ignore(true)
        .git_exclude(true)
        .git_global(true)
        .require_git(false)
        .follow_links(false);
    for name in IGNORE_FILES {
        builder.add_custom_ignore_filename(name);
    }
    let walker = builder
        .filter_entry(|entry| {
            let name = entry.file_name().to_string_lossy();
            !(name.starts_with('.') && !TOOL_DIRS.contains(&name.as_ref()) && entry.depth() > 0)
        })
        .build_parallel();

    let results: Mutex<Vec<(String, Outcome)>> = Mutex::new(Vec::new());
    let walk_error: Mutex<Option<String>> = Mutex::new(None);
    walker.run(|| {
        let results = &results;
        let walk_error = &walk_error;
        Box::new(move |entry| {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    walk_error
                        .lock()
                        .expect("not poisoned")
                        .get_or_insert(e.to_string());
                    return WalkState::Quit;
                }
            };
            if !entry.file_type().is_some_and(|t| t.is_file()) {
                return WalkState::Continue;
            }
            let Ok(rel) = entry.path().strip_prefix(root) else {
                return WalkState::Continue;
            };
            let rel = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            let Some(kind) = classify(&rel) else {
                return WalkState::Continue;
            };
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            let outcome = read_one(entry.path(), &rel, kind, size);
            results.lock().expect("not poisoned").push((rel, outcome));
            WalkState::Continue
        })
    });
    if let Some(e) = walk_error.into_inner().expect("not poisoned") {
        return Err(Error::Io(std::io::Error::other(e)));
    }

    let mut results = results.into_inner().expect("not poisoned");
    results.sort_by(|a, b| a.0.cmp(&b.0));
    let mut found = Found::default();
    for (rel, outcome) in results {
        found.files_considered += 1;
        match outcome {
            Outcome::Large => found.skipped_large += 1,
            Outcome::Binary => found.skipped_binary += 1,
            Outcome::NoCards => {}
            Outcome::Unreadable(message) => found.warnings.push(ParseWarning {
                path: rel,
                line: 0,
                message: format!("couldn't read the file: {message}"),
            }),
            Outcome::Parsed(parsed) => {
                found.files_parsed += 1;
                found.cards.extend(parsed.cards);
                found.warnings.extend(parsed.warnings);
            }
        }
    }
    Ok(found)
}

/// Parse one file of a source (`rel` relative to `root`), as a scan would.
pub(crate) fn parse_file(root: &Path, rel: &str) -> Result<super::parse::Parsed> {
    let Some(kind) = classify(rel) else {
        return Err(Error::validation(format!(
            "{rel} isn't a file keepjay reads cards from"
        )));
    };
    let path = root.join(rel);
    let size = std::fs::metadata(&path)?.len();
    match read_one(&path, rel, kind, size) {
        Outcome::Parsed(p) => Ok(p),
        Outcome::NoCards => Ok(super::parse::Parsed::default()),
        Outcome::Large => Err(Error::validation(format!("{rel} is over 1 MiB"))),
        Outcome::Binary => Err(Error::validation(format!("{rel} isn't UTF-8 text"))),
        Outcome::Unreadable(e) => Err(Error::Io(std::io::Error::other(e))),
    }
}

fn read_one(path: &Path, rel: &str, kind: FileKind, size: u64) -> Outcome {
    if size > MAX_FILE_BYTES {
        return Outcome::Large;
    }
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) => return Outcome::Unreadable(e.to_string()),
    };
    if bytes.iter().take(8192).any(|&b| b == 0) {
        return Outcome::Binary;
    }
    let Ok(text) = String::from_utf8(bytes) else {
        return Outcome::Binary;
    };
    Outcome::Parsed(match kind {
        FileKind::Comments(prefix) => {
            if !text.contains("@card") {
                return Outcome::NoCards;
            }
            comments::parse(rel, &text, prefix)
        }
        FileKind::Markdown => markdown::parse(rel, &text),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classification() {
        assert_eq!(classify("src/lib.rs"), Some(FileKind::Comments("//")));
        assert_eq!(classify("a/b.PY"), Some(FileKind::Comments("#")));
        assert_eq!(classify("notes/async.cards.md"), Some(FileKind::Markdown));
        assert_eq!(
            classify(".keepjay/cards/agent.md"),
            Some(FileKind::Markdown)
        );
        assert_eq!(
            classify(".scrubjay/cards/agent.md"),
            Some(FileKind::Markdown),
            "the pre-rename folder is still read (D-037)"
        );
        assert_eq!(
            classify("README.md"),
            None,
            "ordinary Markdown isn't a card file"
        );
        assert_eq!(classify("Makefile"), None);
        assert_eq!(classify("image.png"), None);
    }
}
