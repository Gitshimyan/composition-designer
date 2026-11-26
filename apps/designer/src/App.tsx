import { useEffect, useMemo, useState } from "react";
import type { FormEvent, ReactNode } from "react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { initialProject, isTauri, openProject } from "./api";
import type { Diagnostic, Instance, ProjectSnapshot } from "./types";

const LAST_PATH_KEY = "composition-designer.last-project";

type LoadState = "idle" | "loading" | "ready" | "error";

export default function App() {
  const [snapshot, setSnapshot] = useState<ProjectSnapshot | null>(null);
  const [loadState, setLoadState] = useState<LoadState>("idle");
  const [error, setError] = useState<string | null>(null);
  const [showPath, setShowPath] = useState(false);
  const [pathInput, setPathInput] = useState("");

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

  const orderedInstances = useMemo(() => {
    return [...instances].sort((a, b) => {
      if (a.id === rootId) return -1;
      if (b.id === rootId) return 1;
      return a.id.localeCompare(b.id);
    });
  }, [instances, rootId]);

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
                    Instances <span>{instances.length}</span>
                  </div>
                  <ul className="tree-list">
                    {orderedInstances.map((instance) => (
                      <li key={instance.id}>
                        <div className="tree-row">
                          <span className="instance-dot" />
                          <span className="tree-name">{instance.id}</span>
                          {instance.id === rootId && <span className="badge">root</span>}
                        </div>
                      </li>
                    ))}
                  </ul>
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

        <section className="panel stage" aria-label="Composition preview">
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
              <div className="canvas-surface">
                <div className="canvas-grid" aria-hidden="true" />
                <div className="composition-grid">
                  {orderedInstances.map((instance) => (
                    <InstanceCard
                      key={instance.id}
                      instance={instance}
                      isRoot={instance.id === rootId}
                      moduleKind={
                        modules.find(
                          (module) =>
                            module.id === instance.module.id &&
                            module.version === instance.module.version,
                        )?.kind
                      }
                    />
                  ))}
                </div>
              </div>
              <footer className="stage-footer">
                <span className="stage-note">Read-only preview · canvas interactions land in 013</span>
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
            {project ? (
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
                  Selection, property editing and module contracts arrive in 014.
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
          <span className="status-digest" title={digest ?? undefined}>
            sha {shortDigest}
          </span>
          <span className="status-sep">read-only shell</span>
          <span className="status-version">M3.012</span>
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

function InstanceCard({
  instance,
  isRoot,
  moduleKind,
}: {
  instance: Instance;
  isRoot: boolean;
  moduleKind?: string;
}) {
  const configCount = Object.keys(instance.config ?? {}).length;
  const providerCount = Object.keys(instance.dependency_providers ?? {}).length;
  return (
    <article className={`instance-card ${isRoot ? "is-root" : ""}`}>
      <div className="instance-top">
        <code className="instance-id">{instance.id}</code>
        <div className="instance-badges">
          {isRoot && <span className="badge badge-root">root</span>}
          {moduleKind && <span className={`badge badge-${moduleKind}`}>{moduleKind}</span>}
        </div>
      </div>
      <div className="instance-module">
        {instance.module.id}
        <span>@{instance.module.version}</span>
      </div>
      <div className="instance-meta">
        <span>{configCount} config</span>
        <span>{providerCount} providers</span>
      </div>
    </article>
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
