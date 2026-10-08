import { useRef, useState } from "react";
import { useCategoryStore } from "../stores/categoryStore";
import {
  buildCategoryTree,
  type CategoryNode,
} from "../../../shared/lib/organization";
import { Icon } from "../../../shared/ui/Icon";
import { ExportDialog } from "../../import-export/index";

/**
 * 分类管理器（D1 / catalog/ui）。
 * 树形分类的增删改 + 拖拽排序：节点行可拖拽到另一节点上，作为其同级（并改变父级），
 * 依据 sortOrder 重排兄弟顺序。符合 M2 §3.1「树形分类、拖拽排序」。
 */
export function CategoryManager() {
  const categories = useCategoryStore((s) => s.categories);
  const createCategory = useCategoryStore((s) => s.createCategory);
  const updateCategory = useCategoryStore((s) => s.updateCategory);
  const deleteCategory = useCategoryStore((s) => s.deleteCategory);
  const moveCategory = useCategoryStore((s) => s.moveCategory);

  const tree = buildCategoryTree(categories);
  const [expanded, setExpanded] = useState<Set<string>>(
    () => new Set(categories.map((c) => c.id)),
  );
  const [editingId, setEditingId] = useState<string | null>(null);
  const [draft, setDraft] = useState("");
  const [overId, setOverId] = useState<string | null>(null);
  const [showExport, setShowExport] = useState(false);
  const draggedRef = useRef<string | null>(null);

  function toggleExpand(id: string) {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }

  function startEdit(id: string, name: string) {
    setEditingId(id);
    setDraft(name);
  }

  function commitEdit(id: string) {
    const name = draft.trim();
    if (name) updateCategory(id, { name });
    setEditingId(null);
  }

  async function handleAdd(parentId: string | null) {
    const created = await createCategory({ name: "未命名分类", parentId });
    if (parentId) setExpanded((prev) => new Set(prev).add(parentId));
    startEdit(created.id, created.name);
  }

  function handleDrop(target: CategoryNode) {
    const dragged = draggedRef.current;
    setOverId(null);
    if (dragged && dragged !== target.id) void moveCategory(dragged, target.id);
  }

  const renderNode = (node: CategoryNode, depth: number) => {
    const hasChildren = node.children.length > 0;
    const isOpen = expanded.has(node.id);
    const isEditing = editingId === node.id;
    const isOver = overId === node.id;

    return (
      <div key={node.id}>
        <div
          className={`cat-node${isOver ? " is-over" : ""}`}
          style={{ paddingLeft: depth * 20 }}
          onDragOver={(e) => {
            e.preventDefault();
            e.dataTransfer.dropEffect = "move";
            if (overId !== node.id) setOverId(node.id);
          }}
          onDragLeave={() => {
            if (overId === node.id) setOverId(null);
          }}
          onDrop={(e) => {
            e.preventDefault();
            handleDrop(node);
          }}
        >
          <span
            className="cat-node-drag"
            title="拖拽排序"
            draggable
            onDragStart={(e) => {
              draggedRef.current = node.id;
              e.dataTransfer.effectAllowed = "move";
              e.dataTransfer.setData("text/plain", node.id);
            }}
            onDragEnd={() => {
              draggedRef.current = null;
              setOverId(null);
            }}
          >
            <Icon name="menu" size={14} />
          </span>
          <button
            type="button"
            className="cat-node-toggle"
            onClick={() => hasChildren && toggleExpand(node.id)}
            aria-label={isOpen ? "收起" : "展开"}
          >
            {hasChildren ? (
              <Icon name={isOpen ? "chevron-down" : "chevron-right"} size={14} />
            ) : (
              <span className="cat-node-spacer" />
            )}
          </button>
          <Icon name="folder" size={15} className="cat-node-icon" />
          {isEditing ? (
            <input
              className="cat-node-input"
              value={draft}
              autoFocus
              onChange={(e) => setDraft(e.target.value)}
              onBlur={() => commitEdit(node.id)}
              onKeyDown={(e) => {
                if (e.key === "Enter") commitEdit(node.id);
                if (e.key === "Escape") setEditingId(null);
              }}
            />
          ) : (
            <span className="cat-node-name">{node.name}</span>
          )}
          <span className="cat-node-actions">
            <button
              type="button"
              className="icon-btn"
              title="新建子分类"
              onClick={() => handleAdd(node.id)}
            >
              <Icon name="plus" size={14} />
            </button>
            <button
              type="button"
              className="icon-btn"
              title="重命名"
              onClick={() => startEdit(node.id, node.name)}
            >
              <Icon name="edit" size={14} />
            </button>
            <button
              type="button"
              className="icon-btn is-danger"
              title="删除"
              onClick={() => deleteCategory(node.id)}
            >
              <Icon name="trash" size={14} />
            </button>
          </span>
        </div>
        {isOpen && hasChildren && node.children.map((c) => renderNode(c, depth + 1))}
      </div>
    );
  };

  return (
    <div className="manager-page">
      <header className="manager-header">
        <div className="page-identity">
          <span className="page-identity-icon">
            <Icon name="folder" size={18} />
          </span>
          <div className="page-identity-text">
            <span className="page-title">分类管理</span>
            <span className="page-subtitle">共 {categories.length} 个分类</span>
          </div>
        </div>
        <button
          type="button"
          className="btn-ghost"
          onClick={() => setShowExport(true)}
        >
          <Icon name="download" size={16} />
          批量导出
        </button>
        <button type="button" className="btn-primary" onClick={() => handleAdd(null)}>
          <Icon name="plus" size={16} />
          新建分类
        </button>
      </header>

      <div className="tree-container">
        {tree.length === 0 ? (
          <div className="manager-empty">暂无分类，点击「新建分类」开始。</div>
        ) : (
          tree.map((n) => renderNode(n, 0))
        )}
      </div>
      <p className="manager-hint">提示：拖拽分类到另一分类上可调整层级与顺序。</p>

      {showExport && (
        <ExportDialog
          title="按分类批量导出"
          dimension="category"
          options={categories.map((c) => ({ id: c.id, name: c.name }))}
          onClose={() => setShowExport(false)}
        />
      )}
    </div>
  );
}