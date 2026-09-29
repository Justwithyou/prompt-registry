/**
 * search 功能切片 —— 公开 API 出口。
 *
 * FSD 约束：其他 feature 只能通过本文件 import，不得直接 import 内部文件。
 * 导出：
 *  - SearchBar：搜索框（联动提示词列表）
 *  - CopyButton：一键复制「原始」入口（供编辑器等复用）
 */
export { SearchBar } from "./ui/SearchBar";
export { CopyButton } from "./ui/CopyButton";