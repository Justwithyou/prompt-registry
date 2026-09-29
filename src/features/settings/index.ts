/**
 * settings 功能切片：设置。
 * 对应架构方案 3.2 的 settingsStore。
 *
 * 公开 API：SettingsPage（供 app 层路由使用）。
 */
export { SettingsPage } from "./ui/SettingsPage";
export { useSettingsStore } from "./stores/settingsStore";
export type { Theme } from "./stores/settingsStore";