/**
 * 同步领域模型（前端类型）。
 *
 * 契约单一事实源在 `shared/api/commands.ts`（对应架构方案 3.5）。本文件仅做再导出，
 * 供 feature 切片通过 entities 层引用实体类型，避免 shared 与 entities 出现两份定义。
 */
export type { SyncConfig, SyncResult, SyncStatus } from "../../shared/api/commands";