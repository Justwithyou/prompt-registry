import { useEffect, useRef, useState } from "react";
import type { ChangeEvent } from "react";
import { usePromptStore } from "../../prompt-library/index";
import { useSettingsStore } from "../stores/settingsStore";
import { Icon } from "../../../shared/ui/Icon";
import { VaultPanel } from "./VaultPanel";
import { FONT_OPTIONS, IMPORTED_FONT_KEY, saveImportedFont } from "../../../shared/lib/fonts";
import { open, save } from "@tauri-apps/plugin-dialog";
import {
  getStorageLocation,
  setStorageLocation as changeStorageLocation,
  exportData,
  importData,
  getAiConfig,
  setAiConfig,
  setAiApiKey,
  type ImportReport,
  type AiConfig,
} from "../../../shared/api/commands";

/** 把导入报告渲染成一段可读摘要文本。 */
function summarizeImport(report: ImportReport): string {
  const line = (label: string, c: { imported: number; skipped: number }) =>
    `${label}：新增 ${c.imported}，跳过 ${c.skipped}`;
  return [
    line("分类", report.categories),
    line("标签", report.tags),
    line("项目", report.projects),
    line("提示词", report.prompts),
    line("历史版本", report.versions),
  ].join("；");
}

/**
 * 设置页骨架（settings/ui）。
 * 可交互项：主题 / 全局字体（含导入本地字体）/ 存储位置 / 清空数据 / 数据导入导出 / 关于。
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
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [clearing, setClearing] = useState(false);
  const [cleared, setCleared] = useState(false);
  const [clearError, setClearError] = useState<string | null>(null);
  const [importing, setImporting] = useState(false);
  const [importError, setImportError] = useState<string | null>(null);
  const [storageDraft, setStorageDraft] = useState("");
  const [storageSaving, setStorageSaving] = useState(false);
  const [storageMessage, setStorageMessage] = useState<string | null>(null);
  const [storageError, setStorageError] = useState<string | null>(null);
  const [exporting, setExporting] = useState(false);
  const [exportMessage, setExportMessage] = useState<string | null>(null);
  const [exportError, setExportError] = useState<string | null>(null);
  const [overwrite, setOverwrite] = useState(false);
  const [importingData, setImportingData] = useState(false);
  const [importMessage, setImportMessage] = useState<string | null>(null);
  const [importDataError, setImportDataError] = useState<string | null>(null);
  const [aiDraft, setAiDraft] = useState<AiConfig>({
    provider: "openai",
    baseUrl: "https://api.openai.com/v1",
    model: "gpt-4o-mini",
    temperature: 0.7,
    maxTokens: 2048,
    timeoutSecs: 60,
  });
  const [aiHasKey, setAiHasKey] = useState(false);
  const [aiKey, setAiKey] = useState("");
  const [aiSaving, setAiSaving] = useState(false);
  const [aiKeySaving, setAiKeySaving] = useState(false);
  const [aiMessage, setAiMessage] = useState<string | null>(null);
  const [aiError, setAiError] = useState<string | null>(null);

  useEffect(() => {
    void getStorageLocation().then(setStorageLocation);
  }, [setStorageLocation]);

  useEffect(() => {
    getAiConfig()
      .then((cfg) => {
        const { hasApiKey, ...rest } = cfg;
        setAiDraft(rest);
        setAiHasKey(hasApiKey);
      })
      .catch(() => {
        /* 配置读取失败时保留默认值 */
      });
  }, []);

  async function handleConfirmClear() {
    setClearing(true);
    setClearError(null);
    try {
      await clearAll();
      setCleared(true);
      setConfirmOpen(false);
    } catch (e) {
      setClearError(String(e));
    } finally {
      setClearing(false);
    }
  }

  async function handleBrowseStorage() {
    try {
      const dir = await open({ directory: true, title: "选择数据存储目录" });
      if (typeof dir === "string") {
        setStorageDraft(dir);
        setStorageMessage(null);
        setStorageError(null);
      }
    } catch (e) {
      setStorageError(String(e));
    }
  }

  async function handleChangeStorage() {
    const draft = storageDraft.trim();
    if (!draft) return;
    setStorageSaving(true);
    setStorageMessage(null);
    setStorageError(null);
    try {
      const newDir = await changeStorageLocation(draft);
      setStorageLocation(newDir);
      setStorageDraft("");
      setStorageMessage(
        `存储位置已更新：数据已迁移到「${newDir}」，旧位置的原数据库文件已保留作为备份。`,
      );
    } catch (e) {
      setStorageError(String(e));
    } finally {
      setStorageSaving(false);
    }
  }

  async function handleExportData() {
    setExporting(true);
    setExportMessage(null);
    setExportError(null);
    try {
      const p = await save({
        title: "导出数据",
        defaultPath: "prompt-manager-备份.json",
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (!p) return;
      const wrote = await exportData(p);
      setExportMessage(`已导出到：${wrote}`);
    } catch (e) {
      setExportError(String(e));
    } finally {
      setExporting(false);
    }
  }

  async function handleImportData() {
    setImportingData(true);
    setImportMessage(null);
    setImportDataError(null);
    try {
      const p = await open({
        title: "选择要导入的数据文件",
        multiple: false,
        directory: false,
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (typeof p !== "string") return;
      const report = await importData(p, overwrite);
      setImportMessage(summarizeImport(report));
    } catch (e) {
      setImportDataError(String(e));
    } finally {
      setImportingData(false);
    }
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

  function updateAi(patch: Partial<AiConfig>) {
    setAiDraft((d) => ({ ...d, ...patch }));
    setAiMessage(null);
    setAiError(null);
  }

  async function handleAiSave() {
    setAiSaving(true);
    setAiMessage(null);
    setAiError(null);
    try {
      const cfg = await setAiConfig(aiDraft);
      setAiHasKey(cfg.hasApiKey);
      setAiMessage("已保存 AI 配置。");
    } catch (e) {
      setAiError(String(e));
    } finally {
      setAiSaving(false);
    }
  }

  async function handleAiKeySave() {
    setAiKeySaving(true);
    setAiMessage(null);
    setAiError(null);
    try {
      await setAiApiKey(aiKey);
      setAiKey("");
      const cfg = await getAiConfig();
      setAiHasKey(cfg.hasApiKey);
      setAiMessage(
        cfg.hasApiKey ? "API Key 已保存（仅存于系统钥匙串）。" : "已清空 API Key。",
      );
    } catch (e) {
      setAiError(String(e));
    } finally {
      setAiKeySaving(false);
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

      <section className="settings-section">
        <h3>AI 辅助</h3>
        <div style={{ display: "flex", flexDirection: "column", gap: "0.75rem" }}>
          <div className="settings-row">
            <div className="settings-grow">
              <div className="settings-label">服务地址（baseUrl）</div>
              <div className="settings-hint" style={{ marginTop: 0 }}>
                OpenAI 兼容接口根地址，生成时自动追加 /chat/completions
              </div>
              <input
                type="text"
                value={aiDraft.baseUrl}
                placeholder="https://api.openai.com/v1"
                onChange={(e) => updateAi({ baseUrl: e.target.value })}
              />
            </div>
          </div>

          <div className="settings-row">
            <div className="settings-grow">
              <div className="settings-label">模型（model）</div>
              <input
                type="text"
                value={aiDraft.model}
                placeholder="gpt-4o-mini"
                onChange={(e) => updateAi({ model: e.target.value })}
              />
            </div>
            <div className="settings-grow" style={{ marginLeft: "0.75rem" }}>
              <div className="settings-label">提供商标识</div>
              <input
                type="text"
                value={aiDraft.provider}
                placeholder="openai"
                onChange={(e) => updateAi({ provider: e.target.value })}
              />
            </div>
          </div>

          <div className="settings-row">
            <div style={{ display: "flex", gap: "0.75rem", width: "100%" }}>
              <div className="settings-grow">
                <div className="settings-label">temperature</div>
                <input
                  type="number"
                  min={0}
                  max={2}
                  step={0.1}
                  value={aiDraft.temperature}
                  onChange={(e) =>
                    updateAi({ temperature: Number(e.target.value) })
                  }
                />
              </div>
              <div className="settings-grow">
                <div className="settings-label">maxTokens</div>
                <input
                  type="number"
                  min={1}
                  value={aiDraft.maxTokens}
                  onChange={(e) => updateAi({ maxTokens: Number(e.target.value) })}
                />
              </div>
              <div className="settings-grow">
                <div className="settings-label">超时（秒）</div>
                <input
                  type="number"
                  min={1}
                  value={aiDraft.timeoutSecs}
                  onChange={(e) =>
                    updateAi({ timeoutSecs: Number(e.target.value) })
                  }
                />
              </div>
            </div>
          </div>

          <div className="settings-row" style={{ alignItems: "flex-end" }}>
            <div className="settings-grow">
              <div className="settings-label">
                API Key {aiHasKey ? "（已配置）" : "（未配置）"}
              </div>
              <div className="settings-hint" style={{ marginTop: 0 }}>
                只存于系统钥匙串，保存后不回显；留空保存即清空。
              </div>
              <input
                type="password"
                value={aiKey}
                placeholder={aiHasKey ? "已配置，如需更换请输入新 Key" : "输入 API Key"}
                onChange={(e) => setAiKey(e.target.value)}
              />
            </div>
            <button
              type="button"
              className="btn-ghost"
              disabled={aiKeySaving || !aiKey.trim()}
              onClick={handleAiKeySave}
            >
              {aiKeySaving ? "保存中…" : "保存 Key"}
            </button>
          </div>

          {aiMessage && (
            <div className="settings-hint" style={{ marginTop: 0 }}>
              {aiMessage}
            </div>
          )}
          {aiError && (
            <div
              className="settings-hint"
              style={{ marginTop: 0, color: "var(--danger)", wordBreak: "break-all" }}
            >
              {aiError}
            </div>
          )}

          <div>
            <button
              type="button"
              className="btn-primary"
              disabled={aiSaving}
              onClick={handleAiSave}
            >
              {aiSaving ? "保存中…" : "保存 AI 配置"}
            </button>
          </div>
        </div>
      </section>

      <VaultPanel />

      <section className="settings-section">
        <h3>存储</h3>
        <div style={{ display: "flex", flexDirection: "column", gap: "0.75rem" }}>
          <div className="settings-row">
            <div className="settings-grow">
              <div className="settings-label">当前数据存储位置</div>
              <div className="settings-hint" style={{ marginTop: 0, wordBreak: "break-all" }}>
                {storageLocation || "读取中…"}
              </div>
            </div>
          </div>
          <div className="settings-row" style={{ alignItems: "flex-end" }}>
            <div className="settings-grow">
              <div className="settings-label">新存储位置</div>
              <input
                type="text"
                value={storageDraft}
                readOnly
                placeholder="点击「选择目录」选择目标文件夹"
              />
            </div>
            <div style={{ display: "flex", gap: "0.5rem" }}>
              <button
                type="button"
                className="btn-ghost"
                onClick={handleBrowseStorage}
              >
                选择目录
              </button>
              <button
                type="button"
                className="btn-primary"
                disabled={storageSaving || !storageDraft.trim()}
                onClick={handleChangeStorage}
              >
                {storageSaving ? "迁移中…" : "迁移并切换"}
              </button>
            </div>
          </div>
          {storageMessage && (
            <div className="settings-hint" style={{ marginTop: 0 }}>
              {storageMessage}
            </div>
          )}
          {storageError && (
            <div className="settings-hint" style={{ marginTop: 0, color: "var(--danger)" }}>
              {storageError}
            </div>
          )}
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
            {clearError && (
              <div className="settings-hint" style={{ color: "var(--danger)" }}>
                {clearError}
              </div>
            )}
          </div>
          <button
            type="button"
            className="btn-danger"
            onClick={() => setConfirmOpen(true)}
          >
            清空数据
          </button>
        </div>
      </section>

      <section className="settings-section">
        <h3>数据导入 / 导出</h3>
        <div style={{ display: "flex", flexDirection: "column", gap: "0.75rem" }}>
          <div className="settings-row" style={{ alignItems: "flex-end" }}>
            <div className="settings-grow">
              <div className="settings-label">导出数据</div>
              <div className="settings-hint" style={{ marginTop: 0 }}>
                把当前全部数据导出为单个 JSON 文件
              </div>
            </div>
            <button
              type="button"
              className="btn-primary"
              disabled={exporting}
              onClick={handleExportData}
            >
              {exporting ? "导出中…" : "导出数据"}
            </button>
          </div>
          {exportMessage && (
            <div className="settings-hint" style={{ marginTop: 0, wordBreak: "break-all" }}>
              {exportMessage}
            </div>
          )}
          {exportError && (
            <div className="settings-hint" style={{ marginTop: 0, color: "var(--danger)", wordBreak: "break-all" }}>
              {exportError}
            </div>
          )}

          <div
            className="settings-row"
            style={{ alignItems: "flex-end", marginTop: "0.5rem" }}
          >
            <div className="settings-grow">
              <div className="settings-label">导入数据</div>
              <div className="settings-hint" style={{ marginTop: 0 }}>
                从 JSON 文件导入；追加合并会跳过已有 ID，覆盖清空会先删除现有数据
              </div>
              <select
                style={{ marginTop: "0.5rem" }}
                value={overwrite ? "overwrite" : "merge"}
                onChange={(e) => setOverwrite(e.target.value === "overwrite")}
              >
                <option value="merge">追加合并（跳过已有）</option>
                <option value="overwrite">覆盖清空后导入</option>
              </select>
            </div>
            <button
              type="button"
              className="btn-primary"
              disabled={importingData}
              onClick={handleImportData}
            >
              {importingData ? "导入中…" : "导入数据"}
            </button>
          </div>
          {importMessage && (
            <div className="settings-hint" style={{ marginTop: 0 }}>
              {importMessage}
            </div>
          )}
          {importDataError && (
            <div className="settings-hint" style={{ marginTop: 0, color: "var(--danger)", wordBreak: "break-all" }}>
              {importDataError}
            </div>
          )}
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

      {confirmOpen && (
        <div className="editor-overlay" onClick={() => setConfirmOpen(false)}>
          <div
            className="editor-overlay-card"
            style={{ width: "min(420px, 100%)" }}
            onClick={(e) => e.stopPropagation()}
          >
            <button
              type="button"
              className="overlay-close"
              onClick={() => setConfirmOpen(false)}
              title="关闭"
            >
              <Icon name="x" size={18} />
            </button>
            <h3 className="panel-title">
              <Icon name="trash" size={18} />
              清空数据
            </h3>
            <p className="panel-subtitle">
              确定要删除所有提示词吗？此操作不可撤销，请谨慎确认。
            </p>
            <div className="export-actions">
              <button
                type="button"
                className="btn-danger"
                onClick={handleConfirmClear}
                disabled={clearing}
              >
                {clearing ? "清空中…" : "确认清空"}
              </button>
              <button
                type="button"
                className="btn-ghost"
                onClick={() => setConfirmOpen(false)}
              >
                取消
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}