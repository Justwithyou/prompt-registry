import type { Project } from "../../../entities/project/types";
import type { Tag } from "../../../entities/tag/types";
import { tagDisplayName } from "../../../shared/lib/organization";
import { RatingStars } from "../../../shared/ui/RatingStars";
import { Select } from "../../../shared/ui/Select";

/** 分类下拉选项（label 含层级缩进）。 */
export interface CategoryOption {
  id: string;
  label: string;
}

interface MetadataPanelProps {
  categoryId: string;
  categoryOptions: CategoryOption[];
  onCategoryChange: (id: string) => void;
  projectId: string;
  projects: Project[];
  onProjectChange: (id: string) => void;
  tags: Tag[];
  selectedTagIds: string[];
  onToggleTag: (id: string) => void;
  description: string;
  isPinned: boolean;
  isFavorite: boolean;
  rating: number;
  onDescriptionChange: (value: string) => void;
  onPinnedChange: (value: boolean) => void;
  onFavoriteChange: (value: boolean) => void;
  onRatingChange: (value: number) => void;
}

/**
 * 元数据面板（prompt-editor/ui）。
 * 选择分类（树形下拉）、项目、标签（多选），编辑备注，以及置顶 / 收藏 / 评分。
 * 文本字段经父组件防抖保存；分类 / 项目 / 标签经组织关系接口保存，开关 / 评分即时保存。
 */
export function MetadataPanel({
  categoryId,
  categoryOptions,
  onCategoryChange,
  projectId,
  projects,
  onProjectChange,
  tags,
  selectedTagIds,
  onToggleTag,
  description,
  isPinned,
  isFavorite,
  rating,
  onDescriptionChange,
  onPinnedChange,
  onFavoriteChange,
  onRatingChange,
}: MetadataPanelProps) {
  return (
    <div className="metadata-panel">
      <label className="field">
        <span className="field-label">分类</span>
        <Select
          value={categoryId}
          onChange={onCategoryChange}
          options={[
            { value: "", label: "未分类" },
            ...categoryOptions.map((o) => ({ value: o.id, label: o.label })),
          ]}
        />
      </label>

      <label className="field">
        <span className="field-label">项目</span>
        <Select
          value={projectId}
          onChange={onProjectChange}
          options={[
            { value: "", label: "未指定项目" },
            ...projects.map((p) => ({ value: p.id, label: p.name })),
          ]}
        />
      </label>

      <div className="field">
        <span className="field-label">标签</span>
        <div className="tag-select">
          {tags.length === 0 ? (
            <span className="tag-select-empty">暂无标签</span>
          ) : (
            tags.map((t) => {
              const active = selectedTagIds.includes(t.id);
              return (
                <button
                  key={t.id}
                  type="button"
                  className={`chip${active ? " is-active" : ""}`}
                  title={tagDisplayName(tags, t)}
                  onClick={() => onToggleTag(t.id)}
                >
                  <span
                    className="tag-dot"
                    style={{ background: t.color ?? "var(--accent)" }}
                  />
                  {t.name}
                </button>
              );
            })
          )}
        </div>
      </div>

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

      <div className="field">
        <span className="field-label">评分</span>
        <RatingStars
          value={rating}
          size={20}
          onChange={(n) => onRatingChange(rating === n ? 0 : n)}
        />
      </div>
    </div>
  );
}