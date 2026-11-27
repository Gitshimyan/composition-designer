import type { Page } from "@playwright/test";
import { invalidSnapshot, sourceText, validSnapshot } from "./fixtures";

export type MockMode = "valid" | "invalid" | "edit-failure";

interface MockPayload {
  mode: MockMode;
  valid: ReturnType<typeof validSnapshot>;
  invalid: ReturnType<typeof invalidSnapshot>;
  sourceText: string;
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
      get snapshot() {
        return current;
      },
    };

    Object.defineProperty(window, "__COMPOSITION_TEST__", {
      value: testState,
      configurable: true,
    });

    function updatedSnapshot(
      command: { instance: string; property: string; value: unknown },
    ) {
      const next = structuredClone(current);
      const instance = next.model?.composition.instances.find(
        (item) => item.id === command.instance,
      );
      if (instance) {
        instance.config[command.property] = command.value;
        instance.effective_config[command.property] = command.value;
        next.model!.revision = {
          seq: (next.model!.revision?.seq ?? 0) + 1,
          source_digest: `sha256:mock-${testState.applyCalls + 1}`,
        };
        next.sourceDigest = next.model!.revision?.source_digest ?? null;
      }
      return next;
    }

    function transactionResponse(transaction: {
      commands: Array<{
        type: string;
        instance: string;
        property: string;
        value?: unknown;
      }>;
    }) {
      const command = transaction.commands[0];
      const next = updatedSnapshot({
        instance: command.instance,
        property: command.property,
        value: command.value,
      });
      const before = current.model?.composition.instances.find(
        (item) => item.id === command.instance,
      )?.config[command.property];
      return {
        accepted: true,
        replayed: false,
        diff: {
          summary: { added: 0, removed: 0, modified: 1 },
          changes: [
            {
              object_id: command.instance,
              kind: "modified",
              before: { [command.property]: before },
              after: { [command.property]: command.value },
            },
          ],
        },
        diagnostics: [],
        snapshot: next,
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
        case "prepare_transaction":
          testState.prepareCalls += 1;
          if (data.mode === "edit-failure") return rejectedResponse();
          return transactionResponse(
            args.transaction as {
              commands: Array<{
                type: string;
                instance: string;
                property: string;
                value?: unknown;
              }>;
            },
          );
        case "apply_transaction":
          testState.applyCalls += 1;
          if (data.mode === "edit-failure") return rejectedResponse();
          const response = transactionResponse(
            args.transaction as {
              commands: Array<{
                type: string;
                instance: string;
                property: string;
                value?: unknown;
              }>;
            },
          );
          current = response.snapshot;
          return response;
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
