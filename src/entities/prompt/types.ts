/**
 * 提示词领域模型（前端类型，与 crates/core/src/models/prompt.rs 对齐）。
 * 属于 entities/prompt —— 跨 feature 共享的业务实体。
 */

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

/** 创建提示词入参 */
export interface CreatePromptInput {
  title: string;
  content: string;
  description?: string | null;
  category?: string | null;
}

/** 更新提示词入参（字段级局部更新） */
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
  tagIds?: string[];
  projectId?: string;
}