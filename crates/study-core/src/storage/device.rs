// SPDX-License-Identifier: AGPL-3.0-or-later
//! This installation's identity, kept in `<data_dir>/device.toml` so it never syncs
//! (docs/specs/09-data-model.md §8). Every process on one machine (CLI, VS Code, MCP) shares it.

use std::path::Path;

use crate::domain::Id;
use crate::error::{Error, Result};

const FILE: &str = "device.toml";

pub fn load_or_create(data_dir: &Path) -> Result<Id> {
    let path = data_dir.join(FILE);
    if let Ok(text) = std::fs::read_to_string(&path) {
        return parse(&text).ok_or_else(|| {
            Error::Unexpected(format!(
                "{} is malformed; expected device_id = \"…\"",
                path.display()
            ))
        });
    }
    let id = Id::new();
    let body =
        format!("# Keepjay device identity. Do not copy between machines.\ndevice_id = \"{id}\"\n");
    // create_new: if another process won the race, read its id instead of overwriting.
    match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
    {
        Ok(mut f) => {
            use std::io::Write;
            f.write_all(body.as_bytes())?;
            Ok(id)
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => load_or_create(data_dir),
        Err(e) => Err(e.into()),
    }
}

fn parse(text: &str) -> Option<Id> {
    text.lines()
        .map(str::trim)
        .find_map(|l| l.strip_prefix("device_id"))
        .and_then(|rest| rest.trim_start().strip_prefix('='))
        .map(|v| v.trim().trim_matches('"'))
        .and_then(|v| v.parse().ok())
}

pub fn host_name() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "this device".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_across_calls() {
        let dir = tempfile::tempdir().unwrap();
        let a = load_or_create(dir.path()).unwrap();
        let b = load_or_create(dir.path()).unwrap();
        assert_eq!(a, b);
    }
}
