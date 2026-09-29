import { Link } from "react-router";
import type { Prompt } from "../../../entities/prompt/types";

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
 * 提示词卡片（prompt-library/ui）。
 * 纯展示组件：标题、分类、摘要、收藏/置顶/评分与更新时间。
 */
export function PromptCard({ prompt }: { prompt: Prompt }) {
  const rating = Math.max(0, Math.min(5, prompt.rating));

  return (
    <Link to={`/prompts/${prompt.id}`} className="prompt-card">
      <div className="prompt-card-header">
        <span className="prompt-card-title">{prompt.title}</span>
        <span className="prompt-card-badges">
          {prompt.isPinned && <span className="badge badge-pin">置顶</span>}
          {prompt.isFavorite && <span className="badge badge-fav">收藏</span>}
        </span>
      </div>

      <div className="prompt-card-meta">
        {prompt.category && (
          <span className="prompt-card-category">{prompt.category}</span>
        )}
        <span
          className="prompt-card-rating"
          title={`评分 ${prompt.rating} / 5`}
        >
          {rating > 0 ? "★".repeat(rating) : "未评分"}
        </span>
      </div>

      <p className="prompt-card-desc">
        {prompt.description ?? excerpt(prompt.content)}
      </p>

      <div className="prompt-card-footer">
        <span className="prompt-card-time">{formatTime(prompt.updatedAt)}</span>
      </div>
    </Link>
  );
}