import { useEffect, useState } from "react";
import { Icon } from "../../../shared/ui/Icon";
import { useEscapeKey } from "../../../shared/hooks/useEscapeKey";
import {
  aiGeneratePrompt,
  getAiConfig,
  type PromptDraft,
} from "../../../shared/api/commands";

interface AiGenerateDialogProps {
  onClose: () => void;
  /** 用户接受草稿后由上层落库（新建提示词）并跳转编辑器。 */
  onAccept: (draft: PromptDraft) => void;
}

/**
 * 「AI 生成提示词」对话框（ai-assistant/ui）。
 * 输入自然语言需求 → 单次生成结构化草稿 → 预览 → 用户「接受」后交给上层保存。
 * 草稿不落库；生成会示意内容将发送到已配置的 baseUrl 端点。
 */
export function AiGenerateDialog({ onClose, onAccept }: AiGenerateDialogProps) {
  useEscapeKey(onClose);

  const [requirement, setRequirement] = useState("");
  const [tone, setTone] = useState("");
  const [audience, setAudience] = useState("");
  const [useVariables, setUseVariables] = useState(true);

  const [baseUrl, setBaseUrl] = useState("");
  const [hasApiKey, setHasApiKey] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [draft, setDraft] = useState<PromptDraft | null>(null);

  useEffect(() => {
    getAiConfig()
      .then((cfg) => {
        setBaseUrl(cfg.baseUrl);
        setHasApiKey(cfg.hasApiKey);
      })
      .catch(() => {
        /* 配置读取失败不阻塞对话框 */
      });
  }, []);

  async function handleGenerate() {
    if (!requirement.trim()) return;
    setBusy(true);
    setError(null);
    setDraft(null);
    try {
      const d = await aiGeneratePrompt(requirement, {
        tone: tone.trim() || undefined,
        audience: audience.trim() || undefined,
        useVariables,
      });
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
        style={{ width: "min(560px, 100%)" }}
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
          AI 生成提示词
        </h3>
        <p className="panel-subtitle">
          用自然语言描述需求，AI 生成一条结构化提示词草稿供你审阅编辑。
        </p>
        {baseUrl && (
          <p className="panel-subtitle" style={{ marginTop: 0 }}>
            内容将发送至：{baseUrl}
          </p>
        )}

        <label className="field">
          <span className="field-label">需求描述</span>
          <textarea
            rows={4}
            value={requirement}
            placeholder="例如：帮我写一条用于小红书笔记的文案提示词，语气活泼，面向年轻女性"
            onChange={(e) => setRequirement(e.target.value)}
          />
        </label>

        <div className="export-form">
          <label className="field">
            <span className="field-label">语气 / 风格（可选）</span>
            <input
              value={tone}
              placeholder="例如：专业、活泼、简洁"
              onChange={(e) => setTone(e.target.value)}
            />
          </label>
          <label className="field">
            <span className="field-label">目标对象（可选）</span>
            <input
              value={audience}
              placeholder="例如：产品经理、运营、大学生"
              onChange={(e) => setAudience(e.target.value)}
            />
          </label>
        </div>

        <label className="field field-inline" style={{ margin: "0.75rem 0" }}>
          <input
            type="checkbox"
            checked={useVariables}
            onChange={(e) => setUseVariables(e.target.checked)}
          />
          <span>允许使用 {`{{变量}}`} 占位</span>
        </label>

        {!hasApiKey && (
          <p className="export-error">
            尚未配置 API Key，请先到「设置 → AI 辅助」填写。
          </p>
        )}

        <div className="export-actions">
          <button
            type="button"
            className="btn-primary"
            onClick={handleGenerate}
            disabled={busy || !requirement.trim()}
          >
            <Icon name="sparkles" size={16} />
            {busy ? "生成中…" : "生成"}
          </button>
          <button type="button" className="btn-ghost" onClick={onClose}>
            取消
          </button>
        </div>

        {error && <p className="export-error">生成失败：{error}</p>}

        {draft && (
          <div className="export-result">
            {draft.title && (
              <strong style={{ display: "block", color: "var(--fg)" }}>
                {draft.title}
              </strong>
            )}
            {draft.description && (
              <span style={{ display: "block" }}>{draft.description}</span>
            )}
            <pre>{draft.content}</pre>
            {draft.suggestedTags.length > 0 && (
              <span style={{ display: "block" }}>
                建议标签：{draft.suggestedTags.join("、")}
              </span>
            )}
            {draft.variables.length > 0 && (
              <span style={{ display: "block" }}>
                变量：{draft.variables.map((v) => `{{${v.name}}}`).join("  ")}
              </span>
            )}
            <div className="export-actions" style={{ marginTop: "0.75rem" }}>
              <button
                type="button"
                className="btn-primary"
                onClick={() => onAccept(draft)}
              >
                接受并打开编辑器
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