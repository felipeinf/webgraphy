import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import ForceGraph2D, {
  type ForceGraphMethods,
  type LinkObject,
  type NodeObject,
} from "react-force-graph-2d";
import { invoke } from "@tauri-apps/api/core";
import type { GraphData, GraphNode } from "../types/graph";
import { useFavicons } from "../hooks/useFavicons";

interface ForceFn<N> {
  (alpha: number): void;
  initialize?: (nodes: N[], ...args: unknown[]) => void;
}

interface GraphCanvasProps {
  graph: GraphData;
  onNodeSelect: (node: GraphNode | null) => void;
  selectedId: string | null;
  expandedDomainId: number | null;
  onDomainClick: (domainId: number) => void;
  onCollapseAll: () => void;
}

type ForceNode = GraphNode & NodeObject;
type ForceLink = LinkObject<ForceNode>;

const DOMAIN_COLOR = "#7c6af7";
const PAGE_COLOR = "#4ecdc4";
const SELECTED_COLOR = "#ff6b6b";
const EXPANDED_DOMAIN_COLOR = "#9b8cff";
const CLUSTER_STRENGTH = 0.22;

function domainRadius(pageCount: number | undefined): number {
  return Math.min(26, 9 + (pageCount ?? 1) * 1.2);
}

function pageLabelMaxChars(globalScale: number): number {
  if (globalScale < 1.05) return 0;
  if (globalScale < 1.6) return 10;
  if (globalScale < 2.2) return 16;
  if (globalScale < 3.5) return 24;
  return 32;
}

function truncateLabel(text: string, max: number): string {
  if (max <= 0) return "";
  if (text.length <= max) return text;
  return `${text.slice(0, max - 1)}…`;
}

function drawLabel(
  ctx: CanvasRenderingContext2D,
  text: string,
  x: number,
  y: number,
  globalScale: number,
  fontSizeScreen: number,
  withBackground = false,
) {
  const fontSize = fontSizeScreen / globalScale;
  ctx.font = `${fontSize}px Inter, system-ui, sans-serif`;
  ctx.textAlign = "center";
  ctx.textBaseline = "top";

  if (withBackground) {
    const metrics = ctx.measureText(text);
    const padX = 4 / globalScale;
    const padY = 2 / globalScale;
    const w = metrics.width + padX * 2;
    const h = fontSize + padY * 2;
    const left = x - w / 2;
    const top = y;
    ctx.fillStyle = "rgba(13, 13, 18, 0.82)";
    const radius = 3 / globalScale;
    ctx.beginPath();
    ctx.roundRect(left, top, w, h, radius);
    ctx.fill();
  }

  ctx.fillStyle = "rgba(230, 230, 240, 0.94)";
  ctx.fillText(text, x, y + (withBackground ? 2 / globalScale : 0));
}

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function createClusterForce(): ForceFn<ForceNode> {
  let nodes: ForceNode[] = [];
  let domainsById = new Map<number, ForceNode>();

  const force: ForceFn<ForceNode> = (alpha: number) => {
    const k = alpha * CLUSTER_STRENGTH;
    for (const node of nodes) {
      if (node.node_type !== "page" || node.domain_id === undefined) continue;
      const anchor = domainsById.get(node.domain_id);
      if (!anchor || anchor.x === undefined || anchor.y === undefined) continue;
      node.vx = (node.vx ?? 0) + (anchor.x - (node.x ?? 0)) * k;
      node.vy = (node.vy ?? 0) + (anchor.y - (node.y ?? 0)) * k;
    }
  };

  force.initialize = (initialNodes: ForceNode[]) => {
    nodes = initialNodes;
    domainsById = new Map();
    for (const node of nodes) {
      if (node.node_type === "domain" && node.domain_id !== undefined) {
        domainsById.set(node.domain_id, node);
      }
    }
  };

  return force;
}

