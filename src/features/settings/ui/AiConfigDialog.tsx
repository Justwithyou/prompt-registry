import { useEffect, useState } from "react";
import { Icon } from "../../../shared/ui/Icon";
import { Select } from "../../../shared/ui/Select";
import { useEscapeKey } from "../../../shared/hooks/useEscapeKey";
import {
  getAiConfig,
  setAiConfig,
  setAiApiKey,
  type AiConfig,
} from "../../../shared/api/commands";

const AI_PRESETS = [
  {
    key: "openai",
    label: "OpenAI",
    baseUrl: "https://api.openai.com/v1",
    model: "gpt-4o-mini",
    apiKeyRequired: true,
  },
  {
    key: "deepseek",
    label: "DeepSeek",
    baseUrl: "https://api.deepseek.com/v1",
    model: "deepseek-chat",
    apiKeyRequired: true,
  },
  {
    key: "ollama",
    label: "Ollama（本地）",
    baseUrl: "http://localhost:11434/v1",
    model: "llama3.1",
    apiKeyRequired: false,
  },
  {
    key: "lmstudio",
    label: "LM Studio（本地）",
    baseUrl: "http://localhost:1234/v1",
    model: "local-model",
    apiKeyRequired: false,
  },
];

interface AiConfigDialogProps {
  open: boolean;
  onClose: () => void;
}

/**
 * AI 提供商配置弹窗（settings/ui）。
 * 由设置页「AI 辅助」开关打开为覆盖层，不再内联在设置页里。
 * 左栏为提供商列表（当前单条、后续可扩展为多提供商），右栏为选中提供商的详情表单。
 */
