//! HLC 与字段级 LWW 合并的单元测试（M4-C4/C5）。

use std::cmp::Ordering;

use prompt_core::SyncLogEntry;
use sync::{compare_entries, merge_entries, Hlc};

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

#[test]
fn hlc_encode_parse_roundtrip() {
    let h = Hlc {
        wall_millis: 1_700_000_000_123,
        counter: 7,
    };
    let parsed = Hlc::parse(&h.encode()).unwrap();
    assert_eq!(parsed, h);
}

#[test]
fn hlc_tick_after_is_monotonic() {
    let mut h = Hlc {
        wall_millis: 100,
        counter: 0,
    };
    // 远端 wall 更大 → 追平并归零 counter。
    h.tick_after(200);
    assert_eq!(h.wall_millis, 200);
    assert_eq!(h.counter, 0);
    // 远端 wall 更小 → 本地 counter 自增（保证同一 wall 内单调）。
    h.tick_after(150);
    assert_eq!(h.wall_millis, 200);
    assert_eq!(h.counter, 1);
}

#[test]
fn compare_entries_orders_by_hlc_then_device() {
    let a = entry("prompt", "p1", "update", "{}", "100-0", "B");
    let b = entry("prompt", "p1", "update", "{}", "200-0", "A");
    assert_eq!(compare_entries(&b, &a), Ordering::Greater);

    // 同一 wall，counter 大者更新。
    let c = entry("prompt", "p1", "update", "{}", "100-2", "A");
    assert_eq!(compare_entries(&c, &a), Ordering::Greater);

    // wall/counter 全同，device_id 字典序打破。
    let d = entry("prompt", "p1", "update", "{}", "100-0", "A");
    assert_eq!(compare_entries(&a, &d), Ordering::Greater);
}

#[test]
fn merge_field_level_lww_unions_latest_fields() {
    // 设备 A 改 title+content（较早），设备 B 只改 content（较晚）。
    let local = entry(
        "prompt",
        "p1",
        "update",
        r#"{"title":"T1","content":"C1"}"#,
        "100-0",
        "A",
    );
    let remote = entry(
        "prompt",
        "p1",
        "update",
        r#"{"content":"C2"}"#,
        "200-0",
        "B",
    );

    let merged = merge_entries(&[local], &[remote]);
    assert_eq!(merged.len(), 1);
    let m = &merged[0];
    assert_eq!(m.operation, "update");

    let payload: serde_json::Value = serde_json::from_str(&m.payload).unwrap();
    // title 仅 A 写过 → 保留；content 双方都写但 B 更新 → 取 B。
    assert_eq!(payload["title"], "T1");
    assert_eq!(payload["content"], "C2");
}

#[test]
fn merge_delete_is_tombstone_when_last() {
    let local = entry("prompt", "p1", "update", r#"{"title":"T1"}"#, "100-0", "A");
    let remote = entry("prompt", "p1", "delete", "{}", "200-0", "B");

    let merged = merge_entries(&[local], &[remote]);
    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].operation, "delete");
}

#[test]
fn merge_delete_is_revived_by_later_update() {
    let remote = entry("prompt", "p1", "delete", "{}", "100-0", "B");
    let local = entry("prompt", "p1", "update", r#"{"title":"T2"}"#, "200-0", "A");

    let merged = merge_entries(&[local], &[remote]);
    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].operation, "update");
}

#[test]
fn merge_version_entries_are_appended_not_merged() {
    let v1 = entry(
        "version",
        "p1",
        "create",
        r#"{"version":"0.1.0"}"#,
        "100-0",
        "A",
    );
    let v2 = entry(
        "version",
        "p1",
        "create",
        r#"{"version":"0.2.0"}"#,
        "200-0",
        "B",
    );

    let merged = merge_entries(std::slice::from_ref(&v1), std::slice::from_ref(&v2));
    // 追加式：两条历史版本都保留。
    assert_eq!(merged.len(), 2);
}

#[test]
fn merge_skips_deleted_entity_when_no_entries() {
    // 空输入不产出。
    assert!(merge_entries(&[], &[]).is_empty());
}
