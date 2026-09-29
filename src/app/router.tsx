import { createHashRouter, Navigate } from "react-router";
import { PromptLibraryPage } from "../features/prompt-library/index";
import { PromptEditorPage } from "../features/prompt-editor/index";
import { SettingsPage } from "../features/settings/index";
import { AppLayout } from "./AppLayout";

/**
 * 路由编排（FSD app 层）。
 * 使用 hash 路由（Tauri 桌面环境无服务器历史路径，hash 更稳妥）。
 * 目标路由表见《架构设计方案》3.3，M1 仅落地核心三页：
 *   / → /prompts、/prompts、/prompts/:id、/settings。
 */
export const router = createHashRouter([
  {
    path: "/",
    element: <AppLayout />,
    children: [
      { index: true, element: <Navigate to="/prompts" replace /> },
      { path: "prompts", element: <PromptLibraryPage /> },
      { path: "prompts/:id", element: <PromptEditorPage /> },
      { path: "settings", element: <SettingsPage /> },
    ],
  },
]);