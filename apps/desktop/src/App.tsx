// SPDX-License-Identifier: AGPL-3.0-or-later
import { useCallback, useEffect, useRef, useState, type FormEvent } from "react";
import { core, errorMessage, type Status, type Workspace } from "./core";

/** How often to check whether another process (e.g. the CLI) changed the data (08 §8). */
const POLL_MS = 1000;

export function App() {
  const [status, setStatus] = useState<Status | null>(null);
  const [workspaces, setWorkspaces] = useState<Workspace[]>([]);
  const [name, setName] = useState("");
  const [error, setError] = useState<string | null>(null);
  const lastVersion = useRef<number | null>(null);

  const refresh = useCallback(async () => {
    try {
      const [s, ws] = await Promise.all([core.status(), core.listWorkspaces()]);
      setStatus(s);
      setWorkspaces(ws);
    } catch (e) {
      setError(errorMessage(e));
    }
  }, []);

  useEffect(() => {
    void refresh();
    const timer = setInterval(async () => {
      try {
        const v = await core.dataVersion();
        if (lastVersion.current !== null && v !== lastVersion.current) void refresh();
        lastVersion.current = v;
      } catch {
        /* transient: try again next tick */
      }
    }, POLL_MS);
    return () => clearInterval(timer);
  }, [refresh]);

  async function create(e: FormEvent) {
    e.preventDefault();
    setError(null);
    try {
      await core.createWorkspace(name);
      setName("");
      await refresh();
    } catch (err) {
      setError(errorMessage(err));
    }
  }

  return (
    <main className="shell">
      <header>
        <h1>Scrubjay</h1>
        <p className="muted">Scaffold build: the desktop shell talking to the shared Rust core.</p>
      </header>

      <section className="card" aria-labelledby="ws-heading">
        <h2 id="ws-heading">Workspaces</h2>
        {workspaces.length === 0 ? (
          <p className="muted">
            No workspaces yet. Create one here, or run <code>scrubjay init</code> in a terminal and
            watch it appear.
          </p>
        ) : (
          <ul className="list">
            {workspaces.map((w) => (
              <li key={w.id}>
                <span>{w.name}</span>
                <code className="muted">{w.short_id}</code>
              </li>
            ))}
          </ul>
        )}
        <form onSubmit={create} className="row">
          <label htmlFor="ws-name" className="visually-hidden">
            New workspace name
          </label>
          <input
            id="ws-name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="New workspace name"
          />
          <button type="submit" disabled={!name.trim()}>
            Create
          </button>
        </form>
        {error && (
          <p role="alert" className="error">
            {error}
          </p>
        )}
      </section>

      {status && (
        <section className="card" aria-labelledby="status-heading">
          <h2 id="status-heading">Status</h2>
          <dl className="grid">
            <dt>Core</dt>
            <dd>v{status.core_version}</dd>
            <dt>Schema</dt>
            <dd>v{status.schema_version}</dd>
            <dt>Data</dt>
            <dd>
              <code>{status.data_dir}</code>
            </dd>
            <dt>Device</dt>
            <dd>
              <code>{status.device_id.slice(-8)}</code>
            </dd>
          </dl>
        </section>
      )}
    </main>
  );
}
