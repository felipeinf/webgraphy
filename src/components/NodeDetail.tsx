import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { GraphNode } from "../types/graph";
import { BrowserBadges } from "./BrowserBadges";
import { loadFaviconDataUrl } from "../hooks/useFavicons";

interface NodeDetailProps {
  node: GraphNode | null;
  onClose: () => void;
  onArchive: () => void;
}

export function NodeDetail({ node, onClose, onArchive }: NodeDetailProps) {
  const [favicon, setFavicon] = useState<string | null>(null);
  const hostname = node?.hostname ?? null;

  useEffect(() => {
    let cancelled = false;
    setFavicon(null);
    if (!hostname) return;
    loadFaviconDataUrl(hostname).then((dataUrl) => {
      if (!cancelled) setFavicon(dataUrl);
    });
    return () => {
      cancelled = true;
    };
  }, [hostname]);

  if (!node) return null;

  const handleOpen = async () => {
    if (node.url) {
      await invoke("open_url", { url: node.url });
    }
  };

  const handleArchive = async () => {
    if (node.node_type !== "page") return;
    const pageId = parseInt(node.id.replace("page-", ""), 10);
    await invoke("archive_page_cmd", { pageId });
    onArchive();
  };

  return (
    <aside className="node-detail">
      <button className="close-btn" onClick={onClose} type="button">
        ×
      </button>

      {favicon && (
        <img
          src={favicon}
          alt=""
          className="detail-favicon"
          width={32}
          height={32}
        />
      )}

      <h2>{node.label}</h2>

      {node.node_type === "domain" && (
        <p className="detail-meta">{node.page_count ?? 0} pages</p>
      )}

      {node.node_type === "page" && (
        <>
          {node.title && node.title !== "New Tab" && (
            <p className="detail-title">{node.title}</p>
          )}
          {node.url && <p className="detail-url">{node.url}</p>}
          <BrowserBadges browsers={node.browsers} />
          <div className="detail-actions">
            <button className="action-btn primary" onClick={handleOpen} type="button">
              Open in browser
            </button>
            <button className="action-btn danger" onClick={handleArchive} type="button">
              Remove from graph
            </button>
          </div>
        </>
      )}
    </aside>
  );
}
