---
tags: [lang::rust, sqlite]
---

# Rust and SQLite in this codebase

Cards with code in them, taken from how Keepjay is built. `keepjay scan` picks them up.

Q: What can go wrong with this, and how does `replace_file` (`sources/writeback.rs`) avoid it?
```rust
std::fs::write(path, contents)?;
```
A: A crash or power cut halfway leaves the file truncated. Write a temp file in the same folder, flush it, then rename it over the original; the rename is atomic:
```rust
let mut f = File::create(&tmp)?;
f.write_all(contents.as_bytes())?;
f.sync_all()?;
std::fs::rename(&tmp, path)?;
```

Q: What does each of these settings do in `storage::open`?
```rust
conn.busy_timeout(Duration::from_millis(5000))?;
conn.pragma_update(None, "journal_mode", "WAL")?;
conn.pragma_update(None, "synchronous", "NORMAL")?;
conn.pragma_update(None, "foreign_keys", "ON")?;
```
A: `busy_timeout`: wait up to 5 s for a lock instead of failing with `SQLITE_BUSY`. `WAL`: readers and the writer don't block each other. `NORMAL`: in WAL mode, safe from corruption, though a power cut can lose the last commits. `foreign_keys`: SQLite enforces foreign keys only when this is on, per connection.

C: SQLite enforces foreign keys only after {{c1::`PRAGMA foreign_keys = ON`}}, and the setting lasts for {{c2::one connection}}.

Q: Why do writes here begin with `TransactionBehavior::Immediate`, not the default?
```rust
let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
```
A: A default (deferred) transaction takes the write lock only at its first write. If two connections read and then both try to write, one gets `SQLITE_BUSY` straight away, and `busy_timeout` can't help. `Immediate` takes the write lock at `BEGIN`, so the other connection simply waits.

Q: Why does `MediaStore::put` read one byte more than the limit?
```rust
io::copy(&mut reader.take(max_bytes.saturating_add(1)), &mut w)?;
if w.written > max_bytes { /* too large */ }
```
A: Reading `max_bytes + 1` is enough to tell that a file is too large without reading the rest of it. `saturating_add` stops `u64::MAX + 1` overflowing.

Q: Which Rust feature is this, and what does it need?
```rust
if let Some(w) = &mut s.watch
    && w.child.try_wait()?.is_some()
{
    return Ok(Input::CommandExited);
}
```
A: A let chain: `if let` patterns and boolean conditions joined with `&&`. It needs edition 2024 (Rust 1.88 or later).

Q: Why does `flush_to_disk` call `rustix::fs::fsync` on Apple platforms instead of `File::sync_all`?
```rust
#[cfg(target_vendor = "apple")]
{ rustix::fs::fsync(f).map_err(std::io::Error::from) }
#[cfg(not(target_vendor = "apple"))]
{ f.sync_all() }
```
A: On macOS, `sync_all` issues `F_FULLFSYNC`, which also flushes the drive's cache: about 4 ms a file, over 4 s for a first scan writing ids into 1,000 files. Plain `fsync` matches what `sync_all` guarantees elsewhere, as Git does by default.
