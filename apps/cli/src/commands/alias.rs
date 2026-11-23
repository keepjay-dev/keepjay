// SPDX-License-Identifier: AGPL-3.0-or-later
//! `scrubjay alias install|uninstall|status`: the official short alias `sj` (CLI-018).
//!
//! Not to be confused with the `[aliases]` table in `cli.toml`, which shortens subcommands.
//!
//! `sj` is put next to the `scrubjay` binary (or in `--dir`): a relative symlink on Unix, so
//! upgrading `scrubjay` upgrades `sj` too, and a hard link (or a copy) on Windows. It's only
//! installed when no other command called `sj` is on `PATH`. `scrubjay doctor` reports when
//! another `sj` earlier on `PATH` shadows it, or when a Windows copy is older than `scrubjay`.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};
use serde::Serialize;
use study_core::Error;

use super::Ctx;
use crate::output::CliError;

/// The short alias: the initials of *scrub jay* (CLI-018, D-035).
pub const ALIAS: &str = "sj";

/// A string every `scrubjay` build contains, to recognise an older copy of it.
const MARKER: &[u8] = crate::output::SCHEMA.as_bytes();

#[derive(Args)]
pub struct AliasArgs {
    #[command(subcommand)]
    cmd: Option<AliasCmd>,
}

pub fn run(ctx: &mut Ctx, args: AliasArgs) -> Result<(), CliError> {
    let out = ctx.out;
    match args.cmd.unwrap_or(AliasCmd::Status) {
        AliasCmd::Status => {
            let s = status()?;
            out.emit(&s, || s.line())
        }
        AliasCmd::Install { dir } => {
            let r = install(dir.as_deref())?;
            out.emit(&r, || {
                let mut text = if r.changed {
                    format!("Installed `sj` ({}): try `sj status`.", r.path)
                } else {
                    format!("`sj` is already installed ({}).", r.path)
                };
                if !r.dir_on_path {
                    text.push_str("\nThat folder isn't on your PATH: add it, or use --dir.");
                }
                text
            })
        }
        AliasCmd::Uninstall { dir } => {
            let r = uninstall(dir.as_deref())?;
            out.emit(&r, || {
                if r.removed {
                    format!("Removed `sj` ({}).", r.path)
                } else {
                    format!("No `sj` alias at {}.", r.path)
                }
            })
        }
    }
}

#[derive(Subcommand)]
enum AliasCmd {
    /// Install `sj` next to scrubjay (or in --dir), unless another `sj` is on your PATH
    Install {
        /// Folder to put `sj` in (default: the folder scrubjay is in)
        #[arg(long, value_name = "DIR")]
        dir: Option<PathBuf>,
    },
    /// Remove the `sj` that `alias install` created
    Uninstall {
        /// Folder it was installed in (default: the folder scrubjay is in)
        #[arg(long, value_name = "DIR")]
        dir: Option<PathBuf>,
    },
    /// Show whether `sj` is installed and what it runs (the default)
    Status,
}

/// What an `sj` file is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// Runs this `scrubjay`.
    Ours,
    /// An older copy of `scrubjay` (Windows, or a hard link left by an upgrade).
    Outdated,
    /// Some other program.
    Other,
}

#[derive(Debug, Clone, Serialize)]
pub struct Found {
    pub path: String,
    pub kind: Kind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    NotInstalled,
    Installed,
    /// Ours, but an older copy: `scrubjay alias install` refreshes it.
    Outdated,
    /// Ours is on PATH, but another `sj` comes first.
    Shadowed,
    /// Another program called `sj` is on PATH, and ours isn't.
    Taken,
}

#[derive(Debug, Clone, Serialize)]
pub struct AliasStatus {
    pub alias: &'static str,
    pub state: State,
    /// Every `sj` on PATH, in the order the shell finds them.
    pub on_path: Vec<Found>,
}

impl AliasStatus {
    /// One line for `alias status` and `doctor`.
    pub fn line(&self) -> String {
        let first = self.on_path.first().map(|f| f.path.as_str()).unwrap_or("");
        match self.state {
            State::NotInstalled => {
                "`sj` isn't installed; `scrubjay alias install` adds it".to_string()
            }
            State::Installed => format!("`sj` runs this scrubjay ({first})"),
            State::Outdated => format!(
                "`sj` ({first}) is an older copy of scrubjay; run `scrubjay alias install` to update it"
            ),
            State::Shadowed => format!(
                "another `sj` ({first}) comes before scrubjay's on your PATH, so `sj` doesn't run scrubjay"
            ),
            State::Taken => format!(
                "`sj` is already another program ({first}); use `scrubjay`, or a shell alias with another name"
            ),
        }
    }

