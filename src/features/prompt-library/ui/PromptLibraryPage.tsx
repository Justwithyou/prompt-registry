import { useEffect, useState } from "react";
import type { ReactNode } from "react";
import { useNavigate } from "react-router";
import { usePromptStore } from "../stores/promptStore";
import { Icon } from "../../../shared/ui/Icon";
import { PromptList } from "./PromptList";
import { PromptCard } from "./PromptCard";
import { PromptFilterBar } from "./PromptFilterBar";

/**
 * 提示词库列表页（prompt-library/ui，M2 D3/D4）。
 *
 * 组成：命令栏（页面标识 + 视图切换 + 批量 + 新建）+ 联合筛选条 + 批量工具栏插槽 + 网格/列表。
 * M2-I1 起数据来自真实 promptStore（服务器端组合筛选），支持排序、批量、收藏/置顶/评分。
 * 批量工具栏由 app 层组合注入（FSD：prompt-library 不直接依赖 batch）。
 */

type ViewMode = "grid" | "list";
const VIEW_KEY = "prompt-manager.view-mode";

function initialViewMode(): ViewMode {
  return localStorage.getItem(VIEW_KEY) === "list" ? "list" : "grid";
}

export function PromptLibraryPage({ batchToolbar }: { batchToolbar?: ReactNode }) {
  const prompts = usePromptStore((s) => s.prompts);
  const selectionMode = usePromptStore((s) => s.selectionMode);
  const setSelectionMode = usePromptStore((s) => s.setSelectionMode);
  const createPrompt = usePromptStore((s) => s.createPrompt);
  const fetchPrompts = usePromptStore((s) => s.fetchPrompts);
  const navigate = useNavigate();

  const [viewMode, setViewMode] = useState<ViewMode>(initialViewMode);

  useEffect(() => {
    void fetchPrompts();
  }, [fetchPrompts]);

  async function handleCreate() {
    const created = await createPrompt({ title: "未命名提示词", content: "" });
    navigate(`/prompts/${created.id}`);
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

        <button
          type="button"
          className={`btn-ghost${selectionMode ? " is-active" : ""}`}
          onClick={() => setSelectionMode(!selectionMode)}
        >
          <Icon name="check" size={16} />
          批量
        </button>

        <button type="button" className="btn-primary" onClick={handleCreate}>
          <Icon name="plus" size={16} />
          新建
        </button>
      </header>

      {batchToolbar}

      <PromptFilterBar />

      {viewMode === "grid" ? (
        <div className="prompt-grid">
          {prompts.length === 0 ? (
            <div className="prompt-list-empty">暂无提示词。</div>
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