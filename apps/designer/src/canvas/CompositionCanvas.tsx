import { useEffect, useMemo, useState } from "react";
import {
  Background,
  BackgroundVariant,
  Controls,
  Handle,
  MarkerType,
  Position,
  ReactFlow,
  type Edge,
  type Node,
  type NodeChange,
  type NodeProps,
  type ReactFlowInstance,
} from "@xyflow/react";
import type { Binding, Instance, LayoutNodes, NodePosition } from "../types";

type InstanceNodeData = {
  instance: Instance;
  isRoot: boolean;
  moduleKind?: string;
  configCount: number;
  providerCount: number;
};

type InstanceNode = Node<InstanceNodeData, "instance">;

const nodeTypes = { instance: InstanceNodeView };

interface CompositionCanvasProps {
  rootId: string;
  instances: Instance[];
  bindings: Binding[];
  positions: LayoutNodes;
  selectedId: string | null;
  focusRequest: { id: string; nonce: number } | null;
  fitNonce: number;
  moduleKinds: Map<string, string>;
  onSelect: (id: string | null) => void;
  onMoveNode: (id: string, position: NodePosition) => void;
  onCommitNode: (id: string, position: NodePosition) => void;
  onResetLayout: () => void;
}

export default function CompositionCanvas({
  rootId,
  instances,
  bindings,
  positions,
  selectedId,
  focusRequest,
  fitNonce,
  moduleKinds,
  onSelect,
  onMoveNode,
  onCommitNode,
  onResetLayout,
}: CompositionCanvasProps) {
  const [flow, setFlow] = useState<ReactFlowInstance<InstanceNode, Edge> | null>(
    null,
  );

  const nodes = useMemo<InstanceNode[]>(() => {
    return instances.map((instance) => {
      const position = positions[instance.id] ?? { x: 0, y: 0 };
      return {
        id: instance.id,
        type: "instance",
        position,
        selected: selectedId === instance.id,
        data: {
          instance,
          isRoot: instance.id === rootId,
          moduleKind: moduleKinds.get(instance.module.id),
          configCount: Object.keys(instance.config ?? {}).length,
          providerCount: Object.keys(instance.dependency_providers ?? {}).length,
        },
      };
    });
  }, [instances, moduleKinds, positions, selectedId, rootId]);

  const edges = useMemo<Edge[]>(() => {
    return bindings.map((binding) => ({
      id: `binding:${binding.id}`,
      source: binding.parent,
      target: binding.child,
      label: binding.slot,
      type: "smoothstep",
      animated: false,
      style: { stroke: "#345363", strokeWidth: 1.5 },
      labelStyle: {
        fill: "#91a2b7",
        fontSize: 9,
        fontFamily: "Cascadia Code, Consolas, monospace",
      },
      labelBgStyle: {
        fill: "#0e151e",
        stroke: "#1d2a38",
        strokeWidth: 1,
      },
      labelBgPadding: [5, 3],
      labelBgBorderRadius: 4,
      markerEnd: { type: MarkerType.ArrowClosed, color: "#345363" },
    }));
  }, [bindings]);

  useEffect(() => {
    if (!focusRequest || !flow) return;
    const focused = nodes.find((node) => node.id === focusRequest.id);
    if (!focused) return;
    flow.fitView({
      nodes: [focused],
      duration: 450,
      padding: 0.9,
      maxZoom: 1.2,
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [focusRequest, flow]);

  useEffect(() => {
    if (!flow) return;
    const timer = window.setTimeout(() => {
      flow.fitView({ padding: 0.24, duration: 300, maxZoom: 1.1 });
    }, 150);
    return () => window.clearTimeout(timer);
  }, [fitNonce, flow]);

  function handleNodesChange(changes: NodeChange<InstanceNode>[]) {
    for (const change of changes) {
      if (change.type === "position" && change.position) {
        onMoveNode(change.id, change.position);
      }
    }
  }

  if (instances.length === 0) {
    return (
      <div className="flow-empty">
        <span className="empty-glyph">◇</span>
        <p>No instances to render.</p>
      </div>
    );
  }

  return (
    <div className="flow-wrap">
      <div className="canvas-toolbar">
        <span className="canvas-toolbar-label">Structural canvas</span>
        <div className="canvas-toolbar-actions">
          <button
            type="button"
            className="canvas-tool"
            onClick={() =>
              flow?.fitView({ padding: 0.22, duration: 300, maxZoom: 1.1 })
            }
          >
            Fit view
          </button>
          <button type="button" className="canvas-tool" onClick={onResetLayout}>
            Reset layout
          </button>
        </div>
      </div>
      <ReactFlow<InstanceNode, Edge>
        nodes={nodes}
        edges={edges}
        nodeTypes={nodeTypes}
        onInit={setFlow}
        onNodesChange={handleNodesChange}
        onNodeClick={(_event, node) => onSelect(node.id)}
        onPaneClick={() => onSelect(null)}
        onNodeDragStop={(_event, node) => {
          onCommitNode(node.id, node.position);
        }}
        fitView
        fitViewOptions={{ padding: 0.24, maxZoom: 1.1 }}
        minZoom={0.3}
        maxZoom={1.6}
        nodesConnectable={false}
        elementsSelectable
      >
        <Background
          variant={BackgroundVariant.Dots}
          gap={22}
          size={1}
          color="rgba(145, 162, 183, 0.18)"
        />
        <Controls showInteractive={false} position="bottom-right" />
      </ReactFlow>
    </div>
  );
}

function InstanceNodeView({ data, selected }: NodeProps<InstanceNode>) {
  return (
    <div
      className={`flow-node ${data.isRoot ? "is-root" : ""} ${selected ? "is-selected" : ""}`}
    >
      <Handle
        type="target"
        position={Position.Top}
        isConnectable={false}
        className="flow-handle"
      />
      <div className="flow-node-top">
        <code>{data.instance.id}</code>
        <div className="flow-node-badges">
          {data.isRoot && <span className="badge badge-root">root</span>}
          {data.moduleKind && (
            <span className={`badge badge-${data.moduleKind}`}>
              {data.moduleKind}
            </span>
          )}
        </div>
      </div>
      <div className="flow-node-module">
        {data.instance.module.id}
        <span>@{data.instance.module.version}</span>
      </div>
      <div className="flow-node-meta">
        <span>{data.configCount} config</span>
        <span>{data.providerCount} providers</span>
      </div>
      <Handle
        type="source"
        position={Position.Bottom}
        isConnectable={false}
        className="flow-handle"
      />
    </div>
  );
}
