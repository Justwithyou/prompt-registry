# CLI 与 Skills 输出使用手册

本手册面向「提示词管理平台」的 **CLI 输出工具（`prompt-cli`）** 与 **Skills（SKILL.md）** 导出能力。

> 背景：平台内的提示词 / 分类 / 标签 / 项目资产沉淀在本地 SQLite（`prompt.db`）中。CLI 与 Skills 输出的作用，是把这些资产「带出去」——既能在命令行查览、批量导出，又能把提示词导出为 Agent（Claude Code / Trae / Codex 等）可直接加载的技能。

---

## 1. 运行环境与形态

当前 CLI 为 **开发态**，不打包独立二进制、不放入 sidecar，通过 Cargo 工作区直接运行：

```bash
# 在 src-tauri 目录下执行（Cargo 工作区根）
cargo run -p prompt-cli -- <子命令> [参数]
```

- 包名：`prompt-cli`
- 二进制名：`prompt-cli`
- 依赖：已安装 Rust 工具链（cargo），且能在 `src-tauri` 下正常编译。

---

## 2. 数据库位置（`--db`）

CLI 默认读取 **当前目录下的 `prompt.db`**，可通过全局参数 `--db <路径>` 覆盖。

```bash
# 显式指定数据库文件（注意 Windows 路径用引号包裹）
cargo run -p prompt-cli -- --db "C:\path\to\prompt.db" list
```

桌面应用的正式数据库位于应用数据目录：

```text
%APPDATA%\com.promptmanager.app\prompt.db
```

例如 Windows 下通常为：

```text
C:\Users\<你的用户名>\AppData\Roaming\com.promptmanager.app\prompt.db
```

如需用 CLI 读取桌面应用里的真实数据，把 `--db` 指向上述路径即可。CLI 以只读方式打开，不会改写数据。

> `--db` 为全局参数，可放在子命令前或后。

---

## 3. 子命令总览

| 子命令 | 作用 |
| --- | --- |
| `list` | 列出提示词（可按分类 / 标签 / 项目 / 关键词过滤） |
| `get` | 读取单条提示词 |
| `export` | 按分类 / 标签 / 项目批量导出 |
| `run` | 渲染单条模板提示词为成品 |
| `help` | 查看帮助（整体或某个子命令） |

---

## 4. `list` —— 列出提示词

```bash
cargo run -p prompt-cli -- list [--category <id>] [--tag <id> ...] [--project <id>] [--keyword <词>] [--format table|json]
```

| 参数 | 说明 |
| --- | --- |
| `--category <id>` | 按分类过滤 |
| `--tag <id>` | 按标签过滤，可多次传入（命中需同时包含） |
| `--project <id>` | 按项目过滤 |
| `--keyword <词>` | 关键词 |
| `--format table\|json` | 输出格式，默认 `table` |

示例：

```bash
# 默认表格列出全部
cargo run -p prompt-cli -- list

# 输出 JSON（含完整字段，便于脚本解析）
cargo run -p prompt-cli -- list --format json

# 组合过滤
cargo run -p prompt-cli -- list --keyword 周报 --project <项目ID>
```

> 过滤参数按 **ID** 匹配；ID 可从 `list --format json` 输出的 `id` 字段，或桌面应用内查看。

---

## 5. `get` —— 读取单条提示词

```bash
cargo run -p prompt-cli -- get <id> [--format skill|markdown|json]
```

| 参数 | 说明 |
| --- | --- |
| `<id>` | 提示词 ID |
| `--format` | 输出格式，默认 `markdown`；`skill` 输出 SKILL.md 文本 |

示例：

```bash
cargo run -p prompt-cli -- get <id>
cargo run -p prompt-cli -- get <id> --format skill
cargo run -p prompt-cli -- get <id> --format json
```

---

## 6. `export` —— 批量导出

```bash
cargo run -p prompt-cli -- export [--category <id>] [--tag <id> ...] [--project <id>] [--format skill|markdown|json] [--out <目录>] [--render] [--var name=value ...]
```

| 参数 | 说明 |
| --- | --- |
| `--category <id>` | 按分类导出 |
| `--tag <id>` | 按标签导出，可多次 |
| `--project <id>` | 按项目导出 |
| `--format` | 导出格式，默认 `skill` |
| `--out <目录>` | 输出目录，默认 `./prompt-export` |
| `--render` | 使用变量默认值渲染后再导出 |
| `--var name=value` | 变量填值，可多次；覆盖默认值 |

