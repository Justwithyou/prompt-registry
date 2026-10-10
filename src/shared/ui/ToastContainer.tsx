import { useToastStore } from "./toastStore";
import { Icon } from "./Icon";

/**
 * 全局 Toast 容器（shared/ui）。
 * 固定在右下角，渲染当前所有待显示的通知。
 */
export function ToastContainer() {
  const toasts = useToastStore((s) => s.toasts);
  const dismiss = useToastStore((s) => s.dismiss);

  if (toasts.length === 0) return null;

  return (
    <div className="toast-container">
      {toasts.map((t) => (
        <div
          key={t.id}
          className={`toast toast-${t.kind}`}
          onClick={() => dismiss(t.id)}
          role="status"
        >
          <Icon
            name={t.kind === "error" ? "x" : t.kind === "info" ? "info" : "check"}
            size={16}
          />
          <span>{t.message}</span>
        </div>
      ))}
    </div>
  );
}