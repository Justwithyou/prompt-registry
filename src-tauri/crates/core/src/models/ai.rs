use serde::{Deserialize, Serialize};

use super::template::VariableDefinition;

/// AI 提供商配置（非敏感项，对应《AI 辅助生成提示词设计方案》§5）。
///
/// 序列化契约：`rename_all = "camelCase"`。API Key 不入本结构，走 OS 钥匙串。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiConfig {
    /// 提供商标识（`openai` / `deepseek` / `ollama` / `lmstudio` / 其他 OpenAI 兼容服务）。
    pub provider: String,
    /// 服务根地址（末尾斜杠不敏感，请求时追加 `/chat/completions`）。
    pub base_url: String,
    /// 模型名（如 `gpt-4o-mini` / `deepseek-chat`）。
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub timeout_secs: u64,
    /// 是否要求 API Key（本地模型预设为 false，允许留空 Key 跳鉴权）。
    #[serde(default = "default_true")]
    pub api_key_required: bool,
    /// 是否开启脱敏（发送到远端前替换敏感内容，回网后还原）。
    #[serde(default)]
    pub desensitize: bool,
    /// 用户自定义敏感词（精确替换，与内置正则规则叠加）。
    #[serde(default)]
    pub sensitive_terms: Vec<String>,
}

fn default_true() -> bool {
    true
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
            api_key_required: true,
            desensitize: false,
            sensitive_terms: Vec::new(),
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
