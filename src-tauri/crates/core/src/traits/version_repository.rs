use crate::errors::AppError;
use crate::models::{Prompt, PromptVersion};

/// 版本历史仓库抽象（M3 模板与版本）。
///
/// - 由 `storage` crate 提供 SQLite 实现（轨道 A）。
/// - 命令层只面向此 trait 编程，不感知具体存储引擎。
pub trait VersionRepository: Send + Sync {
    /// 列出某提示词的全部历史版本（按创建时间降序）。
    fn list(&self, prompt_id: &str) -> Result<Vec<PromptVersion>, AppError>;
    /// 获取单个版本详情。
    fn get(&self, version_id: &str) -> Result<PromptVersion, AppError>;
    /// 回滚：从指定历史版本生成一条新版本（追加式，不覆盖历史）。
    fn rollback(&self, prompt_id: &str, version_id: &str) -> Result<Prompt, AppError>;
}
