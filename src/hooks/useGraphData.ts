import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { GraphData } from "../types/graph";

export function useGraphData(search: string, expandedDomains: number[]) {
  const [graph, setGraph] = useState<GraphData>({ nodes: [], links: [] });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const hasLoadedRef = useRef(false);

  const expandedKey = expandedDomains.join(",");
  const searchKey = search.trim();

  const refresh = useCallback(
    async (silent = false) => {
      if (!silent) setLoading(true);
      setError(null);
      try {
        const data = await invoke<GraphData>("get_graph", {
          search: searchKey || null,
          expandedDomains:
            expandedDomains.length > 0 ? expandedDomains : null,
        });
        setGraph(data);
        hasLoadedRef.current = true;
      } catch (e) {
        setError(String(e));
      } finally {
        if (!silent) setLoading(false);
      }
    },
    [searchKey, expandedKey],
  );

  useEffect(() => {
    const silent = hasLoadedRef.current;
    refresh(silent);
  }, [refresh]);

  return { graph, loading, error, refresh };
}
