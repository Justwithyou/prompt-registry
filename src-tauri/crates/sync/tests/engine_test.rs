//! 同步引擎端到端测试（M4-C5）：加密往返、双设备拉取、冲突合并。

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use prompt_core::{
    AppError, EncryptedPayload, RemoteStore, SyncCipher, SyncEngine, SyncLogEntry, SyncLogStore,
};
use sync::Engine;

fn entry(
    entity_type: &str,
    entity_id: &str,
    operation: &str,
    payload: &str,
    hlc: &str,
    device_id: &str,
) -> SyncLogEntry {
    SyncLogEntry {
        id: entity_id.to_string(),
        entity_type: entity_type.to_string(),
        entity_id: entity_id.to_string(),
        operation: operation.to_string(),
        payload: payload.to_string(),
        hlc: hlc.to_string(),
        device_id: device_id.to_string(),
    }
}

/// 测试用 XOR 加密管道：可逆往返，避免依赖 crypto crate（加密原语归轨道 A）。
struct XorCipher;

impl SyncCipher for XorCipher {
    fn encrypt(&self, plaintext: &[u8], key: &[u8; 32]) -> Result<EncryptedPayload, AppError> {
        let ciphertext = plaintext
            .iter()
            .zip(key.iter().cycle())
            .map(|(a, b)| a ^ b)
            .collect();
        Ok(EncryptedPayload {
            version: 1,
            salt: vec![0u8; 16],
            nonce: vec![0u8; 12],
            ciphertext,
        })
    }

    fn decrypt(&self, payload: &EncryptedPayload, key: &[u8; 32]) -> Result<Vec<u8>, AppError> {
        Ok(payload
            .ciphertext
            .iter()
            .zip(key.iter().cycle())
            .map(|(a, b)| a ^ b)
            .collect())
    }
}

/// 测试用内存变更日志仓库。
#[derive(Default)]
struct MemStore {
    pending: Mutex<Vec<SyncLogEntry>>,
    applied: Mutex<Vec<SyncLogEntry>>,
}

impl MemStore {
    fn seed(&self, entries: Vec<SyncLogEntry>) {
        *self.pending.lock().unwrap() = entries;
    }

    fn applied_snapshot(&self) -> Vec<SyncLogEntry> {
        self.applied.lock().unwrap().clone()
    }
}

impl SyncLogStore for MemStore {
    fn record(
        &self,
        entity_type: &str,
        entity_id: &str,
        operation: &str,
        payload: &str,
    ) -> Result<(), AppError> {
        let entry = SyncLogEntry {
            id: entity_id.to_string(),
            entity_type: entity_type.to_string(),
            entity_id: entity_id.to_string(),
            operation: operation.to_string(),
            payload: payload.to_string(),
            hlc: "0-0".to_string(),
            device_id: "test".to_string(),
        };
        self.pending.lock().unwrap().push(entry);
        Ok(())
    }

    fn pending(&self) -> Result<Vec<SyncLogEntry>, AppError> {
        Ok(self.pending.lock().unwrap().clone())
    }

    fn apply(&self, entries: &[SyncLogEntry]) -> Result<(), AppError> {
        self.applied.lock().unwrap().extend_from_slice(entries);
        Ok(())
    }

    fn mark_synced(&self, entries: &[SyncLogEntry]) -> Result<(), AppError> {
        let mut pending = self.pending.lock().unwrap();
        pending.retain(|e| !entries.iter().any(|x| x.id == e.id));
        Ok(())
    }
}

/// 测试用内存远端（WebDAV 契约的最小实现）。
#[derive(Default)]
struct MemRemote {
    files: Mutex<HashMap<String, Vec<u8>>>,
}

impl RemoteStore for MemRemote {
    fn put(&self, path: &str, data: &[u8]) -> Result<(), AppError> {
        self.files
            .lock()
            .unwrap()
            .insert(path.to_string(), data.to_vec());
        Ok(())
    }

    fn get(&self, path: &str) -> Result<Vec<u8>, AppError> {
        self.files
            .lock()
            .unwrap()
            .get(path)
            .cloned()
            .ok_or_else(|| AppError::NotFound(format!("远端文件不存在: {path}")))
    }

    fn propfind(&self, _path: &str) -> Result<Vec<String>, AppError> {
        Ok(self.files.lock().unwrap().keys().cloned().collect())
    }

