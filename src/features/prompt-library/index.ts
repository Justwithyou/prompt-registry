/**
 * prompt-library 功能切片 —— 公开 API 出口。
 *
 * FSD 约束：
 *  - 其他 feature 只能 import 本文件，不得直接 import 内部文件。
 *  - 本切片内部结构（components/ hooks/ stores/ api/ types/）实现细节不对外暴露。
 */
export { usePromptStore } from "./stores/promptStore";
export type { PromptStore } from "./stores/promptStore";
export { PromptLibraryPage } from "./ui/PromptLibraryPage";