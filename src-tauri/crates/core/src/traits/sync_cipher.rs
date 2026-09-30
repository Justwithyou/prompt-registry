use crate::errors::AppError;
use crate::models::EncryptedPayload;

/// 同步加密管道抽象（M4 分工清单 §3.2）。
///
/// 轨道 C 依赖它加密/解密同步数据包，不感知 AES-256-GCM 具体实现。
pub trait SyncCipher: Send + Sync {
    fn encrypt(&self, plaintext: &[u8], sync_key: &[u8; 32]) -> Result<EncryptedPayload, AppError>;
    fn decrypt(&self, payload: &EncryptedPayload, sync_key: &[u8; 32])
        -> Result<Vec<u8>, AppError>;
}
