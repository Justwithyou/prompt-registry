//! 密钥管理层级（M4-A2）：Argon2id KEK 派生 + Master Key 生成 / 包装 / 轮换。
//!
//! Master Key 只存在于内存与 OS 钥匙串（可选），绝不落库明文；
//! 库落的包装信封（盐 + nonce + 密文）由上层（轨道 B / D）负责持久化。

use std::sync::Mutex;

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use prompt_core::{AppError, VaultEnvelope, VaultProvider};
use rand::rngs::OsRng;
use rand::RngCore;

use crate::keyring::SecretStore;

const MASTER_KEY_LEN: usize = 32;
const SALT_LEN: usize = 32;
const NONCE_LEN: usize = 12;
// Argon2id 安全参数（M4 分工清单 §3.1；与已加密数据兼容性绑定，不可私改）。
const ARGON2_M_COST: u32 = 65_536; // 64 MiB
const ARGON2_T_COST: u32 = 3;
const ARGON2_P_COST: u32 = 1;

fn crypto_err(e: impl std::fmt::Display) -> AppError {
    AppError::Crypto(e.to_string())
}

/// 包装后的 Master Key 信封：盐 + nonce + 密文。
struct Envelope {
    kek_salt: [u8; SALT_LEN],
    nonce: [u8; NONCE_LEN],
    ciphertext: Vec<u8>,
}

struct VaultState {
    envelope: Option<Envelope>,
    master_key: Option<[u8; MASTER_KEY_LEN]>,
}

/// 用 Argon2id 从口令派生 32 字节 KEK。
fn derive_kek(password: &str, salt: &[u8]) -> Result<[u8; 32], AppError> {
    let params =
        Params::new(ARGON2_M_COST, ARGON2_T_COST, ARGON2_P_COST, Some(32)).map_err(crypto_err)?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut out = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut out)
        .map_err(crypto_err)?;
    Ok(out)
}

/// 用 KEK 经 AES-256-GCM 包装 Master Key，返回 (nonce, 密文含标签)。
fn wrap(
    key: &[u8; 32],
    plaintext: &[u8; MASTER_KEY_LEN],
) -> Result<([u8; NONCE_LEN], Vec<u8>), AppError> {
    let mut nonce = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);
    let cipher = Aes256Gcm::new_from_slice(key).map_err(crypto_err)?;
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext.as_slice())
        .map_err(crypto_err)?;
    Ok((nonce, ciphertext))
}

/// 用 KEK 解包 Master Key。
fn unwrap_key(
    key: &[u8; 32],
    nonce: &[u8; NONCE_LEN],
    ciphertext: &[u8],
) -> Result<[u8; MASTER_KEY_LEN], AppError> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(crypto_err)?;
    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(crypto_err)?;
    plaintext
        .try_into()
        .map_err(|_| AppError::Crypto("Master Key 长度异常".to_string()))
}

/// Argon2id KEK + Master Key 包装 / 轮换的默认实现（`VaultProvider`）。
pub struct CryptoVault {
    state: Mutex<VaultState>,
    secret_store: Option<Box<dyn SecretStore>>,
}

impl CryptoVault {
    /// 无钥匙串（不使用自动解锁）。
    pub fn new() -> Self {
        Self {
            state: Mutex::new(VaultState {
                envelope: None,
                master_key: None,
            }),
            secret_store: None,
        }
    }

    /// 注入钥匙串，`setup` 时把 Master Key 写入以便自动解锁。
    pub fn with_secret_store(secret_store: Box<dyn SecretStore>) -> Self {
        Self {
            state: Mutex::new(VaultState {
                envelope: None,
                master_key: None,
            }),
            secret_store: Some(secret_store),
        }
    }

    /// 尝试用 OS 钥匙串中的 Master Key 自动解锁（无需口令）。
    pub fn auto_unlock(&self) -> Result<(), AppError> {
        let store = self
            .secret_store
            .as_ref()
            .ok_or_else(|| AppError::Crypto("未配置钥匙串".to_string()))?;
        if let Some(raw) = store.get()? {
            let mk: [u8; MASTER_KEY_LEN] = raw
                .try_into()
                .map_err(|_| AppError::Crypto("钥匙串中的 Master Key 长度异常".to_string()))?;
            let mut state = self.state.lock().map_err(|_| lock_poisoned())?;
            state.master_key = Some(mk);
        }
        Ok(())
    }

    /// 清除钥匙串保存的 Master Key（关闭自动解锁）。
    pub fn disable_auto_unlock(&self) -> Result<(), AppError> {
        if let Some(store) = self.secret_store.as_ref() {
            store.delete()?;
        }
        Ok(())
    }
}

fn lock_poisoned() -> AppError {
    AppError::Crypto("保险库状态锁已中毒".to_string())
}

impl Default for CryptoVault {
    fn default() -> Self {
        Self::new()
    }
}

