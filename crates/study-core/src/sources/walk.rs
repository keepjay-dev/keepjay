// SPDX-License-Identifier: AGPL-3.0-or-later
//! Walking a source for cards (SRC-002, SRC-003, SRC-008; docs/specs/13-code-sources.md).
//!
//! Reads only files under the source root. Respects `.gitignore` (also outside Git work
//! trees), `.git/info/exclude`, the user's global Git ignore file and `.scrubjayignore`.
//! Skips hidden files and folders except `.scrubjay/`, binary files, files over
//! [`MAX_FILE_BYTES`] and anything that isn't UTF-8. Symlinks aren't followed. Nothing
//! is executed.

use std::path::Path;

use serde::Serialize;

use super::parse::{ParseWarning, ParsedCard};
use super::{comments, markdown};
use crate::error::{Error, Result};

/// Files larger than this are skipped (SRC-008).
pub const MAX_FILE_BYTES: u64 = 1024 * 1024;

/// The folder (inside a source) whose Markdown files are all card files.
pub const CARDS_DIR: &str = ".scrubjay/cards";

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
    let in_cards_dir = rel.starts_with(&format!("{CARDS_DIR}/"));
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

/// Find all cards under `root`.
pub fn find_cards(root: &Path) -> Result<Found> {
    let mut found = Found::default();
    let walker = ignore::WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .git_exclude(true)
        .git_global(true)
        .require_git(false)
        .follow_links(false)
        .add_custom_ignore_filename(".scrubjayignore")
        .filter_entry(|entry| {
            let name = entry.file_name().to_string_lossy();
            !(name.starts_with('.') && name != ".scrubjay" && entry.depth() > 0)
        })
        .sort_by_file_path(|a, b| a.cmp(b))
        .build();

    for entry in walker {
        let entry = entry.map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?;
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        let Ok(rel) = entry.path().strip_prefix(root) else {
            continue;
        };
        let rel = rel
            .components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        let Some(kind) = classify(&rel) else {
            continue;
        };
        found.files_considered += 1;
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        if size > MAX_FILE_BYTES {
            found.skipped_large += 1;
            continue;
        }
        let bytes = std::fs::read(entry.path())?;
        if bytes.iter().take(8192).any(|&b| b == 0) {
            found.skipped_binary += 1;
            continue;
        }
        let Ok(text) = String::from_utf8(bytes) else {
            found.skipped_binary += 1;
            continue;
        };
        let parsed = match kind {
            FileKind::Comments(prefix) => {
                if !text.contains("@card") {
                    continue;
                }
                comments::parse(&rel, &text, prefix)
            }
            FileKind::Markdown => markdown::parse(&rel, &text),
        };
        found.files_parsed += 1;
        found.cards.extend(parsed.cards);
        found.warnings.extend(parsed.warnings);
    }
    Ok(found)
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
            classify(".scrubjay/cards/agent.md"),
            Some(FileKind::Markdown)
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
