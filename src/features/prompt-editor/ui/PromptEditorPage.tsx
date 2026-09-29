import { useEffect, useState } from "react";
import type { ReactNode } from "react";
import { useNavigate, useParams } from "react-router";
import { usePromptStore } from "../../prompt-library/index";
import { CopyButton } from "../../search/index";
import { useDebouncedCallback } from "../hooks/useDebouncedCallback";
import { renderMarkdown } from "../lib/markdown";
import { MetadataPanel } from "./MetadataPanel";
import { Icon } from "../../../shared/ui/Icon";

interface Draft {
  title: string;
  content: string;
  description: string;
  category: string;
  isPinned: boolean;
  isFavorite: boolean;
  rating: number;
}

/**
 * 提示词编辑器（prompt-editor/ui）。
 * - Markdown 编辑 + 实时预览（分屏）
 * - 文本字段（标题/内容/备注/分类）500ms 防抖自动保存
 * - 元数据面板（置顶/收藏/评分）即时保存
 * - 删除（软删除，二次确认）+ 返回
 *
 * M1-I1：数据来自真实 promptStore；冷启动直接进入编辑器会先 fetchPrompts 再按 id 定位。
 */
interface PromptEditorPageProps {
  templatePanel?: ReactNode;
  versionPanel?: ReactNode;
}

export function PromptEditorPage({
  templatePanel,
  versionPanel,
}: PromptEditorPageProps) {
  const { id } = useParams();
  const navigate = useNavigate();
  const prompts = usePromptStore((s) => s.prompts);
  const isLoading = usePromptStore((s) => s.isLoading);
  const fetchPrompts = usePromptStore((s) => s.fetchPrompts);
  const updatePrompt = usePromptStore((s) => s.updatePrompt);
  const deletePrompt = usePromptStore((s) => s.deletePrompt);

  const source = id ? prompts.find((p) => p.id === id) : undefined;

  // 冷启动时拉取列表；列表就绪后 source 自动变为可用
  useEffect(() => {
    if (id && !source) void fetchPrompts();
  }, [id, source, fetchPrompts]);

  // 首次挂载若有 source 直接同步初始化；后续 source 出现 / id 变化时再初始化一次，
  // 且仅以 id 为依据，避免 autosave 刷新 prompts 覆盖用户正在编辑的草稿。
  const [draft, setDraft] = useState<Draft | null>(() => {
    if (!source) return null;
    return {
      title: source.title,
      content: source.content,
      description: source.description ?? "",
      category: source.category ?? "",
      isPinned: source.isPinned,
      isFavorite: source.isFavorite,
      rating: source.rating,
    };
  });
  const [initializedId, setInitializedId] = useState<string | null>(
    source ? (id ?? null) : null,
  );

  useEffect(() => {
    if (source && initializedId !== id) {
      setDraft({
        title: source.title,
        content: source.content,
        description: source.description ?? "",
        category: source.category ?? "",
        isPinned: source.isPinned,
        isFavorite: source.isFavorite,
        rating: source.rating,
      });
      setInitializedId(id ?? null);
    }
  }, [source, id, initializedId]);

  const [confirming, setConfirming] = useState(false);
  const [activePanel, setActivePanel] = useState<
    "template" | "version" | null
  >(null);

  const autosave = useDebouncedCallback((snapshot: Draft) => {
    if (!id) return;
    void updatePrompt(id, {
      title: snapshot.title,
      content: snapshot.content,
      description: snapshot.description || null,
      category: snapshot.category || null,
    });
  }, 500);

  if (!draft) {
    return (
      <div className="editor-empty">
        <p>
          {isLoading ? "加载中…" : "未找到该提示词，可能已被删除。"}
        </p>
        <button type="button" className="btn-primary" onClick={() => navigate("/prompts")}>
          <Icon name="arrow-left" size={16} />
          返回列表
        </button>
      </div>
    );
  }

  const d = draft;

  function patchText(
    patch: Partial<Pick<Draft, "title" | "content" | "description" | "category">>,
  ) {
    const next = { ...d, ...patch };
    setDraft(next);
    autosave(next);
  }

  function patchMeta(patch: Partial<Pick<Draft, "isPinned" | "isFavorite" | "rating">>) {
    setDraft({ ...d, ...patch });
    if (id) void updatePrompt(id, patch);
  }

  function handleDelete() {
    if (!confirming) {
      setConfirming(true);
      return;
    }
    if (id) void deletePrompt(id).then(() => navigate("/prompts"));
  }

  return (
    <div className="editor-page">
      <header className="editor-header">
        <button type="button" className="btn-ghost" onClick={() => navigate("/prompts")}>
          <Icon name="arrow-left" size={16} />
          返回
        </button>
        <input
          className="editor-title"
          value={d.title}
          placeholder="标题"
          onChange={(e) => patchText({ title: e.target.value })}
        />
        <div className="editor-actions">
          <button
            type="button"
            className={`btn-ghost${activePanel === "template" ? " is-active" : ""}`}
            onClick={() =>
              setActivePanel(activePanel === "template" ? null : "template")
            }
          >
            <Icon name="braces" size={16} />
            模板
          </button>
          <button
            type="button"
            className={`btn-ghost${activePanel === "version" ? " is-active" : ""}`}
            onClick={() =>
              setActivePanel(activePanel === "version" ? null : "version")
            }
          >
            <Icon name="history" size={16} />
            版本
          </button>
          <CopyButton text={d.content} />
          <button
            type="button"
            className={`btn-danger${confirming ? " is-confirm" : ""}`}
            onClick={handleDelete}
          >
            <Icon name="trash" size={16} />
            {confirming ? "再次点击确认删除" : "删除"}
          </button>
        </div>
      </header>

      <div className="editor-body">
        <section className="editor-main">
          <div className="editor-pane">
            <span className="pane-label">编辑</span>
            <textarea
              className="editor-content"
              value={d.content}
              placeholder="在此输入 Markdown 提示词内容…"
              onChange={(e) => patchText({ content: e.target.value })}
            />
          </div>
          <div className="editor-pane">
            <span className="pane-label">预览</span>
            <div
              className="editor-preview markdown-body"
              dangerouslySetInnerHTML={{ __html: renderMarkdown(d.content) }}
            />
          </div>
        </section>

        <aside className="editor-meta">
          <MetadataPanel
            category={d.category}
            description={d.description}
            isPinned={d.isPinned}
            isFavorite={d.isFavorite}
            rating={d.rating}
            onCategoryChange={(v) => patchText({ category: v })}
            onDescriptionChange={(v) => patchText({ description: v })}
            onPinnedChange={(v) => patchMeta({ isPinned: v })}
            onFavoriteChange={(v) => patchMeta({ isFavorite: v })}
            onRatingChange={(v) => patchMeta({ rating: v })}
          />
        </aside>
      </div>

      {activePanel && (
        <div className="editor-overlay" onClick={() => setActivePanel(null)}>
          <div
            className="editor-overlay-card"
            onClick={(e) => e.stopPropagation()}
          >
            <button
              type="button"
              className="overlay-close"
              onClick={() => setActivePanel(null)}
              title="关闭"
            >
              <Icon name="x" size={18} />
            </button>
            {activePanel === "template" ? templatePanel : versionPanel}
          </div>
        </div>
      )}
    </div>
  );
}