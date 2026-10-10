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
    /// M1 遗留的字符串分类（保留兼容读）。
    pub category: Option<String>,
    /// M2 树形分类外键（指向 `categories.id`，供「按分类筛选」）。
    pub category_id: Option<String>,
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
    /// M1 遗留的字符串分类（保留兼容读；M2 起新增 `category_id` 指向树形分类）。
    pub category: Option<String>,
    pub is_pinned: Option<bool>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    // —— M2 组织层新增：多维联合筛选 ——
    pub project_id: Option<String>,
    pub tag_ids: Option<Vec<String>>,
    pub category_id: Option<String>,
    pub is_favorite: Option<bool>,
    pub sort_by: Option<PromptSortBy>,
    // —— M7 批量导出多选：多分类 / 多项目为「并集」（命中任意其一即纳入）——
    pub category_ids: Option<Vec<String>>,
    pub project_ids: Option<Vec<String>>,
    /// 标签筛选语义开关：默认 `false` 保持「交集」（命中全部标签）；`true` 时改为并集（多选批量导出）。
    pub tag_match_any: bool,
}

/// 列表排序字段（M2 新增，对应 `sortBy: updatedAt | rating | createdAt`）。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PromptSortBy {
    UpdatedAt,
    Rating,
    CreatedAt,
}

/// 一次性更新某条提示词的组织关系与快捷标记（供卡片快捷操作与元数据面板复用）。
///
/// 全字段 `Option`，`None` 表示不修改。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PromptRelationsInput {
    pub project_ids: Option<Vec<String>>,
    pub tag_ids: Option<Vec<String>>,
    /// 分配到树形分类（`Some` 设置，`None` 保持不变）。
    pub category_id: Option<String>,
    pub is_pinned: Option<bool>,
    pub is_favorite: Option<bool>,
    pub rating: Option<i32>,
}

/// 批量操作入参（批量打标签 / 移动项目 / 删除）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct BatchUpdateInput {
    pub add_tag_ids: Option<Vec<String>>,
    pub remove_tag_ids: Option<Vec<String>>,
    pub set_project_id: Option<String>,
    pub delete: bool,
}

/// 某条提示词当前的组织关系（读取用，供编辑器回显分类 / 项目 / 标签）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PromptRelations {
    pub project_ids: Vec<String>,
    pub tag_ids: Vec<String>,
    pub category_id: Option<String>,
}
