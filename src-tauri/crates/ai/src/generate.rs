//! 「AI 生成提示词」编排：系统提示词构造 + 结构化解析（含降级）+ 变量提取。

use prompt_core::{AiConfig, AppError, PromptDraft, VariableDefinition};
use serde::Deserialize;

use crate::provider::Provider;

/// 生成选项（可选：语气 / 面向对象 / 是否启用 `{{变量}}` 占位）。
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateOptions {
    pub tone: Option<String>,
    pub audience: Option<String>,
    pub use_variables: Option<bool>,
}

/// 模型返回的结构化 JSON 中间表示。
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DraftJson {
    #[serde(default)]
    pub(crate) title: String,
    #[serde(default)]
    pub(crate) content: String,
    #[serde(default)]
    pub(crate) description: String,
    #[serde(default)]
    pub(crate) suggested_tags: Vec<String>,
    #[serde(default)]
    pub(crate) suggested_category: Option<String>,
    /// 模型自报的变量定义（可选；后端以正文实际占位为准做合并）。
    #[serde(default)]
    pub(crate) variables: Vec<VariableDefinition>,
}

/// 生成结构化提示词草稿（不落库，返回给前端预览）。
pub fn generate_prompt(
    provider: &dyn Provider,
    config: &AiConfig,
    api_key: &str,
    requirement: &str,
    options: &GenerateOptions,
) -> Result<PromptDraft, AppError> {
    let system = build_system_prompt(options);
    let raw = provider.generate(config, api_key, &system, requirement)?;
    let draft = parse_draft(&raw);
    let variables = template::extract_variables(&draft.content);
    Ok(PromptDraft {
        title: draft.title,
        content: draft.content,
        description: draft.description,
        suggested_tags: draft.suggested_tags,
        suggested_category: draft.suggested_category,
        variables,
    })
}

/// 构造系统提示词：约束角色、格式与变量占位约定。
fn build_system_prompt(opts: &GenerateOptions) -> String {
    let use_variables = opts.use_variables.unwrap_or(true);
    let mut s = String::from(
        "你是一名资深的提示词工程（Prompt Engineering）专家。\
         根据用户用自然语言描述的需求/场景，生成一条高质量、可直接使用的结构化提示词。",
    );
    if let Some(tone) = opts.tone.as_deref().filter(|t| !t.trim().is_empty()) {
        s.push_str(&format!("语气/风格要求：{}。", tone.trim()));
    }
    if let Some(aud) = opts.audience.as_deref().filter(|a| !a.trim().is_empty()) {
        s.push_str(&format!("面向对象：{}。", aud.trim()));
    }
    if use_variables {
        s.push_str(
            "若提示词中有需要参数化的输入，用 `{{变量名}}` 占位（变量名用中文或英文均可）。",
        );
    }
    s.push_str(
        "只输出一个 JSON 对象，不要输出任何解释、Markdown 代码块标记或前后缀，字段如下：\
         {\"title\":\"建议标题\",\"content\":\"提示词正文\",\"description\":\"一句话用途说明\",\
         \"suggestedTags\":[\"标签1\",\"标签2\"],\"suggestedCategory\":\"建议分类名\"}。\
         若没有建议分类，suggestedCategory 给空字符串。",
    );
    s
}

/// 解析模型返回文本：优先直接 JSON；否则提取首个 `{...}` 块；再失败则整段当作正文兜底。
pub(crate) fn parse_draft(raw: &str) -> DraftJson {
    let trimmed = raw.trim();
    if let Ok(d) = serde_json::from_str::<DraftJson>(trimmed) {
        return d;
    }
    if let (Some(start), Some(end)) = (trimmed.find('{'), trimmed.rfind('}')) {
        if start <= end {
            if let Ok(d) = serde_json::from_str::<DraftJson>(&trimmed[start..=end]) {
                return d;
            }
        }
    }
    DraftJson {
        content: trimmed.to_string(),
        ..DraftJson::default()
    }
}
