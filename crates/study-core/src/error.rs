// SPDX-License-Identifier: AGPL-3.0-or-later
//! Core error type. [`ErrorKind`] maps onto the CLI exit codes in docs/specs/06-cli.md §6.3.

use serde::Serialize;

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Stable, machine-readable error categories (also used in CLI JSON `error.code`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    Unexpected,
    NotFound,
    Validation,
    Conflict,
    VersionIncompatible,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Conflict(String),
    #[error(
        "the database schema (v{db}) is newer than this version of Scrubjay supports (v{supported}); please update Scrubjay"
    )]
    SchemaTooNew { db: u32, supported: u32 },
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Unexpected(String),
}

impl Error {
    pub fn validation(msg: impl Into<String>) -> Self {
        Error::Validation(msg.into())
    }

    pub fn conflict(msg: impl Into<String>) -> Self {
        Error::Conflict(msg.into())
    }

    pub fn kind(&self) -> ErrorKind {
        match self {
            Error::NotFound(_) => ErrorKind::NotFound,
            Error::Validation(_) => ErrorKind::Validation,
            Error::Conflict(_) => ErrorKind::Conflict,
            Error::SchemaTooNew { .. } => ErrorKind::VersionIncompatible,
            Error::Db(rusqlite::Error::SqliteFailure(e, _))
                if matches!(
                    e.code,
                    rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked
                ) =>
            {
                ErrorKind::Conflict
            }
            Error::Db(_) | Error::Io(_) | Error::Unexpected(_) => ErrorKind::Unexpected,
        }
    }
}
