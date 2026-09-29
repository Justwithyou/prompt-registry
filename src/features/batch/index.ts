/**
 * batch 功能切片：批量操作（多选 + 批量打标签 / 移动项目 / 删除）。
 * 对应 M2 组织层 §3.1「批量操作范围」与架构方案 3.2 的批处理边界。
 * 内部结构：components/ hooks/ stores/ api/ types/
 */
export { BatchToolbar } from "./ui/BatchToolbar";