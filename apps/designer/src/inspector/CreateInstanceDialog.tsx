import { useMemo, useRef, useState } from "react";
import type {
  Binding,
  EditCommand,
  Instance,
  ModuleDefinition,
  ReviewOptions,
  SlotContract,
} from "../types";
import { defaultDraftFor, parseDraft } from "./configDraft";
import {
  acceptedModules,
  isLocalId,
  moduleKey,
  requiredProperties,
  requiredSlots,
  slotOccupancy,
  suggestBindingId,
  suggestInstanceId,
} from "./instanceEditing";
import { ConfigField } from "./InstanceActions";

interface CreateInstanceDialogProps {
  parent: Instance;
  slot: SlotContract;
  instances: Instance[];
  bindings: Binding[];
  modules: ModuleDefinition[];
  suspended?: boolean;
  onCancel: () => void;
  onReview: (
    commands: EditCommand[],
    description: string,
    options?: ReviewOptions,
  ) => void;
}

interface DraftNode {
  key: string;
  moduleKey: string;
  instanceId: string;
  bindingId: string;
  config: Record<string, string>;
  children: DraftChild[];
}

interface DraftChild {
  slot: SlotContract;
  node: DraftNode | null;
}

/** 防止自引用/递归模块契约让向导无限展开；超过深度后必须由用户选择更浅的模块。 */
const MAX_DEPTH = 6;

