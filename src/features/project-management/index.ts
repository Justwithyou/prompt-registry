/**
 * project-management 功能切片：项目管理。
 * 对应架构方案 3.2 的 projectStore。
 * 内部结构：components/ hooks/ stores/ api/ types/
 */
export { useProjectStore } from "./stores/projectStore";
export type { ProjectStore } from "./stores/projectStore";
export { ProjectManager } from "./ui/ProjectManager";