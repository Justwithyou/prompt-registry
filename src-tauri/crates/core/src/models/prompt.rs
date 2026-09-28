use serde::{Deserialize, Serialize};

/// 提示词实体（对应 BRD 数据模型 prompts 表）。
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreatePromptInput {
    pub title: String,
    pub content: String,
    pub description: Option<String>,
    pub category: Option<String>,
}

/// 更新提示词入参（字段级局部更新，None 表示不修改）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdatePromptInput {
    pub title: Option<String>,
    pub content: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub is_pinned: Option<bool>,
    pub is_favorite: Option<bool>,
    pub rating: Option<i32>,
}

/// 列表 / 搜索查询条件。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PromptQuery {
    pub keyword: Option<String>,
    pub category: Option<String>,
    pub tag_ids: Option<Vec<String>>,
    pub project_id: Option<String>,
}