export function AiConfigDialog({ open, onClose }: AiConfigDialogProps) {
  const [aiDraft, setAiDraft] = useState<AiConfig>({
    provider: "openai",
    baseUrl: "https://api.openai.com/v1",
    model: "gpt-4o-mini",
    temperature: 0.7,
    maxTokens: 2048,
    timeoutSecs: 60,
    apiKeyRequired: true,
    desensitize: false,
    sensitiveTerms: [],
  });
  const [aiHasKey, setAiHasKey] = useState(false);
  const [aiKey, setAiKey] = useState("");
  const [sensitiveTermsText, setSensitiveTermsText] = useState("");
  const [aiSaving, setAiSaving] = useState(false);
  const [aiKeySaving, setAiKeySaving] = useState(false);
  const [aiMessage, setAiMessage] = useState<string | null>(null);
  const [aiError, setAiError] = useState<string | null>(null);

  useEscapeKey(onClose, open);

  useEffect(() => {
    if (!open) return;
    getAiConfig()
      .then((cfg) => {
        const { hasApiKey, ...rest } = cfg;
        setAiDraft(rest);
        setAiHasKey(hasApiKey);
        setSensitiveTermsText(rest.sensitiveTerms.join("，"));
      })
      .catch(() => {
        /* 配置读取失败时保留默认值 */
      });
  }, [open]);

  function updateAi(patch: Partial<AiConfig>) {
    setAiDraft((d) => ({ ...d, ...patch }));
    setAiMessage(null);
    setAiError(null);
  }

  function applyAiPreset(key: string) {
    const preset = AI_PRESETS.find((p) => p.key === key);
    if (!preset) return;
    updateAi({
      provider: preset.key,
      baseUrl: preset.baseUrl,
      model: preset.model,
      apiKeyRequired: preset.apiKeyRequired,
    });
  }

  async function handleAiSave() {
    setAiSaving(true);
    setAiMessage(null);
    setAiError(null);
    try {
      const cfg = await setAiConfig({
        ...aiDraft,
        sensitiveTerms: sensitiveTermsText
          .split(/[,，、\n]/)
          .map((s) => s.trim())
          .filter(Boolean),
      });
      setAiHasKey(cfg.hasApiKey);
      setAiSaving(false);
      onClose();
    } catch (e) {
      setAiError(String(e));
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

  if (!open) return null;

  const presetLabel =
    AI_PRESETS.find((p) => p.key === aiDraft.provider)?.label ?? aiDraft.provider;

  return (
    <div className="editor-overlay" onClick={onClose}>
      <div
        className="editor-overlay-card ai-config-card"
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
          AI 提供商
        </h3>
        <p className="panel-subtitle">
          配置模型服务、API Key 与脱敏规则。多提供商管理将在后续版本支持。
        </p>

        <div className="ai-config-body">
          <aside className="ai-provider-list">
            <div className="ai-provider-item is-active">
              <Icon name="sparkles" size={16} />
              <div className="ai-provider-meta">
                <span className="ai-provider-name">{presetLabel}</span>
                <span className="ai-provider-model">
                  {aiDraft.model || "未设置模型"}
                </span>
              </div>
            </div>
            <button
              type="button"
              className="ai-provider-add"
              disabled
              title="后续版本将支持多提供商"
            >
              <Icon name="plus" size={14} />
              添加提供商
            </button>
          </aside>

          <div className="ai-provider-detail">
            <div className="ai-config-section-title">连接</div>
            <div className="ai-form-grid">
              <label className="field">
                <span className="field-label">提供商预设</span>
                <Select
                  value={aiDraft.provider}
                  onChange={applyAiPreset}
                  options={AI_PRESETS.map((p) => ({ value: p.key, label: p.label }))}
                />
              </label>

              <label className="field">
                <span className="field-label">本地模型</span>
                <span className="field-inline">
                  <input
                    type="checkbox"
                    checked={!aiDraft.apiKeyRequired}
                    onChange={(e) => updateAi({ apiKeyRequired: !e.target.checked })}
                  />
                  <span>无需 API Key</span>
                </span>
              </label>

              <label className="field field-span">
                <span className="field-label">服务地址（baseUrl）</span>
                <input
                  type="text"
                  value={aiDraft.baseUrl}
                  placeholder="https://api.openai.com/v1"
                  onChange={(e) => updateAi({ baseUrl: e.target.value })}
                />
                <span className="settings-hint" style={{ marginTop: 0 }}>
                  OpenAI 兼容接口根地址，生成时自动追加 /chat/completions
                </span>
              </label>
            </div>

            <div className="ai-config-section-title">模型与参数</div>
            <div className="ai-form-grid">
              <label className="field">
                <span className="field-label">模型（model）</span>
                <input
                  type="text"
                  value={aiDraft.model}
                  placeholder="gpt-4o-mini"
                  onChange={(e) => updateAi({ model: e.target.value })}
                />
              </label>

              <label className="field">
                <span className="field-label">temperature</span>
                <input
                  type="number"
                  min={0}
                  max={2}
                  step={0.1}
                  value={aiDraft.temperature}
                  onChange={(e) => updateAi({ temperature: Number(e.target.value) })}
                />
              </label>

              <label className="field">
                <span className="field-label">maxTokens</span>
                <input
                  type="number"
                  min={1}
                  value={aiDraft.maxTokens}
                  onChange={(e) => updateAi({ maxTokens: Number(e.target.value) })}
                />
              </label>

              <label className="field">
                <span className="field-label">超时（秒）</span>
                <input
                  type="number"
                  min={1}
                  value={aiDraft.timeoutSecs}
                  onChange={(e) => updateAi({ timeoutSecs: Number(e.target.value) })}
                />
              </label>
            </div>

            <div className="ai-config-section-title">API Key</div>
            <div className="field">
              <span className="field-label">
                API Key {aiHasKey ? "（已配置）" : "（未配置）"}
              </span>
              <div className="ai-key-row">
                <input
                  type="password"
                  value={aiKey}
                  placeholder={aiHasKey ? "已配置，如需更换请输入新 Key" : "输入 API Key"}
                  onChange={(e) => setAiKey(e.target.value)}
                />
                <button
                  type="button"
                  className="btn-ghost"
                  disabled={aiKeySaving || !aiKey.trim()}
                  onClick={handleAiKeySave}
                >
                  {aiKeySaving ? "保存中…" : "保存 Key"}
                </button>
              </div>
              <span className="settings-hint" style={{ marginTop: 0 }}>
                只存于系统钥匙串，保存后不回显；留空保存即清空。
              </span>
            </div>

            <div className="ai-config-section-title">通用 · 脱敏</div>
            <label className="field field-inline">
              <input
                type="checkbox"
                checked={aiDraft.desensitize}
                onChange={(e) => updateAi({ desensitize: e.target.checked })}
              />
              <span>脱敏模式（发送到远端前替换敏感内容，回网后还原）</span>
            </label>

            <label className="field">
              <span className="field-label">敏感词（逗号分隔）</span>
              <input
                type="text"
                value={sensitiveTermsText}
                placeholder="例如：张三，某公司，绝密项目"
                onChange={(e) => setSensitiveTermsText(e.target.value)}
              />
              <span className="settings-hint" style={{ marginTop: 0 }}>
                公司名、人名、项目代号等，精确匹配后替换为占位符
              </span>
            </label>

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

            <div className="ai-config-actions">
              <button type="button" className="btn-ghost" onClick={onClose}>
                取消
              </button>
              <button
                type="button"
                className="btn-primary"
                disabled={aiSaving}
                onClick={handleAiSave}
              >
                {aiSaving ? "保存中…" : "保存配置"}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}