export default function CreateInstanceDialog({
  parent,
  slot,
  instances,
  bindings,
  modules,
  suspended = false,
  onCancel,
  onReview,
}: CreateInstanceDialogProps) {
  const existingInstanceIds = useMemo(
    () => new Set(instances.map((instance) => instance.id)),
    [instances],
  );
  const existingBindingIds = useMemo(
    () => new Set(bindings.map((binding) => binding.id)),
    [bindings],
  );
  const rootCandidates = useMemo(
    () => acceptedModules(modules, slot),
    [modules, slot],
  );
  const keyCounter = useRef(0);
  const newKey = () => `draft-${keyCounter.current++}`;
  const moduleByKey = (key: string) =>
    modules.find((candidate) => moduleKey(candidate) === key);

  const [root, setRoot] = useState<DraftNode | null>(() => {
    const first = rootCandidates[0];
    if (!first) return null;
    const usedInstances = new Set(existingInstanceIds);
    const usedBindings = new Set(existingBindingIds);
    return buildNode(
      first,
      modules,
      usedInstances,
      usedBindings,
      newKey,
      0,
    );
  });
  const [localError, setLocalError] = useState<string | null>(null);

  function updateNode(
    key: string,
    update: (node: DraftNode) => DraftNode,
  ) {
    setRoot((current) => (current ? mapTree(current, key, update) : current));
    setLocalError(null);
  }

  function changeModule(key: string, nextKey: string) {
    const nextModule = moduleByKey(nextKey);
    if (!nextModule) return;
    setRoot((current) => {
      if (!current) return current;
      const usedInstances = new Set(existingInstanceIds);
      const usedBindings = new Set(existingBindingIds);
      collectIds(current, usedInstances, usedBindings, key);
      return mapTree(current, key, (node) => ({
        ...node,
        moduleKey: nextKey,
        config: rebuildConfig(node.config, nextModule),
        children: buildChildren(
          nextModule,
          modules,
          usedInstances,
          usedBindings,
          newKey,
          1,
        ),
      }));
    });
    setLocalError(null);
  }

  function submit() {
    if (!root) {
      setLocalError("Choose a module.");
      return;
    }
    const errors: string[] = [];
    const parsedConfig = new Map<string, Record<string, unknown>>();
    const seenInstances = new Set<string>();
    const seenBindings = new Set<string>();
    let count = 0;

    const walk = (node: DraftNode) => {
      count += 1;
      const module = moduleByKey(node.moduleKey);
      if (!module) {
        errors.push(`${node.instanceId}: module is not loaded.`);
        return;
      }
      const instanceId = node.instanceId.trim();
      const bindingId = node.bindingId.trim();
      if (!isLocalId(instanceId)) {
        errors.push(`${node.instanceId || "(empty)"}: invalid instance id.`);
      } else if (
        existingInstanceIds.has(instanceId) ||
        seenInstances.has(instanceId)
      ) {
        errors.push(`Instance ${instanceId} already exists.`);
      } else {
        seenInstances.add(instanceId);
      }
      if (!isLocalId(bindingId)) {
        errors.push(`${node.bindingId || "(empty)"}: invalid binding id.`);
      } else if (
        existingBindingIds.has(bindingId) ||
        seenBindings.has(bindingId)
      ) {
        errors.push(`Binding ${bindingId} already exists.`);
      } else {
        seenBindings.add(bindingId);
      }

      const config: Record<string, unknown> = {};
      for (const property of requiredProperties(module)) {
        const parsed = parseDraft(node.config[property.id] ?? "", property);
        if ("error" in parsed) {
          errors.push(`${instanceId || node.key}.${property.id}: ${parsed.error}`);
        } else {
          config[property.id] = parsed.value;
        }
      }
      parsedConfig.set(node.key, config);

      for (const child of node.children) {
        if (!child.node) {
          errors.push(
            `${instanceId}.${child.slot.id}: required slot has no accepted module.`,
          );
        } else {
          walk(child.node);
        }
      }
    };
    walk(root);

    if (errors.length > 0) {
      setLocalError(
        errors.length > 1
          ? `${errors[0]} (+${errors.length - 1} more)`
          : errors[0],
      );
      return;
    }

    const instanceCommands: EditCommand[] = [];
    const bindingCommands: EditCommand[] = [];
    const emit = (node: DraftNode) => {
      const module = moduleByKey(node.moduleKey);
      if (!module) return;
      instanceCommands.push({
        type: "create_instance",
        id: node.instanceId.trim(),
        module: module.id,
        version: module.version,
        config: parsedConfig.get(node.key) ?? {},
      });
      for (const child of node.children) {
        if (!child.node) continue;
        emit(child.node);
        bindingCommands.push({
          type: "create_binding",
          id: child.node.bindingId.trim(),
          parent: node.instanceId.trim(),
          slot: child.slot.id,
          child: child.node.instanceId.trim(),
        });
      }
    };
    emit(root);
    bindingCommands.push({
      type: "create_binding",
      id: root.bindingId.trim(),
      parent: parent.id,
      slot: slot.id,
      child: root.instanceId.trim(),
    });

    setLocalError(null);
    onReview(
      [...instanceCommands, ...bindingCommands],
      count > 1
        ? `Create ${root.instanceId.trim()} subtree (${count} instances) under ${parent.id}.${slot.id}`
        : `Create ${root.instanceId.trim()} under ${parent.id}.${slot.id}`,
      { selectAfter: root.instanceId.trim(), focusAfter: true },
    );
  }

  return (
    <div
      className={`dialog-veil ${suspended ? "is-suspended" : ""}`}
      aria-hidden={suspended || undefined}
    >
      <section
        className="dialog-card dialog-card-wide"
        role="dialog"
        aria-modal="true"
        aria-labelledby="create-instance-title"
      >
        <div className="dialog-head">
          <div>
            <div className="eyebrow">New child structure</div>
            <h2 id="create-instance-title">
              {parent.id} · {slot.id}
            </h2>
          </div>
          <button
            type="button"
            className="dialog-close"
            aria-label="Close"
            onClick={onCancel}
          >
            ×
          </button>
        </div>

        <div className="dialog-slot-meta">
          <span>cardinality {slot.cardinality}</span>
          <span>{slot.required ? "required" : "optional"}</span>
          <span>{slotOccupancy(bindings, parent.id, slot.id)} bound</span>
          <span>accepts {slot.accepts.join(", ")}</span>
        </div>

        {root ? (
          <DraftFields
            node={root}
            slot={slot}
            modules={modules}
            root
            onChangeModule={changeModule}
            onChangeInstanceId={(key, value) =>
              updateNode(key, (node) => ({
                ...node,
                instanceId: value,
                bindingId: node.bindingId.startsWith(`bind-${node.instanceId}`)
                  ? suggestBindingId(value, existingBindingIds)
                  : node.bindingId,
              }))
            }
            onChangeBindingId={(key, value) =>
              updateNode(key, (node) => ({ ...node, bindingId: value }))
            }
            onChangeConfig={(key, property, value) =>
              updateNode(key, (node) => ({
                ...node,
                config: { ...node.config, [property]: value },
              }))
            }
          />
        ) : (
          <div className="dialog-note">
            No module accepted by this slot is available.
          </div>
        )}

        {localError && <div className="config-error">{localError}</div>}

        <div className="dialog-actions">
          <button type="button" className="button button-ghost" onClick={onCancel}>
            Cancel
          </button>
          <button
            type="button"
            className="button button-primary"
            disabled={!root}
            onClick={submit}
          >
            Review create
          </button>
        </div>
        <p className="dialog-note">
          Required slots are expanded recursively. Instances and Bindings are
          created in one Core transaction; nothing is written until you apply
          the review.
        </p>
      </section>
    </div>
  );
}

