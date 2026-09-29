/**
 * tag-management 功能切片：标签管理。
 * 对应架构方案 3.2 的 tagStore。
 * 内部结构：components/ hooks/ stores/ api/ types/
 */
export { useTagStore } from "./stores/tagStore";
export type { TagStore } from "./stores/tagStore";
export { TagManager } from "./ui/TagManager";
export { TagCloud } from "./ui/TagCloud";