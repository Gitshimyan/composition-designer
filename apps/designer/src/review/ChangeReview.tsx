import type {
  DependencyResolution,
  EditCommand,
  EditResponse,
  ObjectChange,
  Transaction,
} from "../types";

interface ChangeReviewProps {
  transaction: Transaction;
  preview: EditResponse;
  description: string;
  busy: boolean;
  error: string | null;
  onApply: () => void;
  onClose: () => void;
}

type ObjectKind = "instance" | "binding" | "connection" | "other";

interface ObjectField {
  label: string;
  before: string;
  after: string;
}

interface DependencyImpact {
  instanceId: string;
  requirement: string;
  before: string;
  after: string;
  status?: string;
}

interface InstanceLike {
  id?: string;
  module?: { id?: string; version?: string };
  config?: Record<string, unknown>;
  dependency_providers?: Record<string, string>;
}

const GROUPS: Array<{ kind: ObjectKind; label: string }> = [
  { kind: "instance", label: "Instances" },
  { kind: "binding", label: "Bindings" },
  { kind: "connection", label: "Connections" },
  { kind: "other", label: "Other" },
];

export default function ChangeReview({
  transaction,
  preview,
  description,
  busy,
  error,
  onApply,
  onClose,
}: ChangeReviewProps) {
  const summary = preview.diff?.summary;
  const changes = preview.diff?.changes ?? [];
  const dependencies = preview.snapshot.model?.dependencies ?? [];
  const impacts = dependencyImpacts(changes, dependencies);

  return (
    <div className="review-veil" role="presentation">
      <section
        className="review-card review-card-wide"
        role="dialog"
        aria-modal="true"
        aria-labelledby="review-title"
      >
        <div className="review-head">
          <div>
            <div className="eyebrow">Change review</div>
            <h2 id="review-title">{description}</h2>
          </div>
          <span className={`review-status ${preview.accepted ? "is-ok" : "is-bad"}`}>
            {preview.accepted ? "validated" : "rejected"}
          </span>
        </div>

        <div className="review-summary">
          <span>
            {transaction.commands.length} command
            {transaction.commands.length === 1 ? "" : "s"}
          </span>
          {summary && (
            <span>
              +{summary.added} / -{summary.removed} / ~{summary.modified}
            </span>
          )}
          <span className="mono">seq {transaction.base_revision.seq}</span>
          <span>
            {preview.diagnostics.filter((diagnostic) => diagnostic.severity === "error").length}{" "}
            errors ·{" "}
            {preview.diagnostics.filter((diagnostic) => diagnostic.severity === "warning").length}{" "}
            warnings
          </span>
        </div>

        <section className="review-section">
          <div className="review-section-head">
            <h3>Commands</h3>
            <span className="review-section-count">{transaction.commands.length}</span>
          </div>
          <ol className="review-command-list">
            {transaction.commands.map((command, index) => (
              <li key={`${command.type}-${index}`}>
                <span className="review-command-index">{index + 1}</span>
                <code>{describeCommand(command)}</code>
              </li>
            ))}
          </ol>
        </section>

        {preview.sourceDiff && (
          <section className="review-section">
            <div className="review-section-head">
              <h3>Source change</h3>
              <span className="review-section-count">
                {preview.sourceDiff.after.split("\n").length} lines
              </span>
            </div>
            <div className="review-source-diff">
              <div>
                <span className="review-source-label">Before</span>
                <pre>{preview.sourceDiff.before}</pre>
              </div>
              <div>
                <span className="review-source-label">After</span>
                <pre>{preview.sourceDiff.after}</pre>
              </div>
            </div>
          </section>
        )}

        {changes.length === 0 ? (
          <div className="review-empty">No object changes returned by Core.</div>
        ) : (
          GROUPS.map((group) => {
            const groupChanges = changes.filter(
              (change) => objectKind(change.object_id) === group.kind,
            );
            if (groupChanges.length === 0) return null;
            return (
              <section className="review-section" key={group.kind}>
                <div className="review-section-head">
                  <h3>{group.label}</h3>
                  <span className="review-section-count">{groupChanges.length}</span>
                </div>
                <div className="review-change-list">
                  {groupChanges.map((change) => (
                    <ChangeCard key={`${change.object_id}-${change.kind}`} change={change} />
                  ))}
                </div>
              </section>
            );
          })
        )}

        <section className="review-section">
          <div className="review-section-head">
            <h3>Dependency impact</h3>
            <span className="review-section-count">{impacts.length}</span>
          </div>
          {impacts.length === 0 ? (
            <div className="review-empty">No explicit provider mappings changed.</div>
          ) : (
            <ul className="review-dependency-list">
              {impacts.map((impact) => (
                <li key={`${impact.instanceId}.${impact.requirement}`}>
                  <code>
                    {impact.instanceId}.{impact.requirement}
                  </code>
                  <span className="review-dependency-change">
                    {impact.before} → <strong>{impact.after}</strong>
                  </span>
                  {impact.status && (
                    <span className={`dep-status is-${impact.status.replace(/_/g, "-")}`}>
                      current {impact.status}
                    </span>
                  )}
                </li>
              ))}
            </ul>
          )}
        </section>

        <section className="review-section">
          <div className="review-section-head">
            <h3>Validation</h3>
            <span
              className={`review-validation ${preview.accepted ? "is-ok" : "is-bad"}`}
            >
              {preview.accepted ? "candidate passed L2–L7" : "candidate rejected"}
            </span>
          </div>
          {preview.diagnostics.length === 0 ? (
            <div className="review-empty">No diagnostics.</div>
          ) : (
            <ul className="review-diagnostics">
              {preview.diagnostics.map((diagnostic, index) => (
                <li key={`${diagnostic.code}-${index}`}>
                  <span className={`severity-dot severity-${diagnostic.severity}`} />
                  <code>{diagnostic.code}</code>
                  <span>{diagnostic.message}</span>
                </li>
              ))}
            </ul>
          )}
        </section>

        {error && <div className="review-error">{error}</div>}

        <div className="review-actions">
          <button type="button" className="button button-ghost" onClick={onClose} disabled={busy}>
            Cancel
          </button>
          <button
            type="button"
            className="button button-primary"
            onClick={onApply}
            disabled={busy || !preview.accepted}
          >
            {busy ? "Applying…" : "Apply transaction"}
          </button>
        </div>
        <p className="review-note">
          Applying writes the composition entry through Core, revalidates, and refreshes the
          source digest. Layout positions are unaffected. Cancel leaves the project untouched.
        </p>
      </section>
    </div>
  );
}

