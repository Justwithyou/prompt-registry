import { create } from "zustand";

/**
 * 设置切片状态（settings/stores）。
 * 对应架构方案 3.2 的 settingsStore：用户偏好（主题、存储位置）与版本信息。
 */

export type Theme = "dark" | "light";

export interface SettingsStore {
  theme: Theme;
  storageLocation: string;
  appVersion: string;
  sidebarCollapsed: boolean;
  setTheme: (theme: Theme) => void;
  toggleTheme: () => void;
  setStorageLocation: (path: string) => void;
  toggleSidebar: () => void;
}

const THEME_KEY = "prompt-manager.theme";
const SIDEBAR_KEY = "prompt-manager.sidebar-collapsed";

function initialTheme(): Theme {
  const saved = localStorage.getItem(THEME_KEY);
  return saved === "light" || saved === "dark" ? saved : "dark";
}

function initialSidebarCollapsed(): boolean {
  return localStorage.getItem(SIDEBAR_KEY) === "1";
}

export const useSettingsStore = create<SettingsStore>()((set, get) => ({
  theme: initialTheme(),
  storageLocation: "C:\\Users\\<user>\\AppData\\Roaming\\prompt-manager\\vault.db",
  appVersion: "0.1.0",
  sidebarCollapsed: initialSidebarCollapsed(),

  setTheme(theme) {
    set({ theme });
    localStorage.setItem(THEME_KEY, theme);
  },

  toggleTheme() {
    get().setTheme(get().theme === "dark" ? "light" : "dark");
  },

  setStorageLocation(path) {
    set({ storageLocation: path });
  },

  toggleSidebar() {
    const collapsed = !get().sidebarCollapsed;
    set({ sidebarCollapsed: collapsed });
    localStorage.setItem(SIDEBAR_KEY, collapsed ? "1" : "0");
  },
}));