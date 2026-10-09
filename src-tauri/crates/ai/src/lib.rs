//! AI 辅助服务（对应架构方案 4.2 /《AI 辅助生成提示词设计方案》）。
//!
//! 首版只实现「AI 生成提示词」：
//! - [`Provider`] trait 抽象模型调用，`OpenAiCompatibleProvider` 提供 OpenAI 兼容 HTTP 实现；
//! - [`generate_prompt`] 编排「系统提示词 → 调用模型 → 结构化解析 → 提取变量」。
//!
//! 后续「调优 / 提取变量 / 本地模型 / 脱敏」在同一 trait 上扩展，不改前端主干。

use prompt_core::AppError;

pub mod generate;
pub mod provider;

pub use generate::{generate_prompt, GenerateOptions};
pub use provider::{OpenAiCompatibleProvider, Provider};

/// 构造 AI 错误（统一 `AppError::Ai`，错误信息不携带密钥等敏感内容）。
pub(crate) fn ai_err(msg: impl Into<String>) -> AppError {
    AppError::Ai(msg.into())
}