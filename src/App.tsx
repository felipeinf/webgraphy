import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { GraphCanvas } from "./components/GraphCanvas";
import { NodeDetail } from "./components/NodeDetail";
import { TagBar } from "./components/TagBar";
import { Toolbar } from "./components/Toolbar";
import { useGraphData } from "./hooks/useGraphData";
import { useSync } from "./hooks/useSync";
import { useTags } from "./hooks/useTags";
import type { DomainDetail, GraphNode, Tag } from "./types/graph";
import "./App.css";

function App() {
  const [search, setSearch] = useState("");
  const [selectedNode, setSelectedNode] = useState<GraphNode | null>(null);
  const [expandedDomainId, setExpandedDomainId] = useState<number | null>(null);
  const [activeTagIds, setActiveTagIds] = useState<number[]>([]);
  const [assigning, setAssigning] = useState(false);
  const [assignedTags, setAssignedTags] = useState<Tag[]>([]);

  const expandedDomains =
    expandedDomainId !== null ? [expandedDomainId] : [];
  const selectedDomainId = selectedNode?.domain_id;

  const { tags, refresh: refreshTags, createTag, removeTag } = useTags();

  const { graph, loading, error, refresh } = useGraphData(
    search,
    expandedDomains,
    activeTagIds,
  );

  const loadAssignedTags = useCallback(async (domainId: number) => {
    try {
      const detail = await invoke<DomainDetail>("get_domain_detail_cmd", {
        domainId,
      });
      setAssignedTags(detail.tags ?? []);
    } catch {
      setAssignedTags([]);
    }
  }, []);

  useEffect(() => {
    if (selectedDomainId === undefined) {
      setAssignedTags([]);
      setAssigning(false);
      return;
    }
    void loadAssignedTags(selectedDomainId);
  }, [selectedDomainId, loadAssignedTags]);

  useEffect(() => {
    if (!assigning) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") setAssigning(false);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [assigning]);

  const handleSynced = useCallback(() => {
    refresh(true);
  }, [refresh]);

  const { syncing, status, lastSummary, error: syncError, syncNow } =
    useSync(handleSynced);

  const handleSync = useCallback(async () => {
    await syncNow();
  }, [syncNow]);

  const handleRemoved = useCallback(() => {
    setSelectedNode(null);
    setAssigning(false);
    refresh();
  }, [refresh]);

  const handleDomainClick = useCallback((domainId: number) => {
    setExpandedDomainId((current) => (current === domainId ? null : domainId));
  }, []);

  const handleFocusedDomainChange = useCallback((domainId: number | null) => {
    setExpandedDomainId(domainId);
  }, []);

  const handleSelectNode = useCallback((node: GraphNode | null) => {
    setSelectedNode(node);
    setAssigning(false);
  }, []);

  const handleToggleFilter = useCallback((tagId: number) => {
    setActiveTagIds((current) =>
      current.includes(tagId)
        ? current.filter((id) => id !== tagId)
        : [...current, tagId],
    );
  }, []);

  const handleSetTag = useCallback(
    async (tagId: number, assigned: boolean) => {
      if (selectedDomainId === undefined) return;
      await invoke("set_domain_tag_cmd", {
        domainId: selectedDomainId,
        tagId,
        assigned,
      });
      await loadAssignedTags(selectedDomainId);
      void refreshTags();
      refresh(true);
    },
    [selectedDomainId, loadAssignedTags, refreshTags, refresh],
  );

  const handleBarToggle = useCallback(
    (tagId: number) => {
      if (assigning) {
        void handleSetTag(tagId, true);
        return;
      }
      handleToggleFilter(tagId);
    },
    [assigning, handleSetTag, handleToggleFilter],
  );

  const handleCreateTag = useCallback(
    async (name: string) => {
      const tag = await createTag(name);
      if (assigning && selectedDomainId !== undefined) {
        await invoke("set_domain_tag_cmd", {
          domainId: selectedDomainId,
          tagId: tag.id,
          assigned: true,
        });
        await loadAssignedTags(selectedDomainId);
        refresh(true);
      }
    },
    [createTag, assigning, selectedDomainId, loadAssignedTags, refresh],
  );

  const handleDeleteTag = useCallback(
    async (tagId: number) => {
      await removeTag(tagId);
      setActiveTagIds((current) => current.filter((id) => id !== tagId));
      if (selectedDomainId !== undefined) {
        await loadAssignedTags(selectedDomainId);
      }
      refresh(true);
    },
    [removeTag, refresh, selectedDomainId, loadAssignedTags],
  );

  const handleTagsChange = useCallback(() => {
    void refreshTags();
    refresh(true);
    if (selectedDomainId !== undefined) {
      void loadAssignedTags(selectedDomainId);
    }
  }, [refreshTags, refresh, selectedDomainId, loadAssignedTags]);

  return (
    <div className="app">
      <Toolbar
        search={search}
        onSearchChange={setSearch}
        onSync={handleSync}
        syncing={syncing}
        status={status}
        onImported={() => {
          void refreshTags();
          refresh();
        }}
      />

      <TagBar
        tags={tags}
        activeTagIds={activeTagIds}
        assignedTagIds={assignedTags.map((tag) => tag.id)}
        assigning={assigning}
        onToggle={handleBarToggle}
        onCreate={handleCreateTag}
        onDelete={handleDeleteTag}
      />

      {(error || syncError) && (
        <div className="error-banner">
          {error || syncError}
        </div>
      )}

      {lastSummary && lastSummary.errors.length > 0 && (
        <div className="warning-banner">
          {lastSummary.errors.join(" · ")}
        </div>
      )}

      <main className="main-content">
        <GraphCanvas
          graph={graph}
          onNodeSelect={handleSelectNode}
          selectedId={selectedNode?.id ?? null}
          expandedDomainId={expandedDomainId}
          onDomainClick={handleDomainClick}
          onFocusedDomainChange={handleFocusedDomainChange}
        />

        {selectedNode && (
          <NodeDetail
            node={selectedNode}
            assignedTags={assignedTags}
            assigning={assigning}
            onClose={() => {
              setSelectedNode(null);
              setAssigning(false);
            }}
            onRemoved={handleRemoved}
            onRefresh={handleTagsChange}
            onStartAssign={() => setAssigning((current) => !current)}
            onUnassignTag={(tagId) => void handleSetTag(tagId, false)}
          />
        )}
      </main>

      {loading && !syncing && (
        <div className="loading-indicator">Loading graph…</div>
      )}
    </div>
  );
}

export default App;
