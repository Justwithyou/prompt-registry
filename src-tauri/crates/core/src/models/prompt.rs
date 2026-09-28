use serde::{Deserialize, Serialize};

/// 提示词实体（对应 BRD 数据模型 prompts 表）。
///
/// 序列化契约：`rename_all = "camelCase"` 使前端 camelCase 字段与 Rust snake_case 字段一一对应，
/// 这是前后端 IPC 契约的一部分（Phase 0 冻结，变更需广播）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Prompt {
    pub id: String,
    pub title: String,
    pub content: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub is_pinned: bool,
    pub is_favorite: bool,
    pub rating: i32,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

/// 创建提示词入参。
///
/// `title` / `content` 为必填；其余字段缺省时为 `None`。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreatePromptInput {
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
}

/// 更新提示词入参（字段级局部更新，None 表示不修改）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct UpdatePromptInput {
    pub title: Option<String>,
    pub content: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub is_pinned: Option<bool>,
    pub is_favorite: Option<bool>,
    pub rating: Option<i32>,
}

/// 列表 / 搜索查询条件（M1 仅支持 keyword + limit，其余留空实现）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PromptQuery {
    pub keyword: Option<String>,
    pub category: Option<String>,
    pub is_pinned: Option<bool>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}