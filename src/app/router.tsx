import { createHashRouter, Navigate } from "react-router";
import { PromptLibraryPage } from "../features/prompt-library/index";
import { PromptEditorPage } from "../features/prompt-editor/index";
import { SettingsPage } from "../features/settings/index";
import { CategoryManager } from "../features/catalog/index";
import { TagManager } from "../features/tag-management/index";
import { ProjectManager } from "../features/project-management/index";
import { BatchToolbar } from "../features/batch/index";
import { TemplateUsePanel } from "../features/template-engine/index";
import { VersionHistoryPanel } from "../features/version-control/index";
import { SyncPanel } from "../features/sync/index";
import { AppLayout } from "./AppLayout";

/**
 * 路由编排（FSD app 层）。
 * 使用 hash 路由（Tauri 桌面环境无服务器历史路径，hash 更稳妥）。
 * M2 起补充组织层三页（分类/标签/项目），并在提示词库页组合批量工具栏。
 */
export const router = createHashRouter([
  {
    path: "/",
    element: <AppLayout />,
    children: [
      { index: true, element: <Navigate to="/prompts" replace /> },
      {
        path: "prompts",
        element: <PromptLibraryPage batchToolbar={<BatchToolbar />} />,
      },
      {
        path: "prompts/:id",
        element: (
          <PromptEditorPage
            templatePanel={<TemplateUsePanel />}
            versionPanel={<VersionHistoryPanel />}
          />
        ),
      },
      { path: "categories", element: <CategoryManager /> },
      { path: "tags", element: <TagManager /> },
      { path: "projects", element: <ProjectManager /> },
      { path: "sync", element: <SyncPanel /> },
      { path: "settings", element: <SettingsPage /> },
    ],
  },
]);