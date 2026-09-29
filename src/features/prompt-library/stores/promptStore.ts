import { create } from "zustand";
import type {
  Prompt,
  PromptQuery,
  CreatePromptInput,
  UpdatePromptInput,
} from "../../../entities/prompt/types";
import * as promptApi from "../api";

/**
 * 提示词库状态（Zustand store）。
 * 对应架构方案 3.2 的 promptStore：列表、当前项、筛选条件、加载态，
 * 以及增删改查 / 搜索 action（M1-C2）。
 */
export interface PromptStore {
  prompts: Prompt[];
  selectedPromptId: string | null;
  filters: PromptQuery;
  isLoading: boolean;

  fetchPrompts: (filters?: PromptQuery) => Promise<void>;
  searchPrompts: (keyword: string) => Promise<void>;
  selectPrompt: (id: string | null) => void;
  createPrompt: (data: CreatePromptInput) => Promise<Prompt>;
  updatePrompt: (id: string, data: UpdatePromptInput) => Promise<void>;
  deletePrompt: (id: string) => Promise<void>;
  clearAll: () => Promise<void>;
}

/**
 * 客户端过滤：仅处理 M1 后端契约未覆盖的分类 / 置顶筛选。
 * keyword 由独立的 FTS 命令 `searchPrompts` 处理，不透传后端 list 查询。
 */
function applyFilters(list: Prompt[], filters: PromptQuery): Prompt[] {
  let out = list;
  if (filters.category) {
    out = out.filter((p) => p.category === filters.category);
  }
  if (filters.isPinned != null) {
    out = out.filter((p) => p.isPinned === filters.isPinned);
  }
  return out;
}

export const usePromptStore = create<PromptStore>()((set, get) => ({
  prompts: [],
  selectedPromptId: null,
  filters: {},
  isLoading: false,

  async fetchPrompts(filters) {
    const next = { ...get().filters, ...(filters ?? {}) };
    set({ filters: next, isLoading: true });
    try {
      const all = await promptApi.getPrompts({});
      set({ prompts: applyFilters(all, next) });
    } finally {
      set({ isLoading: false });
    }
  },

  async searchPrompts(keyword) {
    set({ isLoading: true });
    try {
      const prompts = await promptApi.searchPrompts(keyword);
      set({ prompts });
    } finally {
      set({ isLoading: false });
    }
  },

  selectPrompt(id) {
    set({ selectedPromptId: id });
  },

  async createPrompt(data) {
    const created = await promptApi.createPrompt(data);
    set((state) => ({ prompts: [created, ...state.prompts] }));
    return created;
  },

  async updatePrompt(id, data) {
    const updated = await promptApi.updatePrompt(id, data);
    set((state) => ({
      prompts: state.prompts.map((p) => (p.id === id ? updated : p)),
    }));
  },

  async deletePrompt(id) {
    await promptApi.deletePrompt(id);
    set((state) => ({
      prompts: state.prompts.filter((p) => p.id !== id),
      selectedPromptId:
        state.selectedPromptId === id ? null : state.selectedPromptId,
    }));
  },

  async clearAll() {
    const all = await promptApi.getPrompts({});
    await Promise.all(all.map((p) => promptApi.deletePrompt(p.id)));
    set({ prompts: [], selectedPromptId: null, filters: {} });
  },
}));