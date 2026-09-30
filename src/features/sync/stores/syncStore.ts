import { create } from "zustand";
import type {
  SyncConfig,
  SyncResult,
  SyncStatus,
} from "../../../entities/sync/types";
import * as syncApi from "../api";

/**
 * 同步状态（Zustand store），对应架构方案 3.2 的 syncStore。
 * 职责：WebDAV 配置、同步状态查询、一键同步（M4-E2/E4）。
 *
 * 说明：冻结契约无「读取已保存配置」命令（仅 configure_sync 写入），
 *       故 `config` 为本次会话内 configure 成功后的内存镜像，供表单回显；
 *       密码/令牌不落 localStorage，避免明文泄露。
 */
export interface SyncStore {
  status: SyncStatus | null;
  config: SyncConfig | null;
  lastResult: SyncResult | null;
  isLoading: boolean;
  error: string | null;

  fetchStatus: () => Promise<void>;
  configure: (config: SyncConfig) => Promise<void>;
  syncNow: () => Promise<void>;
}

function toError(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

export const useSyncStore = create<SyncStore>()((set) => ({
  status: null,
  config: null,
  lastResult: null,
  isLoading: false,
  error: null,

  async fetchStatus() {
    set({ isLoading: true, error: null });
    try {
      const status = await syncApi.getSyncStatus();
      set({ status });
    } catch (e) {
      set({ error: toError(e) });
    } finally {
      set({ isLoading: false });
    }
  },

  async configure(config) {
    set({ isLoading: true, error: null });
    try {
      await syncApi.configureSync(config);
      set({ config });
    } catch (e) {
      set({ error: toError(e) });
      throw e;
    } finally {
      set({ isLoading: false });
    }
  },

  async syncNow() {
    set({ isLoading: true, error: null });
    try {
      const result = await syncApi.syncNow();
      // 同步后待同步变更可能清零，重拉一次状态保持一致。
      const status = await syncApi.getSyncStatus();
      set({ lastResult: result, status });
    } catch (e) {
      set({ error: toError(e) });
      throw e;
    } finally {
      set({ isLoading: false });
    }
  },
}));