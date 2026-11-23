# Usage ping

Keepjay can send an anonymous **usage ping** once a week, so we can tell whether people
keep using it. It's **off unless you turn it on**, and Keepjay works the same either way
(D-040).

## Turning it on or off

Keepjay asks you once, the first time you run `keepjay source add` or `keepjay init` in a
terminal; the answer defaults to no. Scripts, `--json` output and other commands never ask,
and nothing turns it on except your answer.

```sh
keepjay telemetry status   # on or off, and exactly what would be sent
keepjay telemetry on
keepjay telemetry off      # also deletes this installation's random id
```

Setting `KEEPJAY_NO_TELEMETRY=1` or `DO_NOT_TRACK=1` stops it whatever the setting says.

## What it sends

One small JSON message, with exactly these fields:

| Field | Example | What it is |
|---|---|---|
| `schema` | `1` | The version of this list |
| `install` | `3f0c2a9e-…` | A random id made when you turn the ping on and deleted when you turn it off. It isn't linked to your device id, account, Git identity or anything else |
| `version` | `0.2.0-alpha.4` | Keepjay's version |
| `os` | `macos` | `macos`, `linux`, `windows` or `other` |
| `reviews` | `10-49` | How many cards you reviewed in the last 7 days, as a range: `0`, `1-9`, `10-49` or `50+` |
| `sources` | `true` | Whether you've registered any source |

It never sends card content, deck, tag or file names, paths, repository details, exact
counts or times. `keepjay telemetry status` prints the message as it would be sent.

## When and where

- **When:** at most once a week (weeks are ISO weeks, in UTC). After a command finishes,
  Keepjay starts a small background copy of itself to send the ping, so no command waits on
  the network. If sending fails, it tries again at most every 6 hours.
- **Where:** `https://ping.keepjay.com/v1/ping`, a Cloudflare Worker run by the Keepjay
  project. It stores one row per id per week (the fields above and the week) and doesn't
  store your IP address. Like any web server, Cloudflare sees the address the request comes
  from while handling it.
- **Stored locally:** your answer, the random id and when the ping was last sent, in
  `telemetry.toml` in your data folder ([PRIVACY.md](../PRIVACY.md)).

## Changes

This list changes only in a new Keepjay version, with the change noted in the
[changelog](../CHANGELOG.md) and a new `schema` number.
