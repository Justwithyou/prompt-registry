import { useEffect, useState } from "react";
import { useParams } from "react-router";
import { usePromptStore } from "../../prompt-library/index";
import { useTemplateStore } from "../stores/templateStore";
import { copyText } from "../../../shared/lib/clipboard";
import { Icon } from "../../../shared/ui/Icon";

/**
 * 模板使用面板（template-engine/ui，M3 D1）。
 *
 * 以当前编辑提示词的正文作为模板：提取 `{{变量}}` 列表 → 填写变量 →
 * Nunjucks 实时预览 → 一键复制。变量提取走真实 IPC（extract_variables），
 * 渲染走前端 Nunjucks（M3 决策点 3.1）。
 */
export function TemplateUsePanel() {
  const { id } = useParams();
  const promptId = id ?? "";

  const content = usePromptStore(
    (s) => s.prompts.find((p) => p.id === promptId)?.content ?? "",
  );

  const variables = useTemplateStore((s) => s.variables);
  const values = useTemplateStore((s) => s.values);
  const preview = useTemplateStore((s) => s.preview);
  const extractVariables = useTemplateStore((s) => s.extractVariables);
  const setValue = useTemplateStore((s) => s.setValue);
  const renderPreview = useTemplateStore((s) => s.renderPreview);

  const [copied, setCopied] = useState(false);

  // 正文变化时重新提取变量（异步，提取后按默认值初始化填写值）。
  useEffect(() => {
    if (!content) return;
    void extractVariables(content);
  }, [content, extractVariables]);

  // 变量填写值或正文变化时刷新预览。
  useEffect(() => {
    renderPreview(content);
  }, [content, values, renderPreview]);

  async function handleCopy() {
    const ok = await copyText(preview);
    setCopied(ok);
    if (ok) window.setTimeout(() => setCopied(false), 1500);
  }

  return (
    <div className="template-panel">
      <h3 className="panel-title">
        <Icon name="braces" size={18} />
        使用模板
      </h3>
      <p className="panel-subtitle">当前提示词中的变量会自动提取，填写后预览并复制</p>

      {content === "" ? (
        <p className="panel-hint">当前提示词正文为空，暂无模板变量。</p>
      ) : variables.length > 0 ? (
        <div className="variable-form">
          {variables.map((v) => (
            <label key={v.name} className="field">
              <span className="field-label">{`{{${v.name}}}`}</span>
              <input
                value={values[v.name] ?? ""}
                placeholder={v.name}
                onChange={(e) => setValue(v.name, e.target.value)}
              />
            </label>
          ))}
        </div>
      ) : (
        <p className="panel-hint">该提示词不含 `{"{{变量}}"}`，无可填写项。</p>
      )}

      <div className="template-preview">
        <div className="preview-header">
          <span>渲染预览</span>
          <button type="button" className="copy-button" onClick={handleCopy}>
            <Icon name={copied ? "check" : "copy"} size={16} />
            {copied ? "已复制" : "一键复制"}
          </button>
        </div>
        <pre className="preview-body">{preview}</pre>
      </div>
    </div>
  );
}