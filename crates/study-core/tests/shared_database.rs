// SPDX-License-Identifier: AGPL-3.0-or-later
//! M0.3 exit criterion: two shells (e.g. desktop + CLI) open the same database, and one
//! sees the other's writes via `data_version` + the change log (docs/specs/08 §8).

use study_core::{Core, Paths};

fn paths(dir: &tempfile::TempDir) -> Paths {
    Paths::new(dir.path(), dir.path())
}

#[test]
fn two_handles_share_one_database_and_see_changes() {
    let dir = tempfile::tempdir().unwrap();
    let mut cli = Core::open(paths(&dir)).unwrap();
    let desktop = Core::open(paths(&dir)).unwrap();

    assert_eq!(
        cli.device_id(),
        desktop.device_id(),
        "same machine, same device id"
    );

    let seen_seq = desktop.last_change_seq().unwrap();
    let seen_version = desktop.data_version().unwrap();

    let ws = cli.create_workspace("Uni").unwrap();

    assert_ne!(
        desktop.data_version().unwrap(),
        seen_version,
        "desktop detects the write"
    );
    let changes = desktop.changes_since(seen_seq).unwrap();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].entity, "workspace");
    assert_eq!(changes[0].entity_id, ws.id);
    assert_eq!(desktop.status().unwrap().workspaces, 1);
}

#[test]
fn init_is_idempotent_and_names_are_unique() {
    let dir = tempfile::tempdir().unwrap();
    let mut core = Core::open(paths(&dir)).unwrap();
    let first = core.init("My Studies").unwrap();
    let again = core.init("Something else").unwrap();
    assert_eq!(first, again);

    let err = core.create_workspace("My Studies").unwrap_err();
    assert_eq!(err.kind(), study_core::ErrorKind::Conflict);
    let err = core.create_workspace("   ").unwrap_err();
    assert_eq!(err.kind(), study_core::ErrorKind::Validation);
}

#[test]
fn doctor_reports_healthy_database() {
    let dir = tempfile::tempdir().unwrap();
    let core = Core::open(paths(&dir)).unwrap();
    let report = core.doctor().unwrap();
    assert!(report.ok, "{report:?}");
    assert_eq!(report.schema_version, report.supported_schema_version);
}
