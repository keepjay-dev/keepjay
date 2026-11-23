// SPDX-License-Identifier: AGPL-3.0-or-later
//! Writing card ids back into source files (SRC-005; docs/specs/13-code-sources.md §6).
//!
//! - Comment cards: `@card` becomes `@card(k22k1m8e)` on the card's own line. No line is
//!   added, so the anchored code and other cards' line numbers don't move.
//! - Markdown cards: a `<!-- id: k22k1m8e -->` line is added after the card.
//!
//! Each file is rewritten atomically (temporary file + rename) with its line endings and
//! permissions kept. A line that no longer looks as expected (the file changed during the
//! scan) is skipped with a warning; the next scan matches the card by its text instead.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;

use super::parse::{CardStyle, ParseWarning, ParsedCard};

/// One id to write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IdWrite {
    pub path: String,
    pub line: u32,
    pub markdown: bool,
    pub id: String,
    /// The id currently in the file, when it's being replaced (duplicates, collisions).
    pub replace: Option<String>,
}

impl IdWrite {
    pub(crate) fn new(card: &ParsedCard, id: &str, replace: Option<String>) -> Self {
        IdWrite {
            path: card.path.clone(),
            line: card.id_line,
            markdown: card.style == CardStyle::Markdown,
            id: id.to_string(),
            replace,
        }
    }
}

/// Apply the writes under `root`. Returns how many ids were written, and warnings.
pub(crate) fn apply(root: &Path, writes: Vec<IdWrite>) -> (u64, Vec<ParseWarning>) {
    let mut by_file: BTreeMap<String, Vec<IdWrite>> = BTreeMap::new();
    for w in writes {
        by_file.entry(w.path.clone()).or_default().push(w);
    }
    let mut written = 0;
    let mut warnings = Vec::new();
    for (rel, mut ws) in by_file {
        // Bottom-up, so lines added for Markdown cards don't shift the ones still to do.
        ws.sort_by_key(|w| std::cmp::Reverse(w.line));
        let path = root.join(&rel);
        match rewrite(&path, &ws) {
            Ok((n, skipped)) => {
                written += n;
                for (line, message) in skipped {
                    warnings.push(ParseWarning {
                        path: rel.clone(),
                        line,
                        message,
                    });
                }
            }
            Err(e) => warnings.push(ParseWarning {
                path: rel.clone(),
                line: 0,
                message: format!("couldn't write card ids into the file: {e}"),
            }),
        }
    }
    (written, warnings)
}

type Skipped = Vec<(u32, String)>;

fn rewrite(path: &Path, writes: &[IdWrite]) -> std::io::Result<(u64, Skipped)> {
    let text = std::fs::read_to_string(path)?;
    let eol = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut lines: Vec<String> = text.split_inclusive('\n').map(str::to_string).collect();
    let mut written = 0;
    let mut skipped = Vec::new();
    for w in writes {
        let idx = w.line as usize - 1;
        let ok = if w.markdown {
            markdown(&mut lines, idx, w, eol)
        } else {
            comment(&mut lines, idx, w)
        };
        if ok {
            written += 1;
        } else {
            skipped.push((
                w.line,
                "the file changed during the scan, so its card id wasn't written; scan again"
                    .to_string(),
            ));
        }
    }
    if written > 0 {
        replace_file(path, &lines.concat())?;
    }
    Ok((written, skipped))
}

fn comment(lines: &mut [String], idx: usize, w: &IdWrite) -> bool {
    let Some(line) = lines.get_mut(idx) else {
        return false;
    };
    let (from, to) = match &w.replace {
        Some(old) => (format!("@card({old})"), format!("@card({})", w.id)),
        None => ("@card".to_string(), format!("@card({})", w.id)),
    };
    let Some(at) = line.find(&from) else {
        return false;
    };
    if w.replace.is_none() && line[at + from.len()..].starts_with('(') {
        return false; // already has an id
    }
    line.replace_range(at..at + from.len(), &to);
    true
}

fn markdown(lines: &mut Vec<String>, idx: usize, w: &IdWrite, eol: &str) -> bool {
    let new = format!("<!-- id: {} -->{eol}", w.id);
    if let Some(old) = &w.replace {
        let Some(line) = lines.get_mut(idx) else {
            return false;
        };
        if !line.contains(&format!("<!-- id: {old} -->")) {
            return false;
        }
        *line = new;
        return true;
    }
    if idx > lines.len() {
        return false;
    }
    // Make sure the line before ends with a newline before adding ours after it.
    if idx > 0
        && let Some(prev) = lines.get_mut(idx - 1)
        && !prev.ends_with('\n')
    {
        prev.push_str(eol);
    }
    lines.insert(idx, new);
    true
}

