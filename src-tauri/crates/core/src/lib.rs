//! core 领域层：领域模型、错误类型、Repository 抽象。
//!
//! 设计约束（对应架构方案 4.2）：
//! - 不依赖任何上层 crate，也不依赖具体存储引擎（如 sqlx / rusqlite）。
//! - 业务服务只面向 `traits` 中定义的 trait 编程，由 storage 提供实现。

pub mod errors;
pub mod models;
pub mod traits;

pub use errors::AppError;
pub use models::{
    AiConfig, BatchUpdateInput, Category, CreateCategoryInput, CreateProjectInput,
    CreatePromptInput, CreateTagInput, EncryptedPayload, Project, Prompt, PromptDraft, PromptQuery,
    PromptRelations, PromptRelationsInput, PromptSortBy, PromptVersion, SyncConfig, SyncLogEntry,
    SyncResult, SyncStatus, Tag, UpdateCategoryInput, UpdateProjectInput, UpdatePromptInput,
    UpdateTagInput, VariableDefinition, VaultEnvelope,
};
pub use traits::{
    CategoryRepository, ProjectRepository, PromptRepository, RemoteStore, SyncCipher,
    SyncConfigStore, SyncEngine, SyncLogStore, TagRepository, TemplateExtractor, VaultProvider,
    VersionRepository,
};
