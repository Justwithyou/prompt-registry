import { useEffect, useState } from "react";
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Icon } from "../shared/ui/Icon";

interface TitleBarProps {
  onHelp: () => void;
  onAbout: () => void;
}

// 非 Tauri 环境（如浏览器 `pnpm dev`）下无窗口可操控，仅渲染帮助/关于入口。
const appWindow = isTauri() ? getCurrentWindow() : null;

/**
 * 自定义窗口标题栏（app 层）。
 * 原生标题栏已在 tauri.conf.json 中以 decorations:false 关闭，
 * 此处提供左侧「帮助 / 关于」入口 + 右侧窗口控制（最小化/最大化/关闭），
 * 中间整块为拖拽移动区域（data-tauri-drag-region）。
 */
export function TitleBar({ onHelp, onAbout }: TitleBarProps) {
  const [maximized, setMaximized] = useState(false);

  useEffect(() => {
    if (!appWindow) return;
    let disposed = false;
    let unlisten: (() => void) | undefined;

    const refresh = async () => {
      const isMax = await appWindow.isMaximized();
      if (!disposed) setMaximized(isMax);
    };

    void refresh();
    appWindow.onResized(refresh).then((fn) => {
      if (disposed) fn();
      else unlisten = fn;
    });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  return (
    <header className="app-titlebar">
      <div className="titlebar-actions">
        <span className="titlebar-sep" aria-hidden="true" />
        <button type="button" className="titlebar-btn" title="帮助" onClick={onHelp}>
          <Icon name="help" size={16} />
          <span>帮助</span>
        </button>
        <button type="button" className="titlebar-btn" title="关于" onClick={onAbout}>
          <Icon name="info" size={16} />
          <span>关于</span>
        </button>
      </div>

      <div
        className="titlebar-drag"
        data-tauri-drag-region=""
        onDoubleClick={() => void appWindow?.toggleMaximize()}
      />

      {appWindow && (
        <div className="titlebar-controls">
          <button
            type="button"
            className="titlebar-control"
            title="最小化"
            onClick={() => void appWindow.minimize()}
          >
            <Icon name="minimize" size={16} />
          </button>
          <button
            type="button"
            className="titlebar-control"
            title={maximized ? "还原" : "最大化"}
            onClick={() => void appWindow.toggleMaximize()}
          >
            <Icon name={maximized ? "restore" : "maximize"} size={16} />
          </button>
          <button
            type="button"
            className="titlebar-control is-close"
            title="关闭"
            onClick={() => void appWindow.close()}
          >
            <Icon name="x" size={16} />
          </button>
        </div>
      )}
    </header>
  );
}