    /// Fine for `doctor`: installed, or simply not installed.
    pub fn ok(&self) -> bool {
        matches!(self.state, State::Installed | State::NotInstalled)
    }
}

#[derive(Debug, Serialize)]
pub struct Installed {
    pub path: String,
    /// `symlink`, `hard_link` or `copy`.
    pub method: &'static str,
    /// False when `sj` was already installed and up to date.
    pub changed: bool,
    pub dir_on_path: bool,
}

#[derive(Debug, Serialize)]
pub struct Removed {
    pub path: String,
    pub removed: bool,
}

pub fn file_name() -> String {
    format!("{ALIAS}{}", std::env::consts::EXE_SUFFIX)
}

fn current_exe() -> Result<PathBuf, CliError> {
    Ok(std::env::current_exe()?.canonicalize()?)
}

fn default_dir(exe: &Path) -> PathBuf {
    exe.parent().map(Path::to_path_buf).unwrap_or_default()
}

/// Which `sj` files the shell would find, in PATH order.
fn candidates(path_var: Option<OsString>) -> Vec<PathBuf> {
    let Some(path_var) = path_var else {
        return Vec::new();
    };
    let names: Vec<String> = if cfg!(windows) {
        let exts = std::env::var("PATHEXT").unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".into());
        exts.split(';')
            .filter(|e| !e.is_empty())
            .map(|e| format!("{ALIAS}{}", e.to_ascii_lowercase()))
            .collect()
    } else {
        vec![ALIAS.to_string()]
    };
    let mut out = Vec::new();
    for dir in std::env::split_paths(&path_var) {
        for name in &names {
            let p = dir.join(name);
            if is_runnable(&p) && !out.contains(&p) {
                out.push(p);
            }
        }
    }
    out
}

fn is_runnable(p: &Path) -> bool {
    let Ok(meta) = std::fs::metadata(p) else {
        return false;
    };
    if !meta.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        meta.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

/// Is `p` this `scrubjay`, an older one, or something else?
fn classify(p: &Path, exe: &Path) -> Kind {
    if p.canonicalize().is_ok_and(|c| c == exe) {
        return Kind::Ours; // a symlink to us
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if let (Ok(a), Ok(b)) = (std::fs::metadata(p), std::fs::metadata(exe))
            && a.dev() == b.dev()
            && a.ino() == b.ino()
        {
            return Kind::Ours; // a hard link to us
        }
    }
    let (Ok(theirs), Ok(ours)) = (std::fs::read(p), std::fs::read(exe)) else {
        return Kind::Other;
    };
    if theirs == ours {
        Kind::Ours // a copy of this build
    } else if theirs.windows(MARKER.len()).any(|w| w == MARKER) {
        Kind::Outdated
    } else {
        Kind::Other
    }
}

/// The alias's state, looking at `PATH` as the shell would.
pub fn status() -> Result<AliasStatus, CliError> {
    let exe = current_exe()?;
    let on_path: Vec<Found> = candidates(std::env::var_os("PATH"))
        .into_iter()
        .map(|p| Found {
            kind: classify(&p, &exe),
            path: p.display().to_string(),
        })
        .collect();
    let state = match on_path.first().map(|f| f.kind) {
        None => State::NotInstalled,
        Some(Kind::Ours) => State::Installed,
        Some(Kind::Outdated) => State::Outdated,
        Some(Kind::Other) if on_path.iter().any(|f| f.kind != Kind::Other) => State::Shadowed,
        Some(Kind::Other) => State::Taken,
    };
    Ok(AliasStatus {
        alias: ALIAS,
        state,
        on_path,
    })
}

pub fn install(dir: Option<&Path>) -> Result<Installed, CliError> {
    let exe = current_exe()?;
    let dir = match dir {
        Some(d) => d.canonicalize()?,
        None => default_dir(&exe),
    };
    let target = dir.join(file_name());

    // Never take the name from another program (CLI-018).
    let st = status()?;
    if let Some(other) = st
        .on_path
        .iter()
        .find(|f| f.kind == Kind::Other && Path::new(&f.path) != target)
    {
        return Err(Error::Conflict(format!(
            "another command called `sj` is already on your PATH ({}); not installing the alias. \
             Use `scrubjay`, or a shell alias with another name",
            other.path
        ))
        .into());
    }
    let dir_on_path = std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).any(|d| d.canonicalize().is_ok_and(|d| d == dir)))
        .unwrap_or(false);

    if std::fs::symlink_metadata(&target).is_ok() {
        match classify(&target, &exe) {
            Kind::Ours => {
                return Ok(Installed {
                    path: target.display().to_string(),
                    method: method_of(&target),
                    changed: false,
                    dir_on_path,
                });
            }
            Kind::Outdated => std::fs::remove_file(&target)?,
            Kind::Other => {
                return Err(Error::Conflict(format!(
                    "{} already exists and isn't scrubjay; not replacing it",
                    target.display()
                ))
                .into());
            }
        }
    }
    let method = create(&exe, &dir, &target)?;
    Ok(Installed {
        path: target.display().to_string(),
        method,
        changed: true,
        dir_on_path,
    })
}

