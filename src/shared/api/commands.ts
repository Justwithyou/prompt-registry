import { invoke } from "@tauri-apps/api/core";

/**
 * IPC 契约 —— 前端单一事实源（对应架构方案 3.5）。
 * 命令名与 src-tauri/crates/ipc/src/commands/ 下的 #[tauri::command] 一一对应。
 * 变更契约时：先改本文件 + Rust 结构体，再分别实现两端（「契约先行」）。
 *
 * 字段命名契约：Rust 侧统一 `rename_all = "camelCase"`，与以下 camelCase 类型一致。
 */

// ===== 领域模型类型（与 crates/core/src/models/prompt.rs 对齐）=====

/** 提示词实体（对应 prompts 表） */
export interface Prompt {
  id: string;
  title: string;
  content: string;
  description: string | null;
  category: string | null;
  isPinned: boolean;
  isFavorite: boolean;
  rating: number;
  createdAt: string;
  updatedAt: string;
  deletedAt: string | null;
}

/** 创建提示词入参（title/content 必填，其余可选） */
export interface CreatePromptInput {
  title: string;
  content: string;
  description?: string | null;
  category?: string | null;
}

/** 更新提示词入参（字段级局部更新，缺省表示不修改） */
export interface UpdatePromptInput {
  title?: string;
  content?: string;
  description?: string | null;
  category?: string | null;
  isPinned?: boolean;
  isFavorite?: boolean;
  rating?: number;
}

/** 列表/搜索查询条件（M1 仅支持 keyword + limit，其余留空） */
export interface PromptQuery {
  keyword?: string;
  category?: string;
  isPinned?: boolean;
  limit?: number;
  offset?: number;
}

// ===== 提示词管理命令（M1 冻结契约）=====

/** 获取提示词列表 */
export async function getPrompts(query: PromptQuery): Promise<Prompt[]> {
  return invoke<Prompt[]>("get_prompts", { query });
}

/** 创建提示词 */
export async function createPrompt(input: CreatePromptInput): Promise<Prompt> {
  return invoke<Prompt>("create_prompt", { input });
}

/** 更新提示词（字段级局部更新） */
export async function updatePrompt(
  id: string,
  input: UpdatePromptInput,
): Promise<Prompt> {
  return invoke<Prompt>("update_prompt", { id, input });
}

/** 删除提示词（软删除） */
export async function deletePrompt(id: string): Promise<void> {
  return invoke<void>("delete_prompt", { id });
}

/** 全文搜索提示词 */
export async function searchPrompts(keyword: string): Promise<Prompt[]> {
  return invoke<Prompt[]>("search_prompts", { keyword });
}

// ===== 连通性探针（M0）=====

export async function ping(): Promise<string> {
  return invoke<string>("ping");
}

// ===== 待接入命令契约（按里程碑逐步开放）=====
// 组织：  get_projects / get_tags ...
// 模板：  render_template / extract_variables
// 版本：  get_versions / diff_versions / rollback_version
// 同步：  sync_now / get_sync_status
// 安全：  unlock_vault / lock_vault / change_master_password