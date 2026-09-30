use std::sync::Mutex;

use crypto::{CryptoVault, SecretStore};
use prompt_core::{AppError, VaultProvider};

/// 内存版秘密存储，用于 mock OS 钥匙串。
struct MockStore {
    data: Mutex<Option<Vec<u8>>>,
}

impl MockStore {
    fn new() -> Self {
        Self {
            data: Mutex::new(None),
        }
    }
}

impl SecretStore for MockStore {
    fn get(&self) -> Result<Option<Vec<u8>>, AppError> {
        Ok(self.data.lock().unwrap().clone())
    }

    fn set(&self, secret: &[u8]) -> Result<(), AppError> {
        *self.data.lock().unwrap() = Some(secret.to_vec());
        Ok(())
    }

    fn delete(&self) -> Result<(), AppError> {
        *self.data.lock().unwrap() = None;
        Ok(())
    }
}

#[test]
fn setup_then_master_key_available() {
    let v = CryptoVault::new();
    v.setup("p@ssw0rd").unwrap();
    assert_eq!(v.master_key().unwrap().len(), 32);
}

#[test]
fn setup_is_rejected_twice() {
    let v = CryptoVault::new();
    v.setup("pw").unwrap();
    assert!(v.setup("pw2").is_err());
}

#[test]
fn lock_clears_master_key() {
    let v = CryptoVault::new();
    v.setup("pw").unwrap();
    v.lock().unwrap();
    assert!(matches!(v.master_key(), Err(AppError::Crypto(_))));
}

#[test]
fn unlock_recovers_same_master_key() {
    let v = CryptoVault::new();
    v.setup("pw").unwrap();
    let before = v.master_key().unwrap();
    v.lock().unwrap();
    v.unlock("pw").unwrap();
    assert_eq!(v.master_key().unwrap(), before);
}

#[test]
fn unlock_with_wrong_password_fails() {
    let v = CryptoVault::new();
    v.setup("right").unwrap();
    v.lock().unwrap();
    assert!(matches!(v.unlock("wrong"), Err(AppError::Auth(_))));
}

#[test]
fn unlock_before_setup_fails() {
    let v = CryptoVault::new();
    assert!(v.unlock("pw").is_err());
}

#[test]
fn change_password_keeps_master_key() {
    let v = CryptoVault::new();
    v.setup("old").unwrap();
    let before = v.master_key().unwrap();

    v.change_password("old", "new").unwrap();
    assert_eq!(v.master_key().unwrap(), before);

    // 新口令可再解锁，旧口令失效，且数据密钥不变。
    v.lock().unwrap();
    assert!(matches!(v.unlock("old"), Err(AppError::Auth(_))));
    v.unlock("new").unwrap();
    assert_eq!(v.master_key().unwrap(), before);
}

#[test]
fn change_password_wrong_old_fails() {
    let v = CryptoVault::new();
    v.setup("correct").unwrap();
    assert!(matches!(
        v.change_password("wrong", "new"),
        Err(AppError::Auth(_))
    ));
}

#[test]
fn auto_unlock_reads_from_secret_store() {
    let v = CryptoVault::with_secret_store(Box::new(MockStore::new()));
    v.setup("pw").unwrap();
    let mk = v.master_key().unwrap();

    v.lock().unwrap();
    assert!(v.master_key().is_err());

    v.auto_unlock().unwrap();
    assert_eq!(v.master_key().unwrap(), mk);
}

#[test]
fn disable_auto_unlock_clears_secret_store() {
    let v = CryptoVault::with_secret_store(Box::new(MockStore::new()));
    v.setup("pw").unwrap();

    v.disable_auto_unlock().unwrap();
    v.lock().unwrap();
    // 钥匙串已清空，自动解锁将不恢复 Master Key。
    v.auto_unlock().unwrap();
    assert!(v.master_key().is_err());
}
