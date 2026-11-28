import { useMemo, useState } from "react";
import type {
  DependencyResolution,
  EditCommand,
  Instance,
  ModuleDefinition,
  ReviewOptions,
} from "../types";
import { dependencyDetails, requirementTarget } from "./relationModel";

interface DependencyProviderDialogProps {
  instanceId: string;
  requirementId: string;
  instances: Instance[];
  modules: ModuleDefinition[];
  dependencies: DependencyResolution[];
  suspended?: boolean;
  onCancel: () => void;
  onReview: (
    commands: EditCommand[],
    description: string,
    options?: ReviewOptions,
  ) => void;
}

export default function DependencyProviderDialog({
  instanceId,
  requirementId,
  instances,
  modules,
  dependencies,
  suspended = false,
  onCancel,
  onReview,
}: DependencyProviderDialogProps) {
  const detail = useMemo(
    () =>
      dependencyDetails(instances, modules, dependencies).find(
        (candidate) =>
          candidate.resolution.from_instance === instanceId &&
          candidate.resolution.requirement_id === requirementId,
      ),
    [instances, modules, dependencies, instanceId, requirementId],
  );
  const candidates = detail?.candidates ?? [];
  const current =
    detail?.instance.dependency_providers[requirementId] ?? "";
  const [providerId, setProviderId] = useState(
    current || candidates[0] || "",
  );
  const [localError, setLocalError] = useState<string | null>(null);

  const candidateInstances = candidates
    .map((id) => instances.find((instance) => instance.id === id))
    .filter((instance): instance is Instance => instance !== undefined);

  function submit() {
    if (!detail) {
      setLocalError("This dependency requirement is no longer available.");
      return;
    }
    if (!providerId) {
      setLocalError("Choose a provider instance.");
      return;
    }
    setLocalError(null);
    onReview(
      [
        {
          type: "set_dependency_provider",
          instance: instanceId,
          requirement: requirementId,
          provider: providerId,
        },
      ],
      `Set ${instanceId}.${requirementId} provider to ${providerId}`,
      { selectAfter: instanceId, focusAfter: true },
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
        aria-labelledby="provider-dialog-title"
      >
        <div className="dialog-head">
          <div>
            <div className="eyebrow">Dependency provider</div>
            <h2 id="provider-dialog-title">
              {instanceId} · {requirementId}
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
          <span>
            target {requirementTarget(detail?.requirement)}
          </span>
          <span>
            {detail?.requirement?.optional ? "optional" : "required"}
          </span>
          <span>status {detail?.resolution.status ?? "unknown"}</span>
          {current && <span>current {current} · explicit</span>}
        </div>

        {candidateInstances.length > 0 ? (
          <div className="slot-choice-list">
            <div className="section-label">Provider candidates</div>
            {candidateInstances.map((candidate) => (
              <label className="slot-choice" key={candidate.id}>
                <input
                  type="radio"
                  name="dependency-provider"
                  value={candidate.id}
                  checked={providerId === candidate.id}
                  onChange={() => {
                    setProviderId(candidate.id);
                    setLocalError(null);
                  }}
                />
                <span className="slot-choice-body">
                  <span className="slot-choice-head">
                    <code>{candidate.id}</code>
                    <span className="slot-count">
                      {candidate.module.id}@{candidate.module.version}
                    </span>
                  </span>
                </span>
              </label>
            ))}
          </div>
        ) : (
          <div className="dialog-note">
            No instance implements {requirementTarget(detail?.requirement)}.
            Create a matching instance first.
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
            disabled={candidateInstances.length === 0}
            onClick={submit}
          >
            Review provider
          </button>
        </div>
        <p className="dialog-note">
          Candidates implement the requirement's module or interface identity.
          Core revalidates target, version, self-provision and ambiguity before
          the mapping is written to <code>[instances.dependencies]</code>.
        </p>
      </section>
    </div>
  );
}
