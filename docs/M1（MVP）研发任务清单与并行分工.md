# 个人提示词管理平台 M1（MVP）研发任务清单与并行分工

**文档版本**：V1.0
**日期**：2026-09-28
**状态**：草案
**关联文档**：《详细设计与开发排期计划》§2.2、《M0 研发落地任务清单与每日检查点》
**当前分支**：`m1-mvp`

---

## 0. 文档用途

M0 只验证了「React→IPC→Rust 回传」一条最小链路。本文档把 **M1（MVP，Day 1~3，约 18 小时）** 的功能点拆成**可多任务并行**的任务清单，解决“功能点没按任务切、无法并行开工”的问题。

并行机制一句话：**先把契约冻结（唯一串行前置），之后 A/B/C/D 四条轨道互不依赖、同时开工，最后集成收口。**

---

## 1. M1 完成定义（DoD）

全部为“是”才算 M1 完成，才打 `git tag m1`：

- [ ] 创建 → 存 SQLite → 列表加载 → 编辑 → 删除 → 搜索 → 一键复制，全链路用**真实数据**走通（不再用 mock）。
- [ ] `SqlitePromptRepository` 已实现 `PromptRepository` trait，业务层经 trait 访问，不直连数据库。
- [ ] IPC 命令全部与 `shared/api/commands.ts` 类型一一对应，`tsc --noEmit` 无错误。
- [ ] 列表页 / 编辑器 / 搜索框 / 一键复制 / 设置页五块 UI 可演示。
- [ ] `cargo clippy` 无 warning。
- ✅ 全部达成 → `git checkout main && git merge m1-mvp` 后打 `git tag m1`。

> 范围提醒：M1 用**明文 SQLite**（rusqlite `bundled`），**不做 SQLCipher 加密**。加密、密钥派生属于 M4。这是为了让 M1 聚焦“核心闭环”，不被安全复杂度阻塞。

---

## 2. 并行编排总览

```
         M0 基线 (main / m1-mvp @ 22221df)
                      │
                      ▼
        ┌─────────────────────────────┐
        │  Phase 0 · M1-P1 契约冻结    │  ← 唯一串行前置，1 人 + AI 一起做，约 1h
        │  (命令签名 + 模型字段 + TS 类型)│
        └─────────────────────────────┘
                      │ 冻结后放开，四轨并行 ↓
        ┌─────────────┼─────────────┬─────────────┐
        ▼             ▼             ▼             ▼
   ┌─────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐
   │ 轨道 A   │  │ 轨道 B    │  │ 轨道 C    │  │ 轨道 D    │
   │ 后端·存储 │  │ 后端·IPC  │  │ 前端·API  │  │ 前端·UI  │
   │ 4h      │  │ 3h       │  │ 2h       │  │ 7h       │
   └─────────┘  └──────────┘  └──────────┘  └──────────┘
        │             │             │             │
        └─────────────┴─────────────┴─────────────┘
                      │
                      ▼
        ┌─────────────────────────────┐
        │  Phase 5 · 集成收口 M1-I1/I2  │  ← 串行，约 1h
        │  (端到端联调 + 打 m1 tag)       │
        └─────────────────────────────┘
```

**并行可行性依据**：A/B/C/D 只通过“契约”交互，不直接依赖对方产物——
- B 依赖 `PromptRepository` trait（M0 已定义），不依赖 A 的具体实现 → B 可和 A 并行。
- C 依赖 TS 类型（commands.ts），不依赖 B 实现 → C 可并行。
- D 用 mock 数据先做 UI，C 就绪后接真实 store → D 可并行。

---

## 3. Phase 0 · 契约冻结（串行前置，唯一阻塞点）

> 此阶段**没完成前，四轨都不许开工**。做完后契约锁定，任何人改契约必须广播并等待，否则并行会互相破坏。