    fn mkcol(&self, _path: &str) -> Result<(), AppError> {
        Ok(())
    }
}

fn make_engine(store: Arc<MemStore>, device_id: &str) -> Engine {
    let engine = Engine::new(Arc::new(XorCipher), store, device_id.to_string());
    engine.set_sync_key(Some([7u8; 32])).unwrap();
    engine
}

#[test]
fn first_sync_pushes_local_and_clears_pending() {
    let store = Arc::new(MemStore::default());
    store.seed(vec![
        entry("prompt", "p1", "update", r#"{"title":"A"}"#, "100-0", "A"),
        entry("prompt", "p2", "update", r#"{"title":"B"}"#, "101-0", "A"),
    ]);
    let engine = make_engine(store.clone(), "A");
    let remote = MemRemote::default();

    let res = engine.sync_once(&remote).unwrap();
    assert_eq!(res.pushed, 2);
    assert_eq!(res.pulled, 0);
    assert_eq!(res.merged, 0);
    // 推送成功后本地 pending 清空。
    assert!(store.pending().unwrap().is_empty());
}

#[test]
fn second_device_pulls_and_applies() {
    let a_store = Arc::new(MemStore::default());
    let e1 = entry(
        "prompt",
        "p1",
        "update",
        r#"{"title":"A","content":"c1"}"#,
        "100-0",
        "A",
    );
    let e2 = entry("prompt", "p2", "update", r#"{"title":"B"}"#, "101-0", "A");
    a_store.seed(vec![e1, e2]);
    let remote = MemRemote::default();
    make_engine(a_store.clone(), "A")
        .sync_once(&remote)
        .unwrap();

    // 设备 B 空仓库，从同一远端拉取。
    let b_store = Arc::new(MemStore::default());
    let res = make_engine(b_store.clone(), "B")
        .sync_once(&remote)
        .unwrap();
    assert_eq!(res.pulled, 2);
    assert_eq!(res.merged, 2);
    assert_eq!(res.pushed, 0);

    let applied = b_store.applied_snapshot();
    assert_eq!(applied.len(), 2);
    // 加密→解密往返后 payload 还原（XOR 可逆）。
    let p1 = applied.iter().find(|e| e.entity_id == "p1").unwrap();
    let payload: serde_json::Value = serde_json::from_str(&p1.payload).unwrap();
    assert_eq!(payload["title"], "A");
    assert_eq!(payload["content"], "c1");
}

#[test]
fn conflicting_fields_merge_by_lww_across_devices() {
    // 设备 A 改 title，设备 B 改 content；两者先后同步应字段级合并。
    let remote = MemRemote::default();

    let a_store = Arc::new(MemStore::default());
    a_store.seed(vec![entry(
        "prompt",
        "p1",
        "update",
        r#"{"title":"from-A"}"#,
        "100-0",
        "A",
    )]);
    make_engine(a_store.clone(), "A")
        .sync_once(&remote)
        .unwrap();

    let b_store = Arc::new(MemStore::default());
    b_store.seed(vec![entry(
        "prompt",
        "p1",
        "update",
        r#"{"content":"from-B"}"#,
        "200-0",
        "B",
    )]);
    make_engine(b_store.clone(), "B")
        .sync_once(&remote)
        .unwrap();

    let applied = b_store.applied_snapshot();
    assert_eq!(applied.len(), 1);
    let payload: serde_json::Value = serde_json::from_str(&applied[0].payload).unwrap();
    assert_eq!(payload["title"], "from-A");
    assert_eq!(payload["content"], "from-B");

    // 远端再拉回设备 A，应同样收敛到合并后状态。
    let res = make_engine(a_store.clone(), "A")
        .sync_once(&remote)
        .unwrap();
    assert!(res.merged > 0);
    let a_applied = a_store.applied_snapshot();
    let back: serde_json::Value = serde_json::from_str(&a_applied.last().unwrap().payload).unwrap();
    assert_eq!(back["title"], "from-A");
    assert_eq!(back["content"], "from-B");
}

#[test]
fn empty_sync_reports_zero() {
    let store = Arc::new(MemStore::default());
    let engine = make_engine(store, "A");
    let remote = MemRemote::default();
    let res = engine.sync_once(&remote).unwrap();
    assert_eq!((res.pushed, res.pulled, res.merged), (0, 0, 0));
}
