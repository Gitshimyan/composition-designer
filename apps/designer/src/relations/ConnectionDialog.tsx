import { useMemo, useState } from "react";
import type {
  Connection,
  EditCommand,
  Instance,
  ModuleDefinition,
  PortContract,
  ReviewOptions,
} from "../types";
import { isLocalId, moduleKey } from "../inspector/instanceEditing";
import {
  endpointKey,
  portEndpoints,
  suggestConnectionId,
  type PortEndpoint,
} from "./relationModel";

interface ConnectionDialogProps {
  /** 缺省表示新建。 */
  connection?: Connection;
  instances: Instance[];
  modules: ModuleDefinition[];
  connections: Connection[];
  suspended?: boolean;
  onCancel: () => void;
  onReview: (
    commands: EditCommand[],
    description: string,
    options?: ReviewOptions,
  ) => void;
}

export default function ConnectionDialog({
  connection,
  instances,
  modules,
  connections,
  suspended = false,
  onCancel,
  onReview,
}: ConnectionDialogProps) {
  const outputs = useMemo(
    () => portEndpoints(instances, modules, "output"),
    [instances, modules],
  );
  const inputs = useMemo(
    () => portEndpoints(instances, modules, "input"),
    [instances, modules],
  );
  const [fromKey, setFromKey] = useState(() =>
    connection
      ? endpointKey(connection.from.instance, connection.from.port)
      : outputs[0]
        ? endpointKey(outputs[0].instance.id, outputs[0].port.id)
        : "",
  );
  const [toKey, setToKey] = useState(() =>
    connection
      ? endpointKey(connection.to.instance, connection.to.port)
      : inputs[0]
        ? endpointKey(inputs[0].instance.id, inputs[0].port.id)
        : "",
  );
  const [connectionId, setConnectionId] = useState(
    connection?.id ?? suggestConnectionId(connections),
  );
  const [localError, setLocalError] = useState<string | null>(null);

  const fromEndpoint = outputs.find(
    (endpoint) => endpointKey(endpoint.instance.id, endpoint.port.id) === fromKey,
  );
  const toEndpoint = inputs.find(
    (endpoint) => endpointKey(endpoint.instance.id, endpoint.port.id) === toKey,
  );
  const fromInstances = uniqueInstances(outputs);
  const toInstances = uniqueInstances(inputs);
  const fromPorts = outputs.filter(
    (endpoint) => endpoint.instance.id === fromEndpoint?.instance.id,
  );
  const toPorts = inputs.filter(
    (endpoint) => endpoint.instance.id === toEndpoint?.instance.id,
  );

  function changeFromInstance(instanceId: string) {
    const first = outputs.find((endpoint) => endpoint.instance.id === instanceId);
    if (first) setFromKey(endpointKey(first.instance.id, first.port.id));
    setLocalError(null);
  }

  function changeToInstance(instanceId: string) {
    const first = inputs.find((endpoint) => endpoint.instance.id === instanceId);
    if (first) setToKey(endpointKey(first.instance.id, first.port.id));
    setLocalError(null);
  }

  function submit() {
    if (!fromEndpoint || !toEndpoint) {
      setLocalError("Choose an output endpoint and an input endpoint.");
      return;
    }
    const id = connection ? connection.id : connectionId.trim();
    if (!connection) {
      if (!isLocalId(id)) {
        setLocalError(
          "Connection id must start with a lowercase letter and use a-z, 0-9, - or _.",
        );
        return;
      }
      if (connections.some((candidate) => candidate.id === id)) {
        setLocalError(`Connection ${id} already exists.`);
        return;
      }
    }
    setLocalError(null);
    onReview(
      [
        connection
          ? {
              type: "update_connection",
              id: connection.id,
              from: {
                instance: fromEndpoint.instance.id,
                port: fromEndpoint.port.id,
              },
              to: {
                instance: toEndpoint.instance.id,
                port: toEndpoint.port.id,
              },
            }
          : {
              type: "create_connection",
              id,
              from: {
                instance: fromEndpoint.instance.id,
                port: fromEndpoint.port.id,
              },
              to: {
                instance: toEndpoint.instance.id,
                port: toEndpoint.port.id,
              },
            },
      ],
      connection
        ? `Update connection ${connection.id}`
        : `Create connection ${id}`,
      { selectAfter: fromEndpoint.instance.id },
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
        aria-labelledby="connection-dialog-title"
      >
        <div className="dialog-head">
          <div>
            <div className="eyebrow">Port connection</div>
            <h2 id="connection-dialog-title">
              {connection ? "Edit connection" : "New connection"}
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
          <span>output → input</span>
          <span>type · protocol · multi validated by Core</span>
        </div>

        <div className="dialog-fields">
          {!connection && (
            <>
              <label htmlFor="connection-id">Connection id</label>
              <input
                id="connection-id"
                aria-label="Connection id"
                value={connectionId}
                spellCheck={false}
                autoComplete="off"
                onChange={(event) => {
                  setConnectionId(event.target.value);
                  setLocalError(null);
                }}
              />
            </>
          )}

          <EndpointFields
            title="From (output)"
            instances={fromInstances}
            endpoints={fromPorts}
            endpoint={fromEndpoint}
            instanceLabel="From instance"
            portLabel="From port"
            onChangeInstance={changeFromInstance}
            onChangePort={(portId) => {
              if (fromEndpoint) {
                setFromKey(
                  endpointKey(fromEndpoint.instance.id, portId),
                );
              }
            }}
          />

          <EndpointFields
            title="To (input)"
            instances={toInstances}
            endpoints={toPorts}
            endpoint={toEndpoint}
            instanceLabel="To instance"
            portLabel="To port"
            onChangeInstance={changeToInstance}
            onChangePort={(portId) => {
              if (toEndpoint) {
                setToKey(endpointKey(toEndpoint.instance.id, portId));
              }
            }}
          />
        </div>

        {localError && <div className="config-error">{localError}</div>}

        <div className="dialog-actions">
          <button type="button" className="button button-ghost" onClick={onCancel}>
            Cancel
          </button>
          <button
            type="button"
            className="button button-primary"
            disabled={!fromEndpoint || !toEndpoint}
            onClick={submit}
          >
            Review connection
          </button>
        </div>
        <p className="dialog-note">
          Endpoints are grouped by port direction. Core validates direction,
          type, protocol, multiplicity and duplicate endpoints before commit.
        </p>
      </section>
    </div>
  );
}

function EndpointFields({
  title,
  instances,
  endpoints,
  endpoint,
  instanceLabel,
  portLabel,
  onChangeInstance,
  onChangePort,
}: {
  title: string;
  instances: Instance[];
  endpoints: PortEndpoint[];
  endpoint?: PortEndpoint;
  instanceLabel: string;
  portLabel: string;
  onChangeInstance: (id: string) => void;
  onChangePort: (id: string) => void;
}) {
  return (
    <div className="endpoint-fields">
      <div className="section-label">{title}</div>
      <label htmlFor={`${instanceLabel}-select`}>{instanceLabel}</label>
      <select
        id={`${instanceLabel}-select`}
        aria-label={instanceLabel}
        value={endpoint?.instance.id ?? ""}
        onChange={(event) => onChangeInstance(event.target.value)}
      >
        {instances.map((instance) => (
          <option key={instance.id} value={instance.id}>
            {instance.id} · {moduleKey(instance.module)}
          </option>
        ))}
      </select>

      <label htmlFor={`${portLabel}-select`}>{portLabel}</label>
      <select
        id={`${portLabel}-select`}
        aria-label={portLabel}
        value={endpoint?.port.id ?? ""}
        onChange={(event) => onChangePort(event.target.value)}
      >
        {endpoints.map((candidate) => (
          <option key={candidate.port.id} value={candidate.port.id}>
            {candidate.port.id} · {candidate.port.type}
          </option>
        ))}
      </select>

      {endpoint && <EndpointMeta port={endpoint.port} />}
    </div>
  );
}

function EndpointMeta({ port }: { port: PortContract }) {
  return (
    <div className="endpoint-meta">
      <span className={`port-direction is-${port.direction}`}>
        {port.direction}
      </span>
      <code>{port.type}</code>
      {port.protocol && <span>protocol {port.protocol}</span>}
      {port.multi && <span>multi</span>}
    </div>
  );
}

function uniqueInstances(endpoints: PortEndpoint[]): Instance[] {
  const seen = new Map<string, Instance>();
  for (const endpoint of endpoints) {
    if (!seen.has(endpoint.instance.id)) {
      seen.set(endpoint.instance.id, endpoint.instance);
    }
  }
  return [...seen.values()].sort((a, b) => a.id.localeCompare(b.id));
}
