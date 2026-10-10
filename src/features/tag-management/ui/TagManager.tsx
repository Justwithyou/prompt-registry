import { useMemo, useState } from "react";
import type { Tag } from "../../../entities/tag/types";
import { useTagStore } from "../stores/tagStore";
import { tagDisplayName } from "../../../shared/lib/organization";
import { Icon } from "../../../shared/ui/Icon";
import { Select } from "../../../shared/ui/Select";
import { EmptyState } from "../../../shared/ui/EmptyState";
import { ExportDialog } from "../../import-export/index";
import { TagCloud } from "./TagCloud";
import { ConfirmDialog } from "../../../shared/ui/ConfirmDialog";
import { toast } from "../../../shared/ui/toastStore";

/**
 * 标签管理器（D1 / tag-management/ui）。
 * 层级标签（父/子，如「场景/客服」）的增删改；支持「列表 / 标签云」两种视图。
 */

type View = "list" | "cloud";

const PALETTE = ["#2f6fed", "#51cf66", "#f5b942", "#ff6b6b", "#9b5cf6", "#22b8cf"];

export function TagManager() {
  const tags = useTagStore((s) => s.tags);
  const createTag = useTagStore((s) => s.createTag);
  const updateTag = useTagStore((s) => s.updateTag);
  const deleteTag = useTagStore((s) => s.deleteTag);

  const [view, setView] = useState<View>("list");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);
  const [name, setName] = useState("");
  const [parentId, setParentId] = useState<string>("");
  const [color, setColor] = useState<string>(PALETTE[0]);
  const [showExport, setShowExport] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<{
    id: string;
    name: string;
  } | null>(null);

  const sorted = useMemo(
    () => [...tags].sort((a, b) => tagDisplayName(tags, a).localeCompare(tagDisplayName(tags, b), "zh")),
    [tags],
  );

  function resetForm() {
    setEditingId(null);
    setCreating(false);
    setName("");
    setParentId("");
    setColor(PALETTE[0]);
  }

  function startEdit(tag: Tag) {
    setEditingId(tag.id);
    setCreating(false);
    setName(tag.name);
    setParentId(tag.parentId ?? "");
    setColor(tag.color ?? PALETTE[0]);
  }

  async function submit() {
    const trimmed = name.trim();
    if (!trimmed) return;
    if (editingId) {
      await updateTag(editingId, {
        name: trimmed,
        parentId: parentId || null,
        color: color || null,
      });
      toast("已更新标签");
    } else {
      await createTag({ name: trimmed, parentId: parentId || null, color: color || null });
      toast("已创建标签");
    }
    resetForm();
  }

  async function confirmDelete() {
    if (!deleteTarget) return;
    await deleteTag(deleteTarget.id);
    toast("已删除标签");
    setDeleteTarget(null);
  }

  const curParents = tags;

  return (
    <div className="manager-page">
      <header className="manager-header">
        <div className="page-identity">
          <span className="page-identity-icon">
            <Icon name="tag" size={18} />
          </span>
          <div className="page-identity-text">
            <span className="page-title">标签管理</span>
            <span className="page-subtitle">共 {tags.length} 个标签</span>
          </div>
        </div>
        <nav className="view-switcher" aria-label="标签视图">
          <button
            type="button"
            className={`view-switcher-item${view === "list" ? " active" : ""}`}
            onClick={() => setView("list")}
          >
            <Icon name="list" size={16} />
            <span>列表</span>
          </button>
          <button
            type="button"
            className={`view-switcher-item${view === "cloud" ? " active" : ""}`}
            onClick={() => setView("cloud")}
          >
            <Icon name="tag" size={16} />
            <span>标签云</span>
          </button>
        </nav>
        <button
          type="button"
          className="btn-ghost"
          onClick={() => setShowExport(true)}
        >
          <Icon name="download" size={16} />
          批量导出
        </button>
        <button
          type="button"
          className="btn-primary"
          onClick={() => {
            resetForm();
            setCreating(true);
          }}
        >
          <Icon name="plus" size={16} />
          新建标签
        </button>
      </header>

      {creating || editingId ? (
        <div className="tag-form">
          <input
            className="tag-form-name"
            value={name}
            placeholder="标签名，如：客服"
            autoFocus
            onChange={(e) => setName(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && submit()}
          />
          <Select
            className="tag-form-parent"
            value={parentId}
            onChange={setParentId}
            options={[
              { value: "", label: "无父级（根标签）" },
              ...curParents.map((t) => ({ value: t.id, label: tagDisplayName(tags, t) })),
            ]}
          />
          <input
            className="tag-form-color"
            type="color"
            value={color}
            onChange={(e) => setColor(e.target.value)}
          />
          <button type="button" className="btn-primary" onClick={submit}>
            {editingId ? "保存" : "创建"}
          </button>
          <button type="button" className="btn-ghost" onClick={resetForm}>
            取消
          </button>
        </div>
      ) : null}

      {view === "cloud" ? (
        <TagCloud
          onCreate={() => {
            resetForm();
            setCreating(true);
          }}
        />
      ) : (
        <div className="tag-list">
          {sorted.length === 0 ? (
            <EmptyState
              icon="tag"
              title="还没有标签"
              description="用标签给提示词打上跨分类的标记"
              actionLabel="新建标签"
              onAction={() => {
                resetForm();
                setCreating(true);
              }}
            />
          ) : (
            sorted.map((tag) => (
              <div key={tag.id} className="tag-row">
                <span className="tag-dot" style={{ background: tag.color ?? "var(--accent)" }} />
                <span className="tag-row-name" title={tagDisplayName(tags, tag)}>
                  {tagDisplayName(tags, tag)}
                </span>
                <span className="tag-row-actions">
                  <button
                    type="button"
                    className="icon-btn"
                    title="编辑"
                    onClick={() => startEdit(tag)}
                  >
                    <Icon name="edit" size={14} />
                  </button>
                  <button
                    type="button"
                    className="icon-btn is-danger"
                    title="删除"
                    onClick={() =>
                      setDeleteTarget({ id: tag.id, name: tagDisplayName(tags, tag) })
                    }
                  >
                    <Icon name="trash" size={14} />
                  </button>
                </span>
              </div>
            ))
          )}
        </div>
      )}

      {showExport && (
        <ExportDialog
          title="按标签批量导出"
          dimension="tag"
          options={tags.map((t) => ({ id: t.id, name: tagDisplayName(tags, t) }))}
          onClose={() => setShowExport(false)}
        />
      )}

      <ConfirmDialog
        open={deleteTarget !== null}
        title="删除标签"
        message={
          <>
            确定删除标签「<strong>{deleteTarget?.name ?? ""}</strong>」吗？
          </>
        }
        confirmLabel="删除"
        danger
        onConfirm={() => void confirmDelete()}
        onClose={() => setDeleteTarget(null)}
      />
    </div>
  );
}