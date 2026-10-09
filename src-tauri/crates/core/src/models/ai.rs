use serde::{Deserialize, Serialize};

use super::template::VariableDefinition;

/// AI 提供商配置（非敏感项，对应《AI 辅助生成提示词设计方案》§5）。
///
/// 序列化契约：`rename_all = "camelCase"`。API Key 不入本结构，走 OS 钥匙串。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiConfig {
    /// 提供商标识（`openai` / `deepseek` / 其他 OpenAI 兼容服务）。
    pub provider: String,
    /// 服务根地址（末尾斜杠不敏感，请求时追加 `/chat/completions`）。
    pub base_url: String,
    /// 模型名（如 `gpt-4o-mini` / `deepseek-chat`）。
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub timeout_secs: u64,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-4o-mini".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
            timeout_secs: 60,
        }
    }
}

/// AI 生成的结构化提示词草稿（`ai_generate_prompt` 返回，对应设计文档 §4）。
///
/// 草稿不落库，由用户审阅后走「新建/编辑提示词」流程保存。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptDraft {
    pub title: String,
    pub content: String,
    pub description: String,
    pub suggested_tags: Vec<String>,
    pub suggested_category: Option<String>,
    /// 从 `content` 中用 `template::extract_variables` 提取的变量定义。
    pub variables: Vec<VariableDefinition>,
}