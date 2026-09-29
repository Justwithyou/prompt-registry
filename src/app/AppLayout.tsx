import { useEffect } from "react";
import { NavLink, Outlet, useLocation } from "react-router";
import { SearchBar } from "../features/search/index";
import { useSettingsStore } from "../features/settings/index";
import { Icon } from "../shared/ui/Icon";

/**
 * 应用外壳布局（app 层）。
 * 左侧导航 + 顶部（提示词列表页时显示搜索框）+ 内容区。
 * 搜索框与列表页分属不同 feature，此处由 app 层组合，符合 FSD 依赖方向（app → features）。
 */
export function AppLayout() {
  const location = useLocation();
  const theme = useSettingsStore((s) => s.theme);
  const sidebarCollapsed = useSettingsStore((s) => s.sidebarCollapsed);
  const toggleSidebar = useSettingsStore((s) => s.toggleSidebar);

  useEffect(() => {
    document.documentElement.setAttribute("data-theme", theme);
  }, [theme]);

  const showSearch = location.pathname === "/prompts";

  return (
    <div className="app-shell">
      <aside className={`app-sidebar${sidebarCollapsed ? " is-collapsed" : ""}`}>
        <div className="app-brand">
          <img src="/logo.svg" className="app-brand-logo" alt="提示词管理" />
          {!sidebarCollapsed && (
            <span className="app-brand-name">提示词管理</span>
          )}
        </div>
        <nav className="app-nav">
          <NavLink
            to="/prompts"
            end
            title="提示词库"
            className={({ isActive }) => `nav-link${isActive ? " is-active" : ""}`}
          >
            <Icon name="file-text" size={18} />
            {!sidebarCollapsed && <span className="nav-label">提示词库</span>}
          </NavLink>
          <NavLink
            to="/settings"
            end
            title="设置"
            className={({ isActive }) => `nav-link${isActive ? " is-active" : ""}`}
          >
            <Icon name="settings" size={18} />
            {!sidebarCollapsed && <span className="nav-label">设置</span>}
          </NavLink>
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
          {!sidebarCollapsed && <span className="nav-label">收起</span>}
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