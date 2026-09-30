use crate::errors::AppError;

/// 密钥/保险库抽象（M4 分工清单 §3.2）。
///
/// 轨道 B（SQLCipher 集成）与轨道 C（同步）都只依赖本 trait，不感知 Argon2id 具体实现。
/// Master Key 只存在于内存与 OS 钥匙串，绝不落库明文。
pub trait VaultProvider: Send + Sync {
    /// 首次设置口令：派生 KEK、生成并包装 Master Key、`PRAGMA rekey` 现库。
    fn setup(&self, password: &str) -> Result<(), AppError>;
    /// 解锁：用口令恢复 Master Key。
    fn unlock(&self, password: &str) -> Result<(), AppError>;
    /// 锁定：清除内存中的 Master Key。
    fn lock(&self) -> Result<(), AppError>;
    /// 修改口令：只重新包装 Master Key，数据密钥不变（密钥轮换）。
    fn change_password(&self, old_pw: &str, new_pw: &str) -> Result<(), AppError>;
    /// 当前 Master Key；未解锁时返回错误。
    fn master_key(&self) -> Result<[u8; 32], AppError>;
}