export function GraphCanvas({
  graph,
  onNodeSelect,
  selectedId,
  expandedDomainId,
  onDomainClick,
  onCollapseAll,
}: GraphCanvasProps) {
  const fgRef = useRef<ForceGraphMethods<ForceNode, ForceLink> | undefined>(
    undefined,
  );
  const containerRef = useRef<HTMLDivElement>(null);
  const positionsRef = useRef<Map<string, { x: number; y: number }>>(new Map());
  const [dimensions, setDimensions] = useState({ width: 0, height: 0 });
  const [hoveredId, setHoveredId] = useState<string | null>(null);

  const isCollapsed = expandedDomainId === null;
  const domainCount = graph.nodes.filter((n) => n.node_type === "domain").length;

  const hostnames = useMemo(
    () =>
      Array.from(
        new Set(
          graph.nodes
            .filter((n) => n.node_type === "domain" && n.hostname)
            .map((n) => n.hostname as string),
        ),
      ),
    [graph.nodes],
  );
  const favicons = useFavicons(hostnames);

  const forceData = useMemo(() => {
    const nodes = graph.nodes.map((n) => {
      const node = { ...n } as ForceNode;
      const saved = positionsRef.current.get(node.id);
      if (saved) {
        node.x = saved.x;
        node.y = saved.y;
      } else if (
        node.node_type === "page" &&
        node.domain_id !== undefined &&
        expandedDomainId === node.domain_id
      ) {
        const parentPos = positionsRef.current.get(`domain-${node.domain_id}`);
        if (parentPos) {
          const angle = Math.random() * Math.PI * 2;
          const dist = 55 + Math.random() * 45;
          node.x = parentPos.x + Math.cos(angle) * dist;
          node.y = parentPos.y + Math.sin(angle) * dist;
        }
      } else if (node.node_type === "domain" && !saved) {
        const angle = Math.random() * Math.PI * 2;
        const dist = 30 + Math.random() * 80;
        node.x = Math.cos(angle) * dist;
        node.y = Math.sin(angle) * dist;
      }
      return node;
    });

    const links = graph.links.map((l) => ({
      source: l.source,
      target: l.target,
    })) as ForceLink[];

    return { nodes, links };
  }, [graph, expandedDomainId]);

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;

    const observer = new ResizeObserver((entries) => {
      const entry = entries[0];
      if (entry) {
        setDimensions({
          width: entry.contentRect.width,
          height: entry.contentRect.height,
        });
      }
    });
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    const fg = fgRef.current;
    if (!fg) return;

    const charge = fg.d3Force("charge");
    if (charge) {
      if (isCollapsed) {
        charge.strength(-90).distanceMax(140);
      } else {
        charge.strength((node: ForceNode) =>
          node.node_type === "domain" ? -320 : -50,
        );
        charge.distanceMax(280);
      }
    }

    const center = fg.d3Force("center");
    if (center) {
      center.strength(isCollapsed ? 0.35 : 0.08);
    }

    const link = fg.d3Force("link");
    if (link) {
      link.distance((linkObj: ForceLink) => {
        const target = linkObj.target as ForceNode;
        return target?.node_type === "page" ? 72 : 24;
      });
      link.strength(0.6);
    }

    fg.d3Force("cluster", createClusterForce());
    fg.d3ReheatSimulation();
  }, [forceData, isCollapsed]);

  const savePositions = useCallback(() => {
    for (const node of forceData.nodes) {
      if (node.x !== undefined && node.y !== undefined) {
        positionsRef.current.set(node.id, { x: node.x, y: node.y });
      }
    }
  }, [forceData.nodes]);

  const handleNodeClick = useCallback(
    (node: ForceNode) => {
      savePositions();

      if (node.node_type === "domain" && node.domain_id !== undefined) {
        onDomainClick(node.domain_id);
        onNodeSelect(node);
        return;
      }

      if (node.node_type === "page") {
        onNodeSelect(node);
        if (node.url) {
          void invoke("open_url", { url: node.url });
        }
      }
    },
    [onDomainClick, onNodeSelect, savePositions],
  );

  const handleNodeRightClick = useCallback(
    (node: ForceNode, event: MouseEvent) => {
      event.preventDefault();
      onNodeSelect(node);
    },
    [onNodeSelect],
  );

  const handleNodeHover = useCallback((node: ForceNode | null) => {
    setHoveredId(node?.id ?? null);
    const el = containerRef.current;
    if (el) {
      el.style.cursor = node ? "pointer" : "default";
    }
  }, []);

  const nodeTooltip = useCallback((node: ForceNode) => {
    const lines: string[] = [];
    if (node.node_type === "domain") {
      lines.push(`<strong>${escapeHtml(node.hostname ?? node.label)}</strong>`);
      lines.push(`${node.page_count ?? 0} pages · click to expand`);
    } else {
      const title =
        node.title && node.title !== "New Tab" ? node.title : node.label;
      lines.push(`<strong>${escapeHtml(title)}</strong>`);
      if (node.url) lines.push(`<span class="tip-url">${escapeHtml(node.url)}</span>`);
      if (node.browsers.length > 0) {
        lines.push(`Browsers: ${escapeHtml(node.browsers.join(", "))}`);
      }
      lines.push("click to open");
    }
    return `<div class="graph-tooltip">${lines.join("<br/>")}</div>`;
  }, []);

  const paintNode = useCallback(
    (node: ForceNode, ctx: CanvasRenderingContext2D, globalScale: number) => {
      const isDomain = node.node_type === "domain";
      const radius = isDomain ? domainRadius(node.page_count) : 5;
      const isSelected = node.id === selectedId;
      const isHovered = node.id === hoveredId;
      const isExpanded =
        isDomain &&
        node.domain_id !== undefined &&
        node.domain_id === expandedDomainId;
      const x = node.x ?? 0;
      const y = node.y ?? 0;

      ctx.beginPath();
      ctx.arc(x, y, radius, 0, 2 * Math.PI);
      ctx.fillStyle = isSelected
        ? SELECTED_COLOR
        : isExpanded
          ? EXPANDED_DOMAIN_COLOR
          : isDomain
            ? DOMAIN_COLOR
            : PAGE_COLOR;
      ctx.fill();

      if (isDomain || isHovered || isSelected) {
        ctx.strokeStyle = isExpanded || isHovered || isSelected
          ? "rgba(255,255,255,0.95)"
          : "rgba(255,255,255,0.25)";
        ctx.lineWidth = (isExpanded || isSelected ? 2 : 1.5) / globalScale;
        ctx.stroke();
      }

      if (isDomain && node.hostname) {
        const img = favicons.get(node.hostname);
        if (img) {
          const size = Math.max(10, radius * 1.1);
          ctx.save();
          ctx.beginPath();
          ctx.arc(x, y, size / 2, 0, 2 * Math.PI);
          ctx.clip();
          ctx.drawImage(img, x - size / 2, y - size / 2, size, size);
          ctx.restore();
        }
      }

      const pageMaxChars = pageLabelMaxChars(globalScale);
      const showPageLabel =
        !isDomain && (isHovered || isSelected || pageMaxChars > 0);

      if (isDomain) {
        const label = truncateLabel(
          `${node.hostname ?? node.label} (${node.page_count ?? 0})`,
          22,
        );
        drawLabel(ctx, label, x, y + radius + 3, globalScale, 11);
      } else if (showPageLabel) {
        const raw =
          node.title && node.title !== "New Tab" ? node.title : node.label;
        const maxChars = isHovered || isSelected ? 36 : pageMaxChars;
        const label = truncateLabel(raw, maxChars);
        if (label) {
          drawLabel(
            ctx,
            label,
            x,
            y + radius + 3,
            globalScale,
            9,
            pageMaxChars > 0,
          );
        }
      }
    },
    [selectedId, hoveredId, expandedDomainId, favicons],
  );

  const paintPointerArea = useCallback(
    (node: ForceNode, color: string, ctx: CanvasRenderingContext2D) => {
      const radius = node.node_type === "domain" ? domainRadius(node.page_count) : 9;
      ctx.beginPath();
      ctx.arc(node.x ?? 0, node.y ?? 0, radius, 0, 2 * Math.PI);
      ctx.fillStyle = color;
      ctx.fill();
    },
    [],
  );

  const initialZoom = useMemo(() => {
    if (domainCount <= 3) return 1.4;
    if (domainCount <= 8) return 1.1;
    if (domainCount <= 20) return 0.85;
    return 0.65;
  }, [domainCount]);

  useEffect(() => {
    const fg = fgRef.current;
    if (!fg || forceData.nodes.length === 0 || !isCollapsed) return;
    fg.zoom(initialZoom, 0);
  }, [forceData.nodes.length, initialZoom, isCollapsed]);

  return (
    <div ref={containerRef} className="graph-container">
      {dimensions.width > 0 && (
        <ForceGraph2D
          ref={fgRef}
          width={dimensions.width}
          height={dimensions.height}
          graphData={forceData}
          nodeId="id"
          nodeLabel={nodeTooltip}
          nodeCanvasObject={paintNode}
          nodePointerAreaPaint={paintPointerArea}
          onNodeClick={handleNodeClick}
          onNodeRightClick={handleNodeRightClick}
          onNodeHover={handleNodeHover}
          onNodeDragEnd={savePositions}
          onBackgroundClick={() => {
            savePositions();
            onCollapseAll();
          }}
          linkColor={() => "rgba(124,106,247,0.2)"}
          linkWidth={0.8}
          backgroundColor="#0d0d12"
          cooldownTicks={120}
          d3AlphaDecay={0.032}
          d3VelocityDecay={0.4}
          enableNodeDrag
          minZoom={0.25}
          maxZoom={6}
        />
      )}
      {forceData.nodes.length === 0 && (
        <div className="graph-empty">
          <p>No tabs imported yet.</p>
          <p className="hint">Click Sync now to import open browser tabs.</p>
        </div>
      )}
    </div>
  );
}
