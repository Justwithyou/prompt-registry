import { create } from "zustand";
import type { PromptVersion } from "../../../entities/version/types";
import type { Prompt } from "../../../entities/prompt/types";
import * as versionApi from "../api";

/**
 * 版本控制状态（Zustand store），对应架构方案 3.2 的 versionStore。
 * 职责：版本列表、Diff 目标版本（勾选两个），以及回滚（M3-C2）。
 * 词级 Diff 计算由前端 `diff` 库完成（M3 决策点 3.1），store 只保存勾选的两个版本 ID。
 */
export interface VersionStore {
  versions: PromptVersion[];
  currentPromptId: string | null;
  /** 用于 Diff 并排对比的两个版本 id（最多 2 个，勾选）。 */
  diffSelection: string[];
  isLoading: boolean;

  fetchVersions: (promptId: string) => Promise<void>;
  toggleDiffSelection: (versionId: string) => void;
  getVersion: (versionId: string) => Promise<PromptVersion>;
  rollback: (promptId: string, versionId: string) => Promise<Prompt>;
}

export const useVersionStore = create<VersionStore>()((set, get) => ({
  versions: [],
  currentPromptId: null,
  diffSelection: [],
  isLoading: false,

  async fetchVersions(promptId) {
    set({ currentPromptId: promptId, isLoading: true });
    try {
      const versions = await versionApi.listPromptVersions(promptId);
      set({ versions });
    } finally {
      set({ isLoading: false });
    }
  },

  toggleDiffSelection(versionId) {
    set((s) => {
      if (s.diffSelection.includes(versionId)) {
        return {
          diffSelection: s.diffSelection.filter((id) => id !== versionId),
        };
      }
      const next = [...s.diffSelection, versionId];
      while (next.length > 2) next.shift();
      return { diffSelection: next };
    });
  },

  getVersion(versionId) {
    return versionApi.getPromptVersion(versionId);
  },

  async rollback(promptId, versionId) {
    const updated = await versionApi.rollbackPromptVersion(promptId, versionId);
    // 回滚是追加式（生成一条新版本），刷新版本列表。
    await get().fetchVersions(promptId);
    return updated;
  },
}));