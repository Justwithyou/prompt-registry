import type { MouseEvent } from "react";
import { Icon } from "./Icon";

interface RatingStarsProps {
  /** 当前评分（0-5）。 */
  value: number;
  /** 点击第 n 颗星时回调；不传则为只读展示。 */
  onChange?: (value: number) => void;
  /** 星星图标尺寸（像素）。 */
  size?: number;
  /** 只读展示模式（无 hover/点击反馈）。 */
  readOnly?: boolean;
}

/**
 * 统一样式的星形评分控件（shared/ui）。
 * 供卡片、编辑器元数据面板复用，替换原先「文字星星 / 数字输入框 / 小星星」的不一致呈现。
 * 交互模式点击即可评分；只读模式仅做展示。
 */
export function RatingStars({
  value,
  onChange,
  size = 16,
  readOnly = false,
}: RatingStarsProps) {
  const interactive = !readOnly && !!onChange;
  const clamped = Math.max(0, Math.min(5, Math.round(value)));

  function handleClick(e: MouseEvent, n: number) {
    e.stopPropagation();
    onChange?.(n);
  }

  return (
    <span
      className={`rating-stars${readOnly ? " is-readonly" : ""}`}
      role={interactive ? "radiogroup" : undefined}
      title={readOnly ? `评分 ${clamped} / 5` : undefined}
    >
      {[1, 2, 3, 4, 5].map((n) => (
        <button
          key={n}
          type="button"
          className={`rating-star${clamped >= n ? " is-filled" : ""}`}
          disabled={!interactive}
          title={`${n} 分`}
          aria-label={`${n} 分`}
          onClick={interactive ? (e) => handleClick(e, n) : undefined}
        >
          <Icon name={clamped >= n ? "star-filled" : "star"} size={size} />
        </button>
      ))}
    </span>
  );
}