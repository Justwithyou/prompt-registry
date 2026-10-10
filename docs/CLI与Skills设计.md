# 个人提示词全流程管理平台 CLI 与 Skills 输出能力设计方案

> 状态：设计草案（待评审）
> 关联里程碑：M5 收口后的能力延伸
> 版本：v0.1

## 1. 背景与目标

平台已完成 M1–M5 收口，具备提示词 / 分类 / 标签 / 项目 / 模板 / 版本 / 加密 / 同步的全链路能力，核心资产沉淀在本地 SQLite（`prompt.db`）中。当前这些资产只能"留在应用内"被浏览、编辑、检索，缺乏"带出去"的通道。

本方案为平台增加两类**输出能力**，让沉淀的提示词资产可被外部 Agent / 脚本直接消费：

1. **Skills 输出**：把单条或一批提示词导出为符合规范（`SKILL.md`）的技能目录，供 Claude Code、Trae、Codex 等 Agent 直接加载为可调用技能。
2. **CLI 输出**：提供一个独立的命令行二进制，复用同一套存储与渲染内核，支持查询、导出、模板渲染，便于脚本化与 CI/工具链集成。

两条通道共享同一个**导出内核**，避免逻辑分叉。

## 2. 范围与粒度（已与用户确认）

输出/导出的粒度支持以下三种（多选结论）：

| 粒度 | 说明 |
| --- | --- |
| 单条提示词 | 导出指定一条提示词 |
| 模板渲染后成品 | 对含 `{{变量}}` 的模板提示词填值渲染后输出成品 |
| 按分类 / 标签 / 项目批量 | 按组织层维度一次性导出满足条件的全部提示词 |

触发方式：**先以应用内 UI 为主**（导出对话框 / 按钮）；CLI 作为同一能力在无界面环境下的等价入口（本轮用户未将"命令行触发"列为首要，故 CLI 定位为"同源辅助通道"，不改变 UI 优先的落地顺序）。

## 3. 总体设计

### 3.1 单一导出内核

```
           ┌────────────────────────────────────────────────┐
           │            ExportService（core 新增）          │
           │  读数据：PromptRepository / CategoryRepository   │
           │           TagRepository / ProjectRepository      │
           │  拼装  ：Prompt + 关系（分类/标签/项目）          │
           │  渲染  ：RenderEngine（复用 template crate）      │
           │  序列化：SkillWriter / MarkdownWriter / JsonWriter│
           └───────────────┬────────────────────────────────┘
                           │
        ┌──────────────────┴──────────────────┐
        │                                     │
   ┌────▼─────┐                         ┌─────▼─────┐
   │   UI 导出 │                         │  CLI 导出 │
   │ (新增 IPC)│                         │ (cli crate)│
   └──────────┘                         └───────────┘
```

- 数据读取：复用 `core` 的四个 repository trait（含 `PromptRepository::get_relations`）读取关系。
- 模板渲染：复用 `template::extract_variables` 提取变量，新增 `render(content, vars)` 做回填（见 §7）。
- 序列化：新增纯函数式 writer，输入为内部 `ExportItem` 结构，输出为 SKILL.md / Markdown / JSON 文本。

### 3.2 内部中间表示（IR）

新增 `core::export` 模块，定义与存储解耦的中间结构：

```rust
pub struct ExportItem {
    pub title: String,
    pub description: Option<String>,
    pub content: String,        // 渲染后正文（或原文，视是否渲染）
    pub rendered: bool,
    pub category_path: Vec<String>, // 如 ["工作", "周报"]
    pub tags: Vec<String>,          // 已展开层级显示名
    pub projects: Vec<String>,
    pub variables: Vec<VariableDefinition>,
    pub meta: ExportMeta,       // rating / is_pinned / is_favorite / created_at 等
}
```

序列化层只消费 `ExportItem`，与 SQLite 字段解耦，便于后续替换存储或输出目标。

## 4. 新增 crate：`cli`

现有 Tauri 应用的分层（`core` / `storage` / `template` / `ipc` / `crypto` / `sync` / `versioning`）已把业务与 Tauri 运行时解耦。新增一个**不依赖 Tauri 运行时**的 `cli` crate，直接复用 `core` + `storage` + `template`：

```
src-tauri/crates/cli/
├── Cargo.toml
└── src/
    ├── main.rs          # clap 命令入口
    ├── args.rs          # 子命令与参数定义
    └── export.rs        # 把 ExportService 接到命令行输出
```

要点：

