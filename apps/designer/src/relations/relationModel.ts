import type {
  Connection,
  DependencyContract,
  DependencyResolution,
  Instance,
  ModuleDefinition,
  PortContract,
  PortRef,
} from "../types";
import { findModule } from "../inspector/instanceEditing";

export interface PortEndpoint {
  instance: Instance;
  port: PortContract;
}

export function endpointFor(
  instances: Instance[],
  modules: ModuleDefinition[],
  ref: PortRef,
): PortEndpoint | undefined {
  const instance = instances.find((candidate) => candidate.id === ref.instance);
  if (!instance) return undefined;
  const module = findModule(
    modules,
    instance.module.id,
    instance.module.version,
  );
  const port = module?.contract.ports.find((candidate) => candidate.id === ref.port);
  return port ? { instance, port } : undefined;
}

/** 按方向列出可选择端点；排序保证下拉/投影稳定。 */
export function portEndpoints(
  instances: Instance[],
  modules: ModuleDefinition[],
  direction: "input" | "output",
): PortEndpoint[] {
  const endpoints: PortEndpoint[] = [];
  for (const instance of instances) {
    const module = findModule(
      modules,
      instance.module.id,
      instance.module.version,
    );
    if (!module) continue;
    for (const port of module.contract.ports) {
      if (port.direction === direction) {
        endpoints.push({ instance, port });
      }
    }
  }
  return endpoints.sort(
    (a, b) =>
      a.instance.id.localeCompare(b.instance.id) ||
      a.port.id.localeCompare(b.port.id),
  );
}

export function endpointKey(instanceId: string, portId: string): string {
  return `${instanceId}:${portId}`;
}

export function endpointLabel(endpoint: PortEndpoint): string {
  return `${endpoint.instance.id}.${endpoint.port.id}`;
}

export interface PortConnections {
  outgoing: Connection[];
  incoming: Connection[];
}

export function connectionsForPort(
  connections: Connection[],
  instanceId: string,
  portId: string,
): PortConnections {
  return {
    outgoing: connections.filter(
      (connection) =>
        connection.from.instance === instanceId &&
        connection.from.port === portId,
    ),
    incoming: connections.filter(
      (connection) =>
        connection.to.instance === instanceId &&
        connection.to.port === portId,
    ),
  };
}

export function connectionsForInstance(
  connections: Connection[],
  instanceId: string,
): Connection[] {
  return connections.filter(
    (connection) =>
      connection.from.instance === instanceId ||
      connection.to.instance === instanceId,
  );
}

export function suggestConnectionId(connections: Connection[]): string {
  const used = new Set(connections.map((connection) => connection.id));
  let index = 1;
  while (used.has(`connect-${index}`)) index += 1;
  return `connect-${index}`;
}

export interface DependencyDetail {
  resolution: DependencyResolution;
  instance: Instance;
  module?: ModuleDefinition;
  requirement?: DependencyContract;
  /** 实例 Config 中显式写了 provider 映射。 */
  explicit: boolean;
  /** 目标 module/version 的其他实例，仅用于解释自动/歧义解析。 */
  candidates: string[];
}

export function dependencyDetails(
  instances: Instance[],
  modules: ModuleDefinition[],
  dependencies: DependencyResolution[],
): DependencyDetail[] {
  const details: DependencyDetail[] = [];
  for (const resolution of dependencies) {
    const instance = instances.find(
      (candidate) => candidate.id === resolution.from_instance,
    );
    if (!instance) continue;
    const module = findModule(
      modules,
      instance.module.id,
      instance.module.version,
    );
    const requirement = module?.contract.dependencies.find(
      (candidate) => candidate.id === resolution.requirement_id,
    );
    const candidates = requirement
      ? instances
          .filter((candidate) => {
            if (candidate.id === instance.id) return false;
            if (candidate.module.id !== requirement.module) return false;
            return (
              requirement.version === undefined ||
              requirement.version === null ||
              candidate.module.version === requirement.version
            );
          })
          .map((candidate) => candidate.id)
          .sort()
      : [];
    details.push({
      resolution,
      instance,
      module,
      requirement,
      explicit: Object.prototype.hasOwnProperty.call(
        instance.dependency_providers ?? {},
        resolution.requirement_id,
      ),
      candidates,
    });
  }
  return details.sort(
    (a, b) =>
      a.resolution.from_instance.localeCompare(b.resolution.from_instance) ||
      a.resolution.requirement_id.localeCompare(b.resolution.requirement_id),
  );
}
