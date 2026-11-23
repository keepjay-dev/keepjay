// SPDX-License-Identifier: AGPL-3.0-or-later
//! Data and config directory resolution (docs/specs/09-data-model.md §8).
//!
//! Precedence: explicit path → `KEEPJAY_DATA_DIR` / `KEEPJAY_CONFIG` (or the old
//! `SCRUBJAY_*` and `ADDISCO_*` names) → OS default. The product was called Addisco until
//! 0.2 (D-035), then Scrubjay until 0.2.0-alpha.3 (D-037): an existing data or config folder under
//! either old name is moved to the new name on first run.

use std::path::{Path, PathBuf};

pub const DATA_DIR_ENV: &str = "KEEPJAY_DATA_DIR";
pub const CONFIG_ENV: &str = "KEEPJAY_CONFIG";
/// Names used before the renames (D-035, D-037), newest first; still honoured.
pub const LEGACY_DATA_DIR_ENVS: [&str; 2] = ["SCRUBJAY_DATA_DIR", "ADDISCO_DATA_DIR"];
pub const LEGACY_CONFIG_ENVS: [&str; 2] = ["SCRUBJAY_CONFIG", "ADDISCO_CONFIG"];
pub const DB_FILE: &str = "study.db";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paths {
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
}

impl Paths {
    /// Use explicit directories (tests, portable installs).
    pub fn new(data_dir: impl Into<PathBuf>, config_dir: impl Into<PathBuf>) -> Self {
        Paths {
            data_dir: data_dir.into(),
            config_dir: config_dir.into(),
        }
    }

    /// Resolve using an optional `--data-dir` override, then environment, then OS defaults.
    pub fn resolve(data_dir_override: Option<&Path>) -> Self {
        let data_dir = data_dir_override
            .map(Path::to_path_buf)
            .or_else(|| first_env(DATA_DIR_ENV, &LEGACY_DATA_DIR_ENVS))
            .unwrap_or_else(|| adopt_legacy(default_data_dir(), None));
        let config_dir = first_env(CONFIG_ENV, &LEGACY_CONFIG_ENVS)
            .and_then(|p| p.parent().map(Path::to_path_buf))
            // On macOS the config folder is the data folder: never move the one in use.
            .unwrap_or_else(|| adopt_legacy(default_config_dir(), Some(&data_dir)));
        Paths {
            data_dir,
            config_dir,
        }
    }

    pub fn db_path(&self) -> PathBuf {
        self.data_dir.join(DB_FILE)
    }

    /// Content-addressed media files (`media/ab/cd/<sha256>.<ext>`, 08 §7, D-026).
    pub fn media_dir(&self) -> PathBuf {
        self.data_dir.join("media")
    }

    pub fn backups_dir(&self) -> PathBuf {
        self.data_dir.join("backups")
    }

    pub fn cli_config_path(&self) -> PathBuf {
        first_env(CONFIG_ENV, &LEGACY_CONFIG_ENVS)
            .unwrap_or_else(|| self.config_dir.join("cli.toml"))
    }
}

/// The folder names the product used before the renames (D-035, D-037), newest first.
#[cfg(any(target_os = "windows", target_os = "macos"))]
const LEGACY_DIR_NAMES: [&str; 2] = ["Scrubjay", "Addisco"];
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
const LEGACY_DIR_NAMES: [&str; 2] = ["scrubjay", "addisco"];

/// Use `new`, first moving the newest existing pre-rename folder next to it into place. If
/// the move fails (e.g. the old folder is in use), keep using the old folder as it is.
/// `in_use` (the data folder) is never moved.
fn adopt_legacy(new: PathBuf, in_use: Option<&Path>) -> PathBuf {
    match LEGACY_DIR_NAMES
        .iter()
        .map(|name| new.with_file_name(name))
        .find(|old| old.is_dir())
    {
        Some(old) if in_use == Some(old.as_path()) => new,
        Some(old) => migrate_dir(new, old),
        None => new,
    }
}

// @card(z8dxm6ep) What happens to an existing `Scrubjay` or `Addisco` data folder on first run?
// It's renamed to the Keepjay name unless that folder already exists. If the rename fails,
// the old folder keeps being used.
fn migrate_dir(new: PathBuf, old: PathBuf) -> PathBuf {
    if new.exists() || !old.is_dir() || new == old {
        return new;
    }
    match std::fs::rename(&old, &new) {
        Ok(()) => new,
        Err(_) => old,
    }
}

