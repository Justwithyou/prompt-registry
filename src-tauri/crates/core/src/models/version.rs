use serde::{Deserialize, Serialize};

/// 提示词历史版本实体（对应 `prompt_versions` 表，M3 分工清单 §3.2）。
///
/// 序列化契约：`rename_all = "camelCase"`。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptVersion {
    pub id: String,
    pub prompt_id: String,
    /// 语义化版本号 `主.次.补丁`（默认 `0.1.0` 起）。
    pub version: String,
    pub content: String,
    pub change_summary: Option<String>,
    pub created_at: String,
}
