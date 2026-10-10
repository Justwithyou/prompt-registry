import { useEffect, useState } from "react";
import { useTagStore } from "../stores/tagStore";
import { getPrompts } from "../../../shared/api/commands";
import { EmptyState } from "../../../shared/ui/EmptyState";

/**
 * 标签云（D1 / tag-management/ui）。
 * 以标签关联提示词数量作为字号/不透明度权重，直观展示全部标签分布。
 * M2-I1 起权重来自真实后端（按标签过滤计数），而非 mock 的 tagIds 字段。
 */

/** 依据关联数量计算字号/不透明度档位。 */
function weight(count: number): { fontSize: number; opacity: number } {
  const t = Math.min(count, 5);
  return { fontSize: 12 + t * 2, opacity: 0.55 + t * 0.09 };
}

export function TagCloud({ onCreate }: { onCreate?: () => void }) {
  const tags = useTagStore((s) => s.tags);
  const [counts, setCounts] = useState<Record<string, number>>({});

  useEffect(() => {
    let cancelled = false;
    (async () => {
      const map: Record<string, number> = {};
      await Promise.all(
        tags.map(async (tag) => {
          map[tag.id] = (await getPrompts({ tagIds: [tag.id] })).length;
        }),
      );
      if (!cancelled) setCounts(map);
    })();
    return () => {
      cancelled = true;
    };
  }, [tags]);

  return (
    <div className="tag-cloud">
      {tags.length === 0 ? (
        <EmptyState
          icon="tag"
          title="还没有标签"
          description="用标签给提示词打上跨分类的标记"
          actionLabel="新建标签"
          onAction={onCreate}
        />
      ) : (
        tags.map((tag) => {
          const w = weight(counts[tag.id] ?? 0);
          return (
            <span
              key={tag.id}
              className="tag-cloud-item"
              style={{
                fontSize: w.fontSize,
                opacity: w.opacity,
                borderColor: tag.color ?? undefined,
              }}
            >
              <span
                className="tag-dot"
                style={{ background: tag.color ?? "var(--accent)" }}
              />
              {tag.name}
            </span>
          );
        })
      )}
    </div>
  );
}