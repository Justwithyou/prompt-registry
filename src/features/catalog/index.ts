/**
 * catalog 功能切片：分类（树形分类，支持拖拽排序）。
 * 对应 M2 组织层的分类功能域；层级标签由 tag-management 切片承载。
 *
 * FSD 约束：其他 feature 只 import 本文件，不得直接 import 内部文件。
 */
export { useCategoryStore } from "./stores/categoryStore";
export type { CategoryStore } from "./stores/categoryStore";
export { CategoryManager } from "./ui/CategoryManager";