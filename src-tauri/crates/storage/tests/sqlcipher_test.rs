//! M4-B3 冒烟：SQLCipher 明文→加密迁移与解锁语义。
//!
//! 仅当 `storage` 以 `bundled-sqlcipher` 编译时本组测试才有加密语义；
//! 运行需具备 SQLCipher 依赖的加密库（OpenSSL），即 `cargo test -p storage` 在装好
//! 该依赖的环境下执行（详见 M4 分工清单 §10.3 跨平台风险说明）。

use std::path::{Path, PathBuf};

use storage::Connection;

const KEY: [u8; 32] = [0x42; 32];
const WRONG_KEY: [u8; 32] = [0x24; 32];

/// 生成唯一临时库文件路径（按纳秒时间戳去重，避免多测试串扰）。
fn temp_db_path(tag: &str) -> PathBuf {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("系统时间早于 UNIX 纪元")
        .as_nanos();
    std::env::temp_dir().join(format!("prompt_mgr_m4b3_{tag}_{unique}.db"))
}

/// 以明文方式建表并插入一行数据，随后 rekey 为加密库。
fn seed_and_rekey(path: &Path, key: &[u8; 32]) {
    let mut conn = Connection::open(path).expect("打开明文库失败");
    conn.inner()
        .execute_batch("CREATE TABLE t (id INTEGER PRIMARY KEY, v TEXT);")
        .expect("建表失败");
    conn.inner()
        .execute("INSERT INTO t (v) VALUES ('secret')", [])
        .expect("插入失败");
    conn.rekey(key).expect("rekey 失败");
}

#[test]
fn rekey_migrates_plaintext_to_encrypted_preserving_data() {
    let path = temp_db_path("rekey");
    seed_and_rekey(&path, &KEY);

    let conn = Connection::open_with_key(&path, &KEY).expect("带密钥打开失败");
    let v: String = conn
        .inner()
        .query_row("SELECT v FROM t WHERE id = 1", [], |row| row.get(0))
        .expect("读取失败");
    assert_eq!(v, "secret", "rekey 后旧数据应保留");

    let _ = std::fs::remove_file(&path);
}

#[test]
fn encrypted_db_read_fails_without_key() {
    let path = temp_db_path("locked");
    seed_and_rekey(&path, &KEY);

    // 无密钥（等价于锁定态）打开：读操作失败。
    let conn = Connection::open(&path).expect("打开连接失败");
    let res = conn
        .inner()
        .query_row("SELECT v FROM t WHERE id = 1", [], |row| {
            row.get::<_, String>(0)
        });
    assert!(res.is_err(), "无密钥读取加密库应失败");

    // 错误密钥同样失败。
    let conn = Connection::open_with_key(&path, &WRONG_KEY).expect("打开连接失败");
    let res = conn
        .inner()
        .query_row("SELECT v FROM t WHERE id = 1", [], |row| {
            row.get::<_, String>(0)
        });
    assert!(res.is_err(), "错误密钥读取加密库应失败");

    let _ = std::fs::remove_file(&path);
}

#[test]
fn encrypted_db_read_succeeds_with_correct_key() {
    let path = temp_db_path("unlocked");
    seed_and_rekey(&path, &KEY);

    let conn = Connection::open_with_key(&path, &KEY).expect("带密钥打开失败");
    let v: String = conn
        .inner()
        .query_row("SELECT v FROM t WHERE id = 1", [], |row| row.get(0))
        .expect("读取失败");
    assert_eq!(v, "secret", "正确密钥应能读取加密库");

    let _ = std::fs::remove_file(&path);
}
