import { listen, type UnlistenFn } from "@tauri-apps/api/event";

/**
 * 后端广播事件监听封装（Events 原语，单向 Rust → 前端）。
 */

/** 监听指定事件，返回用于取消监听的函数 */
export async function onEvent<T>(
  name: string,
  handler: (payload: T) => void,
): Promise<UnlistenFn> {
  return listen<T>(name, (event) => handler(event.payload));
}

/** 事件名常量，与 Rust ipc::events 对齐 */
export const Events = {
  syncCompleted: "sync-completed",
} as const;