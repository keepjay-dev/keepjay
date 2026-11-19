// SPDX-License-Identifier: AGPL-3.0-or-later
// Typed wrappers around the Tauri commands in src-tauri/src/main.rs.
// These types mirror study_core; they'll be generated from Rust in M1 (08 §3).
import { invoke } from "@tauri-apps/api/core";

export interface Status {
  core_version: string;
  data_dir: string;
  db_path: string;
  schema_version: number;
  device_id: string;
  workspaces: number;
}

export interface Workspace {
  id: string;
  short_id: string;
  name: string;
  created_at: number;
}

export interface CoreError {
  code: "unexpected" | "not_found" | "validation" | "conflict" | "version_incompatible";
  message: string;
}

export const core = {
  status: () => invoke<Status>("status"),
  listWorkspaces: () => invoke<Workspace[]>("list_workspaces"),
  createWorkspace: (name: string) => invoke<Workspace>("create_workspace", { name }),
  dataVersion: () => invoke<number>("data_version"),
};

export function errorMessage(e: unknown): string {
  if (e && typeof e === "object" && "message" in e) return String((e as CoreError).message);
  return String(e);
}