/// Make the new contents durable before the rename, so a crash can't leave an empty file.
/// On macOS, `File::sync_all` issues `F_FULLFSYNC`, which also flushes the drive's cache:
/// about 4 ms a file, over 4 s for a first scan that writes ids into 1,000 files. Plain
/// `fsync` gives the guarantee `sync_all` gives elsewhere, as Git does by default.
fn flush_to_disk(f: &std::fs::File) -> std::io::Result<()> {
    #[cfg(target_vendor = "apple")]
    {
        rustix::fs::fsync(f).map_err(std::io::Error::from)
    }
    #[cfg(not(target_vendor = "apple"))]
    {
        f.sync_all()
    }
}

/// Replace a file's contents atomically, keeping its permissions.
fn replace_file(path: &Path, contents: &str) -> std::io::Result<()> {
    let dir = path.parent().unwrap_or(Path::new("."));
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let tmp = dir.join(format!(".{name}.scrubjay-tmp"));
    let result = (|| {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(contents.as_bytes())?;
        flush_to_disk(&f)?;
        let perms = std::fs::metadata(path)?.permissions();
        std::fs::set_permissions(&tmp, perms)?;
        std::fs::rename(&tmp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn w(line: u32, markdown: bool, id: &str, replace: Option<&str>) -> IdWrite {
        IdWrite {
            path: "f".into(),
            line,
            markdown,
            id: id.into(),
            replace: replace.map(str::to_string),
        }
    }

    fn run(text: &str, writes: &[IdWrite]) -> (String, u64, Skipped) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("f");
        std::fs::write(&path, text).unwrap();
        let mut ws = writes.to_vec();
        ws.sort_by_key(|w| std::cmp::Reverse(w.line));
        let (n, skipped) = rewrite(&path, &ws).unwrap();
        (std::fs::read_to_string(&path).unwrap(), n, skipped)
    }

    #[test]
    fn comment_ids_are_inserted_in_place() {
        let (out, n, _) = run(
            "// @card Q?\r\n// A.\r\nfn f() {}\r\n",
            &[w(1, false, "k22k1m8e", None)],
        );
        assert_eq!(n, 1);
        assert_eq!(out, "// @card(k22k1m8e) Q?\r\n// A.\r\nfn f() {}\r\n");
    }

    #[test]
    fn duplicate_ids_are_replaced() {
        let (out, _, _) = run(
            "# @card(aaaaaaaa) Q?\n# A.\n",
            &[w(1, false, "bbbbbbbb", Some("aaaaaaaa"))],
        );
        assert_eq!(out, "# @card(bbbbbbbb) Q?\n# A.\n");
    }

    #[test]
    fn markdown_ids_are_added_after_each_card_bottom_up() {
        let text = "Q: one\nA: 1\n\nQ: two\nA: 2";
        let (out, n, _) = run(
            text,
            &[w(3, true, "aaaaaaaa", None), w(6, true, "bbbbbbbb", None)],
        );
        assert_eq!(n, 2);
        assert_eq!(
            out,
            "Q: one\nA: 1\n<!-- id: aaaaaaaa -->\n\nQ: two\nA: 2\n<!-- id: bbbbbbbb -->\n"
        );
    }

    #[test]
    fn changed_lines_are_skipped() {
        let (out, n, skipped) = run("fn f() {}\n", &[w(1, false, "k22k1m8e", None)]);
        assert_eq!((n, skipped.len()), (0, 1));
        assert_eq!(out, "fn f() {}\n", "file untouched");
        let (_, n, _) = run("// @card(abcdefgh) Q?\n", &[w(1, false, "k22k1m8e", None)]);
        assert_eq!(n, 0, "never overwrite an existing id unless asked");
    }

    #[cfg(unix)]
    #[test]
    fn permissions_are_kept() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("run.sh");
        std::fs::write(&path, "# @card Q?\n# A.\n").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        rewrite(&path, &[w(1, false, "k22k1m8e", None)]).unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o755);
    }
}
