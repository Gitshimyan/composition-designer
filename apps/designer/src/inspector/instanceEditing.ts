import type {
  Binding,
  Connection,
  DependencyResolution,
  EditCommand,
  Instance,
  ModuleDefinition,
  PropertyContract,
  SlotContract,
} from "../types";
import { isConfigValueCompatible } from "./configDraft";

const LOCAL_ID = /^[a-z][a-z0-9_-]*$/;

export function isLocalId(value: string): boolean {
  return LOCAL_ID.test(value);
}

export function moduleKey(module: { id: string; version: string }): string {
  return `${module.id}@${module.version}`;
}

export function moduleOptions(modules: ModuleDefinition[]): ModuleDefinition[] {
  return [...modules].sort((a, b) => moduleKey(a).localeCompare(moduleKey(b)));
}

export function findModule(
  modules: ModuleDefinition[],
  id: string,
  version: string,
): ModuleDefinition | undefined {
  return modules.find(
    (module) => module.id === id && module.version === version,
  );
}

export function requiredProperties(
  module: ModuleDefinition,
): PropertyContract[] {
  return module.contract.properties.filter((property) => property.required);
}

/**
 * 能否在创建后立即通过校验：模块不得声明必填 Slot，否则需要先创建其子结构
 * （属于 023 的 Slot 感知结构编辑）。
 */
export function isCreatableStandalone(module: ModuleDefinition): boolean {
  return module.contract.slots.every((slot) => !slot.required);
}

export function creatableModules(
  modules: ModuleDefinition[],
  accepts: string[],
): ModuleDefinition[] {
  return moduleOptions(modules).filter(
    (module) =>
      accepts.includes(module.id) && isCreatableStandalone(module),
  );
}

export function slotOccupancy(
  bindings: Binding[],
  parentId: string,
  slotId: string,
): number {
  return bindings.filter(
    (binding) => binding.parent === parentId && binding.slot === slotId,
  ).length;
}

/** 容量判断只用于交互提示，合法性仍由 Core 的 Slot cardinality 校验决定。 */
export function slotHasCapacity(
  slot: SlotContract,
  count: number,
): boolean {
  if (slot.cardinality === "many") return true;
  return count === 0;
}

export function parentOf(
  bindings: Binding[],
  instanceId: string,
): string | null {
  return bindings.find((binding) => binding.child === instanceId)?.parent ?? null;
}

export function childrenOf(
  bindings: Binding[],
  instanceId: string,
): string[] {
  return bindings
    .filter((binding) => binding.parent === instanceId)
    .map((binding) => binding.child);
}

/** 目标实例 + 全部后代，先序。 */
export function subtreeIds(
  bindings: Binding[],
  instanceId: string,
): string[] {
  const ids: string[] = [];
  const stack = [instanceId];
  const seen = new Set<string>();
  while (stack.length > 0) {
    const current = stack.pop() as string;
    if (seen.has(current)) continue;
    seen.add(current);
    ids.push(current);
    for (const child of childrenOf(bindings, current).reverse()) {
      stack.push(child);
    }
  }
  return ids;
}

export interface DeleteImpact {
  /** 删除事务里逐个 DeleteInstance 的顺序：先子后父。 */
  order: string[];
  instances: string[];
  bindings: Binding[];
  connections: Connection[];
  dependencyLinks: DependencyResolution[];
}

export function deleteImpact(
  targetId: string,
  bindings: Binding[],
  connections: Connection[],
  dependencies: DependencyResolution[],
): DeleteImpact {
  const ids = subtreeIds(bindings, targetId);
  const removed = new Set(ids);
  const ordered = postOrder(bindings, targetId);
  return {
    order: ordered,
    instances: ids,
    bindings: bindings.filter(
      (binding) =>
        removed.has(binding.parent) || removed.has(binding.child),
    ),
    connections: connections.filter(
      (connection) =>
        removed.has(connection.from.instance) ||
        removed.has(connection.to.instance),
    ),
    dependencyLinks: dependencies.filter(
      (dependency) =>
        dependency.provider_instance !== undefined &&
        dependency.provider_instance !== null &&
        removed.has(dependency.provider_instance) &&
        !removed.has(dependency.from_instance),
    ),
  };
}

function postOrder(bindings: Binding[], instanceId: string): string[] {
  const result: string[] = [];
  const visit = (id: string) => {
    for (const child of childrenOf(bindings, id)) {
      visit(child);
    }
    result.push(id);
  };
  visit(instanceId);
  return result;
}

export function deleteCommands(impact: DeleteImpact): EditCommand[] {
  return impact.order.map((id) => ({ type: "delete_instance" as const, id }));
}

export function suggestInstanceId(
  moduleId: string,
  existing: Set<string>,
): string {
  const base = moduleId.split(".").pop() || "instance";
  if (!existing.has(base)) return base;
  let index = 2;
  while (existing.has(`${base}-${index}`)) index += 1;
  return `${base}-${index}`;
}

export function suggestBindingId(
  instanceId: string,
  existing: Set<string>,
): string {
  const base = `bind-${instanceId}`;
  if (!existing.has(base)) return base;
  let index = 2;
  while (existing.has(`${base}-${index}`)) index += 1;
  return `${base}-${index}`;
}

export interface ModuleChangePlan {
  target: ModuleDefinition;
  /** 目标契约认识且类型兼容，保留但不需要 SetConfig 的配置。 */
  keep: Record<string, unknown>;
  /** 目标契约不认识或类型不兼容，需要 UnsetConfig 的键。 */
  unset: string[];
  /** 目标契约必填、但当前没有可用值，需要用户填写。 */
  missing: PropertyContract[];
  /** 目标契约中可选的、需要重置为默认的键。 */
  reset: string[];
}

export function planModuleChange(
  instance: Instance,
  target: ModuleDefinition,
): ModuleChangePlan {
  const properties = new Map(
    target.contract.properties.map((property) => [property.id, property]),
  );
  const keep: Record<string, unknown> = {};
  const unset: string[] = [];
  const reset: string[] = [];

  for (const [key, value] of Object.entries(instance.config ?? {})) {
    const property = properties.get(key);
    if (!property) {
      unset.push(key);
      continue;
    }
    if (isConfigValueCompatible(value, property)) {
      keep[key] = value;
    } else if (property.required) {
      unset.push(key);
    } else {
      reset.push(key);
    }
  }

  const missing = target.contract.properties.filter(
    (property) =>
      property.required && !Object.prototype.hasOwnProperty.call(keep, property.id),
  );

  return { target, keep, unset, missing, reset };
}

export function moduleChangeCommands(
  instanceId: string,
  plan: ModuleChangePlan,
  values: Record<string, unknown>,
): EditCommand[] {
  const commands: EditCommand[] = [
    {
      type: "update_instance_module",
      id: instanceId,
      module: plan.target.id,
      version: plan.target.version,
    },
  ];
  for (const property of plan.unset) {
    commands.push({ type: "unset_config", instance: instanceId, property });
  }
  for (const property of plan.reset) {
    commands.push({ type: "unset_config", instance: instanceId, property });
  }
  for (const property of plan.missing) {
    commands.push({
      type: "set_config",
      instance: instanceId,
      property: property.id,
      value: values[property.id],
    });
  }
  return commands;
}

export function describeModuleChange(
  instanceId: string,
  target: ModuleDefinition,
): string {
  return `Change ${instanceId} module to ${moduleKey(target)}`;
}
