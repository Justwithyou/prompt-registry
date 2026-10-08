import { create } from "zustand";
import { DEFAULT_FONT_KEY, IMPORTED_FONT_KEY } from "../../../shared/lib/fonts";

/**
 * 设置切片状态（settings/stores）。
 * 对应架构方案 3.2 的 settingsStore：用户偏好（主题、存储位置）与版本信息。
 */

/** M5：主题三态（跟随系统 / 亮 / 暗），默认跟随系统（系统解析由 M5-B1 实现）。 */
export type Theme = "system" | "dark" | "light";

/** 解析后的实际主题：system 下由 prefers-color-scheme 解析为 dark/light。 */
export type ResolvedTheme = "dark" | "light";

export interface SettingsStore {
  theme: Theme;
  storageLocation: string;
  appVersion: string;
  sidebarCollapsed: boolean;
  fontKey: string;
  importedFontName: string | null;
  setTheme: (theme: Theme) => void;
  toggleTheme: () => void;
  setStorageLocation: (path: string) => void;
  toggleSidebar: () => void;
  setFontKey: (key: string) => void;
  setImportedFont: (name: string) => void;
}

const THEME_KEY = "prompt-manager.theme";
const SIDEBAR_KEY = "prompt-manager.sidebar-collapsed";
const FONT_KEY = "prompt-manager.font-key";
const IMPORTED_FONT_NAME_KEY = "prompt-manager.imported-font-name";

function initialTheme(): Theme {
  const saved = localStorage.getItem(THEME_KEY);
  return saved === "light" || saved === "dark" ? saved : "dark";
}

function initialSidebarCollapsed(): boolean {
  return localStorage.getItem(SIDEBAR_KEY) === "1";
}

function initialFontKey(): string {
  const saved = localStorage.getItem(FONT_KEY);
  return saved ?? DEFAULT_FONT_KEY;
}

export const useSettingsStore = create<SettingsStore>()((set, get) => ({
  theme: initialTheme(),
  storageLocation: "C:\\Users\\<user>\\AppData\\Roaming\\prompt-manager\\vault.db",
  appVersion: "1.0.0",
  sidebarCollapsed: initialSidebarCollapsed(),
  fontKey: initialFontKey(),
  importedFontName: localStorage.getItem(IMPORTED_FONT_NAME_KEY),

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

  setFontKey(key) {
    set({ fontKey: key });
    localStorage.setItem(FONT_KEY, key);
  },

  setImportedFont(name) {
    set({ fontKey: IMPORTED_FONT_KEY, importedFontName: name });
    localStorage.setItem(FONT_KEY, IMPORTED_FONT_KEY);
    localStorage.setItem(IMPORTED_FONT_NAME_KEY, name);
  },
}));