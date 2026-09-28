use crate::errors::AppError;
use crate::models::{CreatePromptInput, Prompt, PromptQuery, UpdatePromptInput};

/// 提示词仓库抽象（对应架构方案 4.4）。
///
/// - 由 `storage` crate 提供 SQLCipher 实现（`SqlitePromptRepository`）。
/// - 测试场景可提供内存实现，实现依赖注入与可测试性。
/// - 业务服务只面向此 trait 编程，未知晓具体存储引擎，便于后续替换（如官方插件迁移）。
pub trait PromptRepository: Send + Sync {
    fn find_all(&self, query: &PromptQuery) -> Result<Vec<Prompt>, AppError>;
    fn find_by_id(&self, id: &str) -> Result<Option<Prompt>, AppError>;
    fn create(&self, input: &CreatePromptInput) -> Result<Prompt, AppError>;
    fn update(&self, id: &str, input: &UpdatePromptInput) -> Result<Prompt, AppError>;
    fn delete(&self, id: &str) -> Result<(), AppError>;
    fn search(&self, keyword: &str) -> Result<Vec<Prompt>, AppError>;
}