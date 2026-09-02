import { invoke } from "@tauri-apps/api/core";
import type { SyncStatus } from "../types/graph";

interface ToolbarProps {
  search: string;
  onSearchChange: (value: string) => void;
  onSync: () => void;
  syncing: boolean;
  status: SyncStatus | null;
  onExport: (format: "json" | "markdown" | "html") => void;
}

export function Toolbar({
  search,
  onSearchChange,
  onSync,
  syncing,
  status,
  onExport,
}: ToolbarProps) {
  const handleExport = async (format: "json" | "markdown" | "html") => {
    try {
      const content = await invoke<string>("export_graph", { format });
      const saved = await invoke<string | null>("save_export", {
        content,
        format,
      });
      if (saved) {
        onExport(format);
      }
    } catch (e) {
      console.error("Export failed:", e);
    }
  };

  const lastSync = status?.last_sync_at
    ? new Date(status.last_sync_at).toLocaleTimeString()
    : "Never";

  return (
    <header className="toolbar">
      <div className="toolbar-left">
        <h1 className="app-title">Webgraphy</h1>
        <span className="stats">
          {status?.total_domains ?? 0} domains · {status?.total_pages ?? 0} pages
        </span>
      </div>

      <div className="toolbar-center">
        <input
          type="search"
          placeholder="Search domains, titles, URLs…"
          value={search}
          onChange={(e) => onSearchChange(e.target.value)}
          className="search-input"
        />
      </div>

      <div className="toolbar-right">
        <span className="last-sync">Last sync: {lastSync}</span>
        <button
          className="action-btn"
          onClick={onSync}
          disabled={syncing}
          type="button"
        >
          {syncing ? "Syncing…" : "Sync now"}
        </button>
        <div className="export-group">
          <button
            className="action-btn"
            onClick={() => handleExport("json")}
            type="button"
          >
            JSON
          </button>
          <button
            className="action-btn"
            onClick={() => handleExport("markdown")}
            type="button"
          >
            MD
          </button>
          <button
            className="action-btn"
            onClick={() => handleExport("html")}
            type="button"
          >
            HTML
          </button>
        </div>
      </div>
    </header>
  );
}
