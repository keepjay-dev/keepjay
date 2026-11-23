// SPDX-License-Identifier: AGPL-3.0-or-later
//! The opt-in usage ping end to end (D-040): off by default, the documented payload when
//! on, at most once a week, never when an environment override is set. A local mock
//! endpoint stands in for ping.keepjay.com.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::process::{Command, Output};
use std::sync::mpsc;
use std::time::Duration;

/// A one-thread HTTP server that answers 204 and reports each request body.
fn mock_endpoint() -> (String, mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/v1/ping", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut len = 0;
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
                    break;
                }
                if let Some(v) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                    len = v.trim().parse().unwrap_or(0);
                }
            }
            let mut body = vec![0; len];
            let _ = reader.read_exact(&mut body);
            let _ = stream.write_all(b"HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n");
            let _ = tx.send(String::from_utf8_lossy(&body).into_owned());
        }
    });
    (url, rx)
}

fn keepjay(dir: &Path, url: &str, args: &[&str], env: &[(&str, &str)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_keepjay"));
    cmd.args(args)
        .env("KEEPJAY_DATA_DIR", dir)
        .env("KEEPJAY_PING_URL", url)
        .env_remove("KEEPJAY_NO_TELEMETRY")
        .env_remove("DO_NOT_TRACK");
    for (k, v) in env {
        cmd.env(k, v);
    }
    cmd.output().expect("run keepjay")
}

fn json(out: &Output) -> serde_json::Value {
    assert!(out.status.success(), "{out:?}");
    serde_json::from_slice(&out.stdout).expect("JSON")
}

const QUIET: Duration = Duration::from_millis(1500);
const PATIENT: Duration = Duration::from_secs(15);

#[test]
fn off_by_default_and_nothing_is_sent() {
    let dir = tempfile::tempdir().unwrap();
    let (url, rx) = mock_endpoint();
    // Not a terminal, so init doesn't ask; nothing may be inferred from that.
    json(&keepjay(dir.path(), &url, &["init", "--json"], &[]));
    json(&keepjay(dir.path(), &url, &["status", "--json"], &[]));

    let v = json(&keepjay(
        dir.path(),
        &url,
        &["telemetry", "status", "--json"],
        &[],
    ));
    assert_eq!(v["data"]["enabled"], false);
    assert_eq!(v["data"]["asked"], false);
    assert!(v["data"]["payload"].is_null());
    assert!(rx.recv_timeout(QUIET).is_err(), "no request while off");

    let out = keepjay(dir.path(), &url, &["telemetry", "send", "--json"], &[]);
    assert_eq!(out.status.code(), Some(4), "send refuses while off");
}

#[test]
fn on_sends_the_documented_payload_once_a_week() {
    let dir = tempfile::tempdir().unwrap();
    let (url, rx) = mock_endpoint();
    json(&keepjay(dir.path(), &url, &["init", "--json"], &[]));
    json(&keepjay(
        dir.path(),
        &url,
        &["telemetry", "on", "--json"],
        &[],
    ));
    assert!(
        rx.recv_timeout(QUIET).is_err(),
        "`telemetry` itself never pings"
    );

    // `status` previews exactly the payload.
    let v = json(&keepjay(
        dir.path(),
        &url,
        &["telemetry", "status", "--json"],
        &[],
    ));
    let preview = v["data"]["payload"].clone();
    let fields: Vec<&String> = preview.as_object().unwrap().keys().collect();
    assert_eq!(
        fields,
        ["install", "os", "reviews", "schema", "sources", "version"]
    );
    assert_eq!(preview["schema"], 1);
    assert_eq!(preview["reviews"], "0");
    assert_eq!(preview["sources"], false);
    assert_eq!(preview["version"], env!("CARGO_PKG_VERSION"));

    // Any other command starts a detached sender; the command itself returns at once.
    json(&keepjay(dir.path(), &url, &["status", "--json"], &[]));
    let body = rx.recv_timeout(PATIENT).expect("a ping");
    let sent: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(sent, preview);

    // The sender records the week once the endpoint accepts it.
    let deadline = std::time::Instant::now() + PATIENT;
    loop {
        let v = json(&keepjay(
            dir.path(),
            &url,
            &["telemetry", "status", "--json"],
            &[],
        ));
        if v["data"]["last_sent_week"]
            .as_str()
            .is_some_and(|w| w.contains("-W"))
        {
            break;
        }
        assert!(std::time::Instant::now() < deadline, "week never recorded");
        std::thread::sleep(Duration::from_millis(100));
    }
    // Then nothing more this week.
    json(&keepjay(dir.path(), &url, &["status", "--json"], &[]));
    assert!(rx.recv_timeout(QUIET).is_err(), "one ping a week");

    // `send` sends now regardless.
    let v = json(&keepjay(
        dir.path(),
        &url,
        &["telemetry", "send", "--json"],
        &[],
    ));
    assert_eq!(v["data"]["sent"], true);
    assert!(rx.recv_timeout(PATIENT).is_ok());
}

#[test]
fn environment_overrides_always_win() {
    for var in ["KEEPJAY_NO_TELEMETRY", "DO_NOT_TRACK"] {
        let dir = tempfile::tempdir().unwrap();
        let (url, rx) = mock_endpoint();
        json(&keepjay(dir.path(), &url, &["init", "--json"], &[]));
        json(&keepjay(
            dir.path(),
            &url,
            &["telemetry", "on", "--json"],
            &[],
        ));
        json(&keepjay(
            dir.path(),
            &url,
            &["status", "--json"],
            &[(var, "1")],
        ));
        assert!(rx.recv_timeout(QUIET).is_err(), "{var} blocks the ping");
        let v = json(&keepjay(
            dir.path(),
            &url,
            &["telemetry", "status", "--json"],
            &[(var, "1")],
        ));
        assert_eq!(v["data"]["disabled_by"], var);
    }
}

#[test]
fn off_forgets_the_install_id() {
    let dir = tempfile::tempdir().unwrap();
    let (url, _rx) = mock_endpoint();
    json(&keepjay(
        dir.path(),
        &url,
        &["telemetry", "on", "--json"],
        &[],
    ));
    let first = json(&keepjay(
        dir.path(),
        &url,
        &["telemetry", "status", "--json"],
        &[],
    ))["data"]["payload"]["install"]
        .clone();
    json(&keepjay(
        dir.path(),
        &url,
        &["telemetry", "off", "--json"],
        &[],
    ));
    let file = std::fs::read_to_string(dir.path().join("telemetry.toml")).unwrap();
    assert!(!file.contains("install_id"), "{file}");
    json(&keepjay(
        dir.path(),
        &url,
        &["telemetry", "on", "--json"],
        &[],
    ));
    let second = json(&keepjay(
        dir.path(),
        &url,
        &["telemetry", "status", "--json"],
        &[],
    ))["data"]["payload"]["install"]
        .clone();
    assert_ne!(first, second);
}
