import type {
  Binding,
  CompositionModel,
  Connection,
  DependencyResolution,
  Instance,
  ModuleDefinition,
  ProjectSnapshot,
} from "../../src/types";

const digest =
  "sha256:10cef70e22f16ef050fd9d1ea3d0f4ba3502235641aaf4f88b91260daa34768e";

const compositionText = `[composition]
schema_version = 1
id = "example.minimal"
root = "app"

[[instances]]
id = "app"
module = "shell.desktop"
version = "1.0.0"

[instances.config]
title = "Minimal Project"

[[instances]]
id = "hello"
module = "leaf.widget"
version = "1.0.0"

[instances.config]
label = "Hello"

[[bindings]]
id = "app-content"
parent = "app"
slot = "content"
child = "hello"
`;

const instances: Instance[] = [
  {
    id: "app",
    module: { id: "shell.desktop", version: "1.0.0" },
    config: { title: "Minimal Project" },
    effective_config: { title: "Minimal Project" },
    dependency_providers: {},
    source: { file: "composition/main.toml" },
  },
  {
    id: "hello",
    module: { id: "leaf.widget", version: "1.0.0" },
    config: { label: "Hello" },
    effective_config: { label: "Hello" },
    dependency_providers: {},
    source: { file: "composition/main.toml" },
  },
  {
    id: "box",
    module: { id: "ui.panel", version: "1.0.0" },
    config: { heading: "Box" },
    effective_config: { heading: "Box" },
    dependency_providers: { logger: "logger" },
    source: { file: "composition/main.toml" },
  },
  {
    id: "hint",
    module: { id: "leaf.widget", version: "1.0.0" },
    config: { label: "Hint" },
    effective_config: { label: "Hint" },
    dependency_providers: {},
    source: { file: "composition/main.toml" },
  },
  {
    id: "logger",
    module: { id: "svc.logger", version: "1.0.0" },
    config: {},
    effective_config: {},
    dependency_providers: {},
    source: { file: "composition/main.toml" },
  },
  {
    id: "logger-2",
    module: { id: "svc.logger", version: "1.0.0" },
    config: {},
    effective_config: {},
    dependency_providers: {},
    source: { file: "composition/main.toml" },
  },
];

const bindings: Binding[] = [
  {
    id: "app-content-box",
    parent: "app",
    slot: "content",
    child: "box",
    source: { file: "composition/main.toml" },
  },
  {
    id: "app-content",
    parent: "app",
    slot: "content",
    child: "hello",
    source: { file: "composition/main.toml" },
  },
  {
    id: "box-body",
    parent: "box",
    slot: "body",
    child: "hint",
    source: { file: "composition/main.toml" },
  },
  {
    id: "app-content-logger",
    parent: "app",
    slot: "content",
    child: "logger",
    source: { file: "composition/main.toml" },
  },
  {
    id: "app-content-logger-2",
    parent: "app",
    slot: "content",
    child: "logger-2",
    source: { file: "composition/main.toml" },
  },
];

const connections: Connection[] = [
  {
    id: "box-log-line",
    from: { instance: "box", port: "log" },
    to: { instance: "logger", port: "line" },
    source: { file: "composition/main.toml" },
  },
];

const dependencies: DependencyResolution[] = [
  {
    requirement_id: "logger",
    from_instance: "box",
    provider_instance: "logger",
    optional: false,
    status: "resolved",
  },
];

const modules: ModuleDefinition[] = [
  {
    id: "leaf.badge",
    version: "1.0.0",
    kind: "primitive",
    description: "Compact badge widget",
    contract: {
      properties: [
        {
          id: "label",
          type: "string",
          required: true,
          default: null,
          description: null,
          values: [],
        },
        {
          id: "tone",
          type: "enum",
          required: true,
          default: null,
          description: null,
          values: ["info", "warn"],
        },
      ],
      slots: [],
      ports: [],
      dependencies: [],
    },
    source: { file: "modules/badge/module.toml" },
  },
  {
    id: "leaf.widget",
    version: "1.0.0",
    kind: "primitive",
    description: "Leaf widget",
    contract: {
      properties: [
        {
          id: "label",
          type: "string",
          required: true,
          default: null,
          description: null,
          values: [],
        },
      ],
      slots: [],
      ports: [],
      dependencies: [],
    },
    source: { file: "modules/leaf/module.toml" },
  },
  {
    id: "shell.desktop",
    version: "1.0.0",
    kind: "composite",
    description: "Desktop shell with one content area",
    contract: {
      properties: [
        {
          id: "title",
          type: "string",
          required: true,
          default: null,
          description: null,
          values: [],
        },
      ],
      slots: [
        {
          id: "content",
          cardinality: "many",
          required: true,
          accepts: ["leaf.badge", "leaf.widget", "ui.panel", "svc.logger"],
        },
        {
          id: "aside",
          cardinality: "optional_one",
          required: false,
          accepts: ["leaf.badge", "leaf.widget"],
        },
      ],
      ports: [],
      dependencies: [],
    },
    source: { file: "modules/shell/module.toml" },
  },
  {
    id: "ui.panel",
    version: "1.0.0",
    kind: "composite",
    description: "Panel with a required body",
    contract: {
      properties: [
        {
          id: "heading",
          type: "string",
          required: false,
          default: null,
          description: null,
          values: [],
        },
      ],
      slots: [
        {
          id: "body",
          cardinality: "one",
          required: true,
          accepts: ["leaf.badge", "leaf.widget"],
        },
      ],
      ports: [
        {
          id: "log",
          direction: "output",
          type: "log.line",
          multi: true,
          protocol: "log.v1",
        },
      ],
      dependencies: [
        {
          id: "logger",
          module: "svc.logger",
          version: "1.0.0",
          optional: false,
        },
      ],
    },
    source: { file: "modules/panel/module.toml" },
  },
  {
    id: "svc.logger",
    version: "1.0.0",
    kind: "primitive",
    description: "Log sink",
    contract: {
      properties: [],
      slots: [],
      ports: [
        {
          id: "line",
          direction: "input",
          type: "log.line",
          multi: false,
          protocol: "log.v1",
        },
        {
          id: "audit",
          direction: "input",
          type: "log.line",
          multi: false,
          protocol: "log.v1",
        },
      ],
      dependencies: [],
    },
    source: { file: "modules/logger/module.toml" },
  },
];

