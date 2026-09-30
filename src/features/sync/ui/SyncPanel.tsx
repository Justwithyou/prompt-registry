import { useEffect, useState, type FormEvent } from "react";
import { useSyncStore } from "../stores/syncStore";

/**
 * 同步配置页（sync/ui，M4-E4）。
 * 提供 WebDAV 配置表单、同步状态展示与一键同步。
 */

function formatSyncTime(ts: string | null | undefined): string {
  if (!ts) return "从未同步";
  const d = new Date(ts);
  if (Number.isNaN(d.getTime())) return ts;
  return d.toLocaleString();
}

export function SyncPanel() {
  const status = useSyncStore((s) => s.status);
  const config = useSyncStore((s) => s.config);
  const lastResult = useSyncStore((s) => s.lastResult);
  const isLoading = useSyncStore((s) => s.isLoading);
  const error = useSyncStore((s) => s.error);
  const fetchStatus = useSyncStore((s) => s.fetchStatus);
  const configure = useSyncStore((s) => s.configure);
  const syncNow = useSyncStore((s) => s.syncNow);

  const [url, setUrl] = useState("");
  const [username, setUsername] = useState("");
  const [token, setToken] = useState("");
  const [enabled, setEnabled] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);

  // 挂载时拉取同步状态。
  useEffect(() => {
    void fetchStatus();
  }, [fetchStatus]);

  // 会话内已配置过则回显（密码/令牌不回显到明文输入值由用户重填）。
  useEffect(() => {
    if (config) {
      setUrl(config.url);
      setUsername(config.username);
      setEnabled(config.enabled);
    }
  }, [config]);

  async function handleConfigure(e: FormEvent) {
    e.preventDefault();
    if (!url) return;
    setNotice(null);
    try {
      await configure({ url, username, token: token || null, enabled });
      setToken("");
      setNotice(enabled ? "同步已启用，配置已保存。" : "同步已停用，配置已保存。");
    } catch {
      // 错误已写入 store.error
    }
  }

  async function handleSyncNow() {
    setNotice(null);
    try {
      await syncNow();
      setNotice("同步完成。");
    } catch {
      // 错误已写入 store.error
    }
  }

  return (
    <div className="settings-page">
      <h2>同步</h2>

      <section className="settings-section">
        <h3>WebDAV 配置</h3>
        <div className="settings-hint">
          支持坚果云 / 群晖 / Nextcloud 等 WebDAV 服务，同步走端到端加密（AES-256-GCM）。
        </div>
        <form className="settings-stack" onSubmit={handleConfigure}>
          <div className="settings-grow">
            <div className="settings-label">服务器地址</div>
            <input
              type="url"
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              placeholder="https://dav.jianguoyun.com/dav/…"
            />
          </div>
          <div className="settings-grow">
            <div className="settings-label">用户名</div>
            <input
              type="text"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              placeholder="WebDAV 用户名"
              autoComplete="username"
            />
          </div>
          <div className="settings-grow">
            <div className="settings-label">密码 / 令牌</div>
            <input
              type="password"
              value={token}
              onChange={(e) => setToken(e.target.value)}
              placeholder="应用密码或访问令牌"
              autoComplete="current-password"
            />
          </div>
          <label className="settings-check">
            <input
              type="checkbox"
              checked={enabled}
              onChange={(e) => setEnabled(e.target.checked)}
            />
            <span>启用同步</span>
          </label>
          <button
            type="submit"
            className="btn-primary"
            disabled={isLoading || !url}
          >
            保存配置
          </button>
        </form>
      </section>

      <section className="settings-section">
        <h3>同步状态</h3>
        <div className="settings-row settings-stats">
          <div>
            <div className="settings-label">设备标识</div>
            <div className="settings-hint">{status?.deviceId ?? "—"}</div>
          </div>
          <div>
            <div className="settings-label">待同步变更</div>
            <div className="settings-hint">
              {status ? status.pendingChanges : "—"}
            </div>
          </div>
          <div>
            <div className="settings-label">上次同步</div>
            <div className="settings-hint">
              {formatSyncTime(status?.lastSyncAt)}
            </div>
          </div>
        </div>
        <div className="settings-stack">
          <button
            type="button"
            className="btn-primary"
            onClick={handleSyncNow}
            disabled={isLoading || !status}
          >
            立即同步
          </button>
          {lastResult && (
            <div className="settings-hint">
              已推送 {lastResult.pushed} · 已拉取 {lastResult.pulled} · 已合并{" "}
              {lastResult.merged}
            </div>
          )}
        </div>
      </section>

      {error && <div className="settings-error">{error}</div>}
      {notice && <div className="settings-notice">{notice}</div>}
    </div>
  );
}