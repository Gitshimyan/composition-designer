export type Severity = "error" | "warning" | "info";

export interface Position {
  line: number;
  column: number;
}

export interface SourceRange {
  start: Position;
  end: Position;
}

export interface SourceRef {
  file: string;
  range?: SourceRange | null;
}

export interface Diagnostic {
  code: string;
  severity: Severity;
  message: string;
  object_id?: string | null;
  file?: string | null;
  range?: SourceRange | null;
  related?: string[];
  hint?: string | null;
}

export interface Project {
  schema_version: number;
  id: string;
  name: string;
  composition_entry: string;
  module_roots: string[];
  source?: SourceRef | null;
}

export interface ModuleRef {
  id: string;
  version: string;
}

export interface PropertyContract {
  id: string;
  type: string;
  required: boolean;
  default?: unknown;
  description?: string | null;
  values: string[];
  min_length?: number | null;
  max_length?: number | null;
  min?: unknown;
  max?: unknown;
  items?: PropertyContract | null;
}

export type SlotCardinality = "one" | "optional_one" | "many";

export interface SlotContract {
  id: string;
  cardinality: SlotCardinality;
  required: boolean;
  accepts: string[];
}

export type PortDirection = "input" | "output";

export interface PortContract {
  id: string;
  direction: PortDirection;
  type: string;
  multi: boolean;
  protocol?: string | null;
}

export interface DependencyContract {
  id: string;
  module: string;
  version?: string | null;
  optional: boolean;
}

export interface Contract {
  properties: PropertyContract[];
  slots: SlotContract[];
  ports: PortContract[];
  dependencies: DependencyContract[];
}

export interface ModuleDefinition {
  id: string;
  version: string;
  kind: string;
  description?: string | null;
  contract: Contract;
  source?: SourceRef | null;
}

export interface Instance {
  id: string;
  module: ModuleRef;
  config: Record<string, unknown>;
  effective_config: Record<string, unknown>;
  dependency_providers: Record<string, string>;
  source?: SourceRef | null;
}

export interface Binding {
  id: string;
  parent: string;
  slot: string;
  child: string;
  source?: SourceRef | null;
}

export interface PortRef {
  instance: string;
  port: string;
}

export interface Connection {
  id: string;
  from: PortRef;
  to: PortRef;
  source?: SourceRef | null;
}

export interface Composition {
  schema_version: number;
  id: string;
  root: string;
  instances: Instance[];
  bindings: Binding[];
  connections: Connection[];
  source?: SourceRef | null;
}

export interface DependencyResolution {
  requirement_id: string;
  from_instance: string;
  provider_instance?: string | null;
  optional: boolean;
  status: string;
}

export interface Revision {
  seq: number;
  source_digest?: string | null;
}

export interface CompositionModel {
  project: Project;
  composition: Composition;
  modules: ModuleDefinition[];
  dependencies: DependencyResolution[];
  diagnostics: Diagnostic[];
  revision?: Revision | null;
}

export interface ProjectSnapshot {
  root: string;
  valid: boolean;
  errorCount: number;
  warningCount: number;
  sourceDigest?: string | null;
  model?: CompositionModel | null;
  diagnostics: Diagnostic[];
  canEdit: boolean;
}

export type EditCommand =
  | {
      type: "create_instance";
      id: string;
      module: string;
      version: string;
      config: Record<string, unknown>;
    }
  | {
      type: "delete_instance";
      id: string;
    }
  | {
      type: "rename_instance";
      from: string;
      to: string;
    }
  | {
      type: "update_instance_module";
      id: string;
      module: string;
      version: string;
    }
  | {
      type: "set_config";
      instance: string;
      property: string;
      value: unknown;
    }
  | {
      type: "unset_config";
      instance: string;
      property: string;
    }
  | {
      type: "set_dependency_provider";
      instance: string;
      requirement: string;
      provider: string;
    }
  | {
      type: "unset_dependency_provider";
      instance: string;
      requirement: string;
    }
  | {
      type: "create_binding";
      id: string;
      parent: string;
      slot: string;
      child: string;
    }
  | {
      type: "update_binding";
      id: string;
      parent: string;
      slot: string;
      child: string;
    }
  | {
      type: "delete_binding";
      id: string;
    }
  | {
      type: "create_connection";
      id: string;
      from: PortRef;
      to: PortRef;
    }
  | {
      type: "update_connection";
      id: string;
      from: PortRef;
      to: PortRef;
    }
  | {
      type: "delete_connection";
      id: string;
    };

export interface Transaction {
  request_id: string;
  base_revision: Revision;
  commands: EditCommand[];
}

/** UI-only intent carried through Change Review; never sent over IPC. */
export interface ReviewOptions {
  selectAfter?: string | null;
  focusAfter?: boolean;
  alias?: { from: string; to: string };
}

export interface DiffSummary {
  added: number;
  removed: number;
  modified: number;
}

export interface ObjectChange {
  object_id: string;
  kind: "added" | "removed" | "modified";
  before?: unknown;
  after?: unknown;
}

export interface ModelDiff {
  summary: DiffSummary;
  changes: ObjectChange[];
}

export interface EditResponse {
  accepted: boolean;
  replayed: boolean;
  diff?: ModelDiff | null;
  sourceDiff?: { before: string; after: string } | null;
  diagnostics: Diagnostic[];
  snapshot: ProjectSnapshot;
}

export interface SourcePayload {
  entry: string;
  text: string;
}

export interface ExternalChangeState {
  hasSession: boolean;
  dirty: boolean;
  sessionDigest?: string | null;
  diskDigest?: string | null;
  diskValid: boolean;
  diskErrorCount: number;
}

export interface NodePosition {
  x: number;
  y: number;
}

export type LayoutNodes = Record<string, NodePosition>;
