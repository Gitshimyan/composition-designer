import { useEffect, useMemo, useState } from "react";
import type {
  Binding,
  Connection,
  DependencyResolution,
  EditCommand,
  Instance,
  ModuleDefinition,
  PropertyContract,
  ReviewOptions,
  SlotContract,
} from "../types";
import { defaultDraftFor, parseDraft } from "./configDraft";
import {
  acceptedModules,
  childBindings,
  deleteCommands,
  deleteImpact,
  describeModuleChange,
  detachBindingCommands,
  findModule,
  isLocalId,
  moduleChangeCommands,
  moduleKey,
  moduleOptions,
  parentBinding,
  parentOf,
  planModuleChange,
  slotChildren,
  slotHasCapacity,
  slotOccupancy,
} from "./instanceEditing";

interface InstanceActionsProps {
  instance: Instance;
  instances: Instance[];
  bindings: Binding[];
  connections: Connection[];
  dependencies: DependencyResolution[];
  modules: ModuleDefinition[];
  rootId: string;
  canEdit: boolean;
  onCreateChild: (parent: Instance, slot: SlotContract) => void;
  onEditBinding: (binding: Binding) => void;
  onReview: (
    commands: EditCommand[],
    description: string,
    options?: ReviewOptions,
  ) => void;
}

