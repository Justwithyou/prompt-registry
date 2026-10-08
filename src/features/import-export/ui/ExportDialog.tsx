import { useEffect, useState } from "react";
import { Icon } from "../../../shared/ui/Icon";
import {
  exportPrompts,
  extractVariables,
  type ExportFormat,
  type ExportSelectorInput,
  type VariableDefinition,
} from "../../../shared/api/commands";

/** 批量导出维度（分类 / 标签 / 项目）。 */
export type ExportDimension = "category" | "tag" | "project";

interface ExportDialogProps {
  onClose: () => void;
  /** 单条导出：固定按提示词 ID 导出（缺省表示批量/全部）。 */
  ids?: string[];
  /** 单条导出时的模板正文，用于「渲染」时提取变量填值。 */
  content?: string;
  /** 批量导出维度；传入后显示「导出范围」下拉（含「全部」）。 */
  dimension?: ExportDimension;
  /** 批量维度下的候选项（如全部分类）。 */
  options?: { id: string; name: string }[];
  title?: string;
}

const FORMAT_LABELS: Record<ExportFormat, string> = {
  skill: "SKILL.md（Claude/编辑器 Skill）",
  markdown: "Markdown",
  json: "JSON",
};

/**
 * 导出对话框（import-export/ui）。
 * 统一承载：格式选择、输出目录、可选模板渲染与变量填值、批量范围内的维度下拉。
 * 命中 Rust `export_prompts` 命令（锁定态由后端拒绝并提示解锁）。
 */
export function ExportDialog({
  onClose,
  ids,
  content,
  dimension,
  options = [],
  title = "导出提示词",
}: ExportDialogProps) {
  const [format, setFormat] = useState<ExportFormat>("skill");
  const [outDir, setOutDir] = useState("prompt-exports");
  const [render, setRender] = useState(false);
  const [scope, setScope] = useState("");

  const [variables, setVariables] = useState<VariableDefinition[]>([]);
  const [values, setValues] = useState<Record<string, string>>({});

  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<string[] | null>(null);

  // 单条导出时，正文变化 → 提取模板变量（供渲染填值）。
  useEffect(() => {
    if (content) void extractVariables(content).then(setVariables);
  }, [content]);

  const isSingle = !!ids && ids.length > 0;

  function buildSelector(): ExportSelectorInput {
    if (isSingle) return { ids: ids! };
    if (!dimension || scope === "") return {};
    if (dimension === "category") return { categoryId: scope };
    if (dimension === "project") return { projectId: scope };
    return { tagIds: [scope] };
  }

  async function handleExport() {
    setBusy(true);
    setError(null);
    setResult(null);
    try {
      const paths = await exportPrompts(
        buildSelector(),
        format,
        outDir.trim() || "prompt-exports",
        render,
        render ? values : {},
      );
      setResult(paths);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="editor-overlay" onClick={onClose}>
      <div className="editor-overlay-card" onClick={(e) => e.stopPropagation()}>
        <button
          type="button"
          className="overlay-close"
          onClick={onClose}
          title="关闭"
        >
          <Icon name="x" size={18} />
        </button>

        <h3 className="panel-title">
          <Icon name="download" size={18} />
          {title}
        </h3>
        <p className="panel-subtitle">
          选择导出格式与目标目录，导出结果写入本地文件。
        </p>

        {!isSingle && dimension && options.length > 0 && (
          <label className="field" style={{ marginBottom: "0.75rem" }}>
            <span className="field-label">导出范围</span>
            <select value={scope} onChange={(e) => setScope(e.target.value)}>
              <option value="">全部</option>
              {options.map((o) => (
                <option key={o.id} value={o.id}>
                  {o.name}
                </option>
              ))}
            </select>
          </label>
        )}

        <div className="export-form">
          <label className="field">
            <span className="field-label">导出格式</span>
            <select
              value={format}
              onChange={(e) => setFormat(e.target.value as ExportFormat)}
            >
              {(Object.keys(FORMAT_LABELS) as ExportFormat[]).map((f) => (
                <option key={f} value={f}>
                  {FORMAT_LABELS[f]}
                </option>
              ))}
            </select>
          </label>

          <label className="field">
            <span className="field-label">目标目录</span>
            <input
              value={outDir}
              placeholder="例如 prompt-exports"
              onChange={(e) => setOutDir(e.target.value)}
            />
          </label>
        </div>

        <label className="field field-inline" style={{ margin: "0.75rem 0" }}>
          <input
            type="checkbox"
            checked={render}
            onChange={(e) => setRender(e.target.checked)}
          />
          <span>渲染模板变量（`{"{{变量}}"}` 替换为填值/默认值）</span>
        </label>

        {render && isSingle && variables.length > 0 && (
          <div className="export-vars">
            <span className="field-label">变量填值</span>
            {variables.map((v) => (
              <label key={v.name} className="field">
                <span className="field-label">{`{{${v.name}}}`}</span>
                <input
                  value={values[v.name] ?? ""}
                  placeholder={v.defaultValue ?? v.name}
                  onChange={(e) =>
                    setValues((prev) => ({ ...prev, [v.name]: e.target.value }))
                  }
                />
              </label>
            ))}
          </div>
        )}

        <div className="export-actions">
          <button
            type="button"
            className="btn-primary"
            onClick={handleExport}
            disabled={busy}
          >
            <Icon name="download" size={16} />
            {busy ? "导出中…" : result ? "再次导出" : "导出"}
          </button>
          <button type="button" className="btn-ghost" onClick={onClose}>
            取消
          </button>
        </div>

        {error && <p className="export-error">导出失败：{error}</p>}
        {result && (
          <div className="export-result">
            <span>已导出 {result.length} 个文件：</span>
            <pre>{result.join("\n")}</pre>
          </div>
        )}
      </div>
    </div>
  );
}