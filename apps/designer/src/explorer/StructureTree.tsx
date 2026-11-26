import { useState } from "react";
import type { Structure, TreeNode } from "../layout";

interface StructureTreeProps {
  structure: Structure;
  selectedId: string | null;
  onSelect: (id: string) => void;
}

export default function StructureTree({
  structure,
  selectedId,
  onSelect,
}: StructureTreeProps) {
  if (!structure.root && structure.orphans.length === 0) {
    return <div className="tree-empty">No instances in this composition.</div>;
  }
  return (
    <div className="structure-tree">
      {structure.root && (
        <ul className="tree-branches">
          <TreeBranch
            node={structure.root}
            depth={0}
            selectedId={selectedId}
            onSelect={onSelect}
          />
        </ul>
      )}
      {structure.orphans.length > 0 && (
        <>
          <div className="tree-orphan-label">Unbound instances</div>
          <ul className="tree-branches">
            {structure.orphans.map((instance) => (
              <li key={instance.id}>
                <button
                  type="button"
                  className={`tree-node ${selectedId === instance.id ? "is-selected" : ""}`}
                  onClick={() => onSelect(instance.id)}
                  style={{ paddingLeft: 8 }}
                >
                  <span className="tree-chevron is-empty" />
                  <span className="instance-dot" />
                  <span className="tree-label">{instance.id}</span>
                  <span className="tree-module">{instance.module.id}</span>
                </button>
              </li>
            ))}
          </ul>
        </>
      )}
    </div>
  );
}

function TreeBranch({
  node,
  depth,
  selectedId,
  onSelect,
}: {
  node: TreeNode;
  depth: number;
  selectedId: string | null;
  onSelect: (id: string) => void;
}) {
  const [collapsed, setCollapsed] = useState(false);
  const hasChildren = node.children.length > 0;
  return (
    <li>
      <button
        type="button"
        className={`tree-node ${selectedId === node.instance.id ? "is-selected" : ""}`}
        onClick={() => onSelect(node.instance.id)}
        style={{ paddingLeft: 8 + depth * 15 }}
        aria-expanded={hasChildren ? !collapsed : undefined}
      >
        <span
          className={`tree-chevron ${hasChildren ? "" : "is-empty"} ${collapsed ? "is-collapsed" : ""}`}
          onClick={(event) => {
            if (!hasChildren) return;
            event.stopPropagation();
            setCollapsed((value) => !value);
          }}
          aria-hidden="true"
        >
          {hasChildren ? "▾" : "·"}
        </span>
        <span className="tree-branch-dot" />
        <span className="tree-label">{node.instance.id}</span>
        {depth === 0 && <span className="badge badge-root">root</span>}
        {node.slot && <span className="tree-slot">{node.slot}</span>}
        <span className="tree-module">{node.instance.module.id}</span>
      </button>
      {hasChildren && !collapsed && (
        <ul className="tree-branches">
          {node.children.map((child) => (
            <TreeBranch
              key={child.instance.id}
              node={child}
              depth={depth + 1}
              selectedId={selectedId}
              onSelect={onSelect}
            />
          ))}
        </ul>
      )}
    </li>
  );
}
