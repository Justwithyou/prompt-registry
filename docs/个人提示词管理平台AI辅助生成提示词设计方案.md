# 个人提示词管理平台 AI 辅助生成提示词 设计方案

> 状态：待评审（评审通过后再定开发计划，暂不实现）
> 日期：2026-10-09
> 关联能力：AI 辅助（BRD §4.5「AI 生成提示词」P2）
> 关联既有资产：`crates/ai`（占位）、`crates/crypto`（OS 钥匙串 / `SecretStore`）、`crates/template`（变量提取）

## 1. 背景与目标

用户已积累大量提示词，但「从零写一条高质量提示词」本身有门槛。本能力让用户用自然语言描述需求，平台自动生成一条**结构化的提示词草稿**（标题 + 正文 + 建议变量 + 建议分类/标签 + 描述），用户审阅、编辑后一键保存。

目标：

- 输入一句/一段需求描述（可选：语气、目标对象、是否启用变量），输出结构化提示词草稿。
- 生成结果直接落进现有提示词数据模型（标题/content/描述/分类/标签/`{{变量}}`），无需二次手工整理。
- Every 生成结果都是**草稿**，必须经用户审阅编辑后才会写入数据库，AI 不自动落库。
- 敏感信息（API Key）只存 OS 钥匙串，不出设备、不回显、不进日志。

## 2. 范围与边界（本期只做「生成」）

BRD §4.5 的 AI 辅助还有「调优 / 提取变量 / 本地模型接入 / 脱敏」，本方案**只落地「AI 生成提示词」**，其余留待后续，边界如下：

| 能力 | 本期是否做 | 说明 |
| --- | --- | --- |
| AI 生成提示词（需求 → 结构化草稿） | ✅ 本期 | 本方案主题 |
| 在线 API 配置（OpenAI 兼容协议） | ✅ 本期 | 生成能力的前置依赖 |
| AI 调优（更具体/更简洁/更专业） | ❌ 后续 | BRD §4.5 P2 |
| AI 提取变量（已有正文自动改 `{{变量}}`） | ❌ 后续 | 现有 `template::extract_variables` 可复用 |
| 本地模型接入（Ollama / LM Studio） | ❌ 后续 | BRD §4.5 P2，通过 `crates/ai/src/local.rs` 扩展 |
| 脱敏模式（调用前替换敏感变量） | ❌ 后续 | BRD §4.5 P3 |

设计上预留扩展点：`crates/ai` 以 **provider trait** 抽象模型调用，后续「调优/本地模型」在同一 trait 上新增方法/实现，不改前端主干。

## 3. 交互与数据流

```
用户点击编辑器/工具栏「AI 生成」
    │
    ▼
前端打开「AI 生成」对话框（feature/ai-assistant）
    ├── 输入：需求描述（必填）
    ├── 输入：语气/风格、目标对象、是否使用变量（选填）
    └── 点击「生成」
    │
    ▼
invoke('ai_generate_prompt', { requirement, options })
    │
    ▼
Rust ai::generate_prompt()（单次 completion，带超时）
    ├── 读取 provider 配置（baseUrl/model/temperature 等，本地 ai_config.json）
    ├── 从 OS 钥匙串读取 API Key（crypto::SecretStore）
    ├── 构造 system 提示词（「你是提示词工程专家…只输出 JSON」）
    ├── 发起 HTTP 请求（reqwest blocking，与 sync/webdav 同技术栈）
    ├── 解析结构化 JSON（失败时降级为纯文本兜底）
    └── 提取正文中的 {{变量}}（复用 template::extract_variables）
    │
    ▼
返回 PromptDraft（结构化草稿，不落库）
    │
    ▼
前端在对话框预览草稿（标题/正文/变量/建议标签/建议分类）
    │
    ▼
用户「接受并打开编辑器」→ 草稿填入新建提示词表单 → 用户继续编辑后手动保存
```

关键点：

