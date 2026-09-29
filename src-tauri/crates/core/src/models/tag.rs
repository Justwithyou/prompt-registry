use serde::{Deserialize, Serialize};

/// 标签实体（支持层级：parent_id 指向父标签）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub color: Option<String>,
    pub created_at: String,
}

/// 创建标签入参。`name` 必填。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateTagInput {
    pub name: String,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
}

/// 更新标签入参（字段级局部更新，None 表示不修改）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct UpdateTagInput {
    pub name: Option<String>,
    pub parent_id: Option<String>,
    pub color: Option<String>,
}
