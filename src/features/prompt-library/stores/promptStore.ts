import { create } from "zustand";
import type {
  Prompt,
  PromptQuery,
  CreatePromptInput,
  UpdatePromptInput,
} from "../../../entities/prompt/types";

/**
 * 提示词库状态（Zustand store）。
 * 对应架构方案 3.2 的 promptStore：列表、选中状态、筛选条件。
 */
export interface PromptStore {
  prompts: Prompt[];
  selectedPromptId: string | null;
  filters: PromptQuery;
  isLoading: boolean;

  fetchPrompts: (filters?: PromptQuery) => Promise<void>;
  selectPrompt: (id: string | null) => void;
  createPrompt: (data: CreatePromptInput) => Promise<Prompt>;
  updatePrompt: (id: string, data: UpdatePromptInput) => Promise<void>;
  deletePrompt: (id: string) => Promise<void>;
}

// M0 骨架：仅落地 store 形状与初始值，方法体待 M1 接入 IPC 后实现
export const usePromptStore = create<PromptStore>()(() => ({
  prompts: [],
  selectedPromptId: null,
  filters: {},
  isLoading: false,

  async fetchPrompts(_filters) {
    // TODO(M1): 调用 shared/api/commands 获取列表并 set({ prompts, isLoading })
  },
  selectPrompt(_id) {
    // TODO(M1): set({ selectedPromptId: _id })
  },
  async createPrompt(_data): Promise<Prompt> {
    throw new Error("M1 实现 createPrompt");
  },
  async updatePrompt(_id, _data) {
    // TODO(M1)
  },
  async deletePrompt(_id) {
    // TODO(M1)
  },
}));