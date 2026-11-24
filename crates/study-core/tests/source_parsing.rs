// SPDX-License-Identifier: AGPL-3.0-or-later
//! Finding cards in a source tree (SRC-002, SRC-003, SRC-008).

use std::path::Path;

use study_core::sources::{CardStyle, find_cards, walk::MAX_FILE_BYTES};

fn write(root: &Path, rel: &str, contents: &[u8]) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, contents).unwrap();
}

#[test]
fn finds_comment_and_markdown_cards_and_respects_ignores() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(
        root,
        "src/lib.rs",
        b"// @card What does open() do?\n// Opens the database.\nfn open() {}\n",
    );
    write(
        root,
        "scripts/build.py",
        b"# @card {{c1::pathlib}} replaces os.path\nimport pathlib\n",
    );
    write(root, "src/plain.rs", b"fn no_cards_here() {}\n");
    write(
        root,
        "docs/async.cards.md",
        b"Q: What is a future?\nA: A value that will be ready later.\n",
    );
    write(
        root,
        ".keepjay/cards/agent.md",
        b"C: {{c1::FSRS}} schedules reviews.\n",
    );
    write(
        root,
        "README.md",
        b"Q: not a card file\nA: ordinary Markdown\n",
    );

    // Ignored by .gitignore, .keepjayignore, or because they're hidden.
    write(root, ".gitignore", b"target/\n");
    write(root, "target/gen.rs", b"// @card ignored?\n// yes\n");
    write(root, ".keepjayignore", b"vendor/\n");
    write(root, "vendor/dep.rs", b"// @card ignored?\n// yes\n");
    write(root, ".github/ci.yml", b"# @card hidden?\n# yes\n");
    write(root, ".env", b"# @card secrets?\n# never read\n");

    // Skipped: too large, binary, not UTF-8.
    let mut big = b"// @card big?\n// yes\n".to_vec();
    big.resize(MAX_FILE_BYTES as usize + 1, b' ');
    write(root, "src/big.rs", &big);
    write(root, "src/blob.rs", b"// @card binary?\n// yes\n\0\0");
    write(root, "src/latin1.rs", b"// @card caf\xe9?\n// yes\n");

    let found = find_cards(root).unwrap();
    assert!(found.warnings.is_empty(), "{:?}", found.warnings);
    let mut paths: Vec<_> = found.cards.iter().map(|c| c.path.as_str()).collect();
    paths.sort();
    assert_eq!(
        paths,
        [
            ".keepjay/cards/agent.md",
            "docs/async.cards.md",
            "scripts/build.py",
            "src/lib.rs"
        ]
    );
    assert_eq!(found.skipped_large, 1);
    assert_eq!(found.skipped_binary, 2);
    // lib.rs, build.py, plain.rs, big.rs, blob.rs, latin1.rs + the two card files.
    assert_eq!(found.files_considered, 8);
    // plain.rs has no `@card`, so it isn't parsed.
    assert_eq!(found.files_parsed, 4);

    let rust = found.cards.iter().find(|c| c.path == "src/lib.rs").unwrap();
    assert_eq!(
        rust.style,
        CardStyle::Comment {
            marker: "//".into()
        }
    );
    assert!(rust.code_hash.is_some());
    let md = found
        .cards
        .iter()
        .find(|c| c.path == "docs/async.cards.md")
        .unwrap();
    assert_eq!(md.style, CardStyle::Markdown);
    assert!(md.code_hash.is_none());
}

#[test]
fn warnings_name_the_file_and_line() {
    let dir = tempfile::tempdir().unwrap();
    write(
        dir.path(),
        "a/b.go",
        b"package b\n\n// @card no answer?\nfunc F() {}\n",
    );
    let found = find_cards(dir.path()).unwrap();
    assert!(found.cards.is_empty());
    assert_eq!(found.warnings.len(), 1);
    assert_eq!(found.warnings[0].path, "a/b.go");
    assert_eq!(found.warnings[0].line, 3);
}

#[test]
fn card_files_that_git_ignores_are_warned_about() {
    for (file, rule) in [
        (".gitignore", ".keepjay/\n"),
        (".git/info/exclude", ".keepjay/\n"),
        (".gitignore", "*.md\n"),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join(".git")).unwrap();
        write(root, file, rule.as_bytes());
        write(root, ".keepjay/cards/a.md", b"Q: q?\nA: a\n");
        write(root, ".keepjay/cards/sub/b.md", b"Q: r?\nA: b\n");
        write(root, "vendor/.keepjay/cards/c.md", b"Q: s?\nA: c\n");

        let found = find_cards(root).unwrap();
        assert!(found.cards.is_empty());
        assert_eq!(found.warnings.len(), 1, "{:?}", found.warnings);
        let w = &found.warnings[0];
        assert_eq!((w.path.as_str(), w.line), (".keepjay/cards", 0));
        assert!(w.message.starts_with("2 card files"), "{}", w.message);

        // Ignoring them in .keepjayignore is a choice: no warning.
        write(root, ".keepjayignore", b".keepjay/\n");
        assert!(find_cards(root).unwrap().warnings.is_empty());

        // The advice in the warning works for an ignored folder.
        if rule == ".keepjay/\n" {
            write(root, ".keepjayignore", b"!.keepjay/\n");
            let found = find_cards(root).unwrap();
            assert_eq!(found.cards.len(), 2, "{file}");
            assert!(found.warnings.is_empty());
        }
    }
}
