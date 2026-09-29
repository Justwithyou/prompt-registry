import { useLayoutEffect, useRef, useState } from "react";
import type { Prompt } from "../../../entities/prompt/types";
import { PromptCard } from "./PromptCard";

/**
 * 提示词虚拟滚动列表（prompt-library/ui）。
 *
 * 采用「固定行高 + 窗口化」的轻量虚拟滚动：仅渲染可视区附近的卡片，保证万级数据下滚动流畅，
 * 不引入第三方依赖（对应架构方案 3.4 / 12.3 的虚拟滚动要求）。
 */

// 与样式表 `app.css` 中 `.prompt-card` 的固定高度（132）＋间距（16）保持一致。
const CARD_HEIGHT = 148;
// 可视区外多渲染的行数，减少快速滚动时的白屏。
const OVERSCAN = 4;

export function PromptList({ prompts }: { prompts: Prompt[] }) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const [viewport, setViewport] = useState(0);

  useLayoutEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const update = () => setViewport(el.clientHeight);
    update();
    const ro = new ResizeObserver(update);
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  const total = prompts.length;
  const start = Math.max(0, Math.floor(scrollTop / CARD_HEIGHT) - OVERSCAN);
  const end = Math.min(total, Math.ceil((scrollTop + viewport) / CARD_HEIGHT) + OVERSCAN);
  const visible = prompts.slice(start, end);

  return (
    <div
      ref={containerRef}
      className="prompt-list"
      onScroll={(e) => setScrollTop(e.currentTarget.scrollTop)}
    >
      {total === 0 ? (
        <div className="prompt-list-empty">暂无提示词，点击「新建」开始。</div>
      ) : (
        <div className="prompt-list-spacer" style={{ height: total * CARD_HEIGHT }}>
          <div
            className="prompt-list-window"
            style={{ transform: `translateY(${start * CARD_HEIGHT}px)` }}
          >
            {visible.map((p) => (
              <PromptCard key={p.id} prompt={p} />
            ))}
          </div>
        </div>
      )}
    </div>
  );
}