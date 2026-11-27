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

/** required Slot 至少要有一个子实例，才能在创建后立即合法。 */
export function requiredSlots(module: ModuleDefinition): SlotContract[] {
  return module.contract.slots.filter((slot) => slot.required);
}

/** Slot 契约接受范围内的模块（含自身还带 required Slot 的复合模块）。 */
export function acceptedModules(
  modules: ModuleDefinition[],
  slot: SlotContract,
): ModuleDefinition[] {
  return moduleOptions(modules).filter((module) =>
    slot.accepts.includes(module.id),
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

export function parentBinding(
  bindings: Binding[],
  childId: string,
): Binding | undefined {
  return bindings.find((binding) => binding.child === childId);
}

export function childBindings(
  bindings: Binding[],
  parentId: string,
): Binding[] {
  return bindings.filter((binding) => binding.parent === parentId);
}

export function slotChildren(
  bindings: Binding[],
  parentId: string,
  slotId: string,
): string[] {
  return bindings
    .filter(
      (binding) => binding.parent === parentId && binding.slot === slotId,
    )
    .map((binding) => binding.child);
}

/** 目标实例的全部后代（不含自身），用于移动 Binding 时排除成环的父级候选。 */
export function descendantIds(
  bindings: Binding[],
  instanceId: string,
): Set<string> {
  const result = new Set<string>();
  for (const child of childrenOf(bindings, instanceId)) {
    for (const id of subtreeIds(bindings, child)) {
      result.add(id);
    }
  }
  return result;
}

export interface BindingParentOption {
  instance: Instance;
  module: ModuleDefinition;
  /** 该父模块中接受当前 child 模块的 Slot。 */
  slots: SlotContract[];
}

/**
 * 可把子实例移入的父级：模块已加载、至少有一个 Slot 接受子模块、且不是自身后代。
 * 是否满足 cardinality / 单父级等仍由 Core 判定。
 */
export function bindingParentOptions(
  instances: Instance[],
  bindings: Binding[],
  modules: ModuleDefinition[],
  child: Instance,
): BindingParentOption[] {
  const descendants = descendantIds(bindings, child.id);
  const options: BindingParentOption[] = [];
  for (const instance of instances) {
    if (instance.id === child.id || descendants.has(instance.id)) continue;
    const module = findModule(
      modules,
      instance.module.id,
      instance.module.version,
    );
    if (!module) continue;
    const slots = module.contract.slots.filter((slot) =>
      slot.accepts.includes(child.module.id),
    );
    if (slots.length === 0) continue;
    options.push({ instance, module, slots });
  }
  return options.sort((a, b) => a.instance.id.localeCompare(b.instance.id));
}

export function moveBindingCommands(
  bindingId: string,
  parentId: string,
  slotId: string,
  childId: string,
): EditCommand[] {
  return [
    {
      type: "update_binding",
      id: bindingId,
      parent: parentId,
      slot: slotId,
      child: childId,
    },
  ];
}

export function detachBindingCommands(bindingId: string): EditCommand[] {
  return [{ type: "delete_binding", id: bindingId }];
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
