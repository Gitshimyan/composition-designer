import type {
  Connection,
  DependencyResolution,
  Instance,
  ModuleDefinition,
} from "../types";
import { connectionsForInstance } from "./relationModel";

interface ServicesViewProps {
  instances: Instance[];
  modules: ModuleDefinition[];
  connections: Connection[];
  dependencies: DependencyResolution[];
  selectedId: string | null;
  canEdit: boolean;
  projectSchemaVersion: number;
  onSelectInstance: (id: string) => void;
  onNewService: () => void;
}

export default function ServicesView({
  instances,
  modules,
  connections,
  dependencies,
  selectedId,
  canEdit,
  projectSchemaVersion,
  onSelectInstance,
  onNewService,
}: ServicesViewProps) {
  const services = instances
    .filter((instance) => instance.scope === "service")
    .sort((left, right) => left.id.localeCompare(right.id));
  const isV2 = projectSchemaVersion >= 2;

  const usedBy = (serviceId: string) => {
    const providers = instances
      .filter((instance) =>
        Object.values(instance.dependency_providers ?? {}).includes(serviceId),
      )
      .map((instance) => instance.id);
    const resolved = dependencies
      .filter((dependency) => dependency.provider_instance === serviceId)
      .map((dependency) => dependency.from_instance);
    return [...new Set([...providers, ...resolved])].sort();
  };

  return (
    <div className="relation-view">
      <div className="relation-toolbar">
        <div className="relation-toolbar-copy">
          <strong>{services.length} service instances</strong>
          <span>
            Services live outside the Binding tree. They can declare config,
            dependencies and ports, and provide or consume connections.
          </span>
        </div>
        <button
          type="button"
          className="button button-primary"
          disabled={!canEdit || !isV2}
          onClick={onNewService}
          title={
            isV2
              ? "Create a service instance"
              : "Services require a schema v2 project"
          }
        >
          New service
        </button>
      </div>

      <div className="relation-scroll">
        {services.length === 0 ? (
          <div className="relation-empty">
            <span className="status-glyph">◇</span>
            <span>
              {isV2
                ? "No service instances. Create one to model runtime services outside the structure tree."
                : "This project uses schema v1. Services are available in v2 projects only."}
            </span>
          </div>
        ) : (
          services.map((service) => {
            const module = modules.find(
              (candidate) =>
                candidate.id === service.module.id &&
                candidate.version === service.module.version,
            );
            const interfaces = module?.contract.interfaces ?? [];
            const connectionCount = connectionsForInstance(
              connections,
              service.id,
            ).length;
            const consumers = usedBy(service.id);
            return (
              <section
                className={`dependency-group ${
                  selectedId === service.id ? "is-selected" : ""
                }`}
                key={service.id}
              >
                <button
                  type="button"
                  className="dependency-group-head"
                  onClick={() => onSelectInstance(service.id)}
                >
                  <code>{service.id}</code>
                  <span>
                    {service.module.id}@{service.module.version}
                  </span>
                  <span className="dependency-group-count">service</span>
                </button>
                <div className="dependency-row">
                  <div className="dependency-row-main">
                    <span className="dependency-target">
                      {module?.kind ?? "module"} · {interfaces.length} interface
                      {interfaces.length === 1 ? "" : "s"}
                    </span>
                    <span className="dep-status is-resolved">service</span>
                  </div>
                  <div className="dependency-row-meta">
                    <span>
                      config {Object.keys(service.config ?? {}).length} ·
                      providers{" "}
                      {Object.keys(service.dependency_providers ?? {}).length} ·
                      connections {connectionCount}
                    </span>
                    <span>
                      used by{" "}
                      {consumers.length > 0 ? consumers.join(", ") : "—"}
                    </span>
                  </div>
                  {interfaces.length > 0 && (
                    <div className="dependency-row-meta">
                      <span>
                        interfaces:{" "}
                        {interfaces
                          .map(
                            (declaration) =>
                              `${declaration.id}@${declaration.version}`,
                          )
                          .join(", ")}
                      </span>
                    </div>
                  )}
                </div>
              </section>
            );
          })
        )}

        <div className="relation-note">
          Services must use primitive modules. Binding a service is rejected by
          Core with <code>structure-service-in-binding</code>; unused services
          surface a <code>service-unused</code> warning.
        </div>
      </div>
    </div>
  );
}
