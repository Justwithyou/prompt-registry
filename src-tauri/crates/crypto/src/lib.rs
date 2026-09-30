//! 加密与密钥管理 / 同步加密管道（M4，对应架构方案 4.2、第 5 章）。
//!
//! - [`vault`]：Argon2id KEK 派生 + Master Key 生成 / 包装 / 轮换，`VaultProvider` 实现。
//! - [`cipher`]：AES-256-GCM + HKDF 同步加密管道，`SyncCipher` 实现。
//! - [`keyring`]：OS 钥匙串适配（可选自动解锁），`SecretStore` 抽象 + `keyring` 实现。

pub mod cipher;
pub mod keyring;
pub mod vault;

pub use cipher::AesGcmCipher;
pub use keyring::{KeychainStore, SecretStore};
pub use vault::CryptoVault;
