// SPDX-License-Identifier: AGPL-3.0-or-later
//! Data and config directory resolution (docs/specs/09-data-model.md §8).
//!
//! Precedence: explicit path → `ADDISCO_DATA_DIR` / `ADDISCO_CONFIG` → OS default.

use std::path::{Path, PathBuf};

pub const DATA_DIR_ENV: &str = "ADDISCO_DATA_DIR";
pub const CONFIG_ENV: &str = "ADDISCO_CONFIG";
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
            .unwrap_or_else(default_data_dir);
        let config_dir = env_path(CONFIG_ENV)
            .and_then(|p| p.parent().map(Path::to_path_buf))
            .unwrap_or_else(default_config_dir);
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
        env_path(CONFIG_ENV).unwrap_or_else(|| self.config_dir.join("cli.toml"))
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
    base().data_local_dir().join("Addisco")
}
#[cfg(target_os = "windows")]
fn default_config_dir() -> PathBuf {
    base().config_dir().join("Addisco")
}

#[cfg(target_os = "macos")]
fn default_data_dir() -> PathBuf {
    base().data_dir().join("Addisco")
}
#[cfg(target_os = "macos")]
fn default_config_dir() -> PathBuf {
    // 09 §8: same as data on macOS; the CLI also honours ~/.config/addisco/cli.toml.
    let xdg = base().home_dir().join(".config").join("addisco");
    if xdg.join("cli.toml").exists() {
        xdg
    } else {
        default_data_dir()
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn default_data_dir() -> PathBuf {
    base().data_dir().join("addisco")
}
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn default_config_dir() -> PathBuf {
    base().config_dir().join("addisco")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_override_wins() {
        let p = Paths::resolve(Some(Path::new("/tmp/addisco-test")));
        assert_eq!(p.data_dir, PathBuf::from("/tmp/addisco-test"));
        assert_eq!(
            p.db_path(),
            PathBuf::from("/tmp/addisco-test").join(DB_FILE)
        );
    }

    #[test]
    fn default_dir_is_named_after_product() {
        let name = default_data_dir();
        let last = name.file_name().unwrap().to_string_lossy().to_lowercase();
        assert_eq!(last, "addisco");
    }
}
