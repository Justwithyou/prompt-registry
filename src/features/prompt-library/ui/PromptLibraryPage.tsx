import { useEffect, useState } from "react";
import { useNavigate } from "react-router";
import { usePromptStore } from "../stores/promptStore";
import { PromptList } from "./PromptList";
import { PromptCard } from "./PromptCard";
import { Icon } from "../../../shared/ui/Icon";

/**
 * 提示词库列表页（prompt-library/ui）。
 *
 * 组成：命令栏（页面标识 + 卡片/列表视图切换 + 新建）+ 筛选 chips + 网格/列表。
 * 参考 AI Gist 展示布局：顶部命令栏承载「页面身份 + 视图切换 + 主操作」三区。
 * M1-I1 起数据来自真实 promptStore（真实 IPC → SQLite）；分类 chips 由当前列表派生。
 */

type ViewMode = "grid" | "list";
const VIEW_KEY = "prompt-manager.view-mode";

function initialViewMode(): ViewMode {
  return localStorage.getItem(VIEW_KEY) === "list" ? "list" : "grid";
}

export function PromptLibraryPage() {
  const prompts = usePromptStore((s) => s.prompts);
  const filters = usePromptStore((s) => s.filters);
  const fetchPrompts = usePromptStore((s) => s.fetchPrompts);
  const createPrompt = usePromptStore((s) => s.createPrompt);
  const navigate = useNavigate();

  const [viewMode, setViewMode] = useState<ViewMode>(initialViewMode);

  useEffect(() => {
    void fetchPrompts();
  }, [fetchPrompts]);

  const categories = Array.from(
    new Set(prompts.map((p) => p.category).filter((c): c is string => c != null)),
  );

  async function handleCreate() {
    const created = await createPrompt({ title: "未命名提示词", content: "" });
    navigate(`/prompts/${created.id}`);
  }

  function selectCategory(category: string | null) {
    // fetchPrompts 与现有 filters 合并，仅更新 category，保留置顶等其他条件
    fetchPrompts({ category: category ?? undefined });
  }

  function togglePinned() {
    fetchPrompts({ isPinned: filters.isPinned ? undefined : true });
  }

  function switchView(mode: ViewMode) {
    setViewMode(mode);
    localStorage.setItem(VIEW_KEY, mode);
  }

  return (
    <div className="library-page">
      <header className="library-header">
        <div className="page-identity">
          <span className="page-identity-icon">
            <Icon name="file-text" size={18} />
          </span>
          <div className="page-identity-text">
            <span className="page-title">提示词库</span>
            <span className="page-subtitle">共 {prompts.length} 条提示词</span>
          </div>
        </div>

        <nav className="view-switcher" aria-label="视图切换">
          <button
            type="button"
            className={`view-switcher-item${viewMode === "grid" ? " active" : ""}`}
            onClick={() => switchView("grid")}
          >
            <Icon name="grid" size={16} />
            <span>卡片</span>
          </button>
          <button
            type="button"
            className={`view-switcher-item${viewMode === "list" ? " active" : ""}`}
            onClick={() => switchView("list")}
          >
            <Icon name="list" size={16} />
            <span>列表</span>
          </button>
        </nav>

        <button type="button" className="btn-primary" onClick={handleCreate}>
          <Icon name="plus" size={16} />
          新建
        </button>
      </header>

      <div className="filter-chips">
        <button
          type="button"
          className={`chip${filters.category == null ? " is-active" : ""}`}
          onClick={() => selectCategory(null)}
        >
          全部
        </button>
        {categories.map((c) => (
          <button
            key={c}
            type="button"
            className={`chip${filters.category === c ? " is-active" : ""}`}
            onClick={() => selectCategory(c)}
          >
            {c}
          </button>
        ))}
        <span className="filter-divider" />
        <button
          type="button"
          className={`chip${filters.isPinned ? " is-active" : ""}`}
          onClick={togglePinned}
        >
          <Icon name="pin" size={14} />
          仅看置顶
        </button>
      </div>

      {viewMode === "grid" ? (
        <div className="prompt-grid">
          {prompts.length === 0 ? (
            <div className="prompt-list-empty">暂无提示词，点击「新建」开始。</div>
          ) : (
            prompts.map((p) => <PromptCard key={p.id} prompt={p} />)
          )}
        </div>
      ) : (
        <PromptList prompts={prompts} />
      )}
    </div>
  );
}