//! HLC（混合逻辑时钟）+ 字段级 LWW 合并（M4-C4）。
//!
//! - [`Hlc`]：`<wall_millis>-<counter>` 编码，比较时 wall 优先、counter 次之，同一
//!   时刻用 `device_id` 字典序做全序打破，保证跨设备确定性。
//! - [`merge_entries`]：对同一实体的多设备变更做**字段级 LWW** 合并——payload 为 JSON
//!   字段对象的条目按 HLC 升序逐字段覆盖，`delete` 为墓碑（末笔删除则实体删除）；
//!   版本历史（`entity_type == "version"`）采用**追加式**保留，不覆盖、不合并。

use std::cmp::Ordering;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use prompt_core::{AppError, SyncLogEntry};

/// 追加式保留的实体类型：版本历史不可变，同步时只追加不合并。
const APPEND_ONLY_ENTITY: &str = "version";

fn sync_err(msg: impl Into<String>) -> AppError {
    AppError::Sync(msg.into())
}

/// HLC 时间戳：`<wall_millis>-<counter>`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hlc {
    pub wall_millis: i64,
    pub counter: u32,
}

impl Hlc {
    /// 生成当前时刻的 HLC（wall 取 Unix 毫秒，counter 归零）。
    pub fn now() -> Self {
        let wall_millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        Self {
            wall_millis,
            counter: 0,
        }
    }

    /// 观察到远端时间后推进本地时钟，保证严格单调（Lamport 混合逻辑语义）。
    ///
    /// 远端 wall 更大则追平；否则本地 counter 自增。
    pub fn tick_after(&mut self, observed_wall: i64) {
        if observed_wall > self.wall_millis {
            self.wall_millis = observed_wall;
            self.counter = 0;
        } else {
            self.counter += 1;
        }
    }

    /// 编码为 `SyncLogEntry::hlc` 字符串。
    pub fn encode(&self) -> String {
        format!("{}-{}", self.wall_millis, self.counter)
    }

    /// 解析 `SyncLogEntry::hlc` 字符串。
    pub fn parse(s: &str) -> Result<Self, AppError> {
        let (w, c) = s
            .split_once('-')
            .ok_or_else(|| sync_err("HLC 编码格式非法（缺少分隔符）"))?;
        let wall_millis = w.parse::<i64>().map_err(|_| sync_err("HLC wall 非法"))?;
        let counter = c.parse::<u32>().map_err(|_| sync_err("HLC counter 非法"))?;
        Ok(Self {
            wall_millis,
            counter,
        })
    }
}

/// 比较两个 sync 条目新旧：HLC（wall 优先、counter 次之），平手以 device_id 字典序打破。
pub fn compare_entries(a: &SyncLogEntry, b: &SyncLogEntry) -> Ordering {
    let ha = Hlc::parse(&a.hlc).unwrap_or(Hlc {
        wall_millis: 0,
        counter: 0,
    });
    let hb = Hlc::parse(&b.hlc).unwrap_or(Hlc {
        wall_millis: 0,
        counter: 0,
    });
    (ha.wall_millis, ha.counter)
        .cmp(&(hb.wall_millis, hb.counter))
        .then_with(|| a.device_id.cmp(&b.device_id))
}

/// 字段级 LWW 合并：将本地与远端条目按 `entity_id` 分组，输出合并后的确定性结果集。
///
/// - 普通实体（prompt/category/tag/project）：按 HLC 升序逐条覆盖 `payload` 中的字段，
///   末尾 `delete` 生成墓碑条目，否则生成 `update`（字段为合并后的并集）。
/// - 版本历史（`entity_type == "version"`）：全量追加，保持不可变日志。
pub fn merge_entries(local: &[SyncLogEntry], remote: &[SyncLogEntry]) -> Vec<SyncLogEntry> {
    let mut groups: HashMap<String, Vec<SyncLogEntry>> = HashMap::new();
    for entry in local.iter().chain(remote) {
        groups
            .entry(entry.entity_id.clone())
            .or_default()
            .push(entry.clone());
    }

    let mut out = Vec::new();
    for (_entity_id, mut entries) in groups {
        // 追加式实体：全量保留，不做字段合并。
        if entries.iter().any(|e| e.entity_type == APPEND_ONLY_ENTITY) {
            entries.sort_by(compare_entries);
            out.extend(entries);
            continue;
        }

        entries.sort_by(compare_entries);
        out.extend(merge_mutable_group(&entries));
    }
    out
}

/// 合并单个可变实体的条目序列（已按 HLC 升序）。
fn merge_mutable_group(entries: &[SyncLogEntry]) -> Vec<SyncLogEntry> {
    let mut fields: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    let mut deleted = false;

    for entry in entries {
        if entry.operation == "delete" {
            fields.clear();
            deleted = true;
            continue;
        }
        // create / update：payload 为 `{"字段": 值, ...}`，逐字段覆盖。
        if let Ok(serde_json::Value::Object(map)) = serde_json::from_str(&entry.payload) {
            for (k, v) in map {
                fields.insert(k, v);
            }
        }
        deleted = false;
    }

    let Some(tip) = entries.last() else {
        return Vec::new();
    };

    let operation = if deleted { "delete" } else { "update" };
    let mut merged = SyncLogEntry {
        id: tip.entity_id.clone(),
        entity_type: tip.entity_type.clone(),
        entity_id: tip.entity_id.clone(),
        operation: operation.to_string(),
        payload: serde_json::Value::Object(fields).to_string(),
        hlc: tip.hlc.clone(),
        device_id: tip.device_id.clone(),
    };
    // 墓碑的 payload 无意义，统一留空对象以保持确定性输出。
    if deleted {
        merged.payload = "{}".to_string();
    }
    vec![merged]
}
