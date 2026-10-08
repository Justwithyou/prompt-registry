import { useEffect, useMemo, useState } from "react";
import type { ReactNode } from "react";
import { useNavigate, useParams } from "react-router";
import { usePromptStore } from "../../prompt-library/index";
import { useCategoryStore } from "../../catalog/index";
import { useTagStore } from "../../tag-management/index";
import { useProjectStore } from "../../project-management/index";
import type { PromptRelations } from "../../../entities/prompt/types";
import { CopyButton } from "../../search/index";
import { useDebouncedCallback } from "../hooks/useDebouncedCallback";
import { renderMarkdown } from "../lib/markdown";
import { buildCategoryTree, type CategoryNode } from "../../../shared/lib/organization";
import { MetadataPanel, type CategoryOption } from "./MetadataPanel";
import { ExportDialog } from "../../import-export/index";
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

/** 把分类树拍平成带缩进的选项列表（供分类下拉）。 */
function flattenCategories(nodes: CategoryNode[], depth: number): CategoryOption[] {
  let out: CategoryOption[] = [];
  for (const n of nodes) {
    out.push({ id: n.id, label: "\u00A0\u00A0".repeat(depth) + n.name });
    out = out.concat(flattenCategories(n.children, depth + 1));
  }
  return out;
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
  const setPromptRelations = usePromptStore((s) => s.setPromptRelations);
  const getPromptRelations = usePromptStore((s) => s.getPromptRelations);
  const categories = useCategoryStore((s) => s.categories);
  const tags = useTagStore((s) => s.tags);
  const projects = useProjectStore((s) => s.projects);

  const source = id ? prompts.find((p) => p.id === id) : undefined;

  const categoryOptions = useMemo(
    () => flattenCategories(buildCategoryTree(categories), 0),
    [categories],
  );

  // 组织关系（分类 / 项目 / 标签）：打开编辑器时回显，修改后乐观更新。
  const [relations, setRelations] = useState<{
    projectIds: string[];
    tagIds: string[];
    categoryId: string;
  }>({ projectIds: [], tagIds: [], categoryId: "" });

  useEffect(() => {
    if (!id) return;
    let cancelled = false;
    void getPromptRelations(id).then((r: PromptRelations) => {
      if (cancelled) return;
      setRelations({
        projectIds: r.projectIds,
        tagIds: r.tagIds,
        categoryId: r.categoryId ?? "",
      });
    });
    return () => {
      cancelled = true;
    };
  }, [id, getPromptRelations]);

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
  const [showExport, setShowExport] = useState(false);

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

  const projectId = relations.projectIds[0] ?? "";

  function handleCategoryChange(categoryId: string) {
    setRelations((r) => ({ ...r, categoryId }));
    if (id) void setPromptRelations(id, { categoryId });
  }

  function handleProjectChange(nextProjectId: string) {
    const projectIds = nextProjectId ? [nextProjectId] : [];
    setRelations((r) => ({ ...r, projectIds }));
    if (id) void setPromptRelations(id, { projectIds });
  }

  function handleToggleTag(tagId: string) {
    const has = relations.tagIds.includes(tagId);
    const tagIds = has
      ? relations.tagIds.filter((t) => t !== tagId)
      : [...relations.tagIds, tagId];
    setRelations((r) => ({ ...r, tagIds }));
    if (id) void setPromptRelations(id, { tagIds });
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
          <button
            type="button"
            className="btn-ghost"
            onClick={() => setShowExport(true)}
          >
            <Icon name="download" size={16} />
            导出
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
            categoryId={relations.categoryId}
            categoryOptions={categoryOptions}
            onCategoryChange={handleCategoryChange}
            projectId={projectId}
            projects={projects}
            onProjectChange={handleProjectChange}
            tags={tags}
            selectedTagIds={relations.tagIds}
            onToggleTag={handleToggleTag}
            description={d.description}
            isPinned={d.isPinned}
            isFavorite={d.isFavorite}
            rating={d.rating}
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

      {showExport && (
        <ExportDialog
          title="导出提示词"
          ids={id ? [id] : []}
          content={d.content}
          onClose={() => setShowExport(false)}
        />
      )}
    </div>
  );
}