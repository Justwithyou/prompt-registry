/**
 * version-control 功能切片：版本对比与回滚。
 * 对应架构方案 3.2 的 versionStore。
 *
 * 公开 API：versionStore（M3 C2）、VersionHistoryPanel（M3 D2 时间轴）、
 * DiffViewer（M3 D3 并排对比），供 app 层组合注入编辑器。
 */
export { useVersionStore } from "./stores/versionStore";
export type { VersionStore } from "./stores/versionStore";
export { VersionHistoryPanel } from "./ui/VersionHistoryPanel";
export { DiffViewer } from "./ui/DiffViewer";