| ID | 任务 | 产出 | 验收 | 分工 |
|---|---|---|---|---|
| M1-P1 | 冻结 M1 IPC 契约：`PromptQuery`/`CreatePromptInput`/`UpdatePromptInput` 字段终版 + `get_prompts`/`create_prompt`/`update_prompt`/`delete_prompt`/`search_prompts` 命令签名 + `commands.ts` 类型与封装函数声明 + `Prompt` 模型补齐 `is_favorite`/`rating` 等列表字段 | 定稿 `core/src/models/*`、`ipc/src/commands/prompt.rs` 签名、`shared/api/commands.ts` | 前后端签名一致；`core` 编译通过；`tsc --noEmit` 通过 | **人定契约，AI 生成类型** |

锁定内容（写死在文档里，避免并行漂移）：

1. 命令清单（`#[tauri::command]` ↔ `commands.ts` 函数）：
   - `get_prompts(query: PromptQuery) -> Vec<Prompt>`
   - `create_prompt(input: CreatePromptInput) -> Prompt`
   - `update_prompt(id, input: UpdatePromptInput) -> Prompt`
   - `delete_prompt(id) -> ()`
   - `search_prompts(keyword) -> Vec<Prompt>`
2. `Prompt` 字段：`id/title/content/description/category/is_pinned/is_favorite/rating/created_at/updated_at/deleted_at`。
3. `PromptQuery` 字段：`category? / keyword? / is_pinned? / limit? / offset?`（MVP 先支持 `keyword` + `limit`，其余留空实现）。

---

## 4. 并行轨道 A · 后端存储（Rust storage crate）

> 独立轨道，文件只落在 `src-tauri/crates/storage/`，不碰 core/ipc。

| ID | 任务 | 产出 | 验收 | 依赖 | 分工 |
|---|---|---|---|---|---|
| M1-A1 | `Connection` 打开封装 + 迁移 SQL（建 `prompts` 表 + `prompts_fts` FTS5 虚表 + 索引） | `storage/src/sqlite/connection.rs`、`migrations/0001_init.sql` | 打开内存/文件 SQLite 成功，表结构可用 | P1 | AI |
| M1-A2 | `SqlitePromptRepository` 实现 `PromptRepository`（find_all/find_by_id/create/update/delete/search） | `storage/src/repository/sqlite_prompt_repository.rs` | 六个方法走通 CRUD + FTS 搜索 | A1 | 人 review 核心 SQL，AI 生成 |
| M1-A3 | repository 单测（`:memory:` 数据库） | `storage/tests/prompt_repository_test.rs` | `cargo test -p storage` 全绿 | A2 | AI |

---

## 5. 并行轨道 B · 后端 IPC + 装配（Rust ipc + src-tauri）

> 只依赖 trait，不依赖 A 实现。文件落在 `ipc/` 和 `src-tauri/src/lib.rs`。

| ID | 任务 | 产出 | 验收 | 依赖 | 分工 |
|---|---|---|---|---|---|
| M1-B1 | 五个命令实现：从 `State<AppState>` 取 repository 调 trait 方法 | `ipc/src/commands/prompt.rs` | 命令函数编译通过，签名与 P1 一致 | P1 | AI，人 review 签名 |
| M1-B2 | `AppState` 装配：构造 `SqlitePromptRepository` 注入 state，注册 `prompt` 命令 | `src-tauri/src/lib.rs` | `cargo build` 通过，`ping` 仍可用 | B1 | AI |
| M1-B3 | 装配后冒烟：`get_prompts` 空库返回 `[]` | 运行记录 | 命令可被 invoke 且返回正确 | B2 | AI |

---

## 6. 并行轨道 C · 前端 API + 状态（TS）

> 文件落在 `src/shared/api/commands.ts`（仅封装实现）、`src/features/prompt-library/api` 与 `stores`。

| ID | 任务 | 产出 | 验收 | 依赖 | 分工 |
|---|---|---|---|---|---|
| M1-C1 | `commands.ts` 完成五个命令的 `invoke` 封装 + 实体 TS 类型对齐 | `shared/api/commands.ts` | `tsc --noEmit` 通过，类型与 P1 一致 | P1 | AI |
| M1-C2 | `promptStore`（Zustand）：列表/当前项/loading/增删改查/搜索 action | `features/prompt-library/stores/promptStore.ts` | store action 能各自调用正确命令 | C1 | AI，人 review action 边界 |

