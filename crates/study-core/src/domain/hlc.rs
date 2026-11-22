// SPDX-License-Identifier: AGPL-3.0-or-later
//! Hybrid logical clock (D-009, docs/specs/08-architecture.md §9).
//!
//! Text form `<physical ms, 13 digits>-<counter, 4 digits>-<device short id>` sorts
//! lexicographically in causal order, which is what per-field last-writer-wins needs.

use std::fmt;

use super::Id;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hlc {
    physical_ms: i64,
    counter: u32,
}

/// One HLC reading, stamped with the device that produced it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct HlcStamp {
    pub physical_ms: i64,
    pub counter: u32,
    pub device: String,
}

impl Hlc {
    /// Advance the clock for a local event at wall-clock time `now_ms`.
    pub fn tick(&mut self, now_ms: i64, device: Id) -> HlcStamp {
        if now_ms > self.physical_ms {
            self.physical_ms = now_ms;
            self.counter = 0;
        } else {
            self.counter += 1;
        }
        HlcStamp {
            physical_ms: self.physical_ms,
            counter: self.counter,
            device: device.short(),
        }
    }

    /// Merge a remote stamp (used by sync in Phase 2) so later local ticks sort after it.
    pub fn observe(&mut self, remote_physical_ms: i64, remote_counter: u32, now_ms: i64) {
        let max_phys = self.physical_ms.max(remote_physical_ms).max(now_ms);
        self.counter = if max_phys == self.physical_ms && max_phys == remote_physical_ms {
            self.counter.max(remote_counter) + 1
        } else if max_phys == self.physical_ms {
            self.counter + 1
        } else if max_phys == remote_physical_ms {
            remote_counter + 1
        } else {
            0
        };
        self.physical_ms = max_phys;
    }
}

// @card(jrz6zf9b) Why does an HLC stamp zero-pad its physical time and counter (`{:013}-{:04}`)?
// So the text sorts in causal order as a plain string, which per-field last-writer-wins
// relies on.
impl fmt::Display for HlcStamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:013}-{:04}-{}",
            self.physical_ms, self.counter, self.device
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monotonic_even_if_wall_clock_stalls_or_goes_back() {
        let dev = Id::new();
        let mut c = Hlc::default();
        let a = c.tick(1_000, dev).to_string();
        let b = c.tick(1_000, dev).to_string();
        let d = c.tick(900, dev).to_string(); // clock went backwards
        let e = c.tick(2_000, dev).to_string();
        assert!(a < b && b < d && d < e, "{a} {b} {d} {e}");
    }

    #[test]
    fn observe_moves_past_remote() {
        let dev = Id::new();
        let mut c = Hlc::default();
        c.tick(1_000, dev);
        c.observe(5_000, 3, 1_100);
        let next = c.tick(1_200, dev);
        assert_eq!((next.physical_ms, next.counter), (5_000, 5));
    }
}
