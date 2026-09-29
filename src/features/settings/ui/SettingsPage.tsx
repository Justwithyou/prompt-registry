import { useState } from "react";
import { usePromptStore } from "../../prompt-library/index";
import { useSettingsStore } from "../stores/settingsStore";
import { Icon } from "../../../shared/ui/Icon";

/**
 * 设置页骨架（settings/ui）。
 * 四项设置均可交互：主题切换 / 存储位置 / 清空数据（二次确认，真实删除全部提示词）/ 关于（版本信息）。
 */
export function SettingsPage() {
  const theme = useSettingsStore((s) => s.theme);
  const storageLocation = useSettingsStore((s) => s.storageLocation);
  const appVersion = useSettingsStore((s) => s.appVersion);
  const toggleTheme = useSettingsStore((s) => s.toggleTheme);
  const setStorageLocation = useSettingsStore((s) => s.setStorageLocation);
  const clearAll = usePromptStore((s) => s.clearAll);

  const [confirmingClear, setConfirmingClear] = useState(false);
  const [cleared, setCleared] = useState(false);

  function handleClear() {
    if (!confirmingClear) {
      setConfirmingClear(true);
      return;
    }
    void clearAll().then(() => {
      setConfirmingClear(false);
      setCleared(true);
    });
  }

  return (
    <div className="settings-page">
      <h2>设置</h2>

      <section className="settings-section">
        <h3>外观</h3>
        <div className="settings-row">
          <div>
            <div className="settings-label">主题</div>
            <div className="settings-hint">
              当前：{theme === "dark" ? "深色（星空）" : "浅色"}
            </div>
          </div>
          <button type="button" className="btn-primary" onClick={toggleTheme}>
            <Icon name={theme === "dark" ? "sun" : "moon"} size={16} />
            切换到{theme === "dark" ? "浅色" : "深色"}
          </button>
        </div>
      </section>

      <section className="settings-section">
        <h3>存储</h3>
        <div className="settings-row">
          <div className="settings-grow">
            <div className="settings-label">数据存储位置</div>
            <input
              type="text"
              value={storageLocation}
              onChange={(e) => setStorageLocation(e.target.value)}
            />
          </div>
        </div>
      </section>

      <section className="settings-section">
        <h3>数据</h3>
        <div className="settings-row">
          <div>
            <div className="settings-label">清空数据</div>
            <div className="settings-hint">
              {cleared ? "已清空所有提示词。" : "删除所有提示词（不可撤销）。"}
            </div>
          </div>
          <button
            type="button"
            className={`btn-danger${confirmingClear ? " is-confirm" : ""}`}
            onClick={handleClear}
          >
            {confirmingClear ? "再次点击确认清空" : "清空数据"}
          </button>
        </div>
      </section>

      <section className="settings-section">
        <h3>关于</h3>
        <div className="settings-row">
          <div>
            <div className="settings-label">个人提示词管理平台</div>
            <div className="settings-hint">版本 {appVersion}（M1 MVP）</div>
          </div>
        </div>
      </section>
    </div>
  );
}