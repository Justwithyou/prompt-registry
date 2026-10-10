import { useEffect, useRef, useState } from "react";
import type { ChangeEvent } from "react";
import { usePromptStore } from "../../prompt-library/index";
import { useSettingsStore } from "../stores/settingsStore";
import { Icon } from "../../../shared/ui/Icon";
import { Select } from "../../../shared/ui/Select";
import { useEscapeKey } from "../../../shared/hooks/useEscapeKey";
import { VaultPanel } from "./VaultPanel";
import { AiConfigDialog } from "./AiConfigDialog";
import { CollapsibleSection } from "../../../shared/ui/CollapsibleSection";
import { FONT_OPTIONS, IMPORTED_FONT_KEY, saveImportedFont } from "../../../shared/lib/fonts";
import { open, save } from "@tauri-apps/plugin-dialog";
import {
  getStorageLocation,
  setStorageLocation as changeStorageLocation,
  exportData,
  importData,
  importDb,
  type ImportReport,
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
 * AI 配置已抽离到 AiConfigDialog（弹窗），由「AI 辅助」开关控制是否打开。
 */
export function SettingsPage() {
  const theme = useSettingsStore((s) => s.theme);
  const storageLocation = useSettingsStore((s) => s.storageLocation);
  const fontKey = useSettingsStore((s) => s.fontKey);
  const importedFontName = useSettingsStore((s) => s.importedFontName);
  const setTheme = useSettingsStore((s) => s.setTheme);
  const setStorageLocation = useSettingsStore((s) => s.setStorageLocation);
  const setFontKey = useSettingsStore((s) => s.setFontKey);
  const setImportedFont = useSettingsStore((s) => s.setImportedFont);
  const aiEnabled = useSettingsStore((s) => s.aiEnabled);
  const toggleAiEnabled = useSettingsStore((s) => s.toggleAiEnabled);
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
  const [aiDialogOpen, setAiDialogOpen] = useState(false);
  const [openSection, setOpenSection] = useState<string | null>(null);

  function toggleSection(key: string) {
    setOpenSection((prev) => (prev === key ? null : key));
  }

  useEscapeKey(() => setConfirmOpen(false), confirmOpen);

  useEffect(() => {
    void getStorageLocation().then(setStorageLocation);
  }, [setStorageLocation]);

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
        filters: [
          { name: "JSON 数据", extensions: ["json"] },
          { name: "数据库", extensions: ["db"] },
        ],
      });
      if (typeof p !== "string") return;
      const isDb = p.toLowerCase().endsWith(".db");
      const report = isDb
        ? await importDb(p, overwrite)
        : await importData(p, overwrite);
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

  return (
    <div className="settings-page">
      <div className="settings-inner">
      <h2>设置</h2>

      <CollapsibleSection
        title="外观"
        open={openSection === "appearance"}
        onToggle={() => toggleSection("appearance")}
      >
        <div className="settings-row">
          <div>
            <div className="settings-label">主题</div>
            <div className="settings-hint">
              {theme === "system"
                ? "跟随系统"
                : theme === "dark"
                  ? "深色模式"
                  : "浅色模式"}
            </div>
          </div>
          <div className="theme-switcher" role="radiogroup" aria-label="主题模式">
            <button
              type="button"
              className={`theme-switcher-btn${theme === "system" ? " is-active" : ""}`}
              onClick={() => setTheme("system")}
              title="跟随系统"
            >
              <Icon name="monitor" size={16} />
            </button>
            <button
              type="button"
              className={`theme-switcher-btn${theme === "light" ? " is-active" : ""}`}
              onClick={() => setTheme("light")}
              title="浅色模式"
            >
              <Icon name="sun" size={16} />
            </button>
            <button
              type="button"
              className={`theme-switcher-btn${theme === "dark" ? " is-active" : ""}`}
              onClick={() => setTheme("dark")}
              title="深色模式"
            >
              <Icon name="moon" size={16} />
            </button>
          </div>
        </div>

        <div className="settings-row" style={{ marginTop: "1rem" }}>
          <div className="settings-grow">
            <div className="settings-label">全局字体</div>
            <div className="settings-hint">改变整个应用的显示字体</div>
            <Select
              value={fontKey}
              onChange={setFontKey}
              options={[
                ...FONT_OPTIONS.map((f) => ({ value: f.key, label: f.label })),
                ...(importedFontName
                  ? [{ value: IMPORTED_FONT_KEY, label: `${importedFontName}（本地）` }]
                  : []),
              ]}
            />
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
      </CollapsibleSection>

      <CollapsibleSection
        title="AI 辅助"
        open={openSection === "ai"}
        onToggle={() => toggleSection("ai")}
        headerRight={
          <div className="settings-ai-actions">
            <button
              type="button"
              className="btn-ghost"
              onClick={() => setAiDialogOpen(true)}
            >
              <Icon name="settings" size={15} />
              配置
            </button>
            <button
              type="button"
              className={`switch${aiEnabled ? " is-on" : ""}`}
              role="switch"
              aria-checked={aiEnabled}
              onClick={toggleAiEnabled}
            >
              <span className="switch-track" />
              <span className="switch-label">{aiEnabled ? "已开启" : "已关闭"}</span>
            </button>
          </div>
        }
      >
        <div className="settings-hint">
          AI 生成 / 调优 / 提取变量 / 本地模型 / 脱敏
        </div>
      </CollapsibleSection>

      <AiConfigDialog
        open={aiDialogOpen}
        onClose={() => setAiDialogOpen(false)}
      />

      <VaultPanel
        open={openSection === "security"}
        onToggle={() => toggleSection("security")}
      />

      <CollapsibleSection
        title="存储"
        open={openSection === "storage"}
        onToggle={() => toggleSection("storage")}
      >
        <div style={{ display: "flex", flexDirection: "column", gap: "0.75rem" }}>
          <div className="settings-row">
            <div className="settings-grow">
              <div className="settings-label">当前数据存储位置</div>
              <div className="settings-hint" style={{ marginTop: "0.5rem", wordBreak: "break-all" }}>
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
      </CollapsibleSection>

      <CollapsibleSection
        title="数据"
        open={openSection === "data"}
        onToggle={() => toggleSection("data")}
      >
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
      </CollapsibleSection>

      <CollapsibleSection
        title="数据导入 / 导出"
        open={openSection === "transfer"}
        onToggle={() => toggleSection("transfer")}
      >
        <div style={{ display: "flex", flexDirection: "column", gap: "0.75rem" }}>
          <div className="settings-row" style={{ alignItems: "flex-end" }}>
            <div className="settings-grow">
              <div className="settings-label">导出数据</div>
              <div className="settings-hint">把当前全部数据导出为单个 JSON 文件</div>
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
              <div className="settings-hint">
                从 JSON 或本平台 .db 数据库文件导入；追加合并会跳过已有 ID，覆盖清空会先删除现有数据
              </div>
              <Select
                value={overwrite ? "overwrite" : "merge"}
                onChange={(value) => setOverwrite(value === "overwrite")}
                options={[
                  { value: "merge", label: "追加合并（跳过已有）" },
                  { value: "overwrite", label: "覆盖清空后导入" },
                ]}
              />
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
      </CollapsibleSection>

      <CollapsibleSection
        title="关于"
        open={openSection === "about"}
        onToggle={() => toggleSection("about")}
      >
        <div className="settings-row">
          <div>
            <div className="settings-label">提示词管理平台</div>
            <div className="settings-hint">v1.0 正式版</div>
          </div>
        </div>
      </CollapsibleSection>
      </div>

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