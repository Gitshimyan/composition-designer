import { useEffect, useMemo, useState } from "react";
import type { FormEvent, ReactNode } from "react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import {
  applyTransaction,
  checkExternalChanges,
  initialProfile,
  initialProject,
  isTauri,
  loadLayout,
  openProject,
  prepareTransaction,
  readSource,
  redo as performRedo,
  reloadProject,
  saveLayout,
  undo as performUndo,
} from "./api";
import CompositionCanvas from "./canvas/CompositionCanvas";
import StructureTree from "./explorer/StructureTree";
import BindingEditorDialog from "./inspector/BindingEditorDialog";
import CreateInstanceDialog from "./inspector/CreateInstanceDialog";
import CreateServiceDialog from "./inspector/CreateServiceDialog";
import InstanceActions from "./inspector/InstanceActions";
import { parseDraft, toDraftValue } from "./inspector/configDraft";
import { autoLayout, buildStructure } from "./layout";
import ConnectionDialog from "./relations/ConnectionDialog";
import ConnectionView from "./relations/ConnectionView";
import DependencyProviderDialog from "./relations/DependencyProviderDialog";
import DependencyView from "./relations/DependencyView";
import ServicesView from "./relations/ServicesView";
import type { DependencyDetail } from "./relations/relationModel";
import ChangeReview from "./review/ChangeReview";
import SourceView from "./source/SourceView";
import type {
  CompositionModel,
  Connection,
  Diagnostic,
  EditCommand,
  EditResponse,
  ExternalChangeState,
  Instance,
  InstanceScope,
  InterfaceDeclaration,
  LayoutNodes,
  NodePosition,
  ProjectSnapshot,
  PropertyContract,
  ReviewOptions,
  SourcePayload,
  Transaction,
} from "./types";

const LAST_PATH_KEY = "composition-designer.last-project";
const LAST_PROFILE_KEY = "composition-designer.last-profile";

type LoadState = "idle" | "loading" | "ready" | "error";

