import { useMemo } from "react";
import { usePromptStore } from "../stores/promptStore";
import { useCategoryStore } from "../../catalog/index";
import { useTagStore } from "../../tag-management/index";
import { useProjectStore } from "../../project-management/index";
import {
  buildCategoryTree,
  tagDisplayName,
  type CategoryNode,
} from "../../../shared/lib/organization";
import { Icon } from "../../../shared/ui/Icon";

/** 把分类树拍平成带缩进的选项列表。 */
function flattenCategories(nodes: CategoryNode[], depth: number): { id: string; label: string }[] {
  let out: { id: string; label: string }[] = [];
  for (const n of nodes) {
    out.push({ id: n.id, label: "\u00A0\u00A0".repeat(depth) + n.name });
    out = out.concat(flattenCategories(n.children, depth + 1));
  }
  return out;
}

const SORT_OPTIONS: { value: string; label: string }[] = [
  { value: "updatedAt", label: "最近更新" },
  { value: "rating", label: "评分最高" },
  { value: "createdAt", label: "创建时间" },
];

/**
 * 联合筛选条（D1 / prompt-library/ui）。
 * 分类（树形下拉）+ 标签（多选 chips）+ 项目 + 收藏/置顶 + 排序，全部组合生效。
 * M2-I1 起通过真实 store 把筛选透传给后端 SQL 组合查询。
 */
export function PromptFilterBar() {
  const categories = useCategoryStore((s) => s.categories);
  const tags = useTagStore((s) => s.tags);
  const projects = useProjectStore((s) => s.projects);
  const filters = usePromptStore((s) => s.filters);
  const setFilters = usePromptStore((s) => s.setFilters);
  const resetFilters = usePromptStore((s) => s.resetFilters);

  const catOptions = useMemo(
    () => flattenCategories(buildCategoryTree(categories), 0),
    [categories],
  );
  const activeTagIds = filters.tagIds ?? [];

  function toggleTag(id: string) {
    void setFilters({
      tagIds: activeTagIds.includes(id)
        ? activeTagIds.filter((t) => t !== id)
        : [...activeTagIds, id],
    });
  }

  return (
    <div className="filter-bar">
      <div className="filter-search">
        <Icon name="search" size={14} className="filter-search-icon" />
        <input
          type="text"
          value={filters.keyword ?? ""}
          placeholder="关键词…"
          onChange={(e) =>
            void setFilters({ keyword: e.target.value || undefined })
          }
        />
      </div>

      <select
        className="filter-select"
        value={filters.categoryId ?? ""}
        onChange={(e) => void setFilters({ categoryId: e.target.value || undefined })}
      >
        <option value="">全部分类</option>
        {catOptions.map((o) => (
          <option key={o.id} value={o.id}>
            {o.label}
          </option>
        ))}
      </select>

      <select
        className="filter-select"
        value={filters.projectId ?? ""}
        onChange={(e) => void setFilters({ projectId: e.target.value || undefined })}
      >
        <option value="">全部项目</option>
        {projects.map((p) => (
          <option key={p.id} value={p.id}>
            {p.name}
          </option>
        ))}
      </select>

      <div className="filter-tags">
        {tags.map((t) => {
          const active = activeTagIds.includes(t.id);
          return (
            <button
              key={t.id}
              type="button"
              className={`chip${active ? " is-active" : ""}`}
              title={tagDisplayName(tags, t)}
              onClick={() => toggleTag(t.id)}
            >
              <span
                className="tag-dot"
                style={{ background: t.color ?? "var(--accent)" }}
              />
              {t.name}
            </button>
          );
        })}
      </div>

      <span className="filter-divider" />

      <button
        type="button"
        className={`chip${filters.isFavorite ? " is-active" : ""}`}
        onClick={() => void setFilters({ isFavorite: filters.isFavorite ? undefined : true })}
      >
        <Icon name="star" size={14} />
        仅看收藏
      </button>
      <button
        type="button"
        className={`chip${filters.isPinned ? " is-active" : ""}`}
        onClick={() => void setFilters({ isPinned: filters.isPinned ? undefined : true })}
      >
        <Icon name="pin" size={14} />
        仅看置顶
      </button>

      <select
        className="filter-select filter-sort"
        value={filters.sortBy ?? "updatedAt"}
        onChange={(e) =>
          void setFilters({
            sortBy: e.target.value as "updatedAt" | "rating" | "createdAt",
          })
        }
      >
        {SORT_OPTIONS.map((o) => (
          <option key={o.value} value={o.value}>
            {o.label}
          </option>
        ))}
      </select>

      <button type="button" className="btn-ghost btn-sm" onClick={() => void resetFilters()}>
        重置
      </button>
    </div>
  );
}