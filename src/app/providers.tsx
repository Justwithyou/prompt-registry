import type { ReactNode } from "react";

/**
 * 全局 Provider 聚合（FSD app 层）。
 * M1 起在此接入：RouterProvider、TanStack Query 的 QueryClientProvider、主题 Provider。
 */
export function Providers({ children }: { children: ReactNode }) {
  return <>{children}</>;
}