function DraftFields({
  node,
  slot,
  modules,
  root = false,
  onChangeModule,
  onChangeInstanceId,
  onChangeBindingId,
  onChangeConfig,
}: {
  node: DraftNode;
  slot: SlotContract;
  modules: ModuleDefinition[];
  root?: boolean;
  onChangeModule: (key: string, moduleKey: string) => void;
  onChangeInstanceId: (key: string, value: string) => void;
  onChangeBindingId: (key: string, value: string) => void;
  onChangeConfig: (key: string, property: string, value: string) => void;
}) {
  const module = modules.find(
    (candidate) => moduleKey(candidate) === node.moduleKey,
  );
  const candidates = acceptedModules(modules, slot);
  const required = module ? requiredProperties(module) : [];
  const label = root ? "Root instance" : `Slot ${slot.id} child`;

  if (!module) {
    return (
      <div className="draft-missing">
        Module {node.moduleKey} is not loaded.
      </div>
    );
  }

  return (
    <fieldset className={`draft-node ${root ? "is-root" : ""}`} aria-label={label}>
      <legend>
        <code>{slot.id} → {moduleKey(module)}</code>
        <span className="draft-slot-meta">
          {slot.cardinality}
          {slot.required ? " · required" : ""}
        </span>
      </legend>

      <div className="dialog-fields">
        <label htmlFor={`${node.key}-module`}>Module</label>
        <select
          id={`${node.key}-module`}
          aria-label="Module"
          value={node.moduleKey}
          onChange={(event) => onChangeModule(node.key, event.target.value)}
        >
          {candidates.map((candidate) => (
            <option key={moduleKey(candidate)} value={moduleKey(candidate)}>
              {moduleKey(candidate)}
            </option>
          ))}
        </select>

        <label htmlFor={`${node.key}-instance`}>Instance id</label>
        <input
          id={`${node.key}-instance`}
          aria-label="Instance id"
          value={node.instanceId}
          spellCheck={false}
          autoComplete="off"
          onChange={(event) =>
            onChangeInstanceId(node.key, event.target.value)
          }
        />

        <label htmlFor={`${node.key}-binding`}>Binding id</label>
        <input
          id={`${node.key}-binding`}
          aria-label="Binding id"
          value={node.bindingId}
          spellCheck={false}
          autoComplete="off"
          onChange={(event) => onChangeBindingId(node.key, event.target.value)}
        />

        {required.length > 0 && (
          <div className="dialog-config">
            <div className="section-label">Required config</div>
            {required.map((property) => (
              <div className="config-row" key={property.id}>
                <div className="config-label-row">
                  <span className="config-label">{property.id}</span>
                  <span className="config-type">{property.type}</span>
                  <span className="config-required">required</span>
                </div>
                <ConfigField
                  property={property}
                  value={node.config[property.id] ?? ""}
                  onChange={(value) =>
                    onChangeConfig(node.key, property.id, value)
                  }
                />
              </div>
            ))}
          </div>
        )}
      </div>

      {node.children.map((child) => (
        <div className="draft-child" key={child.slot.id}>
          {child.node ? (
            <DraftFields
              node={child.node}
              slot={child.slot}
              modules={modules}
              onChangeModule={onChangeModule}
              onChangeInstanceId={onChangeInstanceId}
              onChangeBindingId={onChangeBindingId}
              onChangeConfig={onChangeConfig}
            />
          ) : (
            <div className="draft-missing">
              Required slot {child.slot.id} has no accepted module. Choose a
              different module above.
            </div>
          )}
        </div>
      ))}
    </fieldset>
  );
}

