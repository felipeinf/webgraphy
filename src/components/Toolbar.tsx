import { invoke } from "@tauri-apps/api/core";
import type { SyncStatus } from "../types/graph";

interface ToolbarProps {
  search: string;
  onSearchChange: (value: string) => void;
  onSync: () => void;
  syncing: boolean;
  status: SyncStatus | null;
  onImported: () => void;
}

export function Toolbar({
  search,
  onSearchChange,
  onSync,
  syncing,
  status,
  onImported,
}: ToolbarProps) {
  const handleExport = async () => {
    try {
      await invoke<string | null>("export_graph");
    } catch (e) {
      console.error("Export failed:", e);
    }
  };

  const handleImport = async () => {
    try {
      const result = await invoke<{
        domains_upserted: number;
        pages_upserted: number;
      } | null>("import_graph");
      if (result) {
        onImported();
      }
    } catch (e) {
      console.error("Import failed:", e);
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
        <button className="action-btn" onClick={handleImport} type="button">
          Import
        </button>
        <button className="action-btn" onClick={handleExport} type="button">
          Export
        </button>
      </div>
    </header>
  );
}
