import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

const imageCache = new Map<string, HTMLImageElement>();
const pending = new Set<string>();
const failed = new Set<string>();

export async function loadFaviconDataUrl(
  hostname: string,
): Promise<string | null> {
  try {
    return await invoke<string | null>("get_favicon", { hostname });
  } catch {
    return null;
  }
}

export function useFavicons(hostnames: string[]) {
  const [version, setVersion] = useState(0);

  useEffect(() => {
    let cancelled = false;
    const missing = hostnames.filter(
      (h) => !imageCache.has(h) && !pending.has(h) && !failed.has(h),
    );
    if (missing.length === 0) return;

    for (const hostname of missing) {
      pending.add(hostname);
      loadFaviconDataUrl(hostname).then((dataUrl) => {
        pending.delete(hostname);
        if (!dataUrl) {
          failed.add(hostname);
          return;
        }
        const img = new Image();
        img.onload = () => {
          imageCache.set(hostname, img);
          if (!cancelled) setVersion((v) => v + 1);
        };
        img.onerror = () => {
          failed.add(hostname);
        };
        img.src = dataUrl;
      });
    }

    return () => {
      cancelled = true;
    };
  }, [hostnames]);

  void version;
  return imageCache;
}