---

## 7. 并行轨道 D · 前端 UI（TS/React）

> 用 mock 数据先做界面，C 完成后接真实 store。文件落在 `src/features/{prompt-library,prompt-editor,search,settings}` 与 `src/app`。

| ID | 任务 | 产出 | 验收 | 依赖 | 分工 |
|---|---|---|---|---|---|
| M1-D1 | 列表页：卡片 + 虚拟滚动 + 快速筛选（分类/置顶） | `features/prompt-library/ui/*` | 列表能渲染 mock 数据、滚动流畅 | P1 | AI |
| M1-D2 | 编辑器：Markdown 编辑 + 预览 + 自动保存（500ms 防抖）+ 元数据面板 | `features/prompt-editor/*` | 输入防抖保存，编辑/新建/删除走通 | D1 | AI |
| M1-D3 | 搜索框 + 一键复制（“复制原始”入口） | `features/search/*` + 复制逻辑 | 搜索联动列表，复制写入剪贴板 | D1 | AI |
| M1-D4 | 设置页骨架：主题切换/存储位置/清空数据/关于 | `features/settings/*` | 四个设置项可交互 | D1 | AI |

---

## 8. Phase 5 · 集成收口（串行）

| ID | 任务 | 产出 | 验收 | 依赖 | 分工 |
|---|---|---|---|---|---|
| M1-I1 | 端到端联调：UI 接真实 store → 真实 IPC → 真实 SQLite，全链路走一遍 | 联调记录 | DoD 第 1 条达成，`pnpm tauri dev` 可演示 | A3+B3+C2+D 全部 | **人主导 + AI 修复** |
| M1-I2 | 合入 main 并打 tag | `git tag m1` | 可回退到 m1 基线 | I1 | 人执行 |

---

## 9. 并行协作约束（务必遵守，否则并行互相破坏）

1. **文件归属唯一**：每个文件只归一条轨道改。跨轨文件（`commands.ts`、`trait`、模型）在 Phase 0 冻结后默认只读，需改动先广播。
2. **提交粒度**：每个 `M1-x` 一 commit，不攒大提交；四轨各自在 `m1-mvp` 上提交会冲突，建议**每轨一个 feature 分支**（`m1-mvp/a-storage`、`m1-mvp/b-ipc` 等），集成阶段再合并回 `m1-mvp`。
3. **契约变更广播**：Phase 0 后若必须改 `Prompt`/命令签名，先通知其余三轨，否则各自按旧契约写会白做。
4. **联调段人工主导**：跨轨接缝（store↔invoke↔命令↔repository）Debug 不裸交 AI，人先定位再让 AI 改。
5. **可回退**：Phase 0 完、四轨各自完、集成完，各打一个轻量 commit 作为回退点。

---

## 10. 与排期文档映射

| 本清单 | 排期 §2.2 | 说明 |
|---|---|---|
| Phase 0（P1） | — | 原排期缺失的“契约冻结”动作，补上作为并行前提 |
| 轨道 A | 2.2.1 数据层（4h） | 拆掉 SQLCipher，改成明文 SQLite（加密归 M4） |
| 轨道 B | 2.2.2 IPC 层（4h） | 拆为命令实现 + 装配，独立成轨 |
| 轨道 C | 2.2.2 / 2.2.3 | 前端 API 封装 + store，原排期与 UI 混在一起 |
| 轨道 D | 2.2.3（6h）+ 2.2.4（2h）+ 2.2.5（2h） | 四块 UI 合并为一条 UI 轨，内部再细分 |
| Phase 5 | — | 联调 + 打 m1 tag 的收口动作 |

---

**附：建议的并行开任务方式**

- 任务 1：Phase 0 契约冻结（人 + 1 AI，最快完成，是闸门）。
- 任务 2～5：轨道 A/B/C/D 各开一个 agent，等 Phase 0 完成后同时启动。
- 任务 6：集成收口（四轨汇合后，人主导）。