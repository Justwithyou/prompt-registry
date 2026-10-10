import { useEffect } from "react";
import type { ReactNode } from "react";
import { Icon } from "./Icon";

/**
 * 通用确认对话框（shared/ui）。
 * 统一危险/关键操作的二次确认：支持 ESC、遮罩点击、关闭按钮三通道关闭。
 */
interface ConfirmDialogProps {
  open: boolean;
  title: string;
  message: ReactNode;
  confirmLabel?: string;
  danger?: boolean;
  busy?: boolean;
  onConfirm: () => void;
  onClose: () => void;
}

export function ConfirmDialog({
  open,
  title,
  message,
  confirmLabel = "确认",
  danger,
  busy,
  onConfirm,
  onClose,
}: ConfirmDialogProps) {
  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onClose]);

  if (!open) return null;

  return (
    <div className="editor-overlay" onClick={onClose}>
      <div
        className="editor-overlay-card"
        style={{ width: "min(420px, 100%)" }}
        onClick={(e) => e.stopPropagation()}
      >
        <button
          type="button"
          className="overlay-close"
          onClick={onClose}
          title="关闭"
        >
          <Icon name="x" size={18} />
        </button>
        <h3 className="panel-title">
          <Icon name={danger ? "trash" : "file-text"} size={18} />
          {title}
        </h3>
        <p className="panel-subtitle">{message}</p>
        <div className="export-actions">
          <button
            type="button"
            className={danger ? "btn-danger" : "btn-primary"}
            onClick={onConfirm}
            disabled={busy}
          >
            {confirmLabel}
          </button>
          <button type="button" className="btn-ghost" onClick={onClose}>
            取消
          </button>
        </div>
      </div>
    </div>
  );
}