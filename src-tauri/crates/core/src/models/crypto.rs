use serde::{Deserialize, Serialize};

/// 同步加密数据包（`SyncCipher::encrypt` 产物，M4 分工清单 §3.2）。
///
/// 序列化契约：`rename_all = "camelCase"`。仅在 Rust 内部（加密/同步管道）流转，不直接跨 IPC。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncryptedPayload {
    /// 协议版本（当前 1），用于未来算法/格式演进。
    pub version: u32,
    /// HKDF 派生盐。
    pub salt: Vec<u8>,
    /// AES-256-GCM nonce。
    pub nonce: Vec<u8>,
    /// 密文（含 GCM 认证标签）。
    pub ciphertext: Vec<u8>,
}