function ChangeCard({ change }: { change: ObjectChange }) {
  const kind = objectKind(change.object_id);
  const fields = diffFields(kind, change.before, change.after);
  return (
    <article className="review-change" data-object-id={change.object_id}>
      <div className="review-change-head">
        <code>{change.object_id}</code>
        <span className={`change-kind kind-${change.kind}`}>{change.kind}</span>
      </div>
      {fields.length === 0 ? (
        <div className="review-empty">No field-level changes.</div>
      ) : (
        <div className="review-field-table">
          <div className="review-field-head">
            <span>Field</span>
            <span>Before</span>
            <span>After</span>
          </div>
          {fields.map((field) => (
            <div className="review-field-row" key={field.label}>
              <span className="review-field-label">{field.label}</span>
              <code className="review-field-before">{field.before}</code>
              <code className="review-field-after">{field.after}</code>
            </div>
          ))}
        </div>
      )}
    </article>
  );
}

function objectKind(objectId: string): ObjectKind {
  const prefix = objectId.split(":", 1)[0];
  return prefix === "instance" || prefix === "binding" || prefix === "connection"
    ? prefix
    : "other";
}

function diffFields(
  kind: ObjectKind,
  beforeValue: unknown,
  afterValue: unknown,
): ObjectField[] {
  if (kind === "other") {
    return [
      {
        label: "value",
        before: beforeValue === undefined ? "—" : formatValue(beforeValue),
        after: afterValue === undefined ? "—" : formatValue(afterValue),
      },
    ];
  }
  const before = objectFields(kind, beforeValue);
  const after = objectFields(kind, afterValue);
  const labels = [...new Set([...before.keys(), ...after.keys()])].sort();
  const fields: ObjectField[] = [];
  for (const label of labels) {
    const previous = before.get(label) ?? "—";
    const next = after.get(label) ?? "—";
    if (previous !== next) {
      fields.push({ label, before: previous, after: next });
    }
  }
  if (fields.length === 0) {
    // Added/removed objects with no flattened fields still deserve one row.
    const side = before.size > 0 ? before : after;
    for (const label of side.keys()) {
      fields.push({ label, before: before.get(label) ?? "—", after: after.get(label) ?? "—" });
    }
  }
  return fields;
}

