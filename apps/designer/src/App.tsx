import { useEffect, useMemo, useState } from "react";
import type { FormEvent, ReactNode } from "react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import {
  initialProject,
  isTauri,
  loadLayout,
  openProject,
  saveLayout,
} from "./api";
import CompositionCanvas from "./canvas/CompositionCanvas";
import StructureTree from "./explorer/StructureTree";
import { autoLayout, buildStructure } from "./layout";
import type {
  Diagnostic,
  Instance,
  LayoutNodes,
  NodePosition,
  ProjectSnapshot,
} from "./types";

const LAST_PATH_KEY = "composition-designer.last-project";

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

  useEffect(() => {
    let cancelled = false;
    async function restore() {
      if (!isTauri()) return;
      try {
        const initial = await initialProject();
        const stored = window.localStorage.getItem(LAST_PATH_KEY);
        const path = initial ?? stored;
        if (path && !cancelled) {
          await loadProject(path);
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

  async function loadProject(rawPath: string) {
    const path = rawPath.trim();
    if (!path) {
      setShowPath(true);
      return;
    }
    setLoadState("loading");
    setError(null);
    try {
      const next = await openProject(path);
      setSnapshot(next);
      setPathInput(path);
      setLoadState("ready");
      window.localStorage.setItem(LAST_PATH_KEY, path);
    } catch (reason) {
      setLoadState("error");
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
        await loadProject(selected);
      }
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    }
  }

  function handlePathSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    void loadProject(pathInput);
  }

  const model = snapshot?.model ?? null;
  const project = model?.project ?? null;
  const composition = model?.composition ?? null;
  const instances = composition?.instances ?? [];
  const bindings = composition?.bindings ?? [];
  const connections = composition?.connections ?? [];
  const dependencies = model?.dependencies ?? [];
  const modules = model?.modules ?? [];
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
    setSelectedId(null);
    setFocusRequest(null);
    setLayoutStatus("auto layout");
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
  }, [snapshot, model]);

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

  const status = describeStatus(loadState, snapshot, error);
  const digest = snapshot?.sourceDigest ?? model?.revision?.source_digest ?? null;
  const shortDigest = digest ? `${digest.slice(0, 7)}…${digest.slice(-6)}` : "—";
  const valid = snapshot?.valid ?? false;

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
            </>
          ) : (
            <span className="muted">No project open</span>
          )}
        </div>

        <div className="topbar-actions">
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

        <section className="panel stage" aria-label="Composition canvas">
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
              <footer className="stage-footer">
                <span className="stage-note">
                  Drag to arrange · positions write to .designer/layout.toml · composition is read-only
                </span>
                <div className="stage-counts">
                  <span>{bindings.length} structural bindings</span>
                  <span>{connections.length} port connections</span>
                  <span>{dependencies.length} dependency resolutions</span>
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
                slots={selectedModule?.contract.slots.length ?? 0}
                ports={selectedModule?.contract.ports.length ?? 0}
                properties={selectedModule?.contract.properties.length ?? 0}
                isRoot={selectedInstance.id === rootId}
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
                  Select a node or tree row to inspect an instance. Editing arrives in 014.
                </div>
              </>
            ) : (
              <div className="inspector-empty">No selection.</div>
            )}
          </div>
        </aside>

        <section className="panel diagnostics" aria-label="Diagnostics">
          <div className="diagnostics-head">
            <div>
              <div className="eyebrow">Validation</div>
              <h2>Diagnostics</h2>
            </div>
            <div className="diagnostic-counts">
              <span className="count count-error">{snapshot?.errorCount ?? 0} errors</span>
              <span className="count count-warning">{snapshot?.warningCount ?? 0} warnings</span>
            </div>
          </div>
          <div className="diagnostics-body">
            {diagnostics.length === 0 ? (
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
                    <span className={`severity-dot severity-${diagnostic.severity}`} />
                    <code className="diagnostic-code">{diagnostic.code}</code>
                    <span className="diagnostic-message">{diagnostic.message}</span>
                    <span className="diagnostic-location">{diagnosticLocation(diagnostic)}</span>
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
        </div>
        <div className="status-right">
          {layoutStatus && <span className="status-layout">{layoutStatus}</span>}
          <span className="status-digest" title={digest ?? undefined}>
            sha {shortDigest}
          </span>
          <span className="status-sep">read-only shell</span>
          <span className="status-version">M3.013</span>
        </div>
      </footer>

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
  slots,
  ports,
  properties,
  isRoot,
}: {
  instance: Instance;
  moduleKind?: string;
  slots: number;
  ports: number;
  properties: number;
  isRoot: boolean;
}) {
  return (
    <>
      <section className="inspector-section">
        <div className="section-label">Instance</div>
        <Property label="Id" value={instance.id} mono />
        <Property label="Module" value={instance.module.id} mono />
        <Property label="Version" value={instance.module.version} mono />
        <Property label="Kind" value={moduleKind ?? "unknown"} />
        <Property label="Role" value={isRoot ? "root" : "child"} />
        <Property
          label="Source"
          value={instance.source?.file ?? "—"}
          mono
        />
      </section>
      <section className="inspector-section">
        <div className="section-label">Contract</div>
        <Property label="Properties" value={String(properties)} />
        <Property label="Slots" value={String(slots)} />
        <Property label="Ports" value={String(ports)} />
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
        Selection is read-only. Property and config editing arrives in 014.
      </div>
    </>
  );
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
        validation diagnostics. This shell is read-only; the desktop host delegates every read to
        the Rust Core.
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
