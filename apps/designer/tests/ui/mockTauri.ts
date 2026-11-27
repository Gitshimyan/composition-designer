import type { Page } from "@playwright/test";
import { invalidSnapshot, sourceText, validSnapshot } from "./fixtures";

export type MockMode = "valid" | "invalid" | "edit-failure";

interface MockPayload {
  mode: MockMode;
  valid: ReturnType<typeof validSnapshot>;
  invalid: ReturnType<typeof invalidSnapshot>;
  sourceText: string;
}

type MockSnapshot = ReturnType<typeof validSnapshot>;
type MockCommand = { type: string } & Record<string, unknown>;

interface Change {
  object_id: string;
  kind: "added" | "removed" | "modified";
  before?: unknown;
  after?: unknown;
}

/**
 * Install a test-only Tauri IPC bridge before the app loads.
 * Production builds never include this module; tests run against Vite preview
 * and only exercise the React UI contract.
 */
export async function installMock(page: Page, mode: MockMode = "valid") {
  const payload: MockPayload = {
    mode,
    valid: validSnapshot(),
    invalid: invalidSnapshot(),
    sourceText,
  };

  await page.addInitScript((data: MockPayload) => {
    let current = structuredClone(data.valid);
    const testState = {
      mode: data.mode,
      dirty: false,
      applyCalls: 0,
      prepareCalls: 0,
      saveLayoutCalls: 0,
      lastLayout: null as unknown,
      lastCommands: [] as MockCommand[],
      get snapshot() {
        return current;
      },
    };

    Object.defineProperty(window, "__COMPOSITION_TEST__", {
      value: testState,
      configurable: true,
    });

    function changed(
      changes: Change[],
      summary: { added: number; removed: number; modified: number },
    ) {
      return { changes, summary };
    }

    function refreshDependencies(
      model: NonNullable<MockSnapshot["model"]>,
    ) {
      const composition = model.composition;
      for (const resolution of model.dependencies) {
        const instance = composition.instances.find(
          (candidate) => candidate.id === resolution.from_instance,
        );
        if (!instance) continue;
        const module = model.modules.find(
          (candidate) =>
            candidate.id === instance.module.id &&
            candidate.version === instance.module.version,
        );
        const requirement = module?.contract.dependencies.find(
          (candidate) => candidate.id === resolution.requirement_id,
        );
        const explicit =
          instance.dependency_providers?.[resolution.requirement_id];
        if (explicit) {
          resolution.provider_instance = explicit;
          resolution.status = "resolved";
          continue;
        }
        const candidates = requirement
          ? composition.instances.filter(
              (candidate) =>
                candidate.id !== instance.id &&
                candidate.module.id === requirement.module &&
                (!requirement.version ||
                  candidate.module.version === requirement.version),
            )
          : [];
        resolution.provider_instance =
          candidates.length === 1 ? candidates[0].id : null;
        resolution.status =
          candidates.length === 1
            ? "resolved"
            : candidates.length === 0
              ? resolution.optional
                ? "optional_unresolved"
                : "unresolved"
              : "ambiguous";
      }
    }

    function applyCommands(commands: MockCommand[]) {
      const next: MockSnapshot = structuredClone(current);
      const model = next.model;
      if (!model) throw new Error("mock transaction requires a valid model");
      const composition = model.composition;
      const changes: Change[] = [];
      const summary = { added: 0, removed: 0, modified: 0 };

      for (const command of commands) {
        switch (command.type) {
          case "create_instance": {
            const id = String(command.id);
            const config = (command.config ?? {}) as Record<string, unknown>;
            composition.instances.push({
              id,
              module: {
                id: String(command.module),
                version: String(command.version),
              },
              config: structuredClone(config),
              effective_config: structuredClone(config),
              dependency_providers: {},
              source: { file: "composition/main.toml" },
            });
            changes.push({ object_id: `instance:${id}`, kind: "added" });
            summary.added += 1;
            break;
          }
          case "create_binding": {
            const id = String(command.id);
            composition.bindings.push({
              id,
              parent: String(command.parent),
              slot: String(command.slot),
              child: String(command.child),
              source: { file: "composition/main.toml" },
            });
            changes.push({ object_id: `binding:${id}`, kind: "added" });
            summary.added += 1;
            break;
          }
          case "update_binding": {
            const id = String(command.id);
            const binding = composition.bindings.find(
              (candidate) => candidate.id === id,
            );
            if (!binding) throw new Error(`missing binding ${id}`);
            binding.parent = String(command.parent);
            binding.slot = String(command.slot);
            binding.child = String(command.child);
            changes.push({ object_id: `binding:${id}`, kind: "modified" });
            summary.modified += 1;
            break;
          }
          case "delete_binding": {
            const id = String(command.id);
            const index = composition.bindings.findIndex(
              (binding) => binding.id === id,
            );
            if (index < 0) throw new Error(`missing binding ${id}`);
            composition.bindings.splice(index, 1);
            changes.push({ object_id: `binding:${id}`, kind: "removed" });
            summary.removed += 1;
            break;
          }
          case "create_connection": {
            const id = String(command.id);
            composition.connections.push({
              id,
              from: command.from as { instance: string; port: string },
              to: command.to as { instance: string; port: string },
              source: { file: "composition/main.toml" },
            });
            changes.push({ object_id: `connection:${id}`, kind: "added" });
            summary.added += 1;
            break;
          }
          case "update_connection": {
            const id = String(command.id);
            const connection = composition.connections.find(
              (candidate) => candidate.id === id,
            );
            if (!connection) throw new Error(`missing connection ${id}`);
            connection.from = command.from as {
              instance: string;
              port: string;
            };
            connection.to = command.to as {
              instance: string;
              port: string;
            };
            changes.push({ object_id: `connection:${id}`, kind: "modified" });
            summary.modified += 1;
            break;
          }
          case "delete_connection": {
            const id = String(command.id);
            const index = composition.connections.findIndex(
              (connection) => connection.id === id,
            );
            if (index < 0) throw new Error(`missing connection ${id}`);
            composition.connections.splice(index, 1);
            changes.push({ object_id: `connection:${id}`, kind: "removed" });
            summary.removed += 1;
            break;
          }
          case "delete_instance": {
            const id = String(command.id);
            if (id === composition.root) {
              throw new Error("root instance cannot be deleted");
            }
            const index = composition.instances.findIndex(
              (instance) => instance.id === id,
            );
            if (index < 0) throw new Error(`missing instance ${id}`);
            composition.instances.splice(index, 1);
            composition.bindings = composition.bindings.filter(
              (binding) => binding.parent !== id && binding.child !== id,
            );
            composition.connections = composition.connections.filter(
              (connection) =>
                connection.from.instance !== id &&
                connection.to.instance !== id,
            );
            for (const instance of composition.instances) {
              for (const [key, provider] of Object.entries(
                instance.dependency_providers,
              )) {
                if (provider === id) delete instance.dependency_providers[key];
              }
            }
            changes.push({ object_id: `instance:${id}`, kind: "removed" });
            summary.removed += 1;
            break;
          }
          case "rename_instance": {
            const from = String(command.from);
            const to = String(command.to);
            const instance = composition.instances.find(
              (candidate) => candidate.id === from,
            );
            if (!instance) throw new Error(`missing instance ${from}`);
            if (
              composition.instances.some((candidate) => candidate.id === to)
            ) {
              throw new Error(`instance ${to} already exists`);
            }
            instance.id = to;
            if (composition.root === from) composition.root = to;
            for (const binding of composition.bindings) {
              if (binding.parent === from) binding.parent = to;
              if (binding.child === from) binding.child = to;
            }
            for (const connection of composition.connections) {
              if (connection.from.instance === from) {
                connection.from.instance = to;
              }
              if (connection.to.instance === from) {
                connection.to.instance = to;
              }
            }
            for (const candidate of composition.instances) {
              for (const [key, provider] of Object.entries(
                candidate.dependency_providers,
              )) {
                if (provider === from) {
                  candidate.dependency_providers[key] = to;
                }
              }
            }
            changes.push(
              { object_id: `instance:${from}`, kind: "removed" },
              { object_id: `instance:${to}`, kind: "added" },
            );
            summary.modified += 1;
            break;
          }
          case "update_instance_module": {
            const id = String(command.id);
            const instance = composition.instances.find(
              (candidate) => candidate.id === id,
            );
            if (!instance) throw new Error(`missing instance ${id}`);
            instance.module = {
              id: String(command.module),
              version: String(command.version),
            };
            changes.push({ object_id: `instance:${id}`, kind: "modified" });
            summary.modified += 1;
            break;
          }
          case "set_config": {
            const instance = composition.instances.find(
              (candidate) => candidate.id === command.instance,
            );
            if (!instance) throw new Error(`missing instance ${command.instance}`);
            instance.config[String(command.property)] = command.value;
            instance.effective_config[String(command.property)] = command.value;
            changes.push({
              object_id: `instance:${command.instance}`,
              kind: "modified",
            });
            summary.modified += 1;
            break;
          }
          case "unset_config": {
            const instance = composition.instances.find(
              (candidate) => candidate.id === command.instance,
            );
            if (!instance) throw new Error(`missing instance ${command.instance}`);
            delete instance.config[String(command.property)];
            delete instance.effective_config[String(command.property)];
            changes.push({
              object_id: `instance:${command.instance}`,
              kind: "modified",
            });
            summary.modified += 1;
            break;
          }
          case "set_dependency_provider": {
            const instance = composition.instances.find(
              (candidate) => candidate.id === command.instance,
            );
            if (!instance) throw new Error(`missing instance ${command.instance}`);
            instance.dependency_providers[String(command.requirement)] =
              String(command.provider);
            refreshDependencies(model);
            changes.push({
              object_id: `instance:${command.instance}`,
              kind: "modified",
            });
            summary.modified += 1;
            break;
          }
          case "unset_dependency_provider": {
            const instance = composition.instances.find(
              (candidate) => candidate.id === command.instance,
            );
            if (!instance) throw new Error(`missing instance ${command.instance}`);
            delete instance.dependency_providers[String(command.requirement)];
            refreshDependencies(model);
            changes.push({
              object_id: `instance:${command.instance}`,
              kind: "modified",
            });
            summary.modified += 1;
            break;
          }
          default:
            throw new Error(`unmocked command: ${command.type}`);
        }
      }

      const seq = (model.revision?.seq ?? 0) + 1;
      model.revision = { seq, source_digest: `sha256:mock-${seq}` };
      next.sourceDigest = model.revision.source_digest;
      next.valid = true;
      next.errorCount = 0;
      return { snapshot: next, diff: changed(changes, summary) };
    }

    function transactionResponse(transaction: { commands: MockCommand[] }) {
      if (
        transaction.commands.some((command) => command.type === "delete_binding")
      ) {
        return detachRejectedResponse();
      }
      const result = applyCommands(transaction.commands);
      return {
        accepted: true,
        replayed: false,
        diff: result.diff,
        diagnostics: [],
        snapshot: result.snapshot,
      };
    }

    function detachRejectedResponse() {
      return {
        accepted: false,
        replayed: false,
        diff: null,
        diagnostics: [
          {
            code: "structure-unreachable-instance",
            severity: "error",
            message: "instance would not be reachable from root",
            file: "composition/main.toml",
            range: {
              start: { line: 1, column: 1 },
              end: { line: 1, column: 1 },
            },
          },
        ],
        snapshot: current,
      };
    }

    function rejectedResponse() {
      return {
        accepted: false,
        replayed: false,
        diff: null,
        diagnostics: [
          {
            code: "contract-property-type-mismatch",
            severity: "error",
            message: "value does not match the property contract",
            file: "composition/main.toml",
            range: {
              start: { line: 1, column: 1 },
              end: { line: 1, column: 1 },
            },
          },
        ],
        snapshot: current,
      };
    }

    const invoke = async (cmd: string, args: Record<string, unknown> = {}) => {
      switch (cmd) {
        case "initial_project":
          return "valid-project";
        case "open_project":
          return data.mode === "invalid"
            ? structuredClone(data.invalid)
            : structuredClone(current);
        case "read_source":
          return { entry: "composition/main.toml", text: data.sourceText };
        case "load_layout":
          return {};
        case "save_layout":
          testState.saveLayoutCalls += 1;
          testState.lastLayout = args.nodes;
          return null;
        case "check_external_changes":
          return {
            hasSession: true,
            dirty: testState.dirty,
            sessionDigest: current.sourceDigest,
            diskDigest: current.sourceDigest,
            diskValid: true,
            diskErrorCount: 0,
          };
        case "reload_project":
          testState.dirty = false;
          return structuredClone(current);
        case "prepare_transaction": {
          testState.prepareCalls += 1;
          const transaction = args.transaction as { commands: MockCommand[] };
          testState.lastCommands = transaction.commands;
          if (data.mode === "edit-failure") return rejectedResponse();
          return transactionResponse(transaction);
        }
        case "apply_transaction": {
          testState.applyCalls += 1;
          const transaction = args.transaction as { commands: MockCommand[] };
          testState.lastCommands = transaction.commands;
          if (data.mode === "edit-failure") return rejectedResponse();
          const response = transactionResponse(transaction);
          current = response.snapshot;
          return response;
        }
        case "plugin:dialog|open":
          return null;
        default:
          throw new Error(`unmocked command: ${cmd}`);
      }
    };

    const internals = {
      invoke,
      transformCallback: (callback: unknown) => {
        const id = Math.floor(Math.random() * 1_000_000);
        (window as unknown as Record<string, unknown>)[`_${id}`] = callback;
        return id;
      },
      unregisterCallback: () => undefined,
      convertFileSrc: (path: string) => path,
      metadata: { currentWebview: { label: "main" } },
    };
    (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = internals;
  }, payload);
}