function model(seq = 0, sourceDigest = digest): CompositionModel {
  return {
    project: {
      schema_version: 1,
      id: "example.minimal",
      name: "Minimal Project",
      composition_entry: "composition/main.toml",
      module_roots: ["modules"],
      source: { file: "project.toml" },
    },
    composition: {
      schema_version: 1,
      id: "example.minimal",
      root: "app",
      instances: structuredClone(instances),
      bindings: structuredClone(bindings),
      connections: structuredClone(connections),
      source: { file: "composition/main.toml" },
    },
    modules: structuredClone(modules),
    dependencies: structuredClone(dependencies),
    diagnostics: [],
    revision: { seq, source_digest: sourceDigest },
  };
}

export function validSnapshot(seq = 0, sourceDigest = digest): ProjectSnapshot {
  return {
    root: "C:\\projects\\minimal",
    valid: true,
    errorCount: 0,
    warningCount: 0,
    sourceDigest,
    model: model(seq, sourceDigest),
    diagnostics: [],
    canEdit: true,
    canUndo: false,
    canRedo: false,
  };
}

export function invalidSnapshot(): ProjectSnapshot {
  return {
    root: "C:\\projects\\broken",
    valid: false,
    errorCount: 1,
    warningCount: 0,
    sourceDigest: "sha256:broken",
    model: null,
    diagnostics: [
      {
        code: "file-project-missing",
        severity: "error",
        message: "project entry was not found",
        file: "project.toml",
      },
    ],
    canEdit: false,
    canUndo: false,
    canRedo: false,
  };
}

const v2CompositionText = `[composition]
schema_version = 2
id = "example.v2"
root = "app"

[[instances]]
id = "app"
module = "example.app"
version = "1.0.0"

[[instances]]
id = "session"
module = "example.session"
version = "1.0.0"
scope = "service"
`;

const v2Modules: ModuleDefinition[] = [
  {
    id: "example.app",
    version: "1.0.0",
    kind: "composite",
    description: "V2 application shell",
    contract: {
      properties: [],
      slots: [],
      ports: [],
      dependencies: [
        {
          id: "session",
          interface: "example.session.api",
          version: "1.0.0",
          optional: false,
        },
      ],
      interfaces: [],
    },
    source: { file: "modules/app/module.toml" },
  },
  {
    id: "example.session",
    version: "1.0.0",
    kind: "primitive",
    description: "Session service",
    contract: {
      properties: [],
      slots: [],
      ports: [
        {
          id: "line",
          direction: "input",
          type: "log.line",
          multi: false,
          protocol: "log.v1",
        },
      ],
      dependencies: [],
      interfaces: [{ id: "example.session.api", version: "1.0.0" }],
    },
    source: { file: "modules/session/module.toml" },
  },
];

const v2Instances: Instance[] = [
  {
    id: "app",
    module: { id: "example.app", version: "1.0.0" },
    scope: "structure",
    config: {},
    effective_config: {},
    dependency_providers: {},
    source: { file: "composition/main.toml" },
  },
  {
    id: "session",
    module: { id: "example.session", version: "1.0.0" },
    scope: "service",
    config: {},
    effective_config: {},
    dependency_providers: {},
    source: { file: "composition/main.toml" },
  },
];

const v2Dependencies: DependencyResolution[] = [
  {
    requirement_id: "session",
    from_instance: "app",
    target: {
      kind: "interface",
      id: "example.session.api",
      version: "1.0.0",
    },
    provider_instance: "session",
    optional: false,
    status: "resolved",
  },
];

export function v2Snapshot(
  seq = 0,
  sourceDigest = "sha256:v2fixture",
): ProjectSnapshot {
  return {
    root: "C:\\projects\\v2",
    valid: true,
    errorCount: 0,
    warningCount: 0,
    sourceDigest,
    model: {
      project: {
        schema_version: 2,
        id: "example.v2",
        name: "V2 Services",
        composition_entry: "composition/main.toml",
        module_roots: ["modules"],
        source: { file: "project.toml" },
      },
      composition: {
        schema_version: 2,
        id: "example.v2",
        root: "app",
        instances: structuredClone(v2Instances),
        bindings: [],
        connections: [],
        source: { file: "composition/main.toml" },
      },
      modules: structuredClone(v2Modules),
      dependencies: structuredClone(v2Dependencies),
      diagnostics: [],
      revision: { seq, source_digest: sourceDigest },
    },
    diagnostics: [],
    canEdit: true,
    canUndo: false,
    canRedo: false,
  };
}

export function v3Snapshot(): ProjectSnapshot {
  const snapshot = v2Snapshot(0, "sha256:v3fixture");
  if (snapshot.model) {
    snapshot.root = "C:\\projects\\v3";
    snapshot.model.project = {
      ...snapshot.model.project,
      schema_version: 3,
      default_composition: "composition/main.toml",
      active_profile: "default",
      profiles: [
        { name: "desktop", composition: "composition/desktop.toml" },
        { name: "headless", composition: "composition/headless.toml" },
      ],
    };
    snapshot.model.composition.schema_version = 3;
  }
  return snapshot;
}

export const sourceText = compositionText;
export const v2SourceText = v2CompositionText;