function buildNode(
  module: ModuleDefinition,
  modules: ModuleDefinition[],
  usedInstances: Set<string>,
  usedBindings: Set<string>,
  newKey: () => string,
  depth: number,
): DraftNode {
  const instanceId = suggestInstanceId(module.id, usedInstances);
  usedInstances.add(instanceId);
  const bindingId = suggestBindingId(instanceId, usedBindings);
  usedBindings.add(bindingId);
  return {
    key: newKey(),
    moduleKey: moduleKey(module),
    instanceId,
    bindingId,
    config: defaultConfig(module),
    children: buildChildren(
      module,
      modules,
      usedInstances,
      usedBindings,
      newKey,
      depth,
    ),
  };
}

function buildChildren(
  module: ModuleDefinition,
  modules: ModuleDefinition[],
  usedInstances: Set<string>,
  usedBindings: Set<string>,
  newKey: () => string,
  depth: number,
): DraftChild[] {
  return requiredSlots(module).map((slot) => {
    const candidates = acceptedModules(modules, slot);
    if (candidates.length === 0 || depth >= MAX_DEPTH) {
      return { slot, node: null };
    }
    return {
      slot,
      node: buildNode(
        candidates[0],
        modules,
        usedInstances,
        usedBindings,
        newKey,
        depth + 1,
      ),
    };
  });
}

function defaultConfig(module: ModuleDefinition): Record<string, string> {
  const config: Record<string, string> = {};
  for (const property of requiredProperties(module)) {
    config[property.id] = defaultDraftFor(property);
  }
  return config;
}

/** 切换模块时保留同名属性草稿，其余按新契约重建。 */
function rebuildConfig(
  previous: Record<string, string>,
  module: ModuleDefinition,
): Record<string, string> {
  const config: Record<string, string> = {};
  for (const property of requiredProperties(module)) {
    config[property.id] = previous[property.id] ?? defaultDraftFor(property);
  }
  return config;
}

function mapTree(
  node: DraftNode,
  key: string,
  update: (node: DraftNode) => DraftNode,
): DraftNode {
  if (node.key === key) return update(node);
  return {
    ...node,
    children: node.children.map((child) =>
      child.node
        ? { ...child, node: mapTree(child.node, key, update) }
        : child,
    ),
  };
}

/**
 * 收集整棵草稿树的 ID；`skipChildrenOf` 指向正在重建的节点，
 * 其旧子树即将被丢弃，因此不占用新 ID。
 */
function collectIds(
  node: DraftNode,
  usedInstances: Set<string>,
  usedBindings: Set<string>,
  skipChildrenOf: string | null = null,
) {
  usedInstances.add(node.instanceId);
  usedBindings.add(node.bindingId);
  if (node.key === skipChildrenOf) return;
  for (const child of node.children) {
    if (child.node) {
      collectIds(child.node, usedInstances, usedBindings, skipChildrenOf);
    }
  }
}
