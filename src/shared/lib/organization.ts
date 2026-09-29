import type { Category } from "../../entities/category/types";
import type { Tag } from "../../entities/tag/types";

/**
 * 组织层纯函数助手（shared/lib，供多个 feature 复用，无副作用）。
 *
 * 源于 Track D 的 mock 纯函数，M2-I1 起迁入 shared 层：
 * 树形分类构建、后代收集、层级标签展示名。
 */

/** 树形分类节点。 */
export interface CategoryNode extends Category {
  children: CategoryNode[];
}

/** 按 parentId 建立「父 id -> 子 id」映射。 */
function buildParentIndex<T extends { id: string; parentId: string | null }>(
  items: T[],
): Map<string, string[]> {
  const map = new Map<string, string[]>();
  for (const item of items) {
    const key = item.parentId ?? "__root__";
    const list = map.get(key) ?? [];
    list.push(item.id);
    map.set(key, list);
  }
  return map;
}

/** 把扁平分类列表组装为树（按 sortOrder 排序）。 */
export function buildCategoryTree(categories: Category[]): CategoryNode[] {
  const byId = new Map<string, CategoryNode>();
  for (const c of categories) {
    byId.set(c.id, { ...c, children: [] });
  }
  const roots: CategoryNode[] = [];
  for (const node of byId.values()) {
    if (node.parentId && byId.has(node.parentId)) {
      byId.get(node.parentId)!.children.push(node);
    } else {
      roots.push(node);
    }
  }
  const sort = (list: CategoryNode[]) => {
    list.sort((a, b) => a.sortOrder - b.sortOrder);
    list.forEach((n) => sort(n.children));
  };
  sort(roots);
  return roots;
}

/** 收集某分类及其所有后代分类的 id 集合（用于树形级联与拖拽防环）。 */
export function collectCategoryDescendants(
  categories: Category[],
  rootId: string,
): Set<string> {
  const parentIndex = buildParentIndex(categories);
  const result = new Set<string>([rootId]);
  const stack = [rootId];
  while (stack.length > 0) {
    const cur = stack.pop()!;
    for (const childId of parentIndex.get(cur) ?? []) {
      if (!result.has(childId)) {
        result.add(childId);
        stack.push(childId);
      }
    }
  }
  return result;
}

/** 计算标签的层级展示名，如「场景/客服」。 */
export function tagDisplayName(tags: Tag[], tag: Tag): string {
  const byId = new Map(tags.map((t) => [t.id, t]));
  const parts: string[] = [tag.name];
  let cur: Tag | undefined = tag;
  while (cur.parentId && byId.has(cur.parentId)) {
    cur = byId.get(cur.parentId)!;
    parts.unshift(cur.name);
  }
  return parts.join("/");
}