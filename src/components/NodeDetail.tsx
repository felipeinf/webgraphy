import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { DomainDetail, GraphNode, PageDetail, Tag } from "../types/graph";
import { BrowserBadges } from "./BrowserBadges";
import { loadFaviconDataUrl } from "../hooks/useFavicons";

function pageLabel(page: PageDetail): string {
  if (page.title && page.title !== "New Tab") return page.title;
  return page.original_url || page.normalized_url;
}

interface NodeDetailProps {
  node: GraphNode | null;
  assignedTags: Tag[];
  assigning: boolean;
  onClose: () => void;
  onRemoved: () => void;
  onRefresh: () => void;
  onStartAssign: () => void;
  onUnassignTag: (tagId: number) => void;
}

export function NodeDetail({
  node,
  assignedTags,
  assigning,
  onClose,
  onRemoved,
  onRefresh,
  onStartAssign,
  onUnassignTag,
}: NodeDetailProps) {
  const [favicon, setFavicon] = useState<string | null>(null);
  const [domainDetail, setDomainDetail] = useState<DomainDetail | null>(null);
  const hostname = node?.hostname ?? null;
  const domainId = node?.domain_id;

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

  useEffect(() => {
    let cancelled = false;
    setDomainDetail(null);

    if (domainId === undefined) return;

    invoke<DomainDetail>("get_domain_detail_cmd", { domainId })
      .then((detail) => {
        if (!cancelled) setDomainDetail(detail);
        if (node?.node_type !== "domain" || detail.meta_title != null) return;
        return invoke<DomainDetail>("fetch_domain_meta_cmd", { domainId }).then(
          (updated) => {
            if (!cancelled) setDomainDetail(updated);
          },
        );
      })
      .catch(() => {
        if (!cancelled) setDomainDetail(null);
      });

    return () => {
      cancelled = true;
    };
  }, [node, domainId]);

  if (!node) return null;

  const handleOpen = async (url?: string) => {
    const target = url ?? node.url;
    if (target) {
      await invoke("open_url", { url: target });
    }
  };

  const reloadDetail = async () => {
    if (domainId === undefined) return;
    const detail = await invoke<DomainDetail>("get_domain_detail_cmd", {
      domainId,
    });
    setDomainDetail(detail);
  };

  const handleRemovePage = async (pageId: number, closePanel: boolean) => {
    await invoke("archive_page_cmd", { pageId });
    if (closePanel) {
      onRemoved();
      return;
    }
    onRefresh();
    await reloadDetail();
  };

  const handleRemoveDomain = async () => {
    if (domainId === undefined) return;
    const count = domainDetail?.page_count ?? node.page_count ?? 0;
    if (count > 1) {
      const host = domainDetail?.hostname ?? node.hostname ?? "this domain";
      const ok = window.confirm(
        `Remove ${host} and ${count} pages from the graph?`,
      );
      if (!ok) return;
    }
    await invoke("archive_domain_cmd", { domainId });
    onRemoved();
  };

  const domainTitle =
    domainDetail?.meta_title?.trim() ||
    node.hostname ||
    node.label;
  const domainHost = domainDetail?.hostname ?? node.hostname;

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

      {node.node_type === "domain" ? (
        <>
          <h2>{domainTitle}</h2>
          {domainHost && domainTitle !== domainHost && (
            <p className="detail-hostname">{domainHost}</p>
          )}
          {domainDetail?.meta_description?.trim() && (
            <p className="detail-description">
              {domainDetail.meta_description.trim()}
            </p>
          )}
          <p className="detail-meta">
            {domainDetail?.page_count ?? node.page_count ?? 0} pages
          </p>
        </>
      ) : (
        <>
          <h2>{node.label}</h2>
          {node.title && node.title !== "New Tab" && (
            <p className="detail-title">{node.title}</p>
          )}
          {node.url && <p className="detail-url">{node.url}</p>}
          <BrowserBadges browsers={node.browsers} />
        </>
      )}

      {domainId !== undefined && (
        <div className="detail-tags">
          {assignedTags.length > 0 && (
            <div className="detail-tag-list">
              {assignedTags.map((tag) => (
                <span key={tag.id} className="tag-chip assigned">
                  {tag.name}
                  <span
                    className="tag-chip-remove"
                    onClick={() => onUnassignTag(tag.id)}
                  >
                    ×
                  </span>
                </span>
              ))}
            </div>
          )}
          <button
            type="button"
            className={`action-btn${assigning ? " primary" : ""}`}
            onClick={onStartAssign}
          >
            {assigning ? "Click a tag above…" : "Add tag"}
          </button>
        </div>
      )}

      {node.node_type === "domain" &&
        domainDetail &&
        domainDetail.subdomains.length > 0 && (
          <div className="detail-subdomains">
            <span className="detail-section-label">Subdomains</span>
            <ul>
              {domainDetail.subdomains.map((sub) => (
                <li key={sub}>{sub}</li>
              ))}
            </ul>
          </div>
        )}

      {node.node_type === "domain" &&
        domainDetail &&
        domainDetail.pages.length > 0 && (
          <div className="detail-pages">
            <span className="detail-section-label">Pages</span>
            <ul className="detail-pages-list">
              {domainDetail.pages.map((page) => (
                <li key={page.id}>
                  <button
                    type="button"
                    className="detail-page-item"
                    onClick={() => handleOpen(page.original_url)}
                  >
                    <span className="detail-page-title">{pageLabel(page)}</span>
                    <span className="detail-page-url">{page.normalized_url}</span>
                  </button>
                  <button
                    type="button"
                    className="detail-page-delete"
                    onClick={() => handleRemovePage(page.id, false)}
                  >
                    ×
                  </button>
                </li>
              ))}
            </ul>
          </div>
        )}

      <div className="detail-actions">
        {node.node_type === "page" && (
          <button
            className="action-btn primary"
            onClick={() => handleOpen()}
            type="button"
          >
            Open in browser
          </button>
        )}
        {node.node_type === "page" && (
          <button
            className="action-btn danger"
            onClick={() => {
              const pageId = parseInt(node.id.replace("page-", ""), 10);
              void handleRemovePage(pageId, true);
            }}
            type="button"
          >
            Remove page
          </button>
        )}
        {node.node_type === "domain" && (
          <button
            className="action-btn danger"
            onClick={() => void handleRemoveDomain()}
            type="button"
          >
            Remove domain
          </button>
        )}
      </div>
    </aside>
  );
}