export default function InstanceActions({
  instance,
  instances,
  bindings,
  connections,
  dependencies,
  modules,
  rootId,
  canEdit,
  onCreateChild,
  onEditBinding,
  onReview,
}: InstanceActionsProps) {
  const module = findModule(modules, instance.module.id, instance.module.version);
  const [renameOpen, setRenameOpen] = useState(false);
  const [renameDraft, setRenameDraft] = useState(instance.id);
  const [renameError, setRenameError] = useState<string | null>(null);
  const [deleteOpen, setDeleteOpen] = useState(false);
  const [targetKey, setTargetKey] = useState(moduleKey(instance.module));
  const [moduleDrafts, setModuleDrafts] = useState<Record<string, string>>({});
  const [moduleError, setModuleError] = useState<string | null>(null);

  const targetModule = useMemo(
    () => modules.find((candidate) => moduleKey(candidate) === targetKey),
    [modules, targetKey],
  );
  const plan = useMemo(
    () =>
      targetModule
        ? planModuleChange(instance, targetModule)
        : null,
    [instance, targetModule],
  );

  useEffect(() => {
    if (!plan) {
      setModuleDrafts({});
      return;
    }
    const drafts: Record<string, string> = {};
    for (const property of plan.missing) {
      drafts[property.id] = defaultDraftFor(property);
    }
    setModuleDrafts(drafts);
    setModuleError(null);
    // Recompute only when the selected module changes; `plan` is derived.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [targetKey, instance.id]);

  const impact = useMemo(
    () =>
      deleteImpact(instance.id, bindings, connections, dependencies),
    [instance.id, bindings, connections, dependencies],
  );
  const parentB = parentBinding(bindings, instance.id);
  const childBs = childBindings(bindings, instance.id);
  const isRoot = instance.id === rootId;
  const displayModule = module ? moduleKey(module) : moduleKey(instance.module);

  function submitRename() {
    const value = renameDraft.trim();
    if (!isLocalId(value)) {
      setRenameError("Use a lowercase id: a-z, 0-9, - or _ (must start with a letter).");
      return;
    }
    if (value === instance.id) {
      setRenameError("The new id is unchanged.");
      return;
    }
    if (instances.some((candidate) => candidate.id === value)) {
      setRenameError(`Instance ${value} already exists.`);
      return;
    }
    setRenameError(null);
    setRenameOpen(false);
    onReview(
      [{ type: "rename_instance", from: instance.id, to: value }],
      `Rename ${instance.id} → ${value}`,
      {
        selectAfter: value,
        focusAfter: true,
        alias: { from: instance.id, to: value },
      },
    );
  }

  function reviewModuleChange() {
    if (!plan) return;
    const values: Record<string, unknown> = {};
    for (const property of plan.missing) {
      const parsed = parseDraft(moduleDrafts[property.id] ?? "", property);
      if ("error" in parsed) {
        setModuleError(`${property.id}: ${parsed.error}`);
        return;
      }
      values[property.id] = parsed.value;
    }
    setModuleError(null);
    onReview(
      moduleChangeCommands(instance.id, plan, values),
      describeModuleChange(instance.id, plan.target),
    );
  }

  function reviewDelete() {
    if (isRoot) return;
    setDeleteOpen(false);
    onReview(
      deleteCommands(impact),
      impact.instances.length > 1
        ? `Delete ${instance.id} and ${impact.instances.length - 1} descendant(s)`
        : `Delete ${instance.id}`,
      {
        selectAfter: parentOf(bindings, instance.id) ?? rootId,
        focusAfter: true,
      },
    );
  }

  function reviewDetach(binding: Binding, childId: string) {
    onReview(
      detachBindingCommands(binding.id),
      `Detach ${childId} from ${binding.parent}.${binding.slot}`,
      { selectAfter: childId },
    );
  }

  return (
    <section className="inspector-section instance-actions">
      <div className="section-label">
        Instance actions {!canEdit && <span className="read-only-tag">read-only</span>}
      </div>

      <div className="action-row">
        <button
          type="button"
          className="config-action"
          disabled={!canEdit}
          onClick={() => {
            setRenameDraft(instance.id);
            setRenameError(null);
            setRenameOpen((value) => !value);
          }}
        >
          Rename
        </button>
        <button
          type="button"
          className="config-action is-muted"
          disabled={!canEdit || isRoot}
          title={isRoot ? "The root instance cannot be deleted." : undefined}
          onClick={() => setDeleteOpen((value) => !value)}
        >
          Delete
        </button>
      </div>

      {renameOpen && (
        <div className="action-form">
          <label htmlFor={`rename-${instance.id}`}>New id</label>
          <div className="config-input-row">
            <input
              id={`rename-${instance.id}`}
              aria-label="New instance id"
              value={renameDraft}
              spellCheck={false}
              autoComplete="off"
              onChange={(event) => setRenameDraft(event.target.value)}
            />
            <button type="button" className="config-action" onClick={submitRename}>
              Review
            </button>
          </div>
          {renameError && <div className="config-error">{renameError}</div>}
        </div>
      )}

      {deleteOpen && !isRoot && (
        <div className="action-form impact-panel">
          <div className="impact-title">Delete {instance.id}</div>
          <Row label="Instances" value={impact.instances.join(", ")} />
          <Row
            label="Bindings"
            value={
              impact.bindings.length === 0
                ? "none"
                : impact.bindings.map((binding) => binding.id).join(", ")
            }
          />
          <Row
            label="Connections"
            value={
              impact.connections.length === 0
                ? "none"
                : impact.connections.map((connection) => connection.id).join(", ")
            }
          />
          {impact.dependencyLinks.length > 0 && (
            <Row
              label="Dependencies"
              value={impact.dependencyLinks
                .map(
                  (dependency) =>
                    `${dependency.from_instance}.${dependency.requirement_id}`,
                )
                .join(", ")}
            />
          )}
          <div className="action-row">
            <button type="button" className="config-action" onClick={reviewDelete}>
              Review delete
            </button>
            <button
              type="button"
              className="config-action is-muted"
              onClick={() => setDeleteOpen(false)}
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      {module ? (
        <div className="action-form module-change">
          <label htmlFor={`module-${instance.id}`}>Module · version</label>
          <div className="config-input-row">
            <select
              id={`module-${instance.id}`}
              aria-label="Instance module"
              value={targetKey}
              disabled={!canEdit}
              onChange={(event) => setTargetKey(event.target.value)}
            >
              {moduleOptions(modules).map((candidate) => (
                <option key={moduleKey(candidate)} value={moduleKey(candidate)}>
                  {moduleKey(candidate)}
                </option>
              ))}
              {!modules.some(
                (candidate) => moduleKey(candidate) === displayModule,
              ) && <option value={displayModule}>{displayModule}</option>}
            </select>
          </div>

          {plan && targetKey !== moduleKey(instance.module) && (
            <>
              {plan.unset.length > 0 && (
                <div className="action-note">
                  Drops config: {plan.unset.join(", ")}
                </div>
              )}
              {plan.reset.length > 0 && (
                <div className="action-note">
                  Resets to default: {plan.reset.join(", ")}
                </div>
              )}
              {plan.missing.map((property) => (
                <div className="config-row" key={property.id}>
                  <div className="config-label-row">
                    <span className="config-label">{property.id}</span>
                    <span className="config-type">{property.type}</span>
                    <span className="config-required">required</span>
                  </div>
                  <ConfigField
                    property={property}
                    value={moduleDrafts[property.id] ?? ""}
                    onChange={(value) =>
                      setModuleDrafts((current) => ({
                        ...current,
                        [property.id]: value,
                      }))
                    }
                  />
                </div>
              ))}
              {moduleError && <div className="config-error">{moduleError}</div>}
              <button
                type="button"
                className="config-action"
                disabled={!canEdit}
                onClick={reviewModuleChange}
              >
                Review module change
              </button>
            </>
          )}
        </div>
      ) : (
        <div className="action-note">
          Module {displayModule} is not loaded, so module actions are unavailable.
        </div>
      )}

      {module && module.contract.slots.length > 0 && (
        <div className="slots-block">
          <div className="section-label">Slots</div>
          {module.contract.slots.map((slot) => {
            const count = slotOccupancy(bindings, instance.id, slot.id);
            const capacity = slotHasCapacity(slot, count);
            const children = slotChildren(bindings, instance.id, slot.id);
            const candidates = acceptedModules(modules, slot);
            return (
              <div className="slot-row" key={slot.id}>
                <div className="slot-row-head">
                  <code>{slot.id}</code>
                  <span className={`slot-cardinality is-${slot.cardinality}`}>
                    {slot.cardinality}
                  </span>
                  <span className="slot-count">{count} bound</span>
                  {!capacity && <span className="slot-full">full</span>}
                </div>
                <div className="slot-accepts">
                  accepts {slot.accepts.join(", ")}
                  {children.length > 0 ? ` · ${children.join(", ")}` : ""}
                </div>
                {capacity && canEdit && candidates.length > 0 && (
                  <button
                    type="button"
                    className="config-action"
                    onClick={() => onCreateChild(instance, slot)}
                  >
                    Add child…
                  </button>
                )}
                {capacity && candidates.length === 0 && (
                  <div className="slot-note">
                    No loaded module is accepted by this Slot.
                  </div>
                )}
              </div>
            );
          })}
        </div>
      )}

      {(parentB || childBs.length > 0) && (
        <div className="bindings-block">
          <div className="section-label">Bindings</div>
          {parentB && (
            <BindingRow
              binding={parentB}
              childId={instance.id}
              parentLabel={`${parentB.parent}.${parentB.slot}`}
              canEdit={canEdit}
              onEdit={() => onEditBinding(parentB)}
              onDetach={() => reviewDetach(parentB, instance.id)}
            />
          )}
          {childBs.map((binding) => (
            <BindingRow
              key={binding.id}
              binding={binding}
              childId={binding.child}
              parentLabel={`${binding.parent}.${binding.slot}`}
              canEdit={canEdit}
              onEdit={() => onEditBinding(binding)}
              onDetach={() => reviewDetach(binding, binding.child)}
            />
          ))}
          <div className="slot-note">
            A standalone Detach leaves the child unreachable, so Core rejects
            it; use Move or Delete instead.
          </div>
        </div>
      )}

      <div className="action-note">
        Changes are previewed through Core and written only after Apply.
      </div>
    </section>
  );
}

function BindingRow({
  binding,
  childId,
  parentLabel,
  canEdit,
  onEdit,
  onDetach,
}: {
  binding: Binding;
  childId: string;
  parentLabel: string;
  canEdit: boolean;
  onEdit: () => void;
  onDetach: () => void;
}) {
  return (
    <div className="binding-row">
      <div className="binding-row-head">
        <code>{binding.id}</code>
        <span className="binding-child">→ {childId}</span>
      </div>
      <div className="binding-row-meta">
        <span>{parentLabel}</span>
        <div className="binding-row-actions">
          <button
            type="button"
            className="config-action"
            disabled={!canEdit}
            onClick={onEdit}
          >
            Move…
          </button>
          <button
            type="button"
            className="config-action is-muted"
            disabled={!canEdit}
            onClick={onDetach}
          >
            Detach
          </button>
        </div>
      </div>
    </div>
  );
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div className="property">
      <span>{label}</span>
      <span className="mono">{value}</span>
    </div>
  );
}

export function ConfigField({
  property,
  value,
  onChange,
}: {
  property: PropertyContract;
  value: string;
  onChange: (value: string) => void;
}) {
  const isEnum = property.values.length > 0;
  if (isEnum) {
    return (
      <div className="config-input-row">
        <select
          aria-label={property.id}
          value={value}
          onChange={(event) => onChange(event.target.value)}
        >
          {property.values.map((option) => (
            <option key={option} value={option}>
              {option}
            </option>
          ))}
        </select>
      </div>
    );
  }
  if (property.type === "boolean") {
    return (
      <div className="config-input-row">
        <select
          aria-label={property.id}
          value={value}
          onChange={(event) => onChange(event.target.value)}
        >
          <option value="true">true</option>
          <option value="false">false</option>
        </select>
      </div>
    );
  }
  return (
    <div className="config-input-row">
      <input
        aria-label={property.id}
        type={
          property.type === "integer" || property.type === "float"
            ? "number"
            : "text"
        }
        step={property.type === "float" ? "any" : undefined}
        value={value}
        spellCheck={false}
        autoComplete="off"
        onChange={(event) => onChange(event.target.value)}
      />
    </div>
  );
}
