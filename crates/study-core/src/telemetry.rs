// SPDX-License-Identifier: AGPL-3.0-or-later
//! The opt-in weekly usage ping: consent, schedule and payload (D-040, NFR-PRIV-002).
//!
//! This module never touches the network; the CLI sends what [`Core::usage_ping`] builds.
//! The state lives in `<data_dir>/telemetry.toml`, next to `device.toml`, so it belongs to
//! this installation and never syncs. Every field sent is listed in `docs/TELEMETRY.md`.

use std::path::Path;

use jiff::Timestamp;
use jiff::tz::TimeZone;
use rusqlite::OptionalExtension;
use serde::Serialize;

use crate::Core;
use crate::error::Result;

const FILE: &str = "telemetry.toml";

/// Version of the payload's fields; the endpoint checks it.
pub const SCHEMA: u32 = 1;

/// The endpoint, unless `KEEPJAY_PING_URL` names another (for testing).
pub const ENDPOINT: &str = "https://ping.keepjay.com/v1/ping";

/// How long a failed send waits before it's tried again.
const RETRY_MS: i64 = 6 * 60 * 60 * 1000;

/// This installation's answer and schedule.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Settings {
    /// The user's answer; `None` if they haven't been asked. Never set any other way.
    pub enabled: Option<bool>,
    /// Random UUIDv4 made on opting in and deleted on opting out; not the device id.
    pub install_id: Option<String>,
    /// ISO week (UTC) of the last ping the endpoint accepted, e.g. `2026-W48`.
    pub last_sent_week: Option<String>,
    /// When a send last started (UTC ms).
    pub last_attempt_ms: Option<i64>,
}

impl Settings {
    pub fn load(data_dir: &Path) -> Settings {
        // Unreadable or malformed means off: never send on a guess.
        std::fs::read_to_string(data_dir.join(FILE))
            .map(|text| parse(&text))
            .unwrap_or_default()
    }

    pub fn save(&self, data_dir: &Path) -> Result<()> {
        let mut body = String::from(
            "# Keepjay usage ping (docs/TELEMETRY.md). Change it with `keepjay telemetry on|off`.\n",
        );
        if let Some(on) = self.enabled {
            body.push_str(&format!("enabled = {on}\n"));
        }
        if let Some(id) = &self.install_id {
            body.push_str(&format!("install_id = \"{id}\"\n"));
        }
        if let Some(week) = &self.last_sent_week {
            body.push_str(&format!("last_sent_week = \"{week}\"\n"));
        }
        if let Some(ms) = self.last_attempt_ms {
            body.push_str(&format!("last_attempt_ms = {ms}\n"));
        }
        let path = data_dir.join(FILE);
        let tmp = data_dir.join(format!(".{FILE}.tmp"));
        std::fs::write(&tmp, body)?;
        std::fs::rename(&tmp, &path)?;
        Ok(())
    }

    /// Record the user's answer. Opting out forgets the install id.
    pub fn set_enabled(&mut self, on: bool) {
        self.enabled = Some(on);
        if on {
            if self.install_id.is_none() {
                self.install_id = Some(uuid::Uuid::new_v4().to_string());
            }
        } else {
            *self = Settings {
                enabled: Some(false),
                ..Settings::default()
            };
        }
    }

    /// Whether the user said yes (an environment override aside).
    pub fn is_on(&self) -> bool {
        self.enabled == Some(true) && self.install_id.is_some()
    }

    /// Whether a ping should start now: on, not yet accepted this week, and no attempt in
    /// the last 6 hours.
    pub fn due(&self, now_ms: i64) -> bool {
        self.is_on()
            && self.last_sent_week.as_deref() != Some(iso_week(now_ms).as_str())
            && self
                .last_attempt_ms
                .is_none_or(|at| now_ms - at >= RETRY_MS || now_ms < at)
    }
}

/// The environment variable that turns the ping off whatever the settings say, if set.
pub fn env_override() -> Option<&'static str> {
    let set = |name: &str| std::env::var_os(name).is_some_and(|v| !v.is_empty() && v != "0");
    ["KEEPJAY_NO_TELEMETRY", "DO_NOT_TRACK"]
        .into_iter()
        .find(|name| set(name))
}

/// The endpoint to send to.
pub fn endpoint() -> String {
    std::env::var("KEEPJAY_PING_URL")
        .ok()
        .filter(|u| !u.is_empty())
        .unwrap_or_else(|| ENDPOINT.to_string())
}

/// ISO 8601 week in UTC, e.g. `2026-W48`.
pub fn iso_week(now_ms: i64) -> String {
    let ts = Timestamp::from_millisecond(now_ms).unwrap_or(Timestamp::UNIX_EPOCH);
    let date = TimeZone::UTC.to_datetime(ts).date().iso_week_date();
    format!("{}-W{:02}", date.year(), date.week())
}

