import { useState } from "react";
import type { BatchUpdateInput } from "../../../shared/api/commands";
import { usePromptStore } from "../../prompt-library/index";
import { useTagStore } from "../../tag-management/index";
import { useProjectStore } from "../../project-management/index";
import { tagDisplayName } from "../../../shared/lib/organization";
import { Icon } from "../../../shared/ui/Icon";

/**
 * 批量操作工具栏（D4 / batch/ui）。
 * 多选模式下，对已选提示词执行：批量打标签、移动项目、删除。
 * 选择器采用原生 select，选择后立即应用并复位占位。
 */
export function BatchToolbar() {
  const selectedIds = usePromptStore((s) => s.selectedIds);
  const selectionMode = usePromptStore((s) => s.selectionMode);
  const batchUpdatePrompts = usePromptStore((s) => s.batchUpdatePrompts);
  const clearSelection = usePromptStore((s) => s.clearSelection);
  const tags = useTagStore((s) => s.tags);
  const projects = useProjectStore((s) => s.projects);

  const [confirming, setConfirming] = useState(false);

  if (!selectionMode) return null;

  const count = selectedIds.length;

  function handleBatch(batch: BatchUpdateInput) {
    if (count === 0) return;
    void batchUpdatePrompts(selectedIds, batch);
    clearSelection();
  }

  function handleDelete() {
    if (!confirming) {
      setConfirming(true);
      return;
    }
    handleBatch({ delete: true });
    setConfirming(false);
  }

  return (
    <div className="batch-toolbar">
      <span className="batch-count">
        <Icon name="check" size={14} />
        已选 {count} 项
      </span>

      <label className="batch-select">
        <Icon name="tag" size={14} />
        <select
          value=""
          onChange={(e) => {
            if (e.target.value) handleBatch({ addTagIds: [e.target.value] });
          }}
        >
          <option value="" disabled>
            打标签…
          </option>
          {tags.map((t) => (
            <option key={t.id} value={t.id}>
              {tagDisplayName(tags, t)}
            </option>
          ))}
        </select>
      </label>

      <label className="batch-select">
        <Icon name="briefcase" size={14} />
        <select
          value=""
          onChange={(e) => {
            if (e.target.value) handleBatch({ setProjectId: e.target.value });
          }}
        >
          <option value="" disabled>
            移动到项目…
          </option>
          {projects.map((p) => (
            <option key={p.id} value={p.id}>
              {p.name}
            </option>
          ))}
        </select>
      </label>

      <button
        type="button"
        className={`btn-danger${confirming ? " is-confirm" : ""}`}
        onClick={handleDelete}
      >
        <Icon name="trash" size={14} />
        {confirming ? "确认删除" : "删除"}
      </button>

      <button type="button" className="btn-ghost" onClick={clearSelection}>
        <Icon name="x" size={14} />
        取消
      </button>
    </div>
  );
}