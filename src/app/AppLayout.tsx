import { useEffect, useState } from "react";
import { NavLink, Outlet, useLocation } from "react-router";
import { SearchBar } from "../features/search/index";
import { useSettingsStore } from "../features/settings/index";
import { usePromptStore } from "../features/prompt-library/index";
import { useProjectStore } from "../features/project-management/index";
import { useCategoryStore } from "../features/catalog/index";
import { useTagStore } from "../features/tag-management/index";
import { Icon } from "../shared/ui/Icon";
import { useEscapeKey } from "../shared/hooks/useEscapeKey";
import { TitleBar } from "./TitleBar";
import { ToastContainer } from "../shared/ui/ToastContainer";
import { loadImportedFont, resolveFontFamily } from "../shared/lib/fonts";

/**
 * 应用外壳布局（app 层）。
 * 顶部标题栏（帮助/关于 + 窗口控制）+ 左侧导航
 * （含库/组织/系统分组导航）+ 顶部搜索 + 内容区。
 * 搜索框与列表页分属不同 feature，此处由 app 层组合，符合 FSD 依赖方向（app → features）。
 * M2-I1 起在挂载时预加载四功能域真实数据（提示词/项目/分类/标签）。
 */
export function AppLayout() {
  const location = useLocation();
  const theme = useSettingsStore((s) => s.theme);
  const sidebarCollapsed = useSettingsStore((s) => s.sidebarCollapsed);
  const toggleSidebar = useSettingsStore((s) => s.toggleSidebar);
  const fontKey = useSettingsStore((s) => s.fontKey);
  const [helpOpen, setHelpOpen] = useState(false);
  const [aboutOpen, setAboutOpen] = useState(false);
  const [systemDark, setSystemDark] = useState(
    () => window.matchMedia("(prefers-color-scheme: dark)").matches,
  );

  useEscapeKey(() => {
    setHelpOpen(false);
    setAboutOpen(false);
  });

  const fetchPrompts = usePromptStore((s) => s.fetchPrompts);
  const fetchProjects = useProjectStore((s) => s.fetchProjects);
  const fetchCategories = useCategoryStore((s) => s.fetchCategories);
  const fetchTags = useTagStore((s) => s.fetchTags);

  useEffect(() => {
    const resolved = theme === "system" ? (systemDark ? "dark" : "light") : theme;
    document.documentElement.setAttribute("data-theme", resolved);
  }, [theme, systemDark]);

  useEffect(() => {
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const onChange = (e: MediaQueryListEvent) => setSystemDark(e.matches);
    mq.addEventListener("change", onChange);
    return () => mq.removeEventListener("change", onChange);
  }, []);

  useEffect(() => {
    document.documentElement.style.setProperty(
      "--font-family",
      resolveFontFamily(fontKey),
    );
  }, [fontKey]);

  useEffect(() => {
    void loadImportedFont();
  }, []);

  useEffect(() => {
    void fetchPrompts();
    void fetchProjects();
    void fetchCategories();
    void fetchTags();
  }, [fetchPrompts, fetchProjects, fetchCategories, fetchTags]);

  const showSearch = location.pathname === "/prompts";

  const navGroups = [
    {
      title: "库",
      items: [
        { to: "/prompts", label: "提示词库", icon: "file-text" as const, end: true },
      ],
    },
    {
      title: "组织",
      items: [
        { to: "/categories", label: "分类", icon: "folder" as const, end: false },
        { to: "/tags", label: "标签", icon: "tag" as const, end: false },
        { to: "/projects", label: "项目", icon: "briefcase" as const, end: false },
      ],
    },
    {
      title: "系统",
      items: [
        { to: "/sync", label: "同步", icon: "sync" as const, end: false },
        { to: "/settings", label: "设置", icon: "settings" as const, end: false },
      ],
    },
  ];

  return (
    <div className="app-root">
      <TitleBar onHelp={() => setHelpOpen(true)} onAbout={() => setAboutOpen(true)} />
      <div className="app-shell">
        <aside className={`app-sidebar${sidebarCollapsed ? " is-collapsed" : ""}`}>
          <div className="app-brand">
            <img
              className="app-brand-logo"
              src="/logo.svg"
              alt="提示词管理平台 Logo"
            />
            {!sidebarCollapsed && <span className="app-brand-name">提示词管理平台</span>}
          </div>

          <nav className="app-nav">
            {navGroups.map((group) => (
              <div key={group.title} className="nav-group">
                {!sidebarCollapsed && (
                  <span className="nav-group-title">{group.title}</span>
                )}
                {group.items.map((item) => (
                  <NavLink
                    key={item.to}
                    to={item.to}
                    end={item.end}
                    title={item.label}
                    className={({ isActive }) => `nav-link${isActive ? " is-active" : ""}`}
                  >
                    <Icon name={item.icon} size={18} />
                    {!sidebarCollapsed && <span className="nav-label">{item.label}</span>}
                  </NavLink>
                ))}
              </div>
            ))}
          </nav>

          <button
            type="button"
            className="sidebar-toggle"
            onClick={toggleSidebar}
            title={sidebarCollapsed ? "展开侧栏" : "收起侧栏"}
          >
            <Icon
              name={sidebarCollapsed ? "panel-left-open" : "panel-left-close"}
              size={18}
            />
          </button>
        </aside>

        <div className="app-content">
          {showSearch && (
            <div className="app-toolbar">
              <SearchBar />
            </div>
          )}
          <main className="app-page">
            <Outlet />
          </main>
        </div>

        {aboutOpen && (
          <div className="editor-overlay" onClick={() => setAboutOpen(false)}>
            <div
              className="editor-overlay-card app-about-card"
              onClick={(e) => e.stopPropagation()}
            >
              <button
                type="button"
                className="overlay-close"
                onClick={() => setAboutOpen(false)}
                title="关闭"
              >
                <Icon name="x" size={18} />
              </button>
              <div className="app-about-body">
                <img
                  className="app-about-logo"
                  src="/logo.svg"
                  alt="提示词管理平台 Logo"
                />
                <h3 className="panel-title">提示词管理平台</h3>
                <p className="panel-subtitle">v1.0 正式版</p>
                <p className="app-about-desc">
                  一款本地优先、注重隐私的个人提示词库管理工具，支持分类、标签、项目组织与 AI 辅助。
                </p>
              </div>
            </div>
          </div>
        )}

        {helpOpen && (
          <div className="editor-overlay" onClick={() => setHelpOpen(false)}>
            <div
              className="editor-overlay-card"
              style={{ width: "min(640px, 92%)" }}
              onClick={(e) => e.stopPropagation()}
            >
              <button
                type="button"
                className="overlay-close"
                onClick={() => setHelpOpen(false)}
                title="关闭"
              >
                <Icon name="x" size={18} />
              </button>
              <h3 className="panel-title">
                <Icon name="help" size={18} />
                使用帮助
              </h3>
              <ul className="app-help-list">
                <li>在「提示词库」创建、编辑、搜索并收藏你的提示词。</li>
                <li>用「分类」「标签」「项目」组织管理提示词。</li>
                <li>在「设置」中切换主题、字体与存储位置，并配置 AI 辅助。</li>
                <li>数据保存在本地，可随时在「设置」中导出或导入备份。</li>
              </ul>
            </div>
          </div>
        )}

        <ToastContainer />
      </div>
    </div>
  );
}