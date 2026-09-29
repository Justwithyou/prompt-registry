interface MetadataPanelProps {
  category: string;
  description: string;
  isPinned: boolean;
  isFavorite: boolean;
  rating: number;
  onCategoryChange: (value: string) => void;
  onDescriptionChange: (value: string) => void;
  onPinnedChange: (value: boolean) => void;
  onFavoriteChange: (value: boolean) => void;
  onRatingChange: (value: number) => void;
}

/**
 * 元数据面板（prompt-editor/ui）。
 * 编辑分类、备注、置顶、收藏、评分；文本字段经父组件防抖保存，开关/评分即时保存。
 */
export function MetadataPanel({
  category,
  description,
  isPinned,
  isFavorite,
  rating,
  onCategoryChange,
  onDescriptionChange,
  onPinnedChange,
  onFavoriteChange,
  onRatingChange,
}: MetadataPanelProps) {
  function handleRating(value: string) {
    const n = Number(value);
    if (Number.isNaN(n)) return;
    onRatingChange(Math.max(0, Math.min(5, Math.round(n))));
  }

  return (
    <div className="metadata-panel">
      <label className="field">
        <span className="field-label">分类</span>
        <input
          type="text"
          value={category}
          placeholder="如：客服 / 编程"
          onChange={(e) => onCategoryChange(e.target.value)}
        />
      </label>

      <label className="field">
        <span className="field-label">备注</span>
        <textarea
          value={description}
          rows={3}
          placeholder="一句话描述该提示词的用途"
          onChange={(e) => onDescriptionChange(e.target.value)}
        />
      </label>

      <label className="field field-inline">
        <input
          type="checkbox"
          checked={isPinned}
          onChange={(e) => onPinnedChange(e.target.checked)}
        />
        <span>置顶</span>
      </label>

      <label className="field field-inline">
        <input
          type="checkbox"
          checked={isFavorite}
          onChange={(e) => onFavoriteChange(e.target.checked)}
        />
        <span>收藏</span>
      </label>

      <label className="field">
        <span className="field-label">评分（0-5）</span>
        <input
          type="number"
          min={0}
          max={5}
          step={1}
          value={rating}
          onChange={(e) => handleRating(e.target.value)}
        />
      </label>
    </div>
  );
}