import { useState } from "react";
import type { Tag } from "../types/graph";

interface TagBarProps {
  tags: Tag[];
  activeTagIds: number[];
  assignedTagIds: number[];
  assigning: boolean;
  onToggle: (tagId: number) => void;
  onCreate: (name: string) => Promise<void>;
  onDelete: (tagId: number) => Promise<void>;
}

export function TagBar({
  tags,
  activeTagIds,
  assignedTagIds,
  assigning,
  onToggle,
  onCreate,
  onDelete,
}: TagBarProps) {
  const [draft, setDraft] = useState("");

  const handleCreate = async () => {
    const name = draft.trim();
    if (!name) return;
    await onCreate(name);
    setDraft("");
  };

  return (
    <div className={`tag-bar${assigning ? " assigning" : ""}`}>
      {assigning && (
        <span className="tag-bar-hint">Pick a tag for this node</span>
      )}
      {tags.map((tag) => {
        const active = assigning
          ? assignedTagIds.includes(tag.id)
          : activeTagIds.includes(tag.id);
        return (
          <button
            key={tag.id}
            type="button"
            className={`tag-chip${active ? " active" : ""}`}
            onClick={() => onToggle(tag.id)}
          >
            {tag.name}
            {!assigning && (
              <span
                className="tag-chip-remove"
                onClick={(event) => {
                  event.stopPropagation();
                  void onDelete(tag.id);
                }}
              >
                ×
              </span>
            )}
          </button>
        );
      })}
      <input
        className="tag-create-input"
        value={draft}
        placeholder="+ tag"
        maxLength={32}
        onChange={(event) => setDraft(event.target.value)}
        onKeyDown={(event) => {
          if (event.key === "Enter") {
            event.preventDefault();
            void handleCreate();
          }
        }}
      />
    </div>
  );
}
