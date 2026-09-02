import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { SyncStatus, SyncSummary } from "../types/graph";

const DEFAULT_INTERVAL_MS = 50_000;

export function useSync(onSynced?: () => void) {
  const [syncing, setSyncing] = useState(false);
  const [status, setStatus] = useState<SyncStatus | null>(null);
  const [lastSummary, setLastSummary] = useState<SyncSummary | null>(null);
  const [error, setError] = useState<string | null>(null);
  const onSyncedRef = useRef(onSynced);
  const inFlightRef = useRef(false);

  useEffect(() => {
    onSyncedRef.current = onSynced;
  }, [onSynced]);

  const refreshStatus = useCallback(async () => {
    try {
      const s = await invoke<SyncStatus>("get_sync_status_cmd");
      setStatus(s);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  const syncNow = useCallback(async () => {
    if (inFlightRef.current) return;
    inFlightRef.current = true;
    setSyncing(true);
    setError(null);
    try {
      const summary = await invoke<SyncSummary>("sync_tabs");
      setLastSummary(summary);
      await refreshStatus();
      onSyncedRef.current?.();
    } catch (e) {
      setError(String(e));
    } finally {
      inFlightRef.current = false;
      setSyncing(false);
    }
  }, [refreshStatus]);

  useEffect(() => {
    refreshStatus();
    syncNow();
    const intervalId = window.setInterval(syncNow, DEFAULT_INTERVAL_MS);
    return () => window.clearInterval(intervalId);
  }, [refreshStatus, syncNow]);

  return {
    syncing,
    status,
    lastSummary,
    error,
    syncNow,
    refreshStatus,
  };
}
