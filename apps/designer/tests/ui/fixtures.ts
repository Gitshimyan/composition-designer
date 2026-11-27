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
  };
}

export const sourceText = compositionText;
