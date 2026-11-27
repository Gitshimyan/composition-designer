import { useMemo } from "react";
import type {
  Connection,
  Instance,
  ModuleDefinition,
} from "../types";
import { moduleKey } from "../inspector/instanceEditing";
import {
  connectionsForInstance,
  connectionsForPort,
  endpointFor,
  endpointLabel,
  portEndpoints,
} from "./relationModel";

interface ConnectionViewProps {
  instances: Instance[];
  modules: ModuleDefinition[];
  connections: Connection[];
  selectedId: string | null;
  canEdit: boolean;
  onSelectInstance: (id: string) => void;
  onCreateConnection: () => void;
  onEditConnection: (connection: Connection) => void;
  onDeleteConnection: (connection: Connection) => void;
}

export default function ConnectionView({
  instances,
  modules,
  connections,
  selectedId,
  canEdit,
  onSelectInstance,
  onCreateConnection,
  onEditConnection,
  onDeleteConnection,
}: ConnectionViewProps) {
  const outputs = useMemo(
    () => portEndpoints(instances, modules, "output"),
    [instances, modules],
  );
  const inputs = useMemo(
    () => portEndpoints(instances, modules, "input"),
    [instances, modules],
  );
  const portHosts = useMemo(
    () =>
      instances
        .map((instance) => ({
          instance,
          module: modules.find(
            (module) =>
              module.id === instance.module.id &&
              module.version === instance.module.version,
          ),
        }))
        .filter(
          (entry) => entry.module && entry.module.contract.ports.length > 0,
        ),
    [instances, modules],
  );

  return (
    <div className="relation-view">
      <div className="relation-toolbar">
        <div className="relation-toolbar-copy">
          <strong>{connections.length} port connections</strong>
          <span>
            Endpoints are grouped by port direction; Core validates type,
            protocol, multiplicity and duplicate endpoints.
          </span>
        </div>
        <button
          type="button"
          className="button button-primary"
          disabled={!canEdit || outputs.length === 0 || inputs.length === 0}
          onClick={onCreateConnection}
        >
          New connection
        </button>
      </div>

      <div className="relation-scroll">
        {connections.length === 0 ? (
          <div className="relation-empty">
            <span className="status-glyph">◇</span>
            <span>No port connections in this project.</span>
          </div>
        ) : (
          <div className="connection-table">
            <div className="connection-table-head">
              <span>Connection</span>
              <span>From</span>
              <span>To</span>
              <span>Type · protocol</span>
              <span />
            </div>
            {connections.map((connection) => {
              const from = endpointFor(instances, modules, connection.from);
              const to = endpointFor(instances, modules, connection.to);
              const selected =
                selectedId !== null &&
                (connection.from.instance === selectedId ||
                  connection.to.instance === selectedId);
              return (
                <div
                  className={`connection-row ${selected ? "is-selected" : ""}`}
                  key={connection.id}
                >
                  <code className="connection-id">{connection.id}</code>
                  <span className="connection-endpoint">
                    {from
                      ? `${endpointLabel(from)}`
                      : `${connection.from.instance}.${connection.from.port}`}
                  </span>
                  <span className="connection-endpoint">
                    {to
                      ? `${endpointLabel(to)}`
                      : `${connection.to.instance}.${connection.to.port}`}
                  </span>
                  <span className="connection-meta">
                    {from?.port.type ?? to?.port.type ?? "—"}
                    {from?.port.protocol || to?.port.protocol
                      ? ` · ${from?.port.protocol ?? to?.port.protocol}`
                      : ""}
                  </span>
                  <span className="connection-actions">
                    <button
                      type="button"
                      className="config-action"
                      disabled={!canEdit}
                      onClick={() => onEditConnection(connection)}
                    >
                      Edit
                    </button>
                    <button
                      type="button"
                      className="config-action is-muted"
                      disabled={!canEdit}
                      onClick={() => onDeleteConnection(connection)}
                    >
                      Delete
                    </button>
                  </span>
                </div>
              );
            })}
          </div>
        )}

        <div className="port-grid">
          {portHosts.length === 0 ? (
            <div className="relation-empty">
              <span className="status-glyph">◇</span>
              <span>No loaded module declares ports.</span>
            </div>
          ) : (
            portHosts.map(({ instance, module }) => {
              const linked = connectionsForInstance(connections, instance.id);
              return (
                <section
                  className={`port-card ${
                    selectedId === instance.id ? "is-selected" : ""
                  }`}
                  key={instance.id}
                >
                  <button
                    type="button"
                    className="port-card-head"
                    onClick={() => onSelectInstance(instance.id)}
                  >
                    <code>{instance.id}</code>
                    <span>{module ? moduleKey(module) : ""}</span>
                    <span className="port-card-count">
                      {linked.length} linked
                    </span>
                  </button>
                  {module?.contract.ports.map((port) => {
                    const portLinks = connectionsForPort(
                      connections,
                      instance.id,
                      port.id,
                    );
                    const ids = [
                      ...portLinks.outgoing.map(
                        (connection) => `→ ${connection.to.instance}.${connection.to.port}`,
                      ),
                      ...portLinks.incoming.map(
                        (connection) => `← ${connection.from.instance}.${connection.from.port}`,
                      ),
                    ];
                    return (
                      <div className="port-row" key={port.id}>
                        <span className={`port-direction is-${port.direction}`}>
                          {port.direction}
                        </span>
                        <code>{port.id}</code>
                        <span className="port-type">{port.type}</span>
                        {port.protocol && (
                          <span className="port-protocol">{port.protocol}</span>
                        )}
                        {port.multi && <span className="port-multi">multi</span>}
                        <span className="port-links">
                          {ids.length > 0 ? ids.join(" · ") : "unconnected"}
                        </span>
                      </div>
                    );
                  })}
                </section>
              );
            })
          )}
        </div>
      </div>
    </div>
  );
}
