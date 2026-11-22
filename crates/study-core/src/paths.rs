// SPDX-License-Identifier: AGPL-3.0-or-later
//! Data and config directory resolution (docs/specs/09-data-model.md §8).
//!
//! Precedence: explicit path → `SCRUBJAY_DATA_DIR` / `SCRUBJAY_CONFIG` (or the old
//! `ADDISCO_*` names) → OS default. The product was called Addisco until 0.2 (D-035): an
//! existing `Addisco` data or config folder is moved to the new name on first run.

use std::path::{Path, PathBuf};

pub const DATA_DIR_ENV: &str = "SCRUBJAY_DATA_DIR";
pub const CONFIG_ENV: &str = "SCRUBJAY_CONFIG";
/// Names used before the rename to Scrubjay (D-035); still honoured.
pub const LEGACY_DATA_DIR_ENV: &str = "ADDISCO_DATA_DIR";
pub const LEGACY_CONFIG_ENV: &str = "ADDISCO_CONFIG";
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
            .or_else(|| env_path(DATA_DIR_ENV))
            .or_else(|| env_path(LEGACY_DATA_DIR_ENV))
            .unwrap_or_else(|| adopt_legacy(default_data_dir()));
        let config_dir = env_path(CONFIG_ENV)
            .or_else(|| env_path(LEGACY_CONFIG_ENV))
            .and_then(|p| p.parent().map(Path::to_path_buf))
            .unwrap_or_else(|| adopt_legacy(default_config_dir()));
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
        env_path(CONFIG_ENV)
            .or_else(|| env_path(LEGACY_CONFIG_ENV))
            .unwrap_or_else(|| self.config_dir.join("cli.toml"))
    }
}

/// The folder name the product used before the rename (D-035), on this OS.
#[cfg(any(target_os = "windows", target_os = "macos"))]
const LEGACY_DIR_NAME: &str = "Addisco";
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
const LEGACY_DIR_NAME: &str = "addisco";

/// Use `new`, first moving an existing pre-rename folder next to it into place. If the
/// move fails (e.g. the old folder is in use), keep using the old folder as it is.
fn adopt_legacy(new: PathBuf) -> PathBuf {
    let old = new.with_file_name(LEGACY_DIR_NAME);
    migrate_dir(new, old)
}

// @card(z8dxm6ep) What happens to an existing `Addisco` data folder on first run?
// It's renamed to the Scrubjay name unless that folder already exists. If the rename fails,
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
    base().data_local_dir().join("Scrubjay")
}
#[cfg(target_os = "windows")]
fn default_config_dir() -> PathBuf {
    base().config_dir().join("Scrubjay")
}

#[cfg(target_os = "macos")]
fn default_data_dir() -> PathBuf {
    base().data_dir().join("Scrubjay")
}
#[cfg(target_os = "macos")]
fn default_config_dir() -> PathBuf {
    // 09 §8: same as data on macOS; the CLI also honours ~/.config/scrubjay/cli.toml.
    let xdg = base().home_dir().join(".config").join("scrubjay");
    if xdg.join("cli.toml").exists() {
        xdg
    } else {
        default_data_dir()
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn default_data_dir() -> PathBuf {
    base().data_dir().join("scrubjay")
}
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn default_config_dir() -> PathBuf {
    base().config_dir().join("scrubjay")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_override_wins() {
        let p = Paths::resolve(Some(Path::new("/tmp/scrubjay-test")));
        assert_eq!(p.data_dir, PathBuf::from("/tmp/scrubjay-test"));
        assert_eq!(
            p.db_path(),
            PathBuf::from("/tmp/scrubjay-test").join(DB_FILE)
        );
    }

    #[test]
    fn pre_rename_folder_is_moved_once() {
        let dir = tempfile::tempdir().unwrap();
        let old = dir.path().join("Addisco");
        let new = dir.path().join("Scrubjay");
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
    fn default_dir_is_named_after_product() {
        let name = default_data_dir();
        let last = name.file_name().unwrap().to_string_lossy().to_lowercase();
        assert_eq!(last, "scrubjay");
    }
}