impl VaultProvider for CryptoVault {
    fn setup(&self, password: &str) -> Result<(), AppError> {
        let master_key = {
            let mut state = self.state.lock().map_err(|_| lock_poisoned())?;
            if state.envelope.is_some() {
                return Err(AppError::Crypto("口令已设置".to_string()));
            }

            let mut kek_salt = [0u8; SALT_LEN];
            OsRng.fill_bytes(&mut kek_salt);
            let kek = derive_kek(password, &kek_salt)?;

            let mut master_key = [0u8; MASTER_KEY_LEN];
            OsRng.fill_bytes(&mut master_key);
            let (nonce, ciphertext) = wrap(&kek, &master_key)?;

            state.envelope = Some(Envelope {
                kek_salt,
                nonce,
                ciphertext,
            });
            state.master_key = Some(master_key);
            master_key
        };

        // 可选：写入 OS 钥匙串以支持自动解锁（仅存原始 Master Key）。
        if let Some(store) = self.secret_store.as_ref() {
            store.set(&master_key)?;
        }
        Ok(())
    }

    fn unlock(&self, password: &str) -> Result<(), AppError> {
        let (kek_salt, nonce, ciphertext) = {
            let state = self.state.lock().map_err(|_| lock_poisoned())?;
            let envelope = state
                .envelope
                .as_ref()
                .ok_or_else(|| AppError::Crypto("尚未设置口令".to_string()))?;
            (
                envelope.kek_salt,
                envelope.nonce,
                envelope.ciphertext.clone(),
            )
        };
        let kek = derive_kek(password, &kek_salt)?;
        let master_key = unwrap_key(&kek, &nonce, &ciphertext)
            .map_err(|_| AppError::Auth("口令错误".to_string()))?;

        let mut state = self.state.lock().map_err(|_| lock_poisoned())?;
        state.master_key = Some(master_key);
        Ok(())
    }

    fn lock(&self) -> Result<(), AppError> {
        let mut state = self.state.lock().map_err(|_| lock_poisoned())?;
        state.master_key = None;
        Ok(())
    }

    fn change_password(&self, old_pw: &str, new_pw: &str) -> Result<(), AppError> {
        let (old_salt, nonce, ciphertext, was_unlocked) = {
            let state = self.state.lock().map_err(|_| lock_poisoned())?;
            let envelope = state
                .envelope
                .as_ref()
                .ok_or_else(|| AppError::Crypto("尚未设置口令".to_string()))?;
            (
                envelope.kek_salt,
                envelope.nonce,
                envelope.ciphertext.clone(),
                state.master_key.is_some(),
            )
        };

        // 用旧口令解出 Master Key（校验口令正确且数据密钥不变）。
        let kek_old = derive_kek(old_pw, &old_salt)?;
        let recovered = unwrap_key(&kek_old, &nonce, &ciphertext)
            .map_err(|_| AppError::Auth("口令错误".to_string()))?;

        // 用新口令重新包装同一 Master Key。
        let mut new_salt = [0u8; SALT_LEN];
        OsRng.fill_bytes(&mut new_salt);
        let kek_new = derive_kek(new_pw, &new_salt)?;
        let (new_nonce, new_ciphertext) = wrap(&kek_new, &recovered)?;

        let mut state = self.state.lock().map_err(|_| lock_poisoned())?;
        state.envelope = Some(Envelope {
            kek_salt: new_salt,
            nonce: new_nonce,
            ciphertext: new_ciphertext,
        });
        // 数据密钥不变；保持原先的锁定 / 解锁状态。
        if was_unlocked {
            state.master_key = Some(recovered);
        }
        Ok(())
    }

    fn master_key(&self) -> Result<[u8; MASTER_KEY_LEN], AppError> {
        let state = self.state.lock().map_err(|_| lock_poisoned())?;
        state
            .master_key
            .ok_or_else(|| AppError::Crypto("保险库已锁定".to_string()))
    }

    fn export_envelope(&self) -> Option<VaultEnvelope> {
        let state = self.state.lock().ok()?;
        state.envelope.as_ref().map(|e| VaultEnvelope {
            kek_salt: e.kek_salt.to_vec(),
            nonce: e.nonce.to_vec(),
            ciphertext: e.ciphertext.clone(),
        })
    }

    fn restore_envelope(&self, envelope: VaultEnvelope) -> Result<(), AppError> {
        let mut state = self.state.lock().map_err(|_| lock_poisoned())?;
        if state.envelope.is_some() {
            return Ok(());
        }
        let kek_salt: [u8; SALT_LEN] = envelope
            .kek_salt
            .try_into()
            .map_err(|_| AppError::Crypto("信封盐长度异常".to_string()))?;
        let nonce: [u8; NONCE_LEN] = envelope
            .nonce
            .try_into()
            .map_err(|_| AppError::Crypto("信封 nonce 长度异常".to_string()))?;
        state.envelope = Some(Envelope {
            kek_salt,
            nonce,
            ciphertext: envelope.ciphertext,
        });
        Ok(())
    }

    fn set_auto_unlock(&self, enabled: bool) -> Result<(), AppError> {
        let store = self
            .secret_store
            .as_ref()
            .ok_or_else(|| AppError::Crypto("未配置 OS 钥匙串".to_string()))?;
        if enabled {
            let key = self.master_key()?;
            store.set(&key)
        } else {
            store.delete()
        }
    }
}
