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

/** 某条提示词当前的组织关系（读取用） */
export interface PromptRelations {
  projectIds: string[];
  tagIds: string[];
  categoryId: string | null;
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

/** 读取某条提示词当前的组织关系（项目 / 标签 / 分类） */
export async function getPromptRelations(
  id: string,
): Promise<PromptRelations> {
  return invoke<PromptRelations>("get_prompt_relations", { id });
}

/** 批量操作提示词 */
export async function batchUpdatePrompts(
  ids: string[],
  batch: BatchUpdateInput,
): Promise<void> {
  return invoke<void>("batch_update_prompts", { ids, batch });
}

// ===== 模板与版本模型类型（M3 冻结契约，与 crates/core/src/models/ 对齐）=====

/** 模板变量定义（`extract_variables` 产物，对应 prompts 模板 `{{变量}}`） */
export interface VariableDefinition {
  name: string;
  defaultValue: string | null;
  options: string[] | null;
  description: string | null;
}

/** 提示词历史版本实体（对应 `prompt_versions` 表） */
export interface PromptVersion {
  id: string;
  promptId: string;
  /** 语义化版本号 `主.次.补丁`（默认 `0.1.0` 起）。 */
  version: string;
  content: string;
  changeSummary: string | null;
  createdAt: string;
}

// ===== 模板与版本命令（M3 冻结契约）=====

/** 从模板正文提取 `{{变量名}}` 列表 */
export async function extractVariables(
  content: string,
): Promise<VariableDefinition[]> {
  return invoke<VariableDefinition[]>("extract_variables", { content });
}

/** 列出某提示词的全部历史版本 */
export async function listPromptVersions(
  promptId: string,
): Promise<PromptVersion[]> {
  return invoke<PromptVersion[]>("list_prompt_versions", { promptId });
}

/** 获取单个版本详情 */
export async function getPromptVersion(
  versionId: string,
): Promise<PromptVersion> {
  return invoke<PromptVersion>("get_prompt_version", { versionId });
}

/** 回滚到指定历史版本（追加式，生成新版本） */
export async function rollbackPromptVersion(
  promptId: string,
  versionId: string,
): Promise<Prompt> {
  return invoke<Prompt>("rollback_prompt_version", { promptId, versionId });
}

// ===== 密钥与同步模型类型（M4 冻结契约，与 crates/core/src/models/ 对齐）=====

/** WebDAV 同步配置（`configure_sync` 入参） */
export interface SyncConfig {
  url: string;
  username: string;
  /** 可选令牌；缺省时回退 Basic Auth。 */
  token?: string | null;
  enabled: boolean;
}

/** 同步状态（`get_sync_status` 返回） */
export interface SyncStatus {
  lastSyncAt: string | null;
  pendingChanges: number;
  deviceId: string;
}

/** 单次同步结果（`sync_now` 返回） */
export interface SyncResult {
  pushed: number;
  pulled: number;
  merged: number;
}

// ===== 密钥与同步命令（M4 冻结契约）=====

/** 首次设置主口令（数据库升级为 SQLCipher 加密） */
export async function setupVault(password: string): Promise<void> {
  return invoke<void>("setup_vault", { password });
}

/** 解锁保险库 */
export async function unlockVault(password: string): Promise<void> {
  return invoke<void>("unlock_vault", { password });
}

/** 锁定保险库 */
export async function lockVault(): Promise<void> {
  return invoke<void>("lock_vault");
}

/** 修改主口令（仅重新包装 Master Key） */
export async function changePassword(
  oldPw: string,
  newPw: string,
): Promise<void> {
  return invoke<void>("change_password", { oldPw, newPw });
}

/** 是否处于锁定态 */
export async function isLocked(): Promise<boolean> {
  return invoke<boolean>("is_locked");
}

/** 配置 WebDAV 同步目标 */
export async function configureSync(config: SyncConfig): Promise<void> {
  return invoke<void>("configure_sync", { config });
}

/** 立即执行一次同步 */
export async function syncNow(): Promise<SyncResult> {
  return invoke<SyncResult>("sync_now");
}

/** 查询当前同步状态 */
export async function getSyncStatus(): Promise<SyncStatus> {
  return invoke<SyncStatus>("get_sync_status");
}

// ===== 连通性探针（M0）=====

export async function ping(): Promise<string> {
  return invoke<string>("ping");
}

// ===== 导出命令（M6 CLI/Skills 输出能力，与 crates/export 及 ipc/commands/export 对齐）=====

/** 导出格式：SKILL.md / Markdown / JSON。 */
export type ExportFormat = "skill" | "markdown" | "json";

/**
 * 导出选择器入参（对应 Rust `ExportSelectorInput`）。
 * 优先级：`ids` > `categoryId` > `projectId` > `tagIds` > 全部。
 */
export interface ExportSelectorInput {
  /** 按提示词 ID 导出（单条/多条）。 */
  ids?: string[];
  /** 按分类导出。 */
  categoryId?: string | null;
  /** 按标签导出。 */
  tagIds?: string[];
  /** 按项目导出。 */
  projectId?: string | null;
}

/** 渲染单条模板提示词为成品（变量未被填值时保留 `{{name}}` 占位）。 */
export async function renderPrompt(
  id: string,
  vars: Record<string, string>,
): Promise<string> {
  return invoke<string>("render_prompt", { id, vars });
}

/** 按选择器导出提示词到目录，返回写入的文件路径列表。 */
export async function exportPrompts(
  selector: ExportSelectorInput,
  format: ExportFormat,
  outDir: string,
  render: boolean,
  vars?: Record<string, string>,
): Promise<string[]> {
  return invoke<string[]>("export_prompts", {
    selector,
    format,
    outDir,
    render,
    vars: vars ?? {},
  });
}

/** 单条导出为 SKILL.md 目录，返回 SKILL.md 文件路径。 */
export async function exportSkill(
  id: string,
  outDir: string,
): Promise<string> {
  return invoke<string>("export_skill", { id, outDir });
}