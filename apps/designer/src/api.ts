import { invoke } from "@tauri-apps/api/core";
import type {
  EditResponse,
  ExternalChangeState,
  LayoutNodes,
  NodePosition,
  ProjectSnapshot,
  SourcePayload,
  Transaction,
} from "./types";

export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export async function openProject(
  path: string,
  profile?: string | null,
): Promise<ProjectSnapshot> {
  return invoke<ProjectSnapshot>("open_project", {
    path,
    profile: profile ?? null,
  });
}

export async function initialProject(): Promise<string | null> {
  if (!isTauri()) return null;
  return invoke<string | null>("initial_project");
}

export async function initialProfile(): Promise<string | null> {
  if (!isTauri()) return null;
  return invoke<string | null>("initial_profile");
}

export async function loadLayout(root: string): Promise<LayoutNodes> {
  return invoke<Record<string, NodePosition>>("load_layout", { root });
}

export async function saveLayout(root: string, nodes: LayoutNodes): Promise<void> {
  await invoke("save_layout", { root, nodes });
}

export async function prepareTransaction(
  transaction: Transaction,
): Promise<EditResponse> {
  return invoke<EditResponse>("prepare_transaction", { transaction });
}

export async function applyTransaction(
  transaction: Transaction,
): Promise<EditResponse> {
  return invoke<EditResponse>("apply_transaction", { transaction });
}

export async function readSource(): Promise<SourcePayload> {
  return invoke<SourcePayload>("read_source");
}

export async function checkExternalChanges(): Promise<ExternalChangeState> {
  return invoke<ExternalChangeState>("check_external_changes");
}

export async function reloadProject(): Promise<ProjectSnapshot> {
  return invoke<ProjectSnapshot>("reload_project");
}