function objectFields(kind: ObjectKind, value: unknown): Map<string, string> {
  const fields = new Map<string, string>();
  if (!value || typeof value !== "object") return fields;
  if (kind === "instance") {
    const instance = value as InstanceLike;
    if (instance.module) {
      fields.set("module", `${instance.module.id ?? "?"}@${instance.module.version ?? "?"}`);
    }
    for (const [key, config] of Object.entries(instance.config ?? {})) {
      fields.set(`config.${key}`, formatValue(config));
    }
    for (const [key, provider] of Object.entries(
      instance.dependency_providers ?? {},
    )) {
      fields.set(`providers.${key}`, provider);
    }
    return fields;
  }
  const record = value as Record<string, unknown>;
  if (kind === "binding") {
    for (const key of ["parent", "slot", "child"]) {
      if (key in record) fields.set(key, formatValue(record[key]));
    }
    return fields;
  }
  for (const endpoint of [
    ["from", record.from],
    ["to", record.to],
  ] as const) {
    if (endpoint[1] && typeof endpoint[1] === "object") {
      const port = endpoint[1] as { instance?: string; port?: string };
      fields.set(endpoint[0], `${port.instance ?? "?"}.${port.port ?? "?"}`);
    }
  }
  return fields;
}

function dependencyImpacts(
  changes: ObjectChange[],
  dependencies: DependencyResolution[],
): DependencyImpact[] {
  const impacts = new Map<string, DependencyImpact>();
  for (const change of changes) {
    if (objectKind(change.object_id) !== "instance") continue;
    const before = (change.before ?? {}) as InstanceLike;
    const after = (change.after ?? {}) as InstanceLike;
    const instanceId =
      before.id ?? after.id ?? change.object_id.replace(/^instance:/, "");
    const beforeProviders = before.dependency_providers ?? {};
    const afterProviders = after.dependency_providers ?? {};
    for (const requirement of new Set([
      ...Object.keys(beforeProviders),
      ...Object.keys(afterProviders),
    ])) {
      const previous = beforeProviders[requirement] ?? "auto";
      const next = afterProviders[requirement] ?? "auto";
      if (previous === next) continue;
      const key = `${instanceId}.${requirement}`;
      impacts.set(key, {
        instanceId,
        requirement,
        before: previous,
        after: next,
        status: dependencies.find(
          (dependency) =>
            dependency.from_instance === instanceId &&
            dependency.requirement_id === requirement,
        )?.status,
      });
    }
  }
  return [...impacts.values()];
}

function describeCommand(command: EditCommand): string {
  switch (command.type) {
    case "create_instance":
      return `create_instance ${command.id} ← ${command.module}@${command.version}`;
    case "delete_instance":
      return `delete_instance ${command.id}`;
    case "rename_instance":
      return `rename_instance ${command.from} → ${command.to}`;
    case "update_instance_module":
      return `update_instance_module ${command.id} → ${command.module}@${command.version}`;
    case "set_config":
      return `set_config ${command.instance}.${command.property} = ${formatValue(command.value)}`;
    case "unset_config":
      return `unset_config ${command.instance}.${command.property}`;
    case "set_dependency_provider":
      return `set_dependency_provider ${command.instance}.${command.requirement} → ${command.provider}`;
    case "unset_dependency_provider":
      return `unset_dependency_provider ${command.instance}.${command.requirement}`;
    case "create_binding":
      return `create_binding ${command.id}: ${command.parent}.${command.slot} → ${command.child}`;
    case "update_binding":
      return `update_binding ${command.id}: ${command.parent}.${command.slot} → ${command.child}`;
    case "delete_binding":
      return `delete_binding ${command.id}`;
    case "create_connection":
      return `create_connection ${command.id}: ${command.from.instance}.${command.from.port} → ${command.to.instance}.${command.to.port}`;
    case "update_connection":
      return `update_connection ${command.id}: ${command.from.instance}.${command.from.port} → ${command.to.instance}.${command.to.port}`;
    case "delete_connection":
      return `delete_connection ${command.id}`;
    case "replace_source":
      return `replace_source ${command.source.length} chars`;
    default:
      return JSON.stringify(command);
  }
}

function formatValue(value: unknown): string {
  if (typeof value === "string") return value;
  if (typeof value === "number" || typeof value === "boolean") return String(value);
  if (value === null) return "null";
  try {
    return JSON.stringify(value);
  } catch {
    return String(value);
  }
}
