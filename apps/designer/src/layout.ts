import type { Binding, Instance, NodePosition } from "./types";

const X_GAP = 280;
const Y_GAP = 190;

export interface TreeNode {
  instance: Instance;
  slot?: string;
  children: TreeNode[];
}

export interface Structure {
  root: TreeNode | null;
  orphans: Instance[];
}

export function autoLayout(
  instances: Instance[],
  bindings: Binding[],
  rootId: string,
): Record<string, NodePosition> {
  const children = new Map<string, string[]>();
  for (const binding of bindings) {
    const bucket = children.get(binding.parent) ?? [];
    bucket.push(binding.child);
    children.set(binding.parent, bucket);
  }

  const depth = new Map<string, number>();
  if (instances.some((instance) => instance.id === rootId)) {
    depth.set(rootId, 0);
    const queue = [rootId];
    while (queue.length > 0) {
      const parent = queue.shift() as string;
      const nextDepth = (depth.get(parent) ?? 0) + 1;
      for (const child of children.get(parent) ?? []) {
        if (!depth.has(child) && instances.some((instance) => instance.id === child)) {
          depth.set(child, nextDepth);
          queue.push(child);
        }
      }
    }
  }

  const maxDepth = Math.max(-1, ...depth.values());
  for (const instance of instances) {
    if (!depth.has(instance.id)) {
      depth.set(instance.id, maxDepth + 1);
    }
  }

  const rows = new Map<number, string[]>();
  const ids = instances.map((instance) => instance.id).sort((a, b) => {
    if (a === rootId) return -1;
    if (b === rootId) return 1;
    return a.localeCompare(b);
  });
  for (const id of ids) {
    const level = depth.get(id) ?? 0;
    const bucket = rows.get(level) ?? [];
    bucket.push(id);
    rows.set(level, bucket);
  }

  const positions: Record<string, NodePosition> = {};
  for (const [level, row] of [...rows.entries()].sort(([a], [b]) => a - b)) {
    row.sort((a, b) => {
      if (a === rootId) return -1;
      if (b === rootId) return 1;
      return a.localeCompare(b);
    });
    row.forEach((id, index) => {
      positions[id] = {
        x: (index - (row.length - 1) / 2) * X_GAP,
        y: level * Y_GAP,
      };
    });
  }
  return positions;
}

export function buildStructure(
  instances: Instance[],
  bindings: Binding[],
  rootId: string,
): Structure {
  const byId = new Map(instances.map((instance) => [instance.id, instance]));
  const children = new Map<string, Binding[]>();
  for (const binding of bindings) {
    const bucket = children.get(binding.parent) ?? [];
    bucket.push(binding);
    children.set(binding.parent, bucket);
  }

  const visited = new Set<string>();
  function build(id: string, slot?: string): TreeNode | null {
    const instance = byId.get(id);
    if (!instance || visited.has(id)) return null;
    visited.add(id);
    const branch = (children.get(id) ?? [])
      .map((binding) => build(binding.child, binding.slot))
      .filter((node): node is TreeNode => node !== null);
    return { instance, slot, children: branch };
  }

  const root = byId.has(rootId) ? build(rootId) : null;
  const orphans = instances.filter((instance) => !visited.has(instance.id));
  return { root, orphans };
}