/// What one ping sends: exactly these fields (docs/TELEMETRY.md).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Ping {
    pub schema: u32,
    pub install: String,
    pub version: String,
    /// `macos`, `linux`, `windows` or `other`.
    pub os: &'static str,
    /// Reviews in the last 7 days, as a bucket: `0`, `1-9`, `10-49` or `50+`.
    pub reviews: &'static str,
    /// Whether any source is registered.
    pub sources: bool,
}

fn os_family() -> &'static str {
    match std::env::consts::OS {
        os @ ("macos" | "linux" | "windows") => os,
        _ => "other",
    }
}

fn bucket(reviews: i64) -> &'static str {
    match reviews {
        ..=0 => "0",
        1..=9 => "1-9",
        10..=49 => "10-49",
        _ => "50+",
    }
}

impl Core {
    /// The ping this installation would send now, or `None` if the user hasn't opted in.
    pub fn usage_ping(&self, version: &str, now_ms: i64) -> Result<Option<Ping>> {
        let settings = Settings::load(&self.paths.data_dir);
        let Some(install) = settings
            .install_id
            .filter(|_| settings.enabled == Some(true))
        else {
            return Ok(None);
        };
        let week_ago = now_ms - 7 * 24 * 60 * 60 * 1000;
        let reviews: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM review_logs WHERE reviewed_at >= ?1",
            [week_ago],
            |r| r.get(0),
        )?;
        let sources = self
            .conn
            .query_row(
                "SELECT 1 FROM sources WHERE deleted_at IS NULL LIMIT 1",
                [],
                |r| r.get::<_, i64>(0),
            )
            .optional()?
            .is_some();
        Ok(Some(Ping {
            schema: SCHEMA,
            install,
            version: version.to_string(),
            os: os_family(),
            reviews: bucket(reviews),
            sources,
        }))
    }
}

fn parse(text: &str) -> Settings {
    let mut s = Settings::default();
    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim();
        let string = || {
            value
                .strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .map(str::to_string)
        };
        match key.trim() {
            "enabled" => s.enabled = value.parse().ok(),
            "install_id" => s.install_id = string().filter(|v| uuid::Uuid::parse_str(v).is_ok()),
            "last_sent_week" => s.last_sent_week = string(),
            "last_attempt_ms" => s.last_attempt_ms = value.parse().ok(),
            _ => {}
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    // 2026-11-23 12:00 UTC, a Monday in ISO week 48.
    const MON: i64 = 1_795_435_200_000;
    const HOUR: i64 = 60 * 60 * 1000;

    #[test]
    fn iso_weeks() {
        assert_eq!(iso_week(MON), "2026-W48");
        assert_eq!(iso_week(MON - 13 * HOUR), "2026-W47"); // Sunday 23:00
        // 2027-01-01 is a Friday in ISO week 53 of 2026.
        assert_eq!(iso_week(1_798_761_600_000), "2026-W53");
    }

    #[test]
    fn buckets() {
        let got: Vec<_> = [0, 1, 9, 10, 49, 50, 5000].map(bucket).into();
        assert_eq!(got, ["0", "1-9", "1-9", "10-49", "10-49", "50+", "50+"]);
    }

    #[test]
    fn off_until_the_user_says_yes() {
        let dir = tempfile::tempdir().unwrap();
        let s = Settings::load(dir.path());
        assert_eq!(s, Settings::default());
        assert!(!s.is_on() && !s.due(MON));
    }

    #[test]
    fn opting_in_and_out_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = Settings::default();
        s.set_enabled(true);
        s.last_attempt_ms = Some(MON);
        s.last_sent_week = Some("2026-W47".into());
        s.save(dir.path()).unwrap();
        let loaded = Settings::load(dir.path());
        assert_eq!(loaded, s);
        assert!(loaded.is_on());

        let id = loaded.install_id.clone().unwrap();
        assert_eq!(uuid::Uuid::parse_str(&id).unwrap().get_version_num(), 4);

        let mut off = loaded;
        off.set_enabled(false);
        assert_eq!(
            off,
            Settings {
                enabled: Some(false),
                ..Settings::default()
            }
        );
        off.set_enabled(true);
        assert_ne!(off.install_id.as_deref(), Some(id.as_str()), "a new id");
    }

    #[test]
    fn due_once_a_week_with_retries_every_six_hours() {
        let mut s = Settings::default();
        s.set_enabled(true);
        assert!(s.due(MON));

        s.last_attempt_ms = Some(MON);
        assert!(!s.due(MON + HOUR), "an attempt is in flight or just failed");
        assert!(s.due(MON + 6 * HOUR), "retry after 6 hours");

        s.last_sent_week = Some("2026-W48".into());
        assert!(!s.due(MON + 6 * HOUR), "already sent this week");
        assert!(s.due(MON + 7 * 24 * HOUR), "next week");
    }

    #[test]
    fn malformed_file_means_off() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(FILE),
            "enabled = yes\ninstall_id = \"not-a-uuid\"\n",
        )
        .unwrap();
        assert!(!Settings::load(dir.path()).is_on());
    }
}
