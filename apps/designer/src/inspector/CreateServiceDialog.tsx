import { useMemo, useState } from "react";
import type {
  EditCommand,
  Instance,
  ModuleDefinition,
  ReviewOptions,
} from "../types";
import { defaultDraftFor, parseDraft } from "./configDraft";
import { isLocalId, moduleKey, requiredProperties } from "./instanceEditing";
import { ConfigField } from "./InstanceActions";

interface CreateServiceDialogProps {
  instances: Instance[];
  modules: ModuleDefinition[];
  suspended?: boolean;
  onCancel: () => void;
  onReview: (
    commands: EditCommand[],
    description: string,
    options?: ReviewOptions,
  ) => void;
}

export default function CreateServiceDialog({
  instances,
  modules,
  suspended = false,
  onCancel,
  onReview,
}: CreateServiceDialogProps) {
  const candidates = useMemo(
    () => modules.filter((module) => module.kind === "primitive"),
    [modules],
  );
  const existingIds = useMemo(
    () => new Set(instances.map((instance) => instance.id)),
    [instances],
  );
  const [moduleKeyValue, setModuleKeyValue] = useState(
    candidates[0] ? moduleKey(candidates[0]) : "",
  );
  const selectedModule = candidates.find(
    (module) => moduleKey(module) === moduleKeyValue,
  );
  const [instanceId, setInstanceId] = useState(() => {
    const base = candidates[0]?.id.split(".").pop() ?? "service";
    return existingIds.has(base) ? `${base}-2` : base;
  });
  const [config, setConfig] = useState<Record<string, string>>(() => {
    const initial: Record<string, string> = {};
    const first = candidates[0];
    if (first) {
      for (const property of requiredProperties(first)) {
        initial[property.id] = defaultDraftFor(property);
      }
    }
    return initial;
  });
  const [localError, setLocalError] = useState<string | null>(null);
  const required = selectedModule ? requiredProperties(selectedModule) : [];

  function changeModule(key: string) {
    setModuleKeyValue(key);
    const module = candidates.find((candidate) => moduleKey(candidate) === key);
    const next: Record<string, string> = {};
    if (module) {
      for (const property of requiredProperties(module)) {
        next[property.id] = defaultDraftFor(property);
      }
    }
    setConfig(next);
    setLocalError(null);
  }

  function submit() {
    if (!selectedModule) {
      setLocalError("Choose a primitive module for the service.");
      return;
    }
    const id = instanceId.trim();
    if (!isLocalId(id)) {
      setLocalError("Service id must be a local id (lowercase, digits, - or _).");
      return;
    }
    if (existingIds.has(id)) {
      setLocalError(`Instance ${id} already exists.`);
      return;
    }
    const values: Record<string, unknown> = {};
    for (const property of required) {
      const parsed = parseDraft(config[property.id] ?? "", property);
      if ("error" in parsed) {
        setLocalError(`${property.id}: ${parsed.error}`);
        return;
      }
      values[property.id] = parsed.value;
    }
    setLocalError(null);
    onReview(
      [
        {
          type: "create_instance",
          id,
          module: selectedModule.id,
          version: selectedModule.version,
          config: values,
          scope: "service",
        },
      ],
      `Create service ${id} (${selectedModule.id}@${selectedModule.version})`,
      { selectAfter: id, focusAfter: true },
    );
  }

  const interfaces = selectedModule?.contract.interfaces ?? [];

  return (
    <div
      className={`dialog-veil ${suspended ? "is-suspended" : ""}`}
      aria-hidden={suspended || undefined}
    >
      <section
        className="dialog-card dialog-card-wide"
        role="dialog"
        aria-modal="true"
        aria-labelledby="create-service-title"
      >
        <div className="dialog-head">
          <div>
            <div className="eyebrow">New service instance</div>
            <h2 id="create-service-title">Create service</h2>
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

        {candidates.length === 0 ? (
          <div className="dialog-note">
            No primitive modules are loaded. A service must use a primitive
            module.
          </div>
        ) : (
          <>
            <div className="dialog-fields">
              <label htmlFor="service-module">Module</label>
              <select
                id="service-module"
                value={moduleKeyValue}
                onChange={(event) => changeModule(event.target.value)}
              >
                {candidates.map((module) => (
                  <option key={moduleKey(module)} value={moduleKey(module)}>
                    {module.id}@{module.version}
                  </option>
                ))}
              </select>

              <label htmlFor="service-id">Instance id</label>
              <input
                id="service-id"
                value={instanceId}
                spellCheck={false}
                autoComplete="off"
                onChange={(event) => {
                  setInstanceId(event.target.value);
                  setLocalError(null);
                }}
              />
            </div>

            {interfaces.length > 0 && (
              <div className="dialog-note">
                Implements:{" "}
                {interfaces
                  .map(
                    (declaration) =>
                      `${declaration.id}@${declaration.version}`,
                  )
                  .join(", ")}
              </div>
            )}

            {required.length > 0 && (
              <>
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
                      value={config[property.id] ?? ""}
                      onChange={(value) => {
                        setConfig((current) => ({
                          ...current,
                          [property.id]: value,
                        }));
                        setLocalError(null);
                      }}
                    />
                  </div>
                ))}
              </>
            )}
          </>
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
            Review service
          </button>
        </div>
        <p className="dialog-note">
          Services are created with <code>scope = "service"</code> and are not
          bound into the structure tree. Core still validates the module kind,
          config and dependencies.
        </p>
      </section>
    </div>
  );
}
