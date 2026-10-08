/**
 * 提示词领域模型（前端类型）。
 *
 * 契约单一事实源在 `shared/api/commands.ts`（对应架构方案 3.5「所有命令通过统一的
 * 类型定义管理」）。本文件仅做再导出，供 feature 切片通过 entities 层引用实体类型，
 * 避免在 shared 与 entities 之间出现两份不同的类型定义。
 */
export type {
  Prompt,
  PromptQuery,
  PromptSortBy,
  CreatePromptInput,
  UpdatePromptInput,
  PromptRelationsInput,
  PromptRelations,
  BatchUpdateInput,
} from "../../shared/api/commands";