export default function App() {
  const [snapshot, setSnapshot] = useState<ProjectSnapshot | null>(null);
  const [loadState, setLoadState] = useState<LoadState>("idle");
  const [error, setError] = useState<string | null>(null);
  const [showPath, setShowPath] = useState(false);
  const [pathInput, setPathInput] = useState("");
  const [positions, setPositions] = useState<LayoutNodes>({});
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [focusRequest, setFocusRequest] = useState<{
    id: string;
    nonce: number;
  } | null>(null);
  const [layoutStatus, setLayoutStatus] = useState<string | null>(null);
  const [fitNonce, setFitNonce] = useState(0);
  const [bottomTab, setBottomTab] = useState<"diagnostics" | "source">("diagnostics");
  const [source, setSource] = useState<SourcePayload | null>(null);
  const [revealLine, setRevealLine] = useState<{ line: number; nonce: number } | null>(null);
  const [pendingReview, setPendingReview] = useState<{
    transaction: Transaction;
    preview: EditResponse;
    description: string;
    options: ReviewOptions;
  } | null>(null);
  const [reviewBusy, setReviewBusy] = useState(false);
  const [editError, setEditError] = useState<string | null>(null);
  const [external, setExternal] = useState<ExternalChangeState | null>(null);
  const [reloading, setReloading] = useState(false);
  const [createDraft, setCreateDraft] = useState<{
    parentId: string;
    slotId: string;
  } | null>(null);
  const [serviceDraft, setServiceDraft] = useState(false);
  const [bindingDraft, setBindingDraft] = useState<string | null>(null);
  const [connectionDraft, setConnectionDraft] = useState<
    { mode: "create" } | { mode: "edit"; id: string } | null
  >(null);
  const [stageView, setStageView] = useState<
    "structure" | "connections" | "dependencies" | "services"
  >("structure");
  const [providerDraft, setProviderDraft] = useState<{
    instanceId: string;
    requirementId: string;
  } | null>(null);

  useEffect(() => {
    let cancelled = false;
    async function restore() {
      if (!isTauri()) return;
      try {
        const initial = await initialProject();
        const initialProfileName = await initialProfile();
        const stored = window.localStorage.getItem(LAST_PATH_KEY);
        const storedProfile = window.localStorage.getItem(LAST_PROFILE_KEY);
        const path = initial ?? stored;
        const profile = initialProfileName ?? storedProfile;
        if (path && !cancelled) {
          await loadProject(path, profile);
        }
      } catch {
        // A missing startup argument is not an application error.
      }
    }
    void restore();
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (!(event.ctrlKey || event.metaKey) || event.altKey) return;
      const target = event.target as HTMLElement | null;
      if (
        target &&
        (target.tagName === "INPUT" ||
          target.tagName === "TEXTAREA" ||
          target.tagName === "SELECT" ||
          target.isContentEditable)
      ) {
        return;
      }
      const key = event.key.toLowerCase();
      if (key === "z" && !event.shiftKey) {
        event.preventDefault();
        void handleUndo();
      } else if ((key === "z" && event.shiftKey) || key === "y") {
        event.preventDefault();
        void handleRedo();
      }
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [snapshot, loadState]);

  async function loadProject(rawPath: string, profile: string | null = null) {
    const path = rawPath.trim();
    if (!path) {
      setShowPath(true);
      return;
    }
    setLoadState("loading");
    setError(null);
    try {
      const next = await openProject(path, profile);
      setSnapshot(next);
      setPathInput(path);
      setLoadState("ready");
      window.localStorage.setItem(LAST_PATH_KEY, path);
      if (profile) {
        window.localStorage.setItem(LAST_PROFILE_KEY, profile);
      } else {
        window.localStorage.removeItem(LAST_PROFILE_KEY);
      }
    } catch (reason) {
      setLoadState("error");
      setError(reason instanceof Error ? reason.message : String(reason));
    }
  }

  function switchProfile(profile: string) {
    if (!snapshot) return;
    void loadProject(snapshot.root, profile);
  }

  async function handleUndo() {
    if (!snapshot?.canUndo || loadState === "loading") return;
    setError(null);
    try {
      const response = await performUndo();
      setSnapshot(response.snapshot);
      if (!response.accepted) {
        setError(
          response.diagnostics[0]?.message ?? "Undo was rejected by the Core.",
        );
      }
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    }
  }

  async function handleRedo() {
    if (!snapshot?.canRedo || loadState === "loading") return;
    setError(null);
    try {
      const response = await performRedo();
      setSnapshot(response.snapshot);
      if (!response.accepted) {
        setError(
          response.diagnostics[0]?.message ?? "Redo was rejected by the Core.",
        );
      }
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    }
  }

  async function handleChooseProject() {
    if (!isTauri()) {
      setShowPath(true);
      return;
    }
    try {
      const selected = await openDialog({
        directory: true,
        multiple: false,
        title: "Open Composition Project",
      });
      if (typeof selected === "string") {
        await loadProject(selected, null);
      }
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    }
  }

  function handlePathSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    void loadProject(pathInput, null);
  }

  const model = snapshot?.model ?? null;
  const project = model?.project ?? null;
  const composition = model?.composition ?? null;
  const instances = composition?.instances ?? [];
  const bindings = composition?.bindings ?? [];
  const connections = composition?.connections ?? [];
  const dependencies = model?.dependencies ?? [];
  const modules = model?.modules ?? [];
  const services = instances.filter(
    (instance) => instance.scope === "service",
  );
  const diagnostics = snapshot?.diagnostics ?? [];
  const rootId = composition?.root ?? "";

  useEffect(() => {
    if (!snapshot || !model) {
      setPositions({});
      setSelectedId(null);
      setFocusRequest(null);
      setLayoutStatus(null);
      return;
    }
    let cancelled = false;
    const auto = autoLayout(
      model.composition.instances,
      model.composition.bindings,
      model.composition.root,
    );
    setPositions(auto);
    setSelectedId(model.composition.root || null);
    setFocusRequest(null);
    setLayoutStatus("auto layout");
    setBottomTab("diagnostics");
    setRevealLine(null);
    setPendingReview(null);
    setCreateDraft(null);
    setServiceDraft(false);
    setBindingDraft(null);
    setConnectionDraft(null);
    setProviderDraft(null);
    setStageView("structure");
    void loadLayout(snapshot.root)
      .then((saved) => {
        if (cancelled) return;
        const merged = { ...auto };
        for (const [id, position] of Object.entries(saved)) {
          if (id in merged) merged[id] = position;
        }
        setPositions(merged);
        setFitNonce((value) => value + 1);
        setLayoutStatus(
          Object.keys(saved).length > 0 ? "layout restored" : "auto layout",
        );
      })
      .catch(() => {
        if (!cancelled) setLayoutStatus("layout load failed");
      });
    return () => {
      cancelled = true;
    };
    // Re-run only when a different project is opened, not after edits.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [snapshot?.root]);

  useEffect(() => {
    let cancelled = false;
    if (!snapshot?.canEdit) {
      setSource(null);
      return;
    }
    void readSource()
      .then((payload) => {
        if (!cancelled) setSource(payload);
      })
      .catch(() => {
        if (!cancelled) setSource(null);
      });
    return () => {
      cancelled = true;
    };
  }, [snapshot?.root, snapshot?.canEdit]);

  useEffect(() => {
    if (!snapshot?.canEdit) {
      setExternal(null);
      return;
    }
    let cancelled = false;
    async function check() {
      try {
        const state = await checkExternalChanges();
        if (!cancelled) setExternal(state);
      } catch {
        if (!cancelled) setExternal(null);
      }
    }
    void check();
    const timer = window.setInterval(() => void check(), 2000);
    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, [snapshot?.root, snapshot?.canEdit]);

  async function persistLayout(next: LayoutNodes) {
    if (!snapshot) return;
    try {
      await saveLayout(snapshot.root, next);
      setLayoutStatus("layout saved");
    } catch {
      setLayoutStatus("layout save failed");
    }
  }

  function handleMoveNode(id: string, position: NodePosition) {
    setPositions((current) => ({ ...current, [id]: position }));
  }

  function handleCommitNode(id: string, position: NodePosition) {
    const next = { ...positions, [id]: position };
    setPositions(next);
    void persistLayout(next);
  }

  function handleResetLayout() {
    if (!model) return;
    const next = autoLayout(
      model.composition.instances,
      model.composition.bindings,
      model.composition.root,
    );
    setPositions(next);
    setFitNonce((value) => value + 1);
    setLayoutStatus("layout reset");
    void persistLayout(next);
  }

  async function handleReloadProject() {
    setReloading(true);
    setEditError(null);
    const review = pendingReview;
    try {
      const next = await reloadProject();
      setSnapshot(next);
      setExternal(null);
      setCreateDraft(null);
      setServiceDraft(false);
      setBindingDraft(null);
      setConnectionDraft(null);
      setProviderDraft(null);
      setRevealLine(null);
      if (next.model) {
        const composition = next.model.composition;
        const auto = autoLayout(
          composition.instances,
          composition.bindings,
          composition.root,
        );
        setPositions(auto);
        setFitNonce((value) => value + 1);
        setSelectedId(composition.root || null);
      } else {
        setSource(null);
        setSelectedId(null);
      }
      if (next.canEdit) {
        try {
          setSource(await readSource());
        } catch {
          setSource(null);
        }
      } else {
        setSource(null);
      }
      if (review && next.model?.revision) {
        const rebased = {
          ...review.transaction,
          base_revision: next.model.revision,
        };
        try {
          const preview = await prepareTransaction(rebased);
          setPendingReview({ ...review, transaction: rebased, preview });
        } catch (reason) {
          setPendingReview(null);
          setEditError(
            reason instanceof Error ? reason.message : String(reason),
          );
        }
      } else if (!next.model) {
        setPendingReview(null);
      }
    } catch (reason) {
      setEditError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setReloading(false);
    }
  }

  function handleSelectTree(id: string) {
    setSelectedId(id);
    setFocusRequest((current) => ({
      id,
      nonce: (current?.nonce ?? 0) + 1,
    }));
  }

  function handleSelectCanvas(id: string | null) {
    setSelectedId(id);
  }

  async function reviewCommands(
    commands: EditCommand[],
    description: string,
    options: ReviewOptions = {},
  ) {
    if (!model?.revision || !snapshot?.canEdit) {
      setEditError("This project is not editable.");
      return;
    }
    if (external?.dirty) {
      setEditError("Files changed on disk. Reload the project before editing.");
      return;
    }
    const requestId =
      typeof crypto !== "undefined" && "randomUUID" in crypto
        ? crypto.randomUUID()
        : `request-${Date.now()}-${Math.random().toString(16).slice(2)}`;
    const transaction: Transaction = {
      request_id: requestId,
      base_revision: {
        seq: model.revision.seq,
        source_digest: model.revision.source_digest ?? null,
      },
      commands,
    };
    setReviewBusy(true);
    setEditError(null);
    try {
      const preview = await prepareTransaction(transaction);
      setPendingReview({ transaction, preview, description, options });
    } catch (reason) {
      setEditError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setReviewBusy(false);
    }
  }

  async function applyPendingReview() {
    if (!pendingReview) return;
    if (external?.dirty) {
      setEditError("Files changed on disk. Reload before applying.");
      return;
    }
    setReviewBusy(true);
    setEditError(null);
    try {
      const response = await applyTransaction(pendingReview.transaction);
      setSnapshot(response.snapshot);
      if (!response.accepted) {
        setPendingReview({ ...pendingReview, preview: response });
        return;
      }
      const nextModel = response.snapshot.model;
      if (nextModel) {
        setPositions((current) =>
          reconcilePositions(
            nextModel,
            current,
            pendingReview.options.alias,
          ),
        );
      }
      if (pendingReview.options.selectAfter !== undefined) {
        const nextId = pendingReview.options.selectAfter;
        setSelectedId(nextId);
        if (nextId && pendingReview.options.focusAfter) {
          setFocusRequest((current) => ({
            id: nextId,
            nonce: (current?.nonce ?? 0) + 1,
          }));
        }
      }
      setPendingReview(null);
      setCreateDraft(null);
    setServiceDraft(false);
      setBindingDraft(null);
      setConnectionDraft(null);
      setProviderDraft(null);
      if (response.snapshot.canEdit) {
        try {
          setSource(await readSource());
        } catch {
          setSource(null);
        }
      }
    } catch (reason) {
      setEditError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setReviewBusy(false);
    }
  }

  function handleDiagnosticClick(diagnostic: Diagnostic) {
    if (!source || diagnostic.file !== source.entry || !diagnostic.range) {
      setBottomTab("diagnostics");
      return;
    }
    setBottomTab("source");
    setRevealLine((current) => ({
      line: diagnostic.range?.start.line ?? 1,
      nonce: (current?.nonce ?? 0) + 1,
    }));
  }

  function handleDeleteConnection(connection: Connection) {
    void reviewCommands(
      [{ type: "delete_connection", id: connection.id }],
      `Delete connection ${connection.id}`,
      { selectAfter: connection.from.instance },
    );
  }

  function handleSetDependencyProvider(detail: DependencyDetail) {
    setProviderDraft({
      instanceId: detail.resolution.from_instance,
      requirementId: detail.resolution.requirement_id,
    });
  }

  function handleClearDependencyProvider(detail: DependencyDetail) {
    const instanceId = detail.resolution.from_instance;
    const requirementId = detail.resolution.requirement_id;
    void reviewCommands(
      [
        {
          type: "unset_dependency_provider",
          instance: instanceId,
          requirement: requirementId,
        },
      ],
      `Clear ${instanceId}.${requirementId} provider`,
      { selectAfter: instanceId },
    );
  }

  const moduleKinds = useMemo(
    () => new Map(modules.map((module) => [module.id, module.kind])),
    [modules],
  );
  const structure = useMemo(
    () => buildStructure(instances, bindings, rootId),
    [instances, bindings, rootId],
  );
  const selectedInstance = instances.find((instance) => instance.id === selectedId);
  const selectedModule = selectedInstance
    ? modules.find(
        (module) =>
          module.id === selectedInstance.module.id &&
          module.version === selectedInstance.module.version,
      )
    : undefined;
  const createParent = createDraft
    ? instances.find((instance) => instance.id === createDraft.parentId)
    : undefined;
  const createParentModule = createParent
    ? modules.find(
        (module) =>
          module.id === createParent.module.id &&
          module.version === createParent.module.version,
      )
    : undefined;
  const createSlot = createParentModule?.contract.slots.find(
    (slot) => slot.id === createDraft?.slotId,
  );
  const bindingDraftValue = bindingDraft
    ? bindings.find((binding) => binding.id === bindingDraft)
    : undefined;
  const bindingDraftChild = bindingDraftValue
    ? instances.find((instance) => instance.id === bindingDraftValue.child)
    : undefined;
  const draftConnection =
    connectionDraft?.mode === "edit"
      ? connections.find((connection) => connection.id === connectionDraft.id)
      : undefined;
  const modulePortCount = modules.reduce(
    (total, module) => total + module.contract.ports.length,
    0,
  );
  const resolvedDependencyCount = dependencies.filter(
    (dependency) => dependency.status === "resolved",
  ).length;

  const status = describeStatus(loadState, snapshot, error);
  const digest = snapshot?.sourceDigest ?? model?.revision?.source_digest ?? null;
  const shortDigest = digest ? `${digest.slice(0, 7)}…${digest.slice(-6)}` : "—";
  const valid = snapshot?.valid ?? false;
  const canEdit = snapshot?.canEdit ?? false;
  const externalDirty = external?.dirty ?? false;

  return (
    <div className="app-shell">
      <header className="topbar">
        <div className="brand">
          <div className="brand-mark" aria-hidden="true">
            CD
          </div>
          <div className="brand-copy">
            <div className="brand-name">Composition Designer</div>
            <div className="brand-sub">compose · inspect · validate</div>
          </div>
        </div>

        <div className="topbar-project" aria-live="polite">
          {project ? (
            <>
              <span className="project-name">{project.name}</span>
              <code>{project.id}</code>
              {project.schema_version === 3 &&
                (project.profiles?.length ?? 0) > 0 && (
                  <label className="profile-picker">
                    <span>Profile</span>
                    <select
                      aria-label="Composition profile"
                      value={project.active_profile ?? "default"}
                      disabled={loadState === "loading"}
                      onChange={(event) => switchProfile(event.target.value)}
                    >
                      <option value="default">default</option>
                      {project.profiles?.map((profile) => (
                        <option key={profile.name} value={profile.name}>
                          {profile.name}
                        </option>
                      ))}
                    </select>
                  </label>
                )}
            </>
          ) : (
            <span className="muted">No project open</span>
          )}
        </div>

        <div className="topbar-actions">
          <button
            type="button"
            className="button button-ghost"
            onClick={() => void handleUndo()}
            disabled={!snapshot?.canUndo || loadState === "loading"}
            title="Undo (Ctrl+Z)"
          >
            Undo
          </button>
          <button
            type="button"
            className="button button-ghost"
            onClick={() => void handleRedo()}
            disabled={!snapshot?.canRedo || loadState === "loading"}
            title="Redo (Ctrl+Shift+Z / Ctrl+Y)"
          >
            Redo
          </button>
          <button
            type="button"
            className="button button-ghost"
            onClick={() => setShowPath((value) => !value)}
          >
            Path
          </button>
          <button
            type="button"
            className="button button-primary"
            onClick={() => void handleChooseProject()}
            disabled={loadState === "loading"}
          >
            Open project
          </button>
        </div>
      </header>

      {showPath && (
        <form className="path-strip" onSubmit={handlePathSubmit}>
          <label htmlFor="project-path">Project folder</label>
          <input
            id="project-path"
            value={pathInput}
            onChange={(event) => setPathInput(event.target.value)}
            placeholder="E:\\Work.Projects\\Composition Designer\\examples\\minimal-project"
            spellCheck={false}
            autoComplete="off"
          />
          <button type="submit" className="button button-primary" disabled={loadState === "loading"}>
            Open path
          </button>
          <button
            type="button"
            className="button button-ghost"
            onClick={() => setShowPath(false)}
          >
            Close
          </button>
          {!isTauri() && (
            <span className="path-note">
              Browser preview: enter an absolute path; the Tauri host is required for IPC.
            </span>
          )}
        </form>
      )}

      {error && (
        <div className="error-banner" role="alert">
          <strong>Could not open project.</strong>
          <span>{error}</span>
          <button type="button" onClick={() => setError(null)} aria-label="Dismiss error">
            ×
          </button>
        </div>
      )}

      {editError && (
        <div className="error-banner" role="alert">
          <strong>Edit failed.</strong>
          <span>{editError}</span>
          <button type="button" onClick={() => setEditError(null)} aria-label="Dismiss edit error">
            ×
          </button>
        </div>
      )}

      {externalDirty && (
        <div className="external-banner" role="alert">
          <div>
            <strong>Files changed on disk.</strong>
            <span>
              Reload before editing. Local apply is blocked while the source digest differs.
              {external && external.diskErrorCount > 0
                ? ` Disk now has ${external.diskErrorCount} error(s).`
                : ""}
            </span>
          </div>
          <button
            type="button"
            className="button button-primary"
            onClick={() => void handleReloadProject()}
            disabled={reloading}
          >
            {reloading ? "Reloading…" : "Reload project"}
          </button>
        </div>
      )}

      <main className="workspace">
        <aside className="panel explorer" aria-label="Project explorer">
          <PanelHeading eyebrow="Workspace" title="Explorer" />
          <div className="explorer-scroll">
            {project ? (
              <>
                <section className="explorer-section">
                  <div className="section-label">Project</div>
                  <div className="project-card">
                    <div className="project-card-name">{project.name}</div>
                    <code>{project.id}</code>
                    <span className="project-entry">{project.composition_entry}</span>
                  </div>
                </section>

                <section className="explorer-section">
                  <div className="section-label">
                    Modules <span>{modules.length}</span>
                  </div>
                  <ul className="tree-list">
                    {modules.map((module) => (
                      <li key={`${module.id}@${module.version}`}>
                        <div className="tree-row">
                          <span className={`kind-dot kind-${module.kind}`} />
                          <span className="tree-name">{module.id}</span>
                          <code className="tree-version">{module.version}</code>
                        </div>
                      </li>
                    ))}
                  </ul>
                </section>

                <section className="explorer-section">
                  <div className="section-label">
                    Structure <span>{instances.length}</span>
                  </div>
                  <StructureTree
                    structure={structure}
                    selectedId={selectedId}
                    onSelect={handleSelectTree}
                  />
                </section>

                <section className="explorer-section">
                  <div className="section-label">Edges</div>
                  <div className="edge-summary">
                    <span>{bindings.length} bindings</span>
                    <span>{connections.length} connections</span>
                  </div>
                </section>
              </>
            ) : (
              <div className="explorer-empty">
                <span className="empty-glyph">◇</span>
                <p>Open a project to inspect its composition tree, module inventory and edges.</p>
              </div>
            )}
          </div>
        </aside>

        <section className="panel stage" aria-label="Composition views">
          {snapshot === null ? (
            <EmptyState
              onChoose={() => void handleChooseProject()}
              onPath={() => setShowPath(true)}
            />
          ) : model === null ? (
            <InvalidState snapshot={snapshot} diagnostics={diagnostics} />
          ) : (
            <>
              <PanelHeading
                eyebrow="Composition"
                title={project?.name ?? "Composition"}
                meta={
                  <span className="meta-pill">
                    root <code>{rootId}</code>
                  </span>
                }
              />
              <div className="stage-views" role="tablist" aria-label="Relation views">
                {(
                  [
                    ["structure", "Structure", bindings.length],
                    ["connections", "Connections", connections.length],
                    ["dependencies", "Dependencies", dependencies.length],
                    ["services", "Services", services.length],
                  ] as const
                ).map(([value, label, count]) => (
                  <button
                    key={value}
                    type="button"
                    role="tab"
                    aria-selected={stageView === value}
                    className={`stage-view-tab ${
                      stageView === value ? "is-active" : ""
                    }`}
                    onClick={() => setStageView(value)}
                  >
                    {label} <span className="tab-count">{count}</span>
                  </button>
                ))}
              </div>
              {stageView === "structure" ? (
                <CompositionCanvas
                  rootId={rootId}
                  instances={instances}
                  bindings={bindings}
                  positions={positions}
                  selectedId={selectedId}
                  focusRequest={focusRequest}
                  fitNonce={fitNonce}
                  moduleKinds={moduleKinds}
                  onSelect={handleSelectCanvas}
                  onMoveNode={handleMoveNode}
                  onCommitNode={handleCommitNode}
                  onResetLayout={handleResetLayout}
                />
              ) : stageView === "connections" ? (
                <ConnectionView
                  instances={instances}
                  modules={modules}
                  connections={connections}
                  selectedId={selectedId}
                  canEdit={canEdit && !externalDirty}
                  onSelectInstance={handleSelectTree}
                  onCreateConnection={() => setConnectionDraft({ mode: "create" })}
                  onEditConnection={(connection) =>
                    setConnectionDraft({ mode: "edit", id: connection.id })
                  }
                  onDeleteConnection={handleDeleteConnection}
                />
                ) : stageView === "services" ? (
                  <ServicesView
                    instances={instances}
                    modules={modules}
                    connections={connections}
                    dependencies={dependencies}
                    selectedId={selectedId}
                    canEdit={canEdit && !externalDirty}
                    projectSchemaVersion={model?.project.schema_version ?? 1}
                    onSelectInstance={handleSelectTree}
                    onNewService={() => setServiceDraft(true)}
                  />
                ) : (
                  <DependencyView
                  instances={instances}
                  modules={modules}
                  dependencies={dependencies}
                  selectedId={selectedId}
                  canEdit={canEdit && !externalDirty}
                  onSelectInstance={handleSelectTree}
                  onSetProvider={handleSetDependencyProvider}
                  onClearProvider={handleClearDependencyProvider}
                />
              )}
              <footer className="stage-footer">
                <span className="stage-note">
                  {stageView === "structure"
                    ? "Drag to arrange · positions write to .designer/layout.toml · structure edits go through Core transactions"
                    : stageView === "connections"
                      ? "Port connections · direction/type/protocol come from module contracts · edits go through Core transactions"
                      : stageView === "services"
                        ? "Services live outside the Binding tree · primitive modules only · edits go through Core transactions"
                        : "Dependency resolutions · explicit providers are editable through Core transactions"}
                </span>
                <div className="stage-counts">
                  {stageView === "structure" ? (
                    <>
                      <span>{bindings.length} structural bindings</span>
                      <span>{connections.length} port connections</span>
                      <span>{dependencies.length} dependency resolutions</span>
                    </>
                  ) : stageView === "connections" ? (
                    <>
                      <span>{connections.length} port connections</span>
                      <span>{modulePortCount} module ports</span>
                    </>
                  ) : stageView === "services" ? (
                    <>
                      <span>{services.length} services</span>
                      <span>
                        {instances.length - services.length} structure instances
                      </span>
                    </>
                  ) : (
                    <>
                      <span>{dependencies.length} dependency resolutions</span>
                      <span>{resolvedDependencyCount} resolved</span>
                    </>
                  )}
                </div>
              </footer>
            </>
          )}
        </section>

        <aside className="panel inspector" aria-label="Inspector">
          <PanelHeading eyebrow="Selection" title="Inspector" />
          <div className="inspector-scroll">
            {selectedInstance ? (
              <SelectionDetails
                instance={selectedInstance}
                moduleKind={selectedModule?.kind}
                scope={selectedInstance.scope}
                interfaces={selectedModule?.contract.interfaces ?? []}
                slots={selectedModule?.contract.slots.length ?? 0}
                ports={selectedModule?.contract.ports.length ?? 0}
                properties={selectedModule?.contract.properties ?? []}
                isRoot={selectedInstance.id === rootId}
                canEdit={canEdit && !externalDirty}
                revisionSeq={model?.revision?.seq ?? 0}
                onReview={(commands, description) =>
                  void reviewCommands(commands, description)
                }
                actions={
                  <InstanceActions
                    key={`${selectedInstance.id}:${model?.revision?.seq ?? 0}`}
                    instance={selectedInstance}
                    instances={instances}
                    bindings={bindings}
                    connections={connections}
                    dependencies={dependencies}
                    modules={modules}
                    rootId={rootId}
                    canEdit={canEdit && !externalDirty}
                    onCreateChild={(parent, slot) =>
                      setCreateDraft({ parentId: parent.id, slotId: slot.id })
                    }
                    onEditBinding={(binding) => setBindingDraft(binding.id)}
                    onReview={(commands, description, options) =>
                      void reviewCommands(commands, description, options)
                    }
                  />
                }
              />
            ) : project ? (
              <>
                <section className="inspector-section">
                  <div className="section-label">Project</div>
                  <Property label="Name" value={project.name} />
                  <Property label="Id" value={project.id} mono />
                  <Property label="Root" value={project.composition_entry} mono />
                  <Property label="Module roots" value={project.module_roots.join(", ")} mono />
                </section>
                <section className="inspector-section">
                  <div className="section-label">Revision</div>
                  <Property label="Seq" value={String(model?.revision?.seq ?? "—")} mono />
                  <Property label="Digest" value={shortDigest} mono />
                  <Property label="Valid" value={valid ? "yes" : "no"} />
                </section>
                <div className="inspector-hint">
                  Select a node or tree row to inspect and edit an instance.
                </div>
              </>
            ) : (
              <div className="inspector-empty">No selection.</div>
            )}
          </div>
        </aside>

        <section className="panel diagnostics" aria-label="Diagnostics and source">
          <div className="diagnostics-head">
            <div className="panel-tabs" role="tablist" aria-label="Bottom panel">
              <button
                type="button"
                role="tab"
                aria-selected={bottomTab === "diagnostics"}
                className={`panel-tab ${bottomTab === "diagnostics" ? "is-active" : ""}`}
                onClick={() => setBottomTab("diagnostics")}
              >
                Diagnostics <span className="tab-count">{diagnostics.length}</span>
              </button>
              <button
                type="button"
                role="tab"
                aria-selected={bottomTab === "source"}
                className={`panel-tab ${bottomTab === "source" ? "is-active" : ""}`}
                onClick={() => setBottomTab("source")}
                disabled={!source}
              >
                TOML Source
              </button>
            </div>
            <div className="diagnostic-counts">
              <span className="count count-error">{snapshot?.errorCount ?? 0} errors</span>
              <span className="count count-warning">{snapshot?.warningCount ?? 0} warnings</span>
            </div>
          </div>
          <div className="diagnostics-body">
            {bottomTab === "source" ? (
              source ? (
                <div className="source-shell">
                  <div className="source-meta">
                    <code>{source.entry}</code>
                    <span>
                      {snapshot?.canEdit
                        ? "editable · Review before writing"
                        : "read-only"}
                    </span>
                  </div>
                  <SourceView
                    text={source.text}
                    revealLine={revealLine}
                    editable={Boolean(snapshot?.canEdit)}
                    blocked={Boolean(external?.dirty)}
                    onReview={(nextSource) =>
                      void reviewCommands(
                        [{ type: "replace_source", source: nextSource }],
                        "Edit composition source",
                      )
                    }
                  />
                </div>
              ) : (
                <div className="diagnostics-empty">
                  <span className="status-glyph">·</span>
                  <span>No editable source session. Open a valid project to inspect TOML.</span>
                </div>
              )
            ) : diagnostics.length === 0 ? (
              <div className={`diagnostics-empty ${valid ? "is-valid" : ""}`}>
                <span className="status-glyph">{valid ? "✓" : "·"}</span>
                <span>
                  {valid
                    ? "No diagnostics. The project is valid against L1–L7."
                    : "No diagnostics returned for this project."}
                </span>
              </div>
            ) : (
              <ul className="diagnostic-list">
                {diagnostics.map((diagnostic, index) => (
                  <li key={`${diagnostic.code}-${index}`}>
                    <button
                      type="button"
                      className="diagnostic-row"
                      onClick={() => handleDiagnosticClick(diagnostic)}
                    >
                      <span className={`severity-dot severity-${diagnostic.severity}`} />
                      <code className="diagnostic-code">{diagnostic.code}</code>
                      <span className="diagnostic-message">{diagnostic.message}</span>
                      <span className="diagnostic-location">{diagnosticLocation(diagnostic)}</span>
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </div>
        </section>

      </main>

      <footer className="statusbar">
        <div className="status-left">
          <span className={`status-dot ${status.tone}`} />
          <span>{status.label}</span>
          {snapshot && (
            <span className="status-sep">
              {snapshot.errorCount} errors · {snapshot.warningCount} warnings
            </span>
          )}
          {externalDirty && <span className="status-sep status-warn">external changes</span>}
        </div>
        <div className="status-right">
          {layoutStatus && <span className="status-layout">{layoutStatus}</span>}
          <span className="status-digest" title={digest ?? undefined}>
            sha {shortDigest}
          </span>
          <span className="status-sep">structure editing</span>
          <span className="status-version">M6.023</span>
        </div>
      </footer>

      {createDraft && createParent && createSlot && (
        <CreateInstanceDialog
          key={`${createParent.id}:${createSlot.id}`}
          parent={createParent}
          slot={createSlot}
          instances={instances}
          bindings={bindings}
          modules={modules}
          suspended={pendingReview !== null}
          onCancel={() => setCreateDraft(null)}
          onReview={(commands, description, options) =>
            void reviewCommands(commands, description, options)
          }
        />
      )}

      {serviceDraft && (
        <CreateServiceDialog
          instances={instances}
          modules={modules}
          suspended={pendingReview !== null}
          onCancel={() => setServiceDraft(false)}
          onReview={(commands, description, options) =>
            void reviewCommands(commands, description, options)
          }
        />
      )}

      {bindingDraftValue && bindingDraftChild && (
        <BindingEditorDialog
          key={bindingDraftValue.id}
          binding={bindingDraftValue}
          child={bindingDraftChild}
          instances={instances}
          bindings={bindings}
          modules={modules}
          suspended={pendingReview !== null}
          onCancel={() => setBindingDraft(null)}
          onReview={(commands, description, options) =>
            void reviewCommands(commands, description, options)
          }
        />
      )}

      {connectionDraft &&
        (connectionDraft.mode === "create" || draftConnection) && (
          <ConnectionDialog
            key={
              connectionDraft.mode === "edit"
                ? connectionDraft.id
                : "new-connection"
            }
            connection={draftConnection}
            instances={instances}
            modules={modules}
            connections={connections}
            suspended={pendingReview !== null}
            onCancel={() => setConnectionDraft(null)}
            onReview={(commands, description, options) =>
              void reviewCommands(commands, description, options)
            }
          />
        )}

      {providerDraft && (
        <DependencyProviderDialog
          key={`${providerDraft.instanceId}:${providerDraft.requirementId}`}
          instanceId={providerDraft.instanceId}
          requirementId={providerDraft.requirementId}
          instances={instances}
          modules={modules}
          dependencies={dependencies}
          suspended={pendingReview !== null}
          onCancel={() => setProviderDraft(null)}
          onReview={(commands, description, options) =>
            void reviewCommands(commands, description, options)
          }
        />
      )}

      {pendingReview && (
        <ChangeReview
          transaction={pendingReview.transaction}
          preview={pendingReview.preview}
          description={pendingReview.description}
          busy={reviewBusy || reloading}
          error={editError}
          externalDirty={externalDirty}
          reloading={reloading}
          onReload={() => void handleReloadProject()}
          onApply={() => void applyPendingReview()}
          onClose={() => {
            setPendingReview(null);
            setEditError(null);
          }}
        />
      )}

      {loadState === "loading" && (
        <div className="loading-veil" role="status" aria-live="polite">
          <div className="loading-card">
            <span className="spinner" />
            <div>
              <strong>Reading project</strong>
              <span>Composition Core is validating files…</span>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

function PanelHeading({
  eyebrow,
  title,
  meta,
}: {
  eyebrow: string;
  title: string;
  meta?: ReactNode;
}) {
  return (
    <div className="panel-head">
      <div>
        <div className="eyebrow">{eyebrow}</div>
        <h2>{title}</h2>
      </div>
      {meta}
    </div>
  );
}

function Property({
  label,
  value,
  mono = false,
}: {
  label: string;
  value: string;
  mono?: boolean;
}) {
  return (
    <div className="property">
      <span>{label}</span>
      <span className={mono ? "mono" : undefined}>{value}</span>
    </div>
  );
}

function SelectionDetails({
  instance,
  moduleKind,
  scope,
  interfaces,
  slots,
  ports,
  properties,
  isRoot,
  canEdit,
  revisionSeq,
  onReview,
  actions,
}: {
  instance: Instance;
  moduleKind?: string;
  scope?: InstanceScope | null;
  interfaces: InterfaceDeclaration[];
  slots: number;
  ports: number;
  properties: PropertyContract[];
  isRoot: boolean;
  canEdit: boolean;
  revisionSeq: number;
  onReview: (commands: EditCommand[], description: string) => void;
  actions: ReactNode;
}) {
  return (
    <>
      <section className="inspector-section">
        <div className="section-label">Instance</div>
        <Property label="Id" value={instance.id} mono />
        <Property label="Module" value={instance.module.id} mono />
        <Property label="Version" value={instance.module.version} mono />
        <Property label="Kind" value={moduleKind ?? "unknown"} />
        <Property label="Scope" value={scope ?? "structure"} />
        <Property label="Role" value={isRoot ? "root" : "child"} />
        <Property
          label="Source"
          value={instance.source?.file ?? "—"}
          mono
        />
      </section>
      <section className="inspector-section config-section">
        <div className="section-label">
          Config {!canEdit && <span className="read-only-tag">read-only</span>}
        </div>
        {properties.length === 0 ? (
          <div className="config-empty">This module declares no properties.</div>
        ) : (
          <div className="config-list">
            {properties.map((property) => (
              <PropertyEditor
                key={`${instance.id}:${property.id}:${revisionSeq}`}
                instance={instance}
                property={property}
                canEdit={canEdit}
                revisionSeq={revisionSeq}
                onReview={onReview}
              />
            ))}
          </div>
        )}
      </section>
      {actions}
      <section className="inspector-section">
        <div className="section-label">Contract</div>
        <Property label="Properties" value={String(properties.length)} />
        <Property label="Slots" value={String(slots)} />
        <Property label="Ports" value={String(ports)} />
        <Property label="Interfaces" value={String(interfaces.length)} />
        {interfaces.map((declaration) => (
          <Property
            key={`${declaration.id}@${declaration.version}`}
            label="Implements"
            value={`${declaration.id}@${declaration.version}`}
            mono
          />
        ))}
        <Property
          label="Configured"
          value={String(Object.keys(instance.config ?? {}).length)}
        />
        <Property
          label="Providers"
          value={String(Object.keys(instance.dependency_providers ?? {}).length)}
        />
      </section>
      <div className="inspector-hint">
        {canEdit
          ? "Changes are validated by Core and written through the editor transaction."
          : "This project is read-only because no editable session could be opened."}
      </div>
    </>
  );
}

function PropertyEditor({
  instance,
  property,
  canEdit,
  revisionSeq,
  onReview,
}: {
  instance: Instance;
  property: PropertyContract;
  canEdit: boolean;
  revisionSeq: number;
  onReview: (commands: EditCommand[], description: string) => void;
}) {
  const current = instance.config[property.id] ?? undefined;
  const initial = toDraftValue(current, property.type);
  const [draft, setDraft] = useState(initial);
  const [localError, setLocalError] = useState<string | null>(null);

  useEffect(() => {
    setDraft(toDraftValue(instance.config[property.id], property.type));
    setLocalError(null);
  }, [instance.id, property.id, property.type, revisionSeq]);

  const label = property.id;
  const readOnly = !canEdit || property.type === "array";
  const isEnum = property.values.length > 0;
  const required = property.required;

  function reviewSet() {
    const parsed = parseDraft(draft, property);
    if ("error" in parsed) {
      setLocalError(parsed.error);
      return;
    }
    onReview(
      [
        {
          type: "set_config",
          instance: instance.id,
          property: label,
          value: parsed.value,
        },
      ],
      `Set ${instance.id}.${label}`,
    );
  }

  function useDefault() {
    onReview(
      [
        {
          type: "unset_config",
          instance: instance.id,
          property: label,
        },
      ],
      `Reset ${instance.id}.${label} to default`,
    );
  }

  return (
    <div className="config-row">
      <div className="config-label-row">
        <span className="config-label">{label}</span>
        <span className="config-type">{property.type}</span>
        {required && <span className="config-required">required</span>}
      </div>
      <div className="config-input-row">
        {isEnum ? (
          <select
            value={draft}
            disabled={readOnly}
            onChange={(event) => setDraft(event.target.value)}
          >
            {!property.required && <option value="">(unset)</option>}
            {property.values.map((value) => (
              <option key={value} value={value}>
                {value}
              </option>
            ))}
          </select>
        ) : property.type === "boolean" ? (
          <select
            value={draft}
            disabled={readOnly}
            onChange={(event) => setDraft(event.target.value)}
          >
            <option value="true">true</option>
            <option value="false">false</option>
          </select>
        ) : (
          <input
            type={property.type === "integer" || property.type === "float" ? "number" : "text"}
            value={draft}
            disabled={readOnly}
            step={property.type === "float" ? "any" : undefined}
            onChange={(event) => setDraft(event.target.value)}
            placeholder={property.required ? "required" : "default"}
          />
        )}
        <button
          type="button"
          className="config-action"
          disabled={readOnly}
          onClick={reviewSet}
        >
          Review
        </button>
        {!required && current !== undefined && (
          <button
            type="button"
            className="config-action is-muted"
            disabled={readOnly}
            onClick={useDefault}
          >
            Default
          </button>
        )}
      </div>
      {localError && <div className="config-error">{localError}</div>}
    </div>
  );
}

function reconcilePositions(
  model: CompositionModel,
  previous: LayoutNodes,
  alias?: { from: string; to: string },
): LayoutNodes {
  const suggested = autoLayout(
    model.composition.instances,
    model.composition.bindings,
    model.composition.root,
  );
  const next: LayoutNodes = {};
  for (const instance of model.composition.instances) {
    const id = instance.id;
    let existing = previous[id];
    if (!existing && alias && id === alias.to) {
      existing = previous[alias.from];
    }
    next[id] = existing ?? suggested[id] ?? { x: 0, y: 0 };
  }
  return next;
}


function EmptyState({ onChoose, onPath }: { onChoose: () => void; onPath: () => void }) {
  return (
    <div className="empty-state">
      <div className="empty-orbit" aria-hidden="true">
        <span className="orbit-dot orbit-1" />
        <span className="orbit-dot orbit-2" />
        <span className="orbit-dot orbit-3" />
      </div>
      <div className="eyebrow">Tauri 2 · React · Rust Core</div>
      <h1>
        Compose first.
        <br />
        <span>Inspect every boundary.</span>
      </h1>
      <p>
        Open a Composition Designer project to inspect its normalized model, module contracts and
        validation diagnostics. Select an instance to edit it through Core transactions; node
        positions stay in <code>.designer/</code>.
      </p>
      <div className="empty-actions">
        <button type="button" className="button button-primary" onClick={onChoose}>
          Choose project folder
        </button>
        <button type="button" className="button button-ghost" onClick={onPath}>
          Enter a path
        </button>
      </div>
      <div className="empty-hints">
        <span>examples/minimal-project</span>
        <span>examples/nested-composition</span>
        <span>invalid-projects/…</span>
      </div>
    </div>
  );
}

function InvalidState({
  snapshot,
  diagnostics,
}: {
  snapshot: ProjectSnapshot;
  diagnostics: Diagnostic[];
}) {
  return (
    <div className="invalid-state">
      <div className="eyebrow">Cannot resolve model</div>
      <h1>The project was read, but no Composition Model could be built.</h1>
      <p>
        <code>{snapshot.root}</code> returned {snapshot.errorCount} error
        {snapshot.errorCount === 1 ? "" : "s"}. Inspect the diagnostics panel for source locations
        and codes.
      </p>
      <div className="invalid-codes">
        {diagnostics.slice(0, 8).map((diagnostic, index) => (
          <code key={`${diagnostic.code}-${index}`}>{diagnostic.code}</code>
        ))}
      </div>
    </div>
  );
}

function diagnosticLocation(diagnostic: Diagnostic): string {
  if (!diagnostic.file) return "project";
  if (diagnostic.range) {
    return `${diagnostic.file}:${diagnostic.range.start.line}:${diagnostic.range.start.column}`;
  }
  return diagnostic.file;
}

function describeStatus(
  loadState: LoadState,
  snapshot: ProjectSnapshot | null,
  error: string | null,
): { label: string; tone: string } {
  if (loadState === "loading") return { label: "Reading project…", tone: "is-busy" };
  if (error) return { label: "Open failed", tone: "is-invalid" };
  if (!snapshot) return { label: "Waiting for a project", tone: "is-idle" };
  if (!snapshot.valid) return { label: "Project has errors", tone: "is-invalid" };
  return { label: "Project valid", tone: "is-valid" };
}
