/**
 * settings 功能切片：设置。
 * 对应架构方案 3.2 的 settingsStore 与 vaultStore。
 *
 * 公开 API：SettingsPage（供 app 层路由使用）、VaultPanel（M4-E3 口令/锁定 UI）、
 * useVaultStore（M4-E2 保险库状态）。
 */
export { SettingsPage } from "./ui/SettingsPage";
export { VaultPanel } from "./ui/VaultPanel";
export { useSettingsStore } from "./stores/settingsStore";
export type { Theme } from "./stores/settingsStore";
export { useVaultStore } from "./stores/vaultStore";
export type { VaultStore } from "./stores/vaultStore";