- 用 `clap` 解析子命令，二进制名为 `prompt-manager`。
- 只读方式打开 `prompt.db`（复用 `storage::Db::open`），不改写数据。
- 数据目录解析与 Tauri 保持一致（`app_data_dir()/prompt.db`），并提供 `--db <path>` 显式覆盖，便于脚本指向备份库。
- 未来可独立打包（`cargo build -p prompt-manager --release`），也可作为 Tauri 的 sidecar；本轮仅需 `cargo run -p` 可跑通验证。

## 5. CLI 子命令详设

### 5.1 `list` —— 查询列表

```
prompt-manager list
    [--category <name|id>]      # 按分类（名或 id）过滤，可多次
    [--tag <name|id>]           # 按标签过滤，可多次
    [--project <name|id>]       # 按项目过滤
    [--keyword <kw>]
    [--format table|json]       # 默认 table
```

输出示例（table）：

```
ID       标题            分类          标签            项目
a1b2c3   周报模板        工作/周报     模板,办公      内部流程
```

### 5.2 `get` —— 读取单条

```
prompt-manager get <id>
    [--format md|json|skill]    # skill 等价 export_skill 单条输出
```

### 5.3 `export` —— 批量导出（分类/标签/项目维度）

```
prompt-manager export
    --category <name|id> | --tag <name|id> | --project <name|id>
    [--format skill|md|json]     # 默认 skill
    [--out <dir>]                # 默认 ./prompt-export
    [--render]                   # 使用变量 default_value 渲染（无默认值的变量留空）
```

### 5.4 `run` —— 模板渲染后成品

```
prompt-manager run <id>
    [--vars name=value ...]      # 变量填值；未提供时回退 default_value
    [--out <file>]               # 缺省输出到 stdout
```

对应 §2 的"模板渲染后成品"粒度。

### 5.5 参数 → 查询映射

CLI 的过滤参数直接映射到现有 `PromptQuery`（`category_id` / `tag_ids` / `project_id` / `keyword` / `sort_by`），复用 `PromptRepository::find_all`，不新增存储实现。

## 6. Skills 输出（SKILL.md）规范

### 6.1 目录与文件结构

批量导出时，每个技能一个独立目录，便于 Agent 直接引用：

```
prompt-export/
├── 周报模板/
│   └── SKILL.md
├── 客服-退款话术/
│   └── SKILL.md
└── ...
```

目录名 = 标题的 slug 化结果（去非法字符、限长、冲突加数字后缀）。

### 6.2 SKILL.md 结构

```markdown
---
name: 周报模板
description: 生成周报的提示词模板，{{变量}} 已填值或保留占位。
category: 工作/周报
tags: [模板, 办公]
projects: [内部流程]
metadata:
  rating: 5
  pinned: true
  favorite: false
  source: <prompt-id>
  generated_at: 2026-10-08T12:00:00Z
---

# 周报模板

（渲染后的正文，或保留 {{变量}} 占位并附变量说明表）
```

- `name` / `description` 为 Agent 加载技能所需的最小识别信息。
- 分类、标签、项目编码进 frontmatter，实现"技能即资产、资产带组织"。
- 含变量时，正文末尾附带一张"变量说明表"（变量名、默认值、可选值、说明），供 Agent 理解可参数化点。

### 6.3 变量说明表（含变量时附加）

```markdown
## 变量

| 变量 | 默认值 | 可选值 | 说明 |
| --- | --- | --- | --- |
| city | 上海 | - | 汇报城市 |
```

## 7. 模板渲染能力（复用并扩展 `template`）

现状：`template` crate 仅提供 `extract_variables(content) -> Vec<VariableDefinition>`（见 `src-tauri/crates/template/src/extractor.rs`，正则 `\{\{(\w+)\}\}`），只提取、不渲染。

本方案在 `template` crate 内新增一个渲染函数（保持现有签名不动）：

```rust
/// 用 vars 回填 content 中的 {{name}}；vars 未覆盖的变量：
/// 有 default_value 则用默认值，否则保留原占位。
pub fn render(content: &str, vars: &HashMap<String, String>) -> String;
```

数据流：

```
Prompt.content
   └─ extract_variables ──> Vec<VariableDefinition>（UI/CLI 展示可填项）
   └─ render(content, vars 合并 default_value) ──> 成品文本
```

变量值来源优先级：**用户显式输入（UI 表单 / CLI `--vars`）> `default_value` > 保留占位**。

## 8. 应用内 UI（导出入口）

以 UI 优先的触发方式，新增以下入口（不改变现有页面结构）：

