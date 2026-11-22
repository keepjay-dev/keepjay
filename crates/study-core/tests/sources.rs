// SPDX-License-Identifier: AGPL-3.0-or-later
//! Registering, listing and removing sources through the public core API (SRC-001).

use std::path::Path;
use std::process::Command;

use study_core::{Core, ErrorKind, Id, NewSource, Paths, SourceKind};

fn setup() -> (tempfile::TempDir, Core, Id) {
    let dir = tempfile::tempdir().unwrap();
    let mut core = Core::open(Paths::new(dir.path().join("data"), dir.path())).unwrap();
    core.init("Dev").unwrap();
    let ws = core.resolve_workspace(None).unwrap();
    (dir, core, ws)
}

fn folder(base: &Path, name: &str) -> std::path::PathBuf {
    let p = base.join(name);
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn git(dir: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.com",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .status()
        .expect("git is installed")
        .success();
    assert!(ok, "git {args:?} failed");
}

fn new(path: &Path) -> NewSource {
    NewSource {
        path: path.to_path_buf(),
        write_ids: true,
        ..NewSource::default()
    }
}

#[test]
fn plain_folder_is_registered_with_a_deck_named_after_it() {
    let (dir, mut core, ws) = setup();
    let path = folder(dir.path(), "notes");
    let s = core.add_source(ws, new(&path)).unwrap();
    assert_eq!(s.name, "notes");
    assert_eq!(s.kind, SourceKind::Folder);
    assert_eq!(s.repo, None);
    assert_eq!(s.deck, "notes");
    assert!(s.write_ids);
    assert!(core.find_deck(ws, "notes").unwrap().is_some());
    assert!(Path::new(&s.path).is_absolute());
}

#[test]
fn git_repository_gets_its_root_commit_as_identity() {
    let (dir, mut core, ws) = setup();
    let repo = folder(dir.path(), "app");
    git(&repo, &["init", "-q"]);
    git(&repo, &["commit", "-q", "--allow-empty", "-m", "first"]);
    git(&repo, &["commit", "-q", "--allow-empty", "-m", "second"]);
    let root = String::from_utf8(
        Command::new("git")
            .arg("-C")
            .arg(&repo)
            .args(["rev-list", "--max-parents=0", "HEAD"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_string();

    // A subfolder of the repository has the same identity.
    let sub = folder(&repo, "src");
    let s = core
        .add_source(
            ws,
            NewSource {
                name: Some("app-src".into()),
                deck: Some("Code::App".into()),
                tags: vec!["lang::rust".into(), "repo::app".into()],
                ..new(&sub)
            },
        )
        .unwrap();
    assert_eq!(s.kind, SourceKind::Repo);
    assert_eq!(s.repo.as_deref(), Some(root.as_str()));
    assert_eq!(s.deck, "Code::App");
    assert_eq!(s.tags, ["lang::rust", "repo::app"]);
}

#[test]
fn repository_without_commits_is_a_repo_without_identity_yet() {
    let (dir, mut core, ws) = setup();
    let repo = folder(dir.path(), "fresh");
    git(&repo, &["init", "-q"]);
    let s = core.add_source(ws, new(&repo)).unwrap();
    assert_eq!(s.kind, SourceKind::Repo);
    assert_eq!(s.repo, None);
}

#[test]
fn duplicates_and_bad_input_are_rejected() {
    let (dir, mut core, ws) = setup();
    let path = folder(dir.path(), "app");
    core.add_source(ws, new(&path)).unwrap();

    let same_path = core
        .add_source(
            ws,
            NewSource {
                name: Some("other".into()),
                ..new(&path)
            },
        )
        .unwrap_err();
    assert_eq!(same_path.kind(), ErrorKind::Conflict);

    let other = folder(dir.path(), "elsewhere");
    let same_name = core
        .add_source(
            ws,
            NewSource {
                name: Some("app".into()),
                ..new(&other)
            },
        )
        .unwrap_err();
    assert_eq!(same_name.kind(), ErrorKind::Conflict);

    let missing = core
        .add_source(ws, new(&dir.path().join("nope")))
        .unwrap_err();
    assert_eq!(missing.kind(), ErrorKind::Validation);

    let file = dir.path().join("file.txt");
    std::fs::write(&file, "x").unwrap();
    assert_eq!(
        core.add_source(ws, new(&file)).unwrap_err().kind(),
        ErrorKind::Validation
    );

    let bad_tag = core
        .add_source(
            ws,
            NewSource {
                tags: vec!["has space".into()],
                ..new(&other)
            },
        )
        .unwrap_err();
    assert_eq!(bad_tag.kind(), ErrorKind::Validation);
    assert_eq!(core.list_sources(ws).unwrap().len(), 1);
}

#[test]
fn sources_are_found_by_name_or_short_id_and_removed() {
    let (dir, mut core, ws) = setup();
    let a = core.add_source(ws, new(&folder(dir.path(), "a"))).unwrap();
    let b = core
        .add_source(
            ws,
            NewSource {
                write_ids: false,
                ..new(&folder(dir.path(), "b"))
            },
        )
        .unwrap();
    assert!(!b.write_ids);

    let names: Vec<_> = core
        .list_sources(ws)
        .unwrap()
        .into_iter()
        .map(|s| s.name)
        .collect();
    assert_eq!(names, ["a", "b"]);
    assert_eq!(core.find_source(ws, &a.short_id).unwrap().name, "a");
    assert_eq!(
        core.find_source(ws, "zzz").unwrap_err().kind(),
        ErrorKind::NotFound
    );

    let before = core.last_change_seq().unwrap();
    core.remove_source(ws, "a").unwrap();
    let changes = core.changes_since(before).unwrap();
    assert!(
        changes
            .iter()
            .any(|c| c.entity == "source" && c.op == "delete" && c.entity_id == a.id),
        "{changes:?}"
    );
    let names: Vec<_> = core
        .list_sources(ws)
        .unwrap()
        .into_iter()
        .map(|s| s.name)
        .collect();
    assert_eq!(names, ["b"]);

    // The same folder can be registered again after removal.
    core.add_source(ws, new(&dir.path().join("a"))).unwrap();
}
