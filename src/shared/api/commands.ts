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
  /** M2 树形分类外键（指向 categories.id，供「按分类筛选」）。 */
  categoryId: string | null;
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

/** 列表/搜索查询条件 */
export interface PromptQuery {
  keyword?: string;
  category?: string;
  isPinned?: boolean;
  limit?: number;
  offset?: number;
  // —— M2 组织层新增：多维联合筛选 ——
  projectId?: string;
  tagIds?: string[];
  categoryId?: string;
  isFavorite?: boolean;
  sortBy?: PromptSortBy;
}

/** 列表排序字段 */
export type PromptSortBy = "updatedAt" | "rating" | "createdAt";

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

// ===== 组织层模型类型（M2 冻结契约，与 crates/core/src/models/ 对齐）=====

/** 项目实体（对应 projects 表） */
export interface Project {
  id: string;
  name: string;
  description: string | null;
  color: string | null;
  createdAt: string;
  updatedAt: string;
}

/** 创建项目入参 */
export interface CreateProjectInput {
  name: string;
  description?: string | null;
  color?: string | null;
}

/** 更新项目入参 */
export interface UpdateProjectInput {
  name?: string;
  description?: string | null;
  color?: string | null;
}

/** 标签实体（支持层级） */
export interface Tag {
  id: string;
  name: string;
  parentId: string | null;
  color: string | null;
  createdAt: string;
}

/** 创建标签入参 */
export interface CreateTagInput {
  name: string;
  parentId?: string | null;
  color?: string | null;
}

/** 更新标签入参 */
export interface UpdateTagInput {
  name?: string;
  parentId?: string | null;
  color?: string | null;
}

/** 分类实体（树形） */
export interface Category {
  id: string;
  name: string;
  parentId: string | null;
  sortOrder: number;
  createdAt: string;
}

/** 创建分类入参 */
export interface CreateCategoryInput {
  name: string;
  parentId?: string | null;
  sortOrder?: number;
}

/** 更新分类入参 */
export interface UpdateCategoryInput {
  name?: string;
  parentId?: string | null;
  sortOrder?: number;
}

/** 一次性更新提示词组织关系与快捷标记 */
export interface PromptRelationsInput {
  projectIds?: string[];
  tagIds?: string[];
  /** 分配到树形分类（设置时覆盖 categoryId）。 */
  categoryId?: string;
  isPinned?: boolean;
  isFavorite?: boolean;
  rating?: number;
}

/** 批量操作入参 */
export interface BatchUpdateInput {
  addTagIds?: string[];
  removeTagIds?: string[];
  setProjectId?: string;
  delete?: boolean;
}

// ===== 组织层命令（M2 冻结契约）=====

/** 获取全部项目 */
export async function listProjects(): Promise<Project[]> {
  return invoke<Project[]>("list_projects");
}

/** 创建项目 */
export async function createProject(input: CreateProjectInput): Promise<Project> {
  return invoke<Project>("create_project", { input });
}

/** 更新项目 */
export async function updateProject(
  id: string,
  input: UpdateProjectInput,
): Promise<Project> {
  return invoke<Project>("update_project", { id, input });
}

/** 删除项目 */
export async function deleteProject(id: string): Promise<void> {
  return invoke<void>("delete_project", { id });
}

/** 获取全部标签 */
export async function listTags(): Promise<Tag[]> {
  return invoke<Tag[]>("list_tags");
}

/** 创建标签 */
export async function createTag(input: CreateTagInput): Promise<Tag> {
  return invoke<Tag>("create_tag", { input });
}

/** 更新标签 */
export async function updateTag(id: string, input: UpdateTagInput): Promise<Tag> {
  return invoke<Tag>("update_tag", { id, input });
}

/** 删除标签 */
export async function deleteTag(id: string): Promise<void> {
  return invoke<void>("delete_tag", { id });
}

/** 获取全部分类 */
export async function listCategories(): Promise<Category[]> {
  return invoke<Category[]>("list_categories");
}

/** 创建分类 */
export async function createCategory(
  input: CreateCategoryInput,
): Promise<Category> {
  return invoke<Category>("create_category", { input });
}

/** 更新分类 */
export async function updateCategory(
  id: string,
  input: UpdateCategoryInput,
): Promise<Category> {
  return invoke<Category>("update_category", { id, input });
}

/** 删除分类 */
export async function deleteCategory(id: string): Promise<void> {
  return invoke<void>("delete_category", { id });
}

/** 一次性更新提示词组织关系与快捷标记 */
export async function setPromptRelations(
  id: string,
  relations: PromptRelationsInput,
): Promise<Prompt> {
  return invoke<Prompt>("set_prompt_relations", { id, relations });
}

/** 批量操作提示词 */
export async function batchUpdatePrompts(
  ids: string[],
  batch: BatchUpdateInput,
): Promise<void> {
  return invoke<void>("batch_update_prompts", { ids, batch });
}

// ===== 连通性探针（M0）=====

export async function ping(): Promise<string> {
  return invoke<string>("ping");
}

// ===== 待接入命令契约（按里程碑逐步开放）=====
// 模板：  render_template / extract_variables
// 版本：  get_versions / diff_versions / rollback_version
// 同步：  sync_now / get_sync_status
// 安全：  unlock_vault / lock_vault / change_master_password