| 位置 | 入口 | 行为 |
| --- | --- | --- |
| 提示词卡片 / 详情 | 「导出」按钮 | 单条导出：选格式 + 变量填值 |
| 分类 / 标签 / 项目管理页 | 「批量导出」按钮 | 按当前维度批量导出 |
| 提示词编辑页 | 「渲染预览」 | 复用现有变量填值 UI 做成品预览 |

导出对话框统一承载：格式选择（SKILL.md / Markdown / JSON）、变量填值（模板渲染）、目标目录（通过 Tauri dialog 选择）。

## 9. 新增前后端契约（IPC 命令）

在 `ipc` crate 新增命令，注册到 `src-tauri/src/lib.rs`：

| 命令 | 签名（要点） | 说明 |
| --- | --- | --- |
| `render_prompt` | `(id, vars: HashMap<String,String>) -> String` | 单条模板渲染 |
| `export_prompts` | `(selector, format, out_dir, render) -> Vec<ExportResult>` | 批量/单条导出到目录 |
| `export_skill` | `(id, out_dir) -> String` | 单条导出为 SKILL.md 目录，返回路径 |

`selector` 为联合类型：`{ ids: Vec<String> }` 或 `{ category_id }` / `{ tag_ids }` / `{ project_id }`，与 §2 粒度一一对应。

前端在 `src/shared/api/commands.ts` 增加对应 `invoke` 封装，供导出对话框调用。

## 10. 落地步骤拆解（阶段化）

| 阶段 | 内容 | 验证 |
| --- | --- | --- |
| P0 | `template::render` + 单测；`export` crate 承载 IR + 三种 writer | `cargo test` |
| P1 | `cli` crate 骨架 + `list` / `get` 子命令 | `cargo run -p prompt-cli -- list` |
| P2 | Skill 生成器 + 导出内核 wiring | 导出单条 SKILL.md 抽查 |
| P3 | CLI `export` / `run` + 应用内导出对话框与 IPC | 端到端导出 + UI 触发 |
| P4 | 批量导出命名/去重规约、变量说明表、测试覆盖 | 按分类/标签/项目批量导出 |

## 11. 决策记录（已确认）

| # | 决策点 | 结论 |
| --- | --- | --- |
| 1 | CLI 打包形态 | 仅开发可用（`cargo run -p`）；本轮不打包独立二进制、不塞 sidecar |
| 2 | 加密（vault 锁定态）导出 | 锁定态拒绝导出，需先解锁（复用 `ensure_unlocked` 网关语义） |
| 3 | SKILL.md frontmatter 兼容 | 标准字段（`name`/`description`）+ 自定义 `metadata`（分类/标签/项目/评分） |
| 4 | 批量导出目录结构 | 扁平目录 + frontmatter 编码组织信息 |

落地约束：

- `ExportService` 与导出类 IPC 命令（`export_prompts` / `export_skill` / `render_prompt`）在入口处校验 `ensure_unlocked()`，锁定态返回统一错误，不导出加密资产。
- CLI 形态保持"开发可用"：`cli` crate 以 `cargo run -p prompt-cli -- <subcommand>` 验证，不引入 release 打包与 sidecar 配置。

## 12. 实施偏差记录（相对 §10/§11 的落地调整）

| # | 设计原述 | 实际落地 | 原因 |
| --- | --- | --- | --- |
| 1 | IR / writer 落在 `core::export` | 独立新 crate `crates/export`（`export`） | `ExportService` 需同时依赖 repos(core traits) + `template::render` + `RegexTemplateExtractor`；`core` 零重型依赖，不宜逆行依赖 `template`。`cli` 与 `ipc` 共同复用 `export`，避免输出逻辑分叉。 |
| 2 | CLI `cargo run -p prompt-manager -- list` | `cargo run -p prompt-cli -- list` | 根 package `prompt-manager` 已有同名二进制，故 cli package 命名为 `prompt-cli`、`[[bin]] name = "prompt-cli"`。 |
| 3 | 未显式提及 help 子命令 | clap 内置 `help` 子命令（`disable_help_subcommand = false` 显式保留） | 用户指令「在 CLI 命令中增加 help 帮助命令」；`prompt-cli help` 与 `prompt-cli <子命令> --help` 均可用。 |
| 4 | §9 `export_prompts(..., render)` 无 `vars` | 增加 `vars: HashMap<String,String>` 入参 | 与 `render_prompt` 口径一致，支持 UI 表单填值后一次性导出（优先级：显式输入 > default_value > 保留占位）。 |
| 5 | 目标目录通过 Tauri dialog 选择（§8） | 前端以文本输入 + 默认目录 `prompt-exports` 实现 | 项目未引入 `@tauri-apps/plugin-dialog`；避免新增依赖，后端 `create_dir_all` 自动建目录。 |