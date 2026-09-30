//! 同步加密管道（M4-A3）：AES-256-GCM + HKDF，实现 `SyncCipher`。
//!
//! 每次加密使用随机盐（HKDF 派生消息密钥）+ 随机 nonce（AES-GCM），
//! 认证标签由 AES-GCM 自动附加到密文末尾，密文被篡改或密钥错误都会解密失败。

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use hkdf::Hkdf;
use prompt_core::{AppError, EncryptedPayload, SyncCipher};
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::Sha256;

const PAYLOAD_VERSION: u32 = 1;
const KEY_LEN: usize = 32;
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
/// HKDF domain separation（与已加密数据兼容性绑定，不可私改）。
const HKDF_INFO: &[u8] = b"prompt-manager/sync/v1";

/// 从 Master Key 派生同步密钥的 HKDF domain separation（与数据库密钥域隔离，不可私改）。
const SYNC_KEY_INFO: &[u8] = b"prompt-manager/sync-key/v1";

/// AES-256-GCM + HKDF 同步加密管道的默认实现。
#[derive(Debug, Default, Clone)]
pub struct AesGcmCipher;

fn crypto_err(e: impl std::fmt::Display) -> AppError {
    AppError::Crypto(e.to_string())
}

/// 从 `sync_key` 与随机盐经 HKDF-SHA256 派生 32 字节消息密钥。
fn derive_key(sync_key: &[u8; KEY_LEN], salt: &[u8]) -> Result<[u8; KEY_LEN], AppError> {
    let hkdf = Hkdf::<Sha256>::new(Some(salt), sync_key);
    let mut okm = [0u8; KEY_LEN];
    hkdf.expand(HKDF_INFO, &mut okm)
        .map_err(|_| AppError::Crypto("HKDF 派生消息密钥失败".to_string()))?;
    Ok(okm)
}

/// 从 Master Key 经 HKDF 派生独立于数据库密钥的同步密钥（M4-C，域隔离）。
///
/// 同步数据加密密钥不与 SQLCipher 数据库密钥共享，避免任一密钥泄露殃及另一侧。
pub fn derive_sync_key(master_key: &[u8; KEY_LEN]) -> [u8; KEY_LEN] {
    let hkdf = Hkdf::<Sha256>::new(None, master_key);
    let mut okm = [0u8; KEY_LEN];
    hkdf.expand(SYNC_KEY_INFO, &mut okm)
        .expect("HKDF 扩展固定长度输出必然成功");
    okm
}

impl SyncCipher for AesGcmCipher {
    fn encrypt(
        &self,
        plaintext: &[u8],
        sync_key: &[u8; KEY_LEN],
    ) -> Result<EncryptedPayload, AppError> {
        let mut salt = [0u8; SALT_LEN];
        let mut nonce = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut salt);
        OsRng.fill_bytes(&mut nonce);

        let key = derive_key(sync_key, &salt)?;
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(crypto_err)?;
        let ciphertext = cipher
            .encrypt(Nonce::from_slice(&nonce), plaintext)
            .map_err(crypto_err)?;

        Ok(EncryptedPayload {
            version: PAYLOAD_VERSION,
            salt: salt.to_vec(),
            nonce: nonce.to_vec(),
            ciphertext,
        })
    }

    fn decrypt(
        &self,
        payload: &EncryptedPayload,
        sync_key: &[u8; KEY_LEN],
    ) -> Result<Vec<u8>, AppError> {
        let key = derive_key(sync_key, &payload.salt)?;
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(crypto_err)?;
        cipher
            .decrypt(
                Nonce::from_slice(&payload.nonce),
                payload.ciphertext.as_ref(),
            )
            .map_err(crypto_err)
    }
}
