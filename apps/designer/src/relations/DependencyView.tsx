import { useMemo } from "react";
import type {
  DependencyResolution,
  Instance,
  ModuleDefinition,
} from "../types";
import { moduleKey } from "../inspector/instanceEditing";
import { dependencyDetails, type DependencyDetail } from "./relationModel";

interface DependencyViewProps {
  instances: Instance[];
  modules: ModuleDefinition[];
  dependencies: DependencyResolution[];
  selectedId: string | null;
  canEdit: boolean;
  onSelectInstance: (id: string) => void;
  onSetProvider: (detail: DependencyDetail) => void;
  onClearProvider: (detail: DependencyDetail) => void;
}

export default function DependencyView({
  instances,
  modules,
  dependencies,
  selectedId,
  canEdit,
  onSelectInstance,
  onSetProvider,
  onClearProvider,
}: DependencyViewProps) {
  const details = useMemo(
    () => dependencyDetails(instances, modules, dependencies),
    [instances, modules, dependencies],
  );
  const groups = useMemo(() => {
    const byInstance = new Map<string, typeof details>();
    for (const detail of details) {
      const bucket = byInstance.get(detail.resolution.from_instance) ?? [];
      bucket.push(detail);
      byInstance.set(detail.resolution.from_instance, bucket);
    }
    return [...byInstance.entries()];
  }, [details]);

  const resolved = dependencies.filter(
    (dependency) => dependency.status === "resolved",
  ).length;
  const explicitCount = details.filter((detail) => detail.explicit).length;

  return (
    <div className="relation-view">
      <div className="relation-toolbar">
        <div className="relation-toolbar-copy">
          <strong>
            {dependencies.length} dependency resolutions · {resolved} resolved ·{" "}
            {explicitCount} explicit
          </strong>
          <span>
            Requirements, candidates and providers come from the same
            Composition Model; explicit mappings are written through Core
            transactions.
          </span>
        </div>
        <span className="relation-badge">provider editing</span>
      </div>

      <div className="relation-scroll">
        {groups.length === 0 ? (
          <div className="relation-empty">
            <span className="status-glyph">◇</span>
            <span>
              No module dependency requirements are declared by this project.
            </span>
          </div>
        ) : (
          groups.map(([instanceId, group]) => {
            const first = group[0];
            return (
              <section
                className={`dependency-group ${
                  selectedId === instanceId ? "is-selected" : ""
                }`}
                key={instanceId}
              >
                <button
                  type="button"
                  className="dependency-group-head"
                  onClick={() => onSelectInstance(instanceId)}
                >
                  <code>{instanceId}</code>
                  <span>
                    {first.module ? moduleKey(first.module) : "module not loaded"}
                  </span>
                  <span className="dependency-group-count">
                    {group.length} requirement{group.length === 1 ? "" : "s"}
                  </span>
                </button>

                {group.map((detail) => {
                  const requirement = detail.requirement;
                  const provider = detail.resolution.provider_instance;
                  const statusClass = detail.resolution.status.replace(
                    /_/g,
                    "-",
                  );
                  return (
                    <div
                      className="dependency-row"
                      key={detail.resolution.requirement_id}
                    >
                      <div className="dependency-row-main">
                        <code>{detail.resolution.requirement_id}</code>
                        <span className="dependency-target">
                          {requirement
                            ? `${requirement.module}${
                                requirement.version
                                  ? `@${requirement.version}`
                                  : ""
                              }`
                            : "unknown target"}
                        </span>
                        <span
                          className={`dependency-required ${
                            requirement?.optional ? "is-optional" : ""
                          }`}
                        >
                          {requirement?.optional ? "optional" : "required"}
                        </span>
                        <span className={`dep-status is-${statusClass}`}>
                          {detail.resolution.status}
                        </span>
                      </div>
                      <div className="dependency-row-meta">
                        <span>
                          provider:{" "}
                          <code>{provider ?? "—"}</code>
                          {provider
                            ? detail.explicit
                              ? " · explicit"
                              : " · auto"
                            : ""}
                        </span>
                        <span>
                          candidates:{" "}
                          {detail.candidates.length > 0
                            ? detail.candidates.join(", ")
                            : "none"}
                        </span>
                      </div>
                      <div className="dependency-row-actions">
                        <button
                          type="button"
                          className="config-action"
                          disabled={
                            !canEdit || detail.candidates.length === 0
                          }
                          onClick={() => onSetProvider(detail)}
                        >
                          Set provider…
                        </button>
                        {detail.explicit && (
                          <button
                            type="button"
                            className="config-action is-muted"
                            disabled={!canEdit}
                            onClick={() => onClearProvider(detail)}
                          >
                            Clear explicit
                          </button>
                        )}
                      </div>
                    </div>
                  );
                })}
              </section>
            );
          })
        )}

        <div className="relation-note">
          Setting an explicit provider writes{" "}
          <code>[instances.dependencies]</code> through Core; clearing it falls
          back to automatic resolution. Unknown targets, wrong modules,
          self-provision and ambiguity are still rejected by Core validation.
        </div>
      </div>
    </div>
  );
}
