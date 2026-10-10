use serde::{Deserialize, Serialize};

/// 包装 Master Key 的持久化信封（盐 / nonce / 密文）。
///
/// 不含明文口令与 Master Key；解包需由口令派生 KEK 后 AES-GCM 解密。
/// 由 crypto 层生成、装配层写入数据目录 `vault.json`，供重启后解锁。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct VaultEnvelope {
    /// Argon2id 派生 KEK 所用盐。
    pub kek_salt: Vec<u8>,
    /// AES-256-GCM 随机 nonce（12 字节）。
    pub nonce: Vec<u8>,
    /// 包装后的密文（含 GCM 认证标签）。
    pub ciphertext: Vec<u8>,
}
