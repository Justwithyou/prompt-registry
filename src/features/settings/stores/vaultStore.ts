import { create } from "zustand";
import * as vaultApi from "../api";

/**
 * 保险库（加密/口令）状态（Zustand store），对应架构方案 3.2 的 vaultStore。
 * 职责：主口令设置 / 解锁 / 锁定 / 修改口令，以及是否开启自动解锁（M4-E2/E3）。
 *
 * 说明：冻结契约只暴露 `is_locked`，未暴露「是否已设置过口令」的查询命令；
 *       故前端用 `initialized` 本地标记补足「未设置 vs 已解锁」的区分，
 *       首次 `setup` / 成功 `unlock` 后置为 true。
 */
export interface VaultStore {
  /** 是否处于锁定态（来自 is_locked 命令）。 */
  isLocked: boolean;
  /** 是否已设置过主口令（本地标记，见文件头说明）。 */
  initialized: boolean;
  /** 是否开启自动解锁（前端偏好，主密钥存 OS 钥匙串）。 */
  autoUnlock: boolean;
  isLoading: boolean;
  error: string | null;

  refresh: () => Promise<void>;
  setup: (password: string) => Promise<void>;
  unlock: (password: string) => Promise<void>;
  lock: () => Promise<void>;
  changePassword: (oldPw: string, newPw: string) => Promise<void>;
  setAutoUnlock: (on: boolean) => Promise<void>;
}

const INITIALIZED_KEY = "prompt-manager.vault-initialized";
const AUTO_UNLOCK_KEY = "prompt-manager.vault-auto-unlock";

function initialInitialized(): boolean {
  return localStorage.getItem(INITIALIZED_KEY) === "1";
}

function initialAutoUnlock(): boolean {
  return localStorage.getItem(AUTO_UNLOCK_KEY) === "1";
}

function toError(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

export const useVaultStore = create<VaultStore>()((set) => ({
  isLocked: false,
  initialized: initialInitialized(),
  autoUnlock: initialAutoUnlock(),
  isLoading: false,
  error: null,

  // 后台轮询锁定态（挂载时调用一次，与后端 is_locked 对齐）。
  async refresh() {
    set({ isLoading: true, error: null });
    try {
      const isLocked = await vaultApi.isLocked();
      set({ isLocked });
    } catch (e) {
      set({ error: toError(e) });
    } finally {
      set({ isLoading: false });
    }
  },

  async setup(password) {
    set({ isLoading: true, error: null });
    try {
      await vaultApi.setupVault(password);
      localStorage.setItem(INITIALIZED_KEY, "1");
      set({ initialized: true, isLocked: false });
    } catch (e) {
      set({ error: toError(e) });
      throw e;
    } finally {
      set({ isLoading: false });
    }
  },

  async unlock(password) {
    set({ isLoading: true, error: null });
    try {
      await vaultApi.unlockVault(password);
      localStorage.setItem(INITIALIZED_KEY, "1");
      set({ initialized: true, isLocked: false });
    } catch (e) {
      set({ error: toError(e) });
      throw e;
    } finally {
      set({ isLoading: false });
    }
  },

  async lock() {
    set({ isLoading: true, error: null });
    try {
      await vaultApi.lockVault();
      set({ isLocked: true });
    } catch (e) {
      set({ error: toError(e) });
      throw e;
    } finally {
      set({ isLoading: false });
    }
  },

  async changePassword(oldPw, newPw) {
    set({ isLoading: true, error: null });
    try {
      await vaultApi.changePassword(oldPw, newPw);
    } catch (e) {
      set({ error: toError(e) });
      throw e;
    } finally {
      set({ isLoading: false });
    }
  },

  async setAutoUnlock(on) {
    try {
      // 真实写入/清除 OS 钥匙串；失败时回滚（不更新本地偏好），错误交由面板展示。
      await vaultApi.setAutoUnlock(on);
      set({ autoUnlock: on });
      localStorage.setItem(AUTO_UNLOCK_KEY, on ? "1" : "0");
    } catch (e) {
      set({ error: toError(e) });
    }
  },
}));