/// The first of `var` and then `legacy` that is set and non-empty.
fn first_env(var: &str, legacy: &[&str]) -> Option<PathBuf> {
    std::iter::once(var)
        .chain(legacy.iter().copied())
        .find_map(env_path)
}

fn env_path(var: &str) -> Option<PathBuf> {
    std::env::var_os(var)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

fn base() -> directories::BaseDirs {
    directories::BaseDirs::new().expect("could not determine the home directory")
}

#[cfg(target_os = "windows")]
fn default_data_dir() -> PathBuf {
    base().data_local_dir().join("Keepjay")
}
#[cfg(target_os = "windows")]
fn default_config_dir() -> PathBuf {
    base().config_dir().join("Keepjay")
}

#[cfg(target_os = "macos")]
fn default_data_dir() -> PathBuf {
    base().data_dir().join("Keepjay")
}
#[cfg(target_os = "macos")]
fn default_config_dir() -> PathBuf {
    // 09 §8: same as data on macOS; the CLI also honours ~/.config/keepjay/cli.toml.
    let xdg = base().home_dir().join(".config").join("keepjay");
    if xdg.join("cli.toml").exists() {
        xdg
    } else {
        default_data_dir()
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn default_data_dir() -> PathBuf {
    base().data_dir().join("keepjay")
}
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn default_config_dir() -> PathBuf {
    base().config_dir().join("keepjay")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_override_wins() {
        let p = Paths::resolve(Some(Path::new("/tmp/keepjay-test")));
        assert_eq!(p.data_dir, PathBuf::from("/tmp/keepjay-test"));
        assert_eq!(
            p.db_path(),
            PathBuf::from("/tmp/keepjay-test").join(DB_FILE)
        );
    }

    #[test]
    fn pre_rename_folder_is_moved_once() {
        let dir = tempfile::tempdir().unwrap();
        let old = dir.path().join("Addisco");
        let new = dir.path().join("Keepjay");
        std::fs::create_dir(&old).unwrap();
        std::fs::write(old.join(DB_FILE), b"db").unwrap();

        assert_eq!(migrate_dir(new.clone(), old.clone()), new);
        assert!(!old.exists());
        assert_eq!(std::fs::read(new.join(DB_FILE)).unwrap(), b"db");

        // Both present (e.g. the old one came back from a backup): the new one wins.
        std::fs::create_dir(&old).unwrap();
        assert_eq!(migrate_dir(new.clone(), old.clone()), new);
        assert!(old.exists(), "nothing is deleted");

        // Neither present: the new name.
        let fresh = dir.path().join("fresh");
        assert_eq!(migrate_dir(fresh.clone(), dir.path().join("none")), fresh);
    }

    #[test]
    fn newest_pre_rename_folder_is_adopted() {
        let dir = tempfile::tempdir().unwrap();
        for old in LEGACY_DIR_NAMES {
            std::fs::create_dir(dir.path().join(old)).unwrap();
            std::fs::write(dir.path().join(old).join(DB_FILE), old).unwrap();
        }
        let new = dir.path().join("Keepjay");
        let scrubjay = dir.path().join(LEGACY_DIR_NAMES[0]);
        assert_eq!(
            adopt_legacy(new.clone(), Some(&scrubjay)),
            new,
            "the folder in use as the data folder isn't moved"
        );
        assert!(scrubjay.exists() && !new.exists());
        assert_eq!(adopt_legacy(new.clone(), None), new);
        assert_eq!(
            std::fs::read_to_string(new.join(DB_FILE)).unwrap(),
            LEGACY_DIR_NAMES[0],
            "the Scrubjay folder wins over the older Addisco one"
        );
        assert!(
            dir.path().join(LEGACY_DIR_NAMES[1]).exists(),
            "nothing is deleted"
        );
    }

    #[test]
    fn default_dir_is_named_after_product() {
        let name = default_data_dir();
        let last = name.file_name().unwrap().to_string_lossy().to_lowercase();
        assert_eq!(last, "keepjay");
    }
}