示例：

```bash
# 全量导出为 SKILL.md 目录（默认）
cargo run -p prompt-cli -- export --out ./out

# 导出为 Markdown / JSON（平铺文件）
cargo run -p prompt-cli -- export --format markdown --out ./out
cargo run -p prompt-cli -- export --format json --out ./out

# 按项目导出并渲染变量
cargo run -p prompt-cli -- export --project <项目ID> --render --var city=上海 --out ./out
```

导出目录结构：

```text
prompt-export/
├── 周报模板/
│   └── SKILL.md          # skill 格式：每个技能一个目录
├── 客服-退款话术/
│   └── SKILL.md
├── 周报模板.md            # markdown 格式：平铺文件
└── 周报模板.json          # json 格式：平铺文件
```

文件名由标题生成（slug）：保留中英文与数字，空格与符号折叠为 `-`，最长 64 字符，空标题回退为 `untitled`；重名自动追加 `-2`、`-3` 序号。

---

## 7. `run` —— 渲染单条模板

```bash
cargo run -p prompt-cli -- run <id> [--var name=value ...] [--out <文件>]
```

| 参数 | 说明 |
| --- | --- |
| `<id>` | 提示词 ID |
| `--var name=value` | 变量填值，可多次 |
| `--out <文件>` | 输出到文件；缺省打印到 stdout |

示例：

```bash
cargo run -p prompt-cli -- run <id> --var name=张三 --var city=上海
cargo run -p prompt-cli -- run <id> --var name=张三 --out result.md
```

渲染规则：正文中的 `{{变量名}}` 会被填值；已填值优先于默认值；未填值且无默认值的占位符会原样保留（作为 `{{变量名}}` 留待补全）。

---

## 8. `help` —— 帮助

clap 内置帮助，顶层与子命令均可查看：

```bash
cargo run -p prompt-cli -- help
cargo run -p prompt-cli -- list --help
cargo run -p prompt-cli -- export --help
```

---

## 9. 模板变量语法

模板正文使用 `{{变量名}}` 占位符，变量名由字母 / 数字 / 下划线组成：

```text
你好 {{name}}，欢迎来到 {{city}}，今天天气 {{weather}}。
```

- `run` / `export --render` 会识别这些占位符并回填。
- 填值从 `--var name=value` 或变量默认值合并而来（显式填值优先）。

---

## 10. SKILL.md 规范

`skill` 格式导出的 `SKILL.md` 结构如下：

```text
---
name: "提示词标题"
description: "描述（缺省回退为「提示词「标题」」）"
category: "工作/周报"          # 无分类时省略此行
tags: ["模板", "周报"]          # 无标签时省略此行
projects: ["内部流程"]         # 无项目时省略此行
metadata:
  rating: 5
  pinned: true
  favorite: false
  source: "提示词ID"
---

# 提示词标题

正文内容……

## 变量
| 变量 | 默认值 | 可选值 | 说明 |
| --- | --- | --- | --- |
| city | - | - | - |
```

说明：
- frontmatter 使用 **标准字段**（`name` / `description`）保证与通用 Agent 兼容，组织信息（分类 / 标签 / 项目）与评分 / 置顶 / 收藏等放在自定义 `metadata` 内。
- `## 变量` 表仅在正文存在 `{{变量}}` 占位符时输出。
- 字符串统一双引号包裹并转义，避免冒号 / 换行 / 引号破坏 frontmatter。

---

## 11. 前端导出入口

桌面应用内同样支持导出，UI 入口与 CLI 共用同一套导出内核（`ExportService`），口径一致：

- 提示词编辑页、分类页、标签页、项目管理页均可点击「批量导出」打开导出对话框。
- 对话框内选择导出格式（SKILL.md / Markdown / JSON）、目标目录，并可为模板变量填值。

---

## 12. 常见问题

- **提示「错误: 未找到该提示词」**：确认 `<id>` 来自 `list` 输出，且 `--db` 指向了正确的数据库文件。
- **`--db` 没生效**：确认使用了全局参数写法 `-- --db <路径> <子命令>` 或 `<子命令> --db <路径>`。
- **导出目录已存在**：会复用该目录，同名词条自动追加序号，不会覆盖。