fn method_of(target: &Path) -> &'static str {
    match std::fs::symlink_metadata(target) {
        Ok(m) if m.file_type().is_symlink() => "symlink",
        _ => "hard_link",
    }
}

#[cfg(unix)]
fn create(exe: &Path, dir: &Path, target: &Path) -> Result<&'static str, CliError> {
    // Relative when side by side, so moving the folder keeps the alias working.
    let link: PathBuf = if exe.parent() == Some(dir) {
        exe.file_name()
            .map(PathBuf::from)
            .unwrap_or_else(|| exe.into())
    } else {
        exe.to_path_buf()
    };
    std::os::unix::fs::symlink(link, target).map_err(|e| unwritable(dir, e))?;
    Ok("symlink")
}

#[cfg(not(unix))]
fn create(exe: &Path, dir: &Path, target: &Path) -> Result<&'static str, CliError> {
    // Symlinks need extra rights on Windows; a hard link is just as fast. Across drives,
    // copy (and `doctor` says when the copy is older than scrubjay).
    if std::fs::hard_link(exe, target).is_ok() {
        return Ok("hard_link");
    }
    std::fs::copy(exe, target).map_err(|e| unwritable(dir, e))?;
    Ok("copy")
}

fn unwritable(dir: &Path, e: std::io::Error) -> CliError {
    if e.kind() == std::io::ErrorKind::PermissionDenied {
        CliError::Usage(format!(
            "can't write to {}; choose a folder on your PATH with --dir, or add `alias sj=scrubjay` to your shell",
            dir.display()
        ))
    } else {
        e.into()
    }
}

pub fn uninstall(dir: Option<&Path>) -> Result<Removed, CliError> {
    let exe = current_exe()?;
    let dir = match dir {
        Some(d) => d.canonicalize()?,
        None => default_dir(&exe),
    };
    let target = dir.join(file_name());
    let path = target.display().to_string();
    if std::fs::symlink_metadata(&target).is_err() {
        return Ok(Removed {
            path,
            removed: false,
        });
    }
    if classify(&target, &exe) == Kind::Other {
        return Err(
            Error::Conflict(format!("{path} isn't scrubjay's alias; not removing it")).into(),
        );
    }
    std::fs::remove_file(&target)?;
    Ok(Removed {
        path,
        removed: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidates_follow_path_order() {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        for d in [&a, &b] {
            let p = d.path().join(file_name());
            std::fs::write(&p, "#!/bin/sh\n").unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
            }
        }
        let path = std::env::join_paths([b.path(), a.path()]).unwrap();
        let found = candidates(Some(path));
        assert_eq!(found.len(), 2);
        assert!(found[0].starts_with(b.path()));
    }

    #[cfg(unix)]
    #[test]
    fn files_that_arent_executable_dont_count() {
        let a = tempfile::tempdir().unwrap();
        std::fs::write(a.path().join(ALIAS), "notes").unwrap();
        assert!(candidates(Some(a.path().as_os_str().to_owned())).is_empty());
    }

    #[test]
    fn classify_tells_copies_and_other_programs_apart() {
        let d = tempfile::tempdir().unwrap();
        let exe = d.path().join("scrubjay");
        std::fs::write(&exe, b"binary scrubjay.cli/1 v2").unwrap();
        let exe = exe.canonicalize().unwrap();
        let copy = d.path().join("copy");
        std::fs::write(&copy, b"binary scrubjay.cli/1 v2").unwrap();
        let old = d.path().join("old");
        std::fs::write(&old, b"binary scrubjay.cli/1 v1").unwrap();
        let other = d.path().join("other");
        std::fs::write(&other, b"#!/bin/sh\necho hi\n").unwrap();
        assert_eq!(classify(&copy, &exe), Kind::Ours);
        assert_eq!(classify(&old, &exe), Kind::Outdated);
        assert_eq!(classify(&other, &exe), Kind::Other);
    }
}
