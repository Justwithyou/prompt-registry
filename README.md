# 提示词管理平台（Prompt Manager）

![Version](https://img.shields.io/badge/version-1.0.0-2f6fed)
![License](https://img.shields.io/badge/license-MIT-green)
![Tauri](https://img.shields.io/badge/Tauri-2.x-24C8DB)
![React](https://img.shields.io/badge/React-19-61DAFB)
![TypeScript](https://img.shields.io/badge/TypeScript-5.7-3178C6)
![Rust](https://img.shields.io/badge/Rust-stable-DEA584)

个人提示词**全流程**管理桌面应用：从提示词的创建、分类、打标、版本管理，到模板渲染、导入导出、跨设备同步、CLI / Skills 输出与 AI 辅助生成，一站式沉淀与管理你的 Prompt 资产。

跨平台桌面应用（Windows / macOS / Linux），基于 **Tauri v2 + React 19 + TypeScript**，数据本地存储并以 **SQLCipher 端到端加密**，内置 **WebDAV 多设备同步**。

---

## ✨ 功能特性

### 组织与管理
- 提示词 CRUD：标题、正文、分类、项目、标签、评分、置顶、收藏
- 多维度组织：分类 / 项目 / 标签三级资产治理
- 模板变量：正文 `{{变量名}}` 占位、变量默认值 / 可选值、一键渲染

### 版本控制
- 每次保存生成版本历史，编辑差异（diff）预览，支持一键回滚

### 导入导出
- JSON 与平台 `.db` 存储文件直接导入（明文库与加密库均可）
- 按分类 / 项目 / 标签 **多选**批量导出
- 导出格式：SKILL.md / Markdown / JSON

### 安全加密
- SQLCipher 全库加密，Master Key 由主口令（Argon2id）派生并经 AES-256-GCM 包装
- 支持 OS 钥匙串自动解锁

### 多设备同步
- WebDAV 变更日志同步（以坚果云为例）
- AES-256-GCM 端到端加密，字段级冲突合并（最后写入者胜出）

### AI 能力
- 自然语言一键生成结构化提示词草稿
- AI 调优 / 改写 / 变量提取 / 脱敏出网
- 支持 OpenAI 兼容接口（在线）与本地模型（Ollama / LM Studio），API Key 仅存 OS 钥匙串

### CLI 与 Skills
- `prompt-cli`：`list` / `get` / `export` / `run` / `help`
- 导出为 `SKILL.md`，供 Claude Code / Trae / Codex 等 Agent 直接加载

---

## 🛠 技术栈

| 层 | 技术 |
| --- | --- |
| 桌面壳 | Tauri v2 |
| 前端 | React 19、TypeScript 5.7、Vite 6、Zustand、React Router 7、Nunjucks |
| 后端 | Rust（Cargo workspace：core / ipc / storage / export / template / crypto / sync / cli / ai / versioning） |
| 存储 | SQLite（SQLCipher 加密） |
| 加密 | Argon2id + AES-256-GCM + HKDF |

---

## 🚀 快速开始

### 环境要求
- Node.js ≥ 18 与 `pnpm`
- Rust 稳定版工具链（`rustup`）
- Tauri 平台系统依赖（见 [Tauri Prerequisites](https://tauri.app/start/prerequisites/)）

### 安装与开发

```bash
pnpm install     # 安装前端依赖
pnpm tauri dev   # 启动桌面应用（开发态，含 HMR）
```

> Windows 下也可直接运行根目录的 `start-dev.cmd` 快速启动。

### 构建

```bash
pnpm tauri build   # 打包当前平台安装包
```

### 仅前端调试 / 类型检查

```bash
pnpm dev      # 仅启动 Vite 前端（不启动桌面壳）
pnpm build    # tsc 类型检查 + vite 构建
```

---

## 📖 使用方式

1. **新建 / 管理提示词**：录入标题、正文（支持 `{{变量}}` 模板），挂到分类 / 项目 / 标签。
2. **版本回滚**：每次保存生成版本记录，可查看 diff 并一键回滚。
3. **导入导出**：在设置或编辑页导入 JSON / `.db`，或按多选范围批量导出为 SKILL.md / Markdown / JSON。
4. **同步**：在「同步」页配置 WebDAV（如坚果云），先设置主口令，即可多设备加密同步。
5. **AI 生成**：在编辑器用自然语言生成草稿，或对既有提示词调优 / 改写 / 提取变量。

---

## ☁️ 多设备同步

同步采用「变更日志」而非整库搬运：每次增删改写入同步日志 → 加密打包 → 经 WebDAV 上传；对端下载解密后做字段级合并。传输内容 AES-256-GCM 端到端加密。

以坚果云 WebDAV 为例的完整配置见 [`坚果云同步手册.md`](坚果云同步手册.md)。

---

## ⌨️ CLI 与 Skills 输出

```bash
# 在 src-tauri 目录下运行
cargo run -p prompt-cli -- list
cargo run -p prompt-cli -- get <id> --format skill
cargo run -p prompt-cli -- export --format skill --out ./out
cargo run -p prompt-cli -- run <id> --var name=张三
```

详细子命令与 `SKILL.md` 规范见 [`CLI与Skills使用手册.md`](CLI与Skills使用手册.md)。

---

## 🏗 项目结构

```text
prompt-hub/
├── src/                        # 前端（React + TS，FSD 组织）
│   ├── features/               # 各功能模块
│   ├── shared/                 # 共享 API / 组件 / 样式
│   └── styles/                 # 全局样式
├── src-tauri/                  # Tauri 后端（Rust workspace）
│   ├── src/                    # 装配层 / 命令入口
│   └── crates/                 # 领域 crate
│       ├── core/               # 领域模型 / 错误 / trait
│       ├── storage/            # SQLite(SQLCipher) 存储与仓储
│       ├── crypto/             # 密钥 / 加密 / 钥匙串
│       ├── sync/               # WebDAV 同步引擎
│       ├── export/             # 导出内核
│       ├── template/           # 模板渲染 / 变量提取
│       ├── ai/                 # AI Provider 与生成能力
│       ├── ipc/                # Tauri 命令层
│       ├── cli/                # prompt-cli
│       └── versioning/         # 版本历史
├── docs/                       # 设计文档与研发清单
└── ...
```

---

## 📚 文档

- [`CLI与Skills使用手册.md`](CLI与Skills使用手册.md)
- [`坚果云同步手册.md`](坚果云同步手册.md)
- [`docs/个人提示词全流程管理平台架构设计方案.md`](docs/个人提示词全流程管理平台架构设计方案.md)
- [`docs/`](docs/)：功能需求、导入导出、AI 能力与各里程碑设计文档

---

## 🤝 开发

提交前请通过质量门禁（格式化 → 编译 → 测试 → 类型检查）：

```bash
cargo fmt --check                     # src-tauri/
cargo check --workspace --all-targets # src-tauri/
cargo test --workspace                # src-tauri/
npx tsc --noEmit                      # 项目根目录
```

---

## 🏷 版本发布

| Tag | 说明 |
| --- | --- |
| `v1.0-tag` | v1.0 正式发布 |
| `m1-tag` 〜 `m5-tag` | M1（MVP）〜 M5（打磨/打包）各里程碑 |

完整标签见 [Tags](https://github.com/Justwithyou/prompt-registry/tags)。

## 📄 License

[MIT](LICENSE) © 2026 Justwithyou