import { create } from "zustand";
import type {
  Category,
  CreateCategoryInput,
  UpdateCategoryInput,
} from "../../../entities/category/types";
import { collectCategoryDescendants } from "../../../shared/lib/organization";
import * as categoryApi from "../api";

/**
 * 分类状态（Zustand store），管理树形分类（对应 M2 组织层 §3.1 分类）。
 * 维护分类列表、当前选中分类与加载态，提供 CRUD action（M2-C2）。
 * 树形结构的组装与拖拽排序在 UI 层（Track D1）基于 `categories` 扁平列表完成。
 */
export interface CategoryStore {
  categories: Category[];
  selectedCategoryId: string | null;
  isLoading: boolean;

  fetchCategories: () => Promise<void>;
  selectCategory: (id: string | null) => void;
  createCategory: (data: CreateCategoryInput) => Promise<Category>;
  updateCategory: (id: string, data: UpdateCategoryInput) => Promise<void>;
  deleteCategory: (id: string) => Promise<void>;
  moveCategory: (draggedId: string, targetId: string) => Promise<void>;
}

export const useCategoryStore = create<CategoryStore>()((set, get) => ({
  categories: [],
  selectedCategoryId: null,
  isLoading: false,

  async fetchCategories() {
    set({ isLoading: true });
    try {
      const categories = await categoryApi.listCategories();
      set({ categories });
    } finally {
      set({ isLoading: false });
    }
  },

  selectCategory(id) {
    set({ selectedCategoryId: id });
  },

  async createCategory(data) {
    const created = await categoryApi.createCategory(data);
    set((state) => ({ categories: [...state.categories, created] }));
    return created;
  },

  async updateCategory(id, data) {
    const updated = await categoryApi.updateCategory(id, data);
    set((state) => ({
      categories: state.categories.map((c) => (c.id === id ? updated : c)),
    }));
  },

  async deleteCategory(id) {
    await categoryApi.deleteCategory(id);
    set((state) => ({
      categories: state.categories.filter((c) => c.id !== id),
      selectedCategoryId:
        state.selectedCategoryId === id ? null : state.selectedCategoryId,
    }));
  },

  // 拖拽排序：把 dragged 移到 target 同级（位置落在 target 原先位置），重排兄弟 sortOrder。
  async moveCategory(draggedId, targetId) {
    const { categories } = get();
    if (draggedId === targetId) return;
    const dragged = categories.find((c) => c.id === draggedId);
    const target = categories.find((c) => c.id === targetId);
    if (!dragged || !target) return;
    // 防止把节点移动到自己的子孙层级之下（防环）。
    if (collectCategoryDescendants(categories, draggedId).has(targetId)) return;

    const newParentId = target.parentId ?? null;
    const siblings = categories.filter((c) => c.parentId === newParentId);
    const without = siblings.filter((c) => c.id !== draggedId);
    const targetIdx = without.findIndex((c) => c.id === targetId);
    const insertIdx = targetIdx < 0 ? without.length : targetIdx;
    const reordered = [...without];
    reordered.splice(insertIdx, 0, { ...dragged, parentId: newParentId });

    for (let i = 0; i < reordered.length; i += 1) {
      await categoryApi.updateCategory(reordered[i].id, {
        parentId: newParentId,
        sortOrder: i,
      });
    }
    await get().fetchCategories();
  },
}));