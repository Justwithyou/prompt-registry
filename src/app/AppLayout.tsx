import { useEffect } from "react";
import { NavLink, Outlet, useLocation, useNavigate } from "react-router";
import { SearchBar } from "../features/search/index";
import { useSettingsStore } from "../features/settings/index";
import { usePromptStore } from "../features/prompt-library/index";
import { useProjectStore } from "../features/project-management/index";
import { useCategoryStore } from "../features/catalog/index";
import { useTagStore } from "../features/tag-management/index";
import { Icon } from "../shared/ui/Icon";
import { loadImportedFont, resolveFontFamily } from "../shared/lib/fonts";

/**
 * 应用外壳布局（app 层）。
 * 左侧导航（含组织层分类/标签/项目入口 + 按项目浏览）+ 顶部搜索 + 内容区。
 * 搜索框与列表页分属不同 feature，此处由 app 层组合，符合 FSD 依赖方向（app → features）。
 * M2-I1 起在挂载时预加载四功能域真实数据（提示词/项目/分类/标签）。
 */
export function AppLayout() {
  const location = useLocation();
  const navigate = useNavigate();
  const theme = useSettingsStore((s) => s.theme);
  const sidebarCollapsed = useSettingsStore((s) => s.sidebarCollapsed);
  const toggleSidebar = useSettingsStore((s) => s.toggleSidebar);
  const fontKey = useSettingsStore((s) => s.fontKey);
  const projects = useProjectStore((s) => s.projects);
  const filters = usePromptStore((s) => s.filters);
  const setFilters = usePromptStore((s) => s.setFilters);

  const fetchPrompts = usePromptStore((s) => s.fetchPrompts);
  const fetchProjects = useProjectStore((s) => s.fetchProjects);
  const fetchCategories = useCategoryStore((s) => s.fetchCategories);
  const fetchTags = useTagStore((s) => s.fetchTags);

  useEffect(() => {
    document.documentElement.setAttribute("data-theme", theme);
  }, [theme]);

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

  function browseProject(projectId: string) {
    void setFilters({ projectId });
    navigate("/prompts");
  }

  const navItems = [
    { to: "/prompts", label: "提示词库", icon: "file-text" as const, end: true },
    { to: "/categories", label: "分类", icon: "folder" as const, end: false },
    { to: "/tags", label: "标签", icon: "tag" as const, end: false },
    { to: "/projects", label: "项目", icon: "briefcase" as const, end: false },
    { to: "/sync", label: "同步", icon: "sync" as const, end: false },
    { to: "/settings", label: "设置", icon: "settings" as const, end: false },
  ];

  return (
    <div className="app-shell">
      <aside className={`app-sidebar${sidebarCollapsed ? " is-collapsed" : ""}`}>
        <nav className="app-nav">
          {navItems.map((item) => (
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
        </nav>

        {!sidebarCollapsed && (
          <div className="app-project-list">
            <span className="app-project-heading">按项目浏览</span>
            {projects.map((p) => {
              const active = filters.projectId === p.id;
              return (
                <button
                  key={p.id}
                  type="button"
                  title={`浏览项目：${p.name}`}
                  className={`app-project-item${active ? " is-active" : ""}`}
                  onClick={() => browseProject(p.id)}
                >
                  <span className="nav-label">{p.name}</span>
                </button>
              );
            })}
          </div>
        )}

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
    </div>
  );
}