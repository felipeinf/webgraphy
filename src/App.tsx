import { useCallback, useState } from "react";
import { GraphCanvas } from "./components/GraphCanvas";
import { NodeDetail } from "./components/NodeDetail";
import { Toolbar } from "./components/Toolbar";
import { useGraphData } from "./hooks/useGraphData";
import { useSync } from "./hooks/useSync";
import type { GraphNode } from "./types/graph";
import "./App.css";

function App() {
  const [search, setSearch] = useState("");
  const [selectedNode, setSelectedNode] = useState<GraphNode | null>(null);
  const [expandedDomainId, setExpandedDomainId] = useState<number | null>(null);

  const expandedDomains =
    expandedDomainId !== null ? [expandedDomainId] : [];

  const { graph, loading, error, refresh } = useGraphData(
    search,
    expandedDomains,
  );

  const handleSynced = useCallback(() => {
    refresh();
  }, [refresh]);

  const { syncing, status, lastSummary, error: syncError, syncNow } =
    useSync(handleSynced);

  const handleSync = useCallback(async () => {
    await syncNow();
    refresh();
  }, [syncNow, refresh]);

  const handleArchive = useCallback(() => {
    setSelectedNode(null);
    refresh();
  }, [refresh]);

  const handleDomainClick = useCallback((domainId: number) => {
    setExpandedDomainId((current) => (current === domainId ? null : domainId));
    setSelectedNode(null);
  }, []);

  const handleCollapseAll = useCallback(() => {
    setExpandedDomainId(null);
    setSelectedNode(null);
  }, []);

  return (
    <div className="app">
      <Toolbar
        search={search}
        onSearchChange={setSearch}
        onSync={handleSync}
        syncing={syncing}
        status={status}
        onExport={() => {}}
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
          onNodeSelect={setSelectedNode}
          selectedId={selectedNode?.id ?? null}
          expandedDomainId={expandedDomainId}
          onDomainClick={handleDomainClick}
          onCollapseAll={handleCollapseAll}
        />

        {selectedNode && (
          <NodeDetail
            node={selectedNode}
            onClose={() => setSelectedNode(null)}
            onArchive={handleArchive}
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
