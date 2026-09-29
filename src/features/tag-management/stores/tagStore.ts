import { create } from "zustand";
import type {
  Tag,
  CreateTagInput,
  UpdateTagInput,
} from "../../../entities/tag/types";
import * as tagApi from "../api";

/**
 * 标签状态（Zustand store），对应架构方案 3.2 的 tagStore。
 * 维护层级标签列表、当前选中标签与加载态，提供 CRUD action（M2-C2）。
 */
export interface TagStore {
  tags: Tag[];
  selectedTagId: string | null;
  isLoading: boolean;

  fetchTags: () => Promise<void>;
  selectTag: (id: string | null) => void;
  createTag: (data: CreateTagInput) => Promise<Tag>;
  updateTag: (id: string, data: UpdateTagInput) => Promise<void>;
  deleteTag: (id: string) => Promise<void>;
}

export const useTagStore = create<TagStore>()((set) => ({
  tags: [],
  selectedTagId: null,
  isLoading: false,

  async fetchTags() {
    set({ isLoading: true });
    try {
      const tags = await tagApi.listTags();
      set({ tags });
    } finally {
      set({ isLoading: false });
    }
  },

  selectTag(id) {
    set({ selectedTagId: id });
  },

  async createTag(data) {
    const created = await tagApi.createTag(data);
    set((state) => ({ tags: [...state.tags, created] }));
    return created;
  },

  async updateTag(id, data) {
    const updated = await tagApi.updateTag(id, data);
    set((state) => ({
      tags: state.tags.map((t) => (t.id === id ? updated : t)),
    }));
  },

  async deleteTag(id) {
    await tagApi.deleteTag(id);
    set((state) => ({
      tags: state.tags.filter((t) => t.id !== id),
      selectedTagId: state.selectedTagId === id ? null : state.selectedTagId,
    }));
  },
}));