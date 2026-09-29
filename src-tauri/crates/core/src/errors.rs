use serde::Serialize;

/// 统一错误类型（对应架构方案 4.5）。
///
/// 为使 core 保持零重型依赖，底层错误（如 `rusqlite::Error`）在此折算为字符串承载，
/// 由 storage / sync / crypto 等 crate 在实现层调用 `map_err(|e| AppError::Database(e.to_string()))` 完成转换。
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Database(String),

    #[error("加密错误: {0}")]
    Crypto(String),

    #[error("同步错误: {0}")]
    Sync(String),

    #[error("模板错误: {0}")]
    Template(String),

    #[error("未找到: {0}")]
    NotFound(String),

    #[error("认证失败: {0}")]
    Auth(String),

    #[error("网络错误: {0}")]
    Network(String),
}

/// IPC 层将其序列化为字符串返回给前端。
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
