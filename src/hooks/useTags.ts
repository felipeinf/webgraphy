import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Tag } from "../types/graph";

export function useTags() {
  const [tags, setTags] = useState<Tag[]>([]);

  const refresh = useCallback(async () => {
    try {
      const list = await invoke<Tag[]>("list_tags_cmd");
      setTags(list);
    } catch {
      setTags([]);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const createTag = useCallback(
    async (name: string) => {
      const tag = await invoke<Tag>("create_tag_cmd", { name });
      await refresh();
      return tag;
    },
    [refresh],
  );

  const removeTag = useCallback(
    async (tagId: number) => {
      await invoke("delete_tag_cmd", { tagId });
      await refresh();
    },
    [refresh],
  );

  return { tags, refresh, createTag, removeTag };
}
