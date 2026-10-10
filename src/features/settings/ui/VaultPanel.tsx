import { useEffect, useState, type FormEvent } from "react";
import { useVaultStore } from "../stores/vaultStore";
import { CollapsibleSection } from "../../../shared/ui/CollapsibleSection";

/**
 * 保险库（加密/口令）设置面板（settings/ui，M4-E3）。
 * 提供三种状态：未设置（设置主口令）、已锁定（解锁）、已解锁（锁定 + 修改口令），
 * 以及「开启自动解锁」开关。
 */
export function VaultPanel({
  open,
  onToggle,
}: {
  open?: boolean;
  onToggle?: () => void;
}) {
  const isLocked = useVaultStore((s) => s.isLocked);
  const initialized = useVaultStore((s) => s.initialized);
  const autoUnlock = useVaultStore((s) => s.autoUnlock);
  const isLoading = useVaultStore((s) => s.isLoading);
  const error = useVaultStore((s) => s.error);
  const refresh = useVaultStore((s) => s.refresh);
  const setup = useVaultStore((s) => s.setup);
  const unlock = useVaultStore((s) => s.unlock);
  const lock = useVaultStore((s) => s.lock);
  const changePassword = useVaultStore((s) => s.changePassword);
  const setAutoUnlock = useVaultStore((s) => s.setAutoUnlock);

  const [password, setPassword] = useState("");
  const [oldPw, setOldPw] = useState("");
  const [newPw, setNewPw] = useState("");
  const [notice, setNotice] = useState<string | null>(null);

  // 挂载时与后端锁定态对齐。
  useEffect(() => {
    void refresh();
  }, [refresh]);

  async function handleSetup(e: FormEvent) {
    e.preventDefault();
    if (!password) return;
    setNotice(null);
    try {
      await setup(password);
      setPassword("");
      setNotice("主口令已设置，数据库已转为加密存储。");
    } catch {
      // 错误已写入 store.error
    }
  }

  async function handleUnlock(e: FormEvent) {
    e.preventDefault();
    if (!password) return;
    setNotice(null);
    try {
      await unlock(password);
      setPassword("");
      setNotice("已解锁。");
    } catch {
      // 错误已写入 store.error
    }
  }

  async function handleLock() {
    setNotice(null);
    try {
      await lock();
      setNotice("保险库已锁定，主密钥已从内存清除。");
    } catch {
      // 错误已写入 store.error
    }
  }

  async function handleChangePassword(e: FormEvent) {
    e.preventDefault();
    if (!oldPw || !newPw) return;
    setNotice(null);
    try {
      await changePassword(oldPw, newPw);
      setOldPw("");
      setNewPw("");
      setNotice("口令已修改，主密钥已重新包装。");
    } catch {
      // 错误已写入 store.error
    }
  }

  return (
    <CollapsibleSection title="安全 · 数据加密" open={open} onToggle={onToggle}>

      {!initialized ? (
        <form className="settings-stack" onSubmit={handleSetup}>
          <div className="settings-hint">
            尚未设置主口令。设置后数据库将使用 SQLCipher 加密，无口令无法读取。
          </div>
          <div className="settings-grow">
            <div className="settings-label">主口令</div>
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="设置一个强口令"
              autoComplete="new-password"
            />
          </div>
          <button
            type="submit"
            className="btn-primary"
            disabled={isLoading || !password}
          >
            设置并加密
          </button>
        </form>
      ) : isLocked ? (
        <form className="settings-stack" onSubmit={handleUnlock}>
          <div className="settings-hint">保险库已锁定，请输入主口令解锁。</div>
          <div className="settings-grow">
            <div className="settings-label">主口令</div>
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="输入主口令"
              autoComplete="current-password"
            />
          </div>
          <button
            type="submit"
            className="btn-primary"
            disabled={isLoading || !password}
          >
            解锁
          </button>
        </form>
      ) : (
        <>
          <div className="settings-row">
            <div>
              <div className="settings-label">当前状态：已解锁</div>
              <div className="settings-hint">主密钥已载入内存，可正常读写。</div>
            </div>
            <button
              type="button"
              className="btn-ghost"
              onClick={handleLock}
              disabled={isLoading}
            >
              锁定
            </button>
          </div>

          <form className="settings-stack" onSubmit={handleChangePassword}>
            <div className="settings-label">修改口令</div>
            <div className="settings-grow">
              <input
                type="password"
                value={oldPw}
                onChange={(e) => setOldPw(e.target.value)}
                placeholder="旧口令"
                autoComplete="current-password"
              />
            </div>
            <div className="settings-grow">
              <input
                type="password"
                value={newPw}
                onChange={(e) => setNewPw(e.target.value)}
                placeholder="新口令"
                autoComplete="new-password"
              />
            </div>
            <button
              type="submit"
              className="btn-primary"
              disabled={isLoading || !oldPw || !newPw}
            >
              修改口令
            </button>
          </form>
        </>
      )}

      <label className="settings-check">
        <input
          type="checkbox"
          checked={autoUnlock}
          onChange={(e) => setAutoUnlock(e.target.checked)}
          disabled={!initialized}
        />
        <span>开启自动解锁（主密钥存入系统钥匙串）</span>
      </label>

      {error && <div className="settings-error">{error}</div>}
      {notice && <div className="settings-notice">{notice}</div>}
    </CollapsibleSection>
  );
}