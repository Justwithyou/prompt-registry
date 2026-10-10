import { useEffect, useRef } from "react";

/**
 * 统一的 ESC 关闭 hook（shared/hooks）。
 * 供各类覆盖层（弹窗 / 面板）复用，实现「ESC + 遮罩 + 关闭按钮」三通道一致关闭。
 */
export function useEscapeKey(onEscape: () => void, active = true) {
  const callbackRef = useRef(onEscape);
  callbackRef.current = onEscape;

  useEffect(() => {
    if (!active) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") callbackRef.current();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [active]);
}