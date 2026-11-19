// SPDX-License-Identifier: AGPL-3.0-or-later
//! Domain primitives shared by every module (docs/specs/09-data-model.md §1).

mod hlc;
mod id;

pub use hlc::{Hlc, HlcStamp};
pub use id::Id;

/// Current UTC time in Unix milliseconds (09 §1: timestamps are UTC ms).
pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
