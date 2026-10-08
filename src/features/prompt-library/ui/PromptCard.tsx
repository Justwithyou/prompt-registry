import type { MouseEvent } from "react";
import { useNavigate } from "react-router";
import type { Prompt } from "../../../entities/prompt/types";
import { usePromptStore } from "../stores/promptStore";
import { useCategoryStore } from "../../catalog/index";
import { Icon } from "../../../shared/ui/Icon";

/** 截取纯文本摘要（去除常见 Markdown 标记）。 */
function excerpt(source: string, max = 72): string {
  const plain = source
    .replace(/```[\s\S]*?```/g, " ")
    .replace(/^#{1,6}\s+/gm, "")
    .replace(/[*_>`~|#-]+/g, " ")
    .replace(/\s+/g, " ")
    .trim();
  return plain.length > max ? `${plain.slice(0, max)}…` : plain;
}

/** 格式化时间戳为简短日期。 */
function formatTime(iso: string): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  return d.toLocaleDateString("zh-CN", { month: "2-digit", day: "2-digit" });
}

/**
 * 提示词卡片（prompt-library/ui，M2 D3/D4）。
 * 纯展示 + 快捷操作：标题、分类、收藏/置顶/评分与更新时间；
 * 卡片底部提供一键收藏/置顶/评分；多选模式下显示选择框。
 * 单击卡片进入编辑器；按钮区域拦截冒泡不触发跳转。
 */
export function PromptCard({ prompt }: { prompt: Prompt }) {
  const navigate = useNavigate();
  const selectionMode = usePromptStore((s) => s.selectionMode);
  const selectedIds = usePromptStore((s) => s.selectedIds);
  const toggleSelected = usePromptStore((s) => s.toggleSelected);
  const setPromptRelations = usePromptStore((s) => s.setPromptRelations);
  const categories = useCategoryStore((s) => s.categories);

  const rating = Math.max(0, Math.min(5, prompt.rating));
  const selected = selectedIds.includes(prompt.id);
  // 优先按树形分类（categoryId）解析展示名，向下兼容 legacy 字符串分类。
  const categoryName = prompt.categoryId
    ? categories.find((c) => c.id === prompt.categoryId)?.name ?? prompt.category
    : prompt.category;

  function open() {
    navigate(`/prompts/${prompt.id}`);
  }

  function stop(e: MouseEvent) {
    e.stopPropagation();
  }

  function handleRatingStar(e: MouseEvent, value: number) {
    stop(e);
    void setPromptRelations(prompt.id, {
      rating: rating === value ? 0 : value,
    });
  }

  return (
    <div
      className={`prompt-card${selected ? " is-selected" : ""}`}
      onClick={open}
    >
      <div className="prompt-card-header">
        {selectionMode && (
          <input
            type="checkbox"
            className="prompt-card-check"
            checked={selected}
            onClick={stop}
            onChange={() => toggleSelected(prompt.id)}
          />
        )}
        <span className="prompt-card-title">{prompt.title}</span>
        <span className="prompt-card-badges">
          {prompt.isPinned && <span className="badge badge-pin">置顶</span>}
          {prompt.isFavorite && <span className="badge badge-fav">收藏</span>}
        </span>
      </div>

      <div className="prompt-card-meta">
        {categoryName && (
          <span className="prompt-card-category">{categoryName}</span>
        )}
        <span className="prompt-card-rating" title={`评分 ${prompt.rating} / 5`}>
          {rating > 0 ? "★".repeat(rating) : "未评分"}
        </span>
      </div>

      <p className="prompt-card-desc">
        {prompt.description ?? excerpt(prompt.content)}
      </p>

      <div className="prompt-card-footer" onClick={stop}>
        <span className="prompt-card-time">{formatTime(prompt.updatedAt)}</span>
        <div className="prompt-card-actions">
          <span className="quick-rating" title="点击星星评分">
            {[1, 2, 3, 4, 5].map((n) => (
              <button
                key={n}
                type="button"
                className="quick-star"
                onClick={(e) => handleRatingStar(e, n)}
              >
                <Icon
                  name={n <= rating ? "star-filled" : "star"}
                  size={12}
                />
              </button>
            ))}
          </span>
        </div>
      </div>
    </div>
  );
}