- **AI 输入永远不进数据库**：用户点「生成」只触发一次远程调用，草稿仅存在于前端状态与返回体，落库仍需用户走正常「新建/编辑提示词」流程。
- **生成 = 单次、非流式**：结构化 JSON 输出一次到位；流式留待「调优/对话调试」再引入（见决策记录 D3）。

## 4. 输出格式（结构化草稿契约）

`ai_generate_prompt` 返回的 `PromptDraft`：

```typescript
interface PromptDraft {
  title: string;              // 建议标题
  content: string;            // 正文（可含 {{变量}} 占位）
  description: string;        // 一段用途/说明
  suggestedTags: string[];    // 建议标签名（可能与现有标签不同，不自动创建）
  suggestedCategory?: string; // 建议分类名（参考值）
  variables: VariableDefinition[]; // 由 template::extract_variables 从 content 提取
}
```

Rust 侧对应结构体在 `crates/ai/src/lib.rs` 或新 `crates/ai/src/prompt.rs`：

```rust
#[derive(Serialize, Deserialize)]
pub struct PromptDraft {
    pub title: String,
    pub content: String,
    pub description: String,
    pub suggested_tags: Vec<String>,
    pub suggested_category: Option<String>,
    pub variables: Vec<VariableDefinition>,
}
```

生成指令约定（system prompt 内置）：

- 要求模型**只输出**一个 JSON 对象，字段与上述契约一致；`content` 中如需参数化，用 `{{变量名}}` 占位。
- 解析策略：先 `serde_json::parse`；失败时用「提取第一个 `{...}` 块」再 parse；仍失败则把整段返回体当 `content`，其余字段置空（**降级但可用，不报错**）。

`VariableDefinition` 直接复用 `crates/template`（或 `prompt_core`）既有类型，保证与模板引擎契约一致。

## 5. 架构落点

沿用既有分层，新增/改动如下（不破坏现有 crate 边界）：

| 层 | 位置 | 改动 |
| --- | --- | --- |
| AI 服务 | `crates/ai/src/lib.rs`（已有占位） | 新增 `Provider` trait、`OpenAiCompatibleProvider`、`generate_prompt`；`Cargo.toml` 增 `reqwest`(blocking+rustls)/`serde_json`/`prompt_core`/`template` |
| 密钥存储 | `crates/crypto/src/keyring.rs` + 新增 `crates/crypto/src/secret.rs` | `KeychainStore` 通用化（入口由固定 `master-key` 改为可传 user 参数），新增 `ai-api-key` 条目；不新增新依赖 |
| 配置存储 | `src-tauri/src/ai_config.rs`（新增） | 持久化 `baseUrl/model/temperature/maxTokens/timeout` 到应用数据目录 `ai_config.json`；读取/写入命令 |
| 命令层 | `src-tauri/src/ai_commands.rs`（新增） | `get_ai_config` / `set_ai_config` / `set_ai_api_key`（写钥匙串）/ `ai_generate_prompt`（读钥匙串 + generate） |
| 装配层 | `src-tauri/src/lib.rs` | `mod ai_config` / `mod ai_commands` + 注册命令；启动时构造 `AppState` 注入 `ai::Provider` 或配置 |
| 前端契约 | `src/shared/api/commands.ts` | 新增类型（`AiConfig`/`PromptDraft`）与 4 个 `invoke` 封装 |
| 前端切片 | `src/features/ai-assistant/`（新增） | `stores/aiStore.ts` + `ui/AiGenerateDialog.tsx`；入口接入 `prompt-editor` |
| 前端配置 | `src/features/settings/ui/SettingsPage.tsx` | 「AI 辅助」分区：baseUrl/model/key 设置（key 只写不回显） |

### 5.1 Provider trait（`crates/ai`）

```rust
pub trait Provider: Send + Sync {
    /// 单次结构化生成；返回模型原始文本（JSON 或纯文本兜底）。
    fn generate(&self, cfg: &AiConfig, api_key: &str, system: &str, user: &str) -> Result<String, AppError>;
}

pub struct OpenAiCompatibleProvider { client: reqwest::blocking::Client }
```

- `AiConfig { provider: String, base_url: String, model: String, temperature: f32, max_tokens: u32, timeout_secs: u64 }`。
- 请求路径：`POST {base_url}/chat/completions`，OpenAI 兼容报文结构（`messages`、`temperature`、`max_tokens`、`response_format` 尽力 JSON 模式）。
- 超时、非 2xx、网络错误统一映射为 `AppError::Ai(String)`，**绝不吞掉 Key 明文进错误信息**。

### 5.2 命令契约（前端 `invoke`）

| 命令 | 入参 | 出参 |
| --- | --- | --- |
| `get_ai_config` | — | `AiConfig`（不含 `apiKey`，仅 `hasApiKey: boolean`） |
| `set_ai_config` | `AiConfig`（不含 key） | `AiConfig` |
| `set_ai_api_key` | `{ apiKey: string }` | `()` |
| `ai_generate_prompt` | `{ requirement: string, options?: { tone?, audience?, useVariables? } }` | `PromptDraft` |

## 6. 安全与隐私

- **API Key 只落 OS 钥匙串**：复用 `crypto::SecretStore`。前端在保存时把 Key 经 `set_ai_api_key` 一次写入，之后仅通过 `get_ai_config` 得知「是否已配置」布尔值，**永不回传 Key 明文**。
- **Key 不进普通配置**：`baseUrl/model/temperature` 等非敏感项才存 `ai_config.json`（可选：后续进 SQLite settings 表，首版用 JSON 与 `storage.json` 一致）。
- **不记录日志**：错误/日志中不输出 Key、不输出用户需求原文到 console（只输出错误类别）。
- **数据出设备边界显式化**：生成必然把「需求描述」发送到用户配置的 API 端点，UI 需在对话框提示「内容将发送至：{base_url}」；脱敏模式（P3）不在本期。
- **无网络/超时**：返回可读错误，不影响本地核心功能（与架构「故障隔离」原则一致）。

## 7. 决策记录（需评审拍板）

- **D1 提供商范围**：首版仅支持 **OpenAI 兼容 HTTP 协议**（可用 OpenAI / DeepSeek / 其他自定义 baseUrl 的兼容服务）；真实的 `local.rs`（Ollama）后续再加。
- **D2 结构化输出**：用「系统指令要求只输出 JSON + 前端/Rust 双重降级解析」实现，不强制依赖厂商的 JSON 模式 API（兼容性最好）。
- **D3 生成非流式**：本期「生成」走单次 completion + 超时（默认 60s）；流式输出留给「调优/对话调试」再同一 provider trait 上扩展。
- **D4 API Key 存储**：OS 钥匙串（`crypto::SecretStore`），前端只写不回显；非敏感配置存 `ai_config.json`。
- **D5 落库边界**：AI 只出**草稿**，走用户「接受 → 填入编辑器 → 手动保存」闭环，绝不自动写 `prompts` 表。
- **D6 默认模型**：不内置任何默认 API Key；首次使用引导用户在「设置 → AI 辅助」配置 baseUrl/model/key。

## 8. 验收要点（评审通过、进入开发后对照）

- [ ] `cargo check`（全工作区）通过。
- [ ] `cargo test -p ai`：用测试 mock `Provider` 验证「JSON 解析 / 纯文本兜底 / 变量提取」三条路径。
- [ ] `cargo test -p crypto`：`SecretStore` 通用化后原 M4 用例仍绿。
- [ ] `tsc --noEmit` 通过。
- [ ] 桌面端：配置 baseUrl/model/key 后，输入需求可生成草稿；无 Key / 错误 baseUrl / 断网时返回友好错误且不崩溃。
- [ ] 验收安全项：`get_ai_config` 返回体不含 Key 明文；日志/报错中无 Key 明文。