use crate::errors::AppError;
use crate::models::{
    BatchUpdateInput, CreatePromptInput, Prompt, PromptQuery, PromptRelationsInput,
    UpdatePromptInput,
};

/// 提示词仓库抽象（对应架构方案 4.4）。
///
/// - 由 `storage` crate 提供 SQLite 实现（`SqlitePromptRepository`）。
/// - 测试场景可提供内存实现，实现依赖注入与可测试性。
/// - 业务服务只面向此 trait 编程，未知晓具体存储引擎，便于后续替换（如官方插件迁移）。
pub trait PromptRepository: Send + Sync {
    fn find_all(&self, query: &PromptQuery) -> Result<Vec<Prompt>, AppError>;
    fn find_by_id(&self, id: &str) -> Result<Option<Prompt>, AppError>;
    fn create(&self, input: &CreatePromptInput) -> Result<Prompt, AppError>;
    fn update(&self, id: &str, input: &UpdatePromptInput) -> Result<Prompt, AppError>;
    fn delete(&self, id: &str) -> Result<(), AppError>;
    fn search(&self, keyword: &str) -> Result<Vec<Prompt>, AppError>;
    /// 一次性更新某条提示词的组织关系（项目 / 标签 / 分类）与快捷标记。
    fn set_relations(&self, id: &str, input: &PromptRelationsInput) -> Result<Prompt, AppError>;
    /// 批量打标签 / 移动项目 / 删除。
    fn batch_update(&self, ids: &[String], batch: &BatchUpdateInput) -> Result<(), AppError>;
}
