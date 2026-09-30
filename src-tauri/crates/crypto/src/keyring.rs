//! OS 钥匙串适配（M4-A4）：可选「自动解锁」Master Key 存取。
//!
//! 通过 [`SecretStore`] 抽象解耦，便于测试用内存实现 mock；生产用 `keyring` crate 落 OS 凭据库。

use prompt_core::AppError;

const KEYRING_SERVICE: &str = "prompt-manager";
const KEYRING_USER: &str = "master-key";

/// 秘密存储抽象，供 `CryptoVault` 注入，便于测试 mock。
pub trait SecretStore: Send + Sync {
    /// 读取秘密；不存在时返回 `None`。
    fn get(&self) -> Result<Option<Vec<u8>>, AppError>;
    /// 写入（覆盖）秘密。
    fn set(&self, secret: &[u8]) -> Result<(), AppError>;
    /// 删除秘密（不存在视为成功）。
    fn delete(&self) -> Result<(), AppError>;
}

/// 基于 `keyring` crate 的 OS 钥匙串实现（Windows 凭据管理器 / macOS 钥匙串 / Secret Service）。
pub struct KeychainStore {
    entry: keyring::Entry,
}

impl KeychainStore {
    pub fn new() -> Result<Self, AppError> {
        let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
            .map_err(|e| AppError::Crypto(e.to_string()))?;
        Ok(Self { entry })
    }
}

impl SecretStore for KeychainStore {
    fn get(&self) -> Result<Option<Vec<u8>>, AppError> {
        match self.entry.get_secret() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(AppError::Crypto(e.to_string())),
        }
    }

    fn set(&self, secret: &[u8]) -> Result<(), AppError> {
        self.entry
            .set_secret(secret)
            .map_err(|e| AppError::Crypto(e.to_string()))
    }

    fn delete(&self) -> Result<(), AppError> {
        match self.entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(AppError::Crypto(e.to_string())),
        }
    }
}
