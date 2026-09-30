/**
 * sync 功能切片：同步配置与状态。
 * 对应架构方案 3.2 的 syncStore。
 *
 * 公开 API：SyncPanel（M4-E4 WebDAV 配置/同步 UI）、useSyncStore（M4-E2 同步状态）。
 */
export { SyncPanel } from "./ui/SyncPanel";
export { useSyncStore } from "./stores/syncStore";
export type { SyncStore } from "./stores/syncStore";