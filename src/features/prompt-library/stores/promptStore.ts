import { create } from "zustand";
import type {
  Prompt,
  PromptQuery,
  CreatePromptInput,
  UpdatePromptInput,
  PromptRelationsInput,
  PromptRelations,
  BatchUpdateInput,
} from "../../../entities/prompt/types";
import * as promptApi from "../api";
import { useVersionStore } from "../../version-control";

/**
 * 提示词库状态（Zustand store）。
 * 对应架构方案 3.2 的 promptStore：列表、当前项、筛选条件、加载态，
 * 以及增删改查 / 搜索 / 关系快捷更新 / 批量操作 action（M1-C2 + M2-C2）。
 */
export interface PromptStore {
  prompts: Prompt[];
  selectedPromptId: string | null;
  filters: PromptQuery;
  isLoading: boolean;
  selectionMode: boolean;
  selectedIds: string[];

  fetchPrompts: (filters?: PromptQuery) => Promise<void>;
  searchPrompts: (keyword: string) => Promise<void>;
  selectPrompt: (id: string | null) => void;
  createPrompt: (data: CreatePromptInput) => Promise<Prompt>;
  updatePrompt: (id: string, data: UpdatePromptInput) => Promise<void>;
  deletePrompt: (id: string) => Promise<void>;
  setPromptRelations: (
    id: string,
    relations: PromptRelationsInput,
  ) => Promise<void>;
  getPromptRelations: (id: string) => Promise<PromptRelations>;
  batchUpdatePrompts: (
    ids: string[],
    batch: BatchUpdateInput,
  ) => Promise<void>;
  clearAll: () => Promise<void>;
  setFilters: (patch: Partial<PromptQuery>) => Promise<void>;
  resetFilters: () => Promise<void>;
  setSelectionMode: (on: boolean) => void;
  toggleSelected: (id: string) => void;
  clearSelection: () => void;
}

export const usePromptStore = create<PromptStore>()((set, get) => ({
  prompts: [],
  selectedPromptId: null,
  filters: {},
  isLoading: false,
  selectionMode: false,
  selectedIds: [],

  // M2 起：把完整组合筛选（关键词/分类/标签/项目/收藏/置顶/排序）透传给后端 SQL 组合查询。
  async fetchPrompts(filters) {
    const next = { ...get().filters, ...(filters ?? {}) };
    set({ filters: next, isLoading: true });
    try {
      const prompts = await promptApi.getPrompts(next);
      set({ prompts });
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
    // M3：防抖保存成功后，后端已写入一条自动版本记录，异步刷新该提示词的版本列表。
    void useVersionStore.getState().fetchVersions(id);
  },

  async deletePrompt(id) {
    await promptApi.deletePrompt(id);
    set((state) => ({
      prompts: state.prompts.filter((p) => p.id !== id),
      selectedPromptId:
        state.selectedPromptId === id ? null : state.selectedPromptId,
    }));
  },

  // 卡片快捷收藏/置顶/评分，或元数据面板设置项目/标签关联（M2-C2）。
  async setPromptRelations(id, relations) {
    const updated = await promptApi.setPromptRelations(id, relations);
    set((state) => ({
      prompts: state.prompts.map((p) => (p.id === id ? updated : p)),
    }));
  },

  // 读取某条提示词当前的组织关系（供编辑器回显分类/项目/标签）。
  async getPromptRelations(id) {
    return promptApi.getPromptRelations(id);
  },

  // 批量打标签 / 移动项目 / 删除，完成后统一重拉以与后端保持一致（M2-C2）。
  async batchUpdatePrompts(ids, batch) {
    await promptApi.batchUpdatePrompts(ids, batch);
    await get().fetchPrompts();
  },

  async clearAll() {
    const all = await promptApi.getPrompts({});
    await Promise.all(all.map((p) => promptApi.deletePrompt(p.id)));
    set({ prompts: [], selectedPromptId: null, filters: {}, selectionMode: false, selectedIds: [] });
  },

  // 合并局部筛选并重拉（M2 联合筛选走服务器端 SQL）。
  setFilters(patch) {
    return get().fetchPrompts(patch);
  },

  // 清空全部筛选条件并恢复默认列表。
  async resetFilters() {
    set({ filters: {} });
    await get().fetchPrompts();
  },

  setSelectionMode(on) {
    set((s) => ({ selectionMode: on, selectedIds: on ? s.selectedIds : [] }));
  },

  toggleSelected(id) {
    set((s) => ({
      selectedIds: s.selectedIds.includes(id)
        ? s.selectedIds.filter((x) => x !== id)
        : [...s.selectedIds, id],
    }));
  },

  clearSelection() {
    set({ selectedIds: [], selectionMode: false });
  },
}));