import { useRef, useState } from "react";
import type { ChangeEvent } from "react";
import { usePromptStore } from "../../prompt-library/index";
import { useSettingsStore } from "../stores/settingsStore";
import { Icon } from "../../../shared/ui/Icon";
import { VaultPanel } from "./VaultPanel";
import { FONT_OPTIONS, IMPORTED_FONT_KEY, saveImportedFont } from "../../../shared/lib/fonts";

/**
 * 设置页骨架（settings/ui）。
 * 可交互项：主题 / 全局字体（含导入本地字体）/ 存储位置 / 清空数据 / 关于。
 */
export function SettingsPage() {
  const theme = useSettingsStore((s) => s.theme);
  const storageLocation = useSettingsStore((s) => s.storageLocation);
  const fontKey = useSettingsStore((s) => s.fontKey);
  const importedFontName = useSettingsStore((s) => s.importedFontName);
  const toggleTheme = useSettingsStore((s) => s.toggleTheme);
  const setStorageLocation = useSettingsStore((s) => s.setStorageLocation);
  const setFontKey = useSettingsStore((s) => s.setFontKey);
  const setImportedFont = useSettingsStore((s) => s.setImportedFont);
  const clearAll = usePromptStore((s) => s.clearAll);

  const fileRef = useRef<HTMLInputElement>(null);
  const [confirmingClear, setConfirmingClear] = useState(false);
  const [cleared, setCleared] = useState(false);
  const [importing, setImporting] = useState(false);
  const [importError, setImportError] = useState<string | null>(null);

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

  async function handleImportFont(e: ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0];
    e.target.value = ""; // 允许重复选择同名文件
    if (!file) return;
    setImporting(true);
    setImportError(null);
    try {
      await saveImportedFont(file);
      setImportedFont(file.name);
    } catch {
      setImportError("导入失败，请确认是有效的字体文件。");
    } finally {
      setImporting(false);
    }
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

        <div className="settings-row" style={{ marginTop: "1rem" }}>
          <div className="settings-grow">
            <div className="settings-label">全局字体</div>
            <div className="settings-hint" style={{ marginTop: 0 }}>
              改变整个应用的显示字体
            </div>
            <select
              value={fontKey}
              onChange={(e) => setFontKey(e.target.value)}
            >
              {FONT_OPTIONS.map((f) => (
                <option key={f.key} value={f.key}>
                  {f.label}
                </option>
              ))}
              {importedFontName && (
                <option value={IMPORTED_FONT_KEY}>
                  {importedFontName}（本地）
                </option>
              )}
            </select>
          </div>
        </div>

        <div className="settings-row" style={{ marginTop: "1rem" }}>
          <div>
            <div className="settings-label">导入本地字体</div>
            <div className="settings-hint">支持 .ttf / .otf / .woff / .woff2</div>
            {importError && (
              <div className="settings-hint" style={{ color: "var(--danger)" }}>
                {importError}
              </div>
            )}
          </div>
          <button
            type="button"
            className="btn-primary"
            disabled={importing}
            onClick={() => fileRef.current?.click()}
          >
            <Icon name="plus" size={16} />
            {importing ? "导入中…" : "选择字体文件"}
          </button>
          <input
            ref={fileRef}
            type="file"
            accept=".ttf,.otf,.woff,.woff2"
            style={{ display: "none" }}
            onChange={handleImportFont}
          />
        </div>
      </section>

      <VaultPanel />

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
            <div className="settings-label">提示词管理平台</div>
            <div className="settings-hint">v1.0 正式版</div>
          </div>
        </div>
      </section>
    </div>
  );
}