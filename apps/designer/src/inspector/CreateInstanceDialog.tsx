import { useEffect, useMemo, useState } from "react";
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
  creatableModules,
  isLocalId,
  moduleKey,
  requiredProperties,
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
  const candidates = useMemo(
    () => creatableModules(modules, slot.accepts),
    [modules, slot.accepts],
  );
  const existingInstanceIds = useMemo(
    () => new Set(instances.map((instance) => instance.id)),
    [instances],
  );
  const existingBindingIds = useMemo(
    () => new Set(bindings.map((binding) => binding.id)),
    [bindings],
  );
  const [selectedKey, setSelectedKey] = useState(
    candidates[0] ? moduleKey(candidates[0]) : "",
  );
  const selected = candidates.find(
    (candidate) => moduleKey(candidate) === selectedKey,
  );
  const [instanceId, setInstanceId] = useState(() =>
    suggestInstanceId(candidates[0]?.id ?? "instance", existingInstanceIds),
  );
  const [bindingId, setBindingId] = useState(() =>
    suggestBindingId(
      suggestInstanceId(candidates[0]?.id ?? "instance", existingInstanceIds),
      existingBindingIds,
    ),
  );
  const [idTouched, setIdTouched] = useState(false);
  const [bindingTouched, setBindingTouched] = useState(false);
  const [drafts, setDrafts] = useState<Record<string, string>>({});
  const [localError, setLocalError] = useState<string | null>(null);

  useEffect(() => {
    if (!selected) return;
    if (!idTouched) {
      const next = suggestInstanceId(selected.id, existingInstanceIds);
      setInstanceId(next);
      if (!bindingTouched) {
        setBindingId(suggestBindingId(next, existingBindingIds));
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedKey]);

  const required = selected ? requiredProperties(selected) : [];

  useEffect(() => {
    setDrafts((current) => {
      const next: Record<string, string> = {};
      for (const property of required) {
        next[property.id] =
          current[property.id] ?? defaultDraftFor(property);
      }
      return next;
    });
    setLocalError(null);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedKey]);

  function submit() {
    const nextInstanceId = instanceId.trim();
    const nextBindingId = bindingId.trim();
    if (!selected) {
      setLocalError("Choose an accepted module.");
      return;
    }
    if (!isLocalId(nextInstanceId)) {
      setLocalError(
        "Instance id must start with a lowercase letter and use a-z, 0-9, - or _.",
      );
      return;
    }
    if (existingInstanceIds.has(nextInstanceId)) {
      setLocalError(`Instance ${nextInstanceId} already exists.`);
      return;
    }
    if (!isLocalId(nextBindingId)) {
      setLocalError(
        "Binding id must start with a lowercase letter and use a-z, 0-9, - or _.",
      );
      return;
    }
    if (existingBindingIds.has(nextBindingId)) {
      setLocalError(`Binding ${nextBindingId} already exists.`);
      return;
    }

    const config: Record<string, unknown> = {};
    for (const property of required) {
      const parsed = parseDraft(drafts[property.id] ?? "", property);
      if ("error" in parsed) {
        setLocalError(`${property.id}: ${parsed.error}`);
        return;
      }
      config[property.id] = parsed.value;
    }

    setLocalError(null);
    onReview(
      [
        {
          type: "create_instance",
          id: nextInstanceId,
          module: selected.id,
          version: selected.version,
          config,
        },
        {
          type: "create_binding",
          id: nextBindingId,
          parent: parent.id,
          slot: slot.id,
          child: nextInstanceId,
        },
      ],
      `Create ${nextInstanceId} under ${parent.id}.${slot.id}`,
      { selectAfter: nextInstanceId, focusAfter: true },
    );
  }

  return (
    <div
      className={`dialog-veil ${suspended ? "is-suspended" : ""}`}
      aria-hidden={suspended || undefined}
    >
      <section
        className="dialog-card"
        role="dialog"
        aria-modal="true"
        aria-labelledby="create-instance-title"
      >
        <div className="dialog-head">
          <div>
            <div className="eyebrow">New child instance</div>
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
          <span>accepts {slot.accepts.join(", ")}</span>
        </div>

        {candidates.length === 0 ? (
          <div className="dialog-note">
            No accepted module can be created standalone in this slot. Modules
            with required slots need their own children first.
          </div>
        ) : (
          <div className="dialog-fields">
            <label htmlFor="create-instance-module">Module</label>
            <select
              id="create-instance-module"
              value={selectedKey}
              onChange={(event) => setSelectedKey(event.target.value)}
            >
              {candidates.map((candidate) => (
                <option key={moduleKey(candidate)} value={moduleKey(candidate)}>
                  {moduleKey(candidate)}
                </option>
              ))}
            </select>

            <label htmlFor="create-instance-id">Instance id</label>
            <input
              id="create-instance-id"
              value={instanceId}
              spellCheck={false}
              autoComplete="off"
              onChange={(event) => {
                setIdTouched(true);
                setInstanceId(event.target.value);
                if (!bindingTouched) {
                  setBindingId(
                    suggestBindingId(event.target.value, existingBindingIds),
                  );
                }
              }}
            />

            <label htmlFor="create-binding-id">Binding id</label>
            <input
              id="create-binding-id"
              value={bindingId}
              spellCheck={false}
              autoComplete="off"
              onChange={(event) => {
                setBindingTouched(true);
                setBindingId(event.target.value);
              }}
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
                      value={drafts[property.id] ?? ""}
                      onChange={(value) =>
                        setDrafts((current) => ({
                          ...current,
                          [property.id]: value,
                        }))
                      }
                    />
                  </div>
                ))}
              </div>
            )}
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
            disabled={candidates.length === 0}
            onClick={submit}
          >
            Review create
          </button>
        </div>
        <p className="dialog-note">
          Creates the instance and its first Binding in one Core transaction.
          Nothing is written until you apply the review.
        </p>
      </section>
    </div>
  );
}
