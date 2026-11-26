import { invoke } from "@tauri-apps/api/core";
import type { LayoutNodes, NodePosition, ProjectSnapshot } from "./types";

export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export async function openProject(path: string): Promise<ProjectSnapshot> {
  return invoke<ProjectSnapshot>("open_project", { path });
}

export async function initialProject(): Promise<string | null> {
  if (!isTauri()) return null;
  return invoke<string | null>("initial_project");
}

export async function loadLayout(root: string): Promise<LayoutNodes> {
  return invoke<Record<string, NodePosition>>("load_layout", { root });
}

export async function saveLayout(root: string, nodes: LayoutNodes): Promise<void> {
  await invoke("save_layout", { root, nodes });
}
