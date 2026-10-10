import { create } from "zustand";

/**
 * 全局 Toast 反馈（shared/ui）。
 * 轻量通知：成功 / 错误 / 提示三类，自动 3 秒后消失，可点击关闭。
 * 供各 feature 在操作成功后统一反馈，替代零散的内联提示。
 */

export type ToastKind = "success" | "error" | "info";

export interface ToastItem {
  id: number;
  kind: ToastKind;
  message: string;
}

interface ToastStore {
  toasts: ToastItem[];
  push: (message: string, kind?: ToastKind) => void;
  dismiss: (id: number) => void;
}

let seq = 0;

export const useToastStore = create<ToastStore>()((set) => ({
  toasts: [],
  push(message, kind = "success") {
    const id = ++seq;
    set((s) => ({ toasts: [...s.toasts, { id, kind, message }] }));
    setTimeout(() => {
      set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) }));
    }, 3000);
  },
  dismiss(id) {
    set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) }));
  },
}));

/** 命令式快捷入口：`toast("已保存")` 或 `toast("失败", "error")`。 */
export function toast(message: string, kind?: ToastKind) {
  useToastStore.getState().push(message, kind);
}