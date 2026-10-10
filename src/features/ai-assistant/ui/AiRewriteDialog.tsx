import { useState } from "react";
import { Icon } from "../../../shared/ui/Icon";
import { useEscapeKey } from "../../../shared/hooks/useEscapeKey";
import {
  aiPolishPrompt,
  aiExtractVariables,
  type PromptDraft,
} from "../../../shared/api/commands";

type AiRewriteMode = "polish" | "extract";

interface AiRewriteDialogProps {
  mode: AiRewriteMode;
  content: string;
  onClose: () => void;
  /** 用户确认后把结果正文应用到编辑器。 */
  onApply: (newContent: string) => void;
}

const STYLES = [
  { label: "更专业", value: "更专业、更规范、术语准确" },
  { label: "更简洁", value: "更简洁、去除冗余" },
  { label: "更具体", value: "更具体、补充细节与示例" },
];

/**
 * 「AI 调优 / AI 提取变量」对话框（ai-assistant/ui）。
 * 对存量正文做单次改写，预览后由用户「应用」回填编辑器，不自动落库。
 */
export function AiRewriteDialog({
  mode,
  content,
  onClose,
  onApply,
}: AiRewriteDialogProps) {
  const [style, setStyle] = useState(STYLES[0].value);
  const [instruction, setInstruction] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [draft, setDraft] = useState<PromptDraft | null>(null);

  const isPolish = mode === "polish";

  useEscapeKey(onClose);

  async function run() {
    setBusy(true);
    setError(null);
    setDraft(null);
    try {
      const d = isPolish
        ? await aiPolishPrompt(content, {
            style,
            instruction: instruction.trim() || undefined,
          })
        : await aiExtractVariables(content);
      setDraft(d);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="editor-overlay" onClick={onClose}>
      <div
        className="editor-overlay-card"
        style={{ width: "min(640px, 100%)" }}
        onClick={(e) => e.stopPropagation()}
      >
        <button
          type="button"
          className="overlay-close"
          onClick={onClose}
          title="关闭"
        >
          <Icon name="x" size={18} />
        </button>

        <h3 className="panel-title">
          <Icon name="sparkles" size={18} />
          {isPolish ? "AI 调优" : "AI 提取变量"}
        </h3>
        <p className="panel-subtitle">
          {isPolish
            ? "对当前提示词正文做方向化改写，预览确认后应用。"
            : "把正文中写死的部分参数化为 {{变量}}，预览确认后应用。"}
        </p>

        {isPolish && (
          <>
            <div className="field">
              <span className="field-label">调优方向</span>
              <div style={{ display: "flex", flexWrap: "wrap", gap: "0.75rem" }}>
                {STYLES.map((s) => (
                  <label key={s.label} className="field field-inline" style={{ margin: 0 }}>
                    <input
                      type="radio"
                      name="polish-style"
                      checked={style === s.value}
                      onChange={() => setStyle(s.value)}
                    />
                    <span>{s.label}</span>
                  </label>
                ))}
              </div>
            </div>
            <label className="field">
              <span className="field-label">补充要求（可选）</span>
              <input
                value={instruction}
                placeholder="例如：保留所有示例、语气沉稳"
                onChange={(e) => setInstruction(e.target.value)}
              />
            </label>
          </>
        )}

        <div className="export-actions">
          <button
            type="button"
            className="btn-primary"
            onClick={run}
            disabled={busy || !content.trim()}
          >
            <Icon name="sparkles" size={16} />
            {busy ? "处理中…" : isPolish ? "开始调优" : "开始提取"}
          </button>
          <button type="button" className="btn-ghost" onClick={onClose}>
            取消
          </button>
        </div>

        {error && <p className="export-error">处理失败：{error}</p>}

        {draft && (
          <div className="export-result">
            <pre>{draft.content}</pre>
            {draft.variables.length > 0 && (
              <span style={{ display: "block" }}>
                变量：{draft.variables.map((v) => `{{${v.name}}}`).join("  ")}
              </span>
            )}
            <div className="export-actions" style={{ marginTop: "0.75rem" }}>
              <button
                type="button"
                className="btn-primary"
                onClick={() => onApply(draft.content)}
              >
                应用正文
              </button>
              <button
                type="button"
                className="btn-ghost"
                onClick={() => setDraft(null)}
              >
                重新生成
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}