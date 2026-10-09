# 个人提示词管理平台 AI 进阶能力 设计方案

> 状态：待评审（评审通过后再定开发计划，暂不实现）
> 日期：2026-10-09
> 关联能力：AI 辅助（BRD §4.5）——AI 调优 / AI 提取变量 / 本地模型接入 / 脱敏模式
> 关联既有资产：`crates/ai`（已落 `Provider` trait + `OpenAiCompatibleProvider` + `generate_prompt`）、`crates/template`（变量提取/渲染）、`crates/crypto`（`SecretStore` / OS 钥匙串）

## 1. 背景与目标

一期已落地「AI 生成提示词」：用户 → 自然语言需求 → 结构化草稿 → 人工审阅落库。经评审确认，剩余四个高频诉求分属两类：

- **对存量内容的加工**：AI 调优（改得更专业/更简洁/更具体）、AI 提取变量（把写死的文案参数化为 `{{变量}}`）。
- **对调用链路的增强**：本地模型接入（不必绑定云端，可离线/省钱/保隐私）、脱敏模式（把敏感占位替换后再出网）。

本方案为这四项能力给出统一设计，复用一期 `crates/ai` 的 `Provider` 抽象与 `AppError::Ai` 错误通道，**不改前端主干、不动既有生成流程**。

目标：

- 四项能力均为现有 `crates/ai` 的**扩展**，而非新起炉灶。
- 每项能力都产出**草稿**，不自动落库，用户审阅后走既有「新建/编辑提示词」保存。
- 敏感信息（API Key、用户自定义敏感词）仍只落 OS 钥匙串 / 本地不落盘不出网。

## 2. 能力总览

| 能力 | 输入 | 输出 | 是否落库 | 前置依赖 | 优先级 |
| --- | --- | --- | --- | --- | --- |
| AI 调优 | 一段提示词正文 + 调优方向 | 调优后草稿（正文为主） | 否（草稿） | 在线/本地 Provider | P2 |
| AI 提取变量 | 一段提示词正文 | 重写为含 `{{变量}}` 的正文 + 变量定义 | 否（草稿） | 在线/本地 Provider（可选：纯规则兜底） | P2 |
| 本地模型接入 | 无（个配置项） | 可选的本地 Provider（Ollama / LM Studio） | — | 无（OpenAI 兼容协议即开箱） | P2 |
| 脱敏模式 | 待出网文本 + 敏感词规则 | 脱敏后文本（出网）+ 还原映射 | — | 无（`crates/ai` 新模块） | P3 |

## 3. 统一扩展点：`crates/ai` Provider trait

一期已定义：

```rust
pub trait Provider: Send + Sync {
    fn generate(&self, cfg: &AiConfig, api_key: &str, system: &str, user: &str)
        -> Result<String, AppError>;
}
```

四项能力的落点：

- **调优 / 提取变量**：都属于「构造 system + 调一次 generate + 解析」的编排，复用 `generate_prompt` 的骨架，新增两个编排函数 `polish_prompt` / `extract_variables_ai`；它们面向 `&dyn Provider`，因此同时支持在线与本地模型。
- **本地模型接入**：`OpenAiCompatibleProvider` 已能直接打 Ollama/LM Studio 的 OpenAI 兼容端点；新增的是「本地 Provider 预设」与「空 Key 跳鉴权」两个小改动（见 §6）。
- **脱敏模式**：独立于 Provider，作为「出网前/回网后」的横切过滤层（见 §7），对任意 Provider 透明。

## 4. AI 调优（polish）

### 4.1 交互

```
编辑器选中一条提示词 → 「AI 调优」按钮
  → 对话框：调优方向（更专业 / 更简洁 / 更具体 / 自定义指令）
  → 后端 polish_prompt()
  → 返回调优后草稿（diff 预览正文）
  → 用户「应用」→ 填入编辑器 → 手动保存
```

### 4.2 契约与编排

`crates/ai/src/polish.rs`：

```rust
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolishOptions {
    pub style: Option<String>,   // 内置方向枚举或自定义指令文本
    pub instruction: Option<String>, // 自定义补充指令
}

pub fn polish_prompt(
    provider: &dyn Provider, config: &AiConfig, api_key: &str,
    content: &str, options: &PolishOptions,
) -> Result<PromptDraft, AppError>;
```

- `style` 固定几个内置 system 片段（更专业/更简洁/更具体），`instruction` 透传为用户自定义要求。
- 输出契约复用 `PromptDraft`：模型只返回一个 JSON `{ "content": 调优后正文, ... }`，`content` 为必填核心字段，其余可空。

### 4.3 关键点

- **只改正文、不动元数据**：调优只面向 `content`，标题/分类/标签/变量由用户决定是否同步。
- **保留变量占位**：system 明确要求「`{{变量}}` 原样保留、不展开、不删除」。
- **非流式**：与一期一致，单次完成 + 超时（决策 D2）。

## 5. AI 提取变量（extract-variables）

### 5.1 与既有规则版的关系

一期 `crates/template::extract_variables` 是**正则规则版**（匹配 `{{变量}}`），只能「识别已被写进正文的占位」，不能「主动发现哪句话该变成变量」。AI 提取变量的价值在于**语义化发现**：把「请给张三写一封周报，部门是研发一部」改写成「请给 `{{收件人}}` 写一封周报，部门是 `{{部门}}`」。

因此：

- **规则版**保留为兜底与唯一定义源（`VariableDefinition` 类型不变）；
- **AI 版**作为增强：模型负责「改写正文 + 给出变量名/默认值/描述」，后端仍用 `template::extract_variables` 对改写后的正文做一次**校验性提取**，保证 `{{变量}}` 与变量定义一致。

### 5.2 契约与编排

`crates/ai/src/extract.rs`：

```rust
pub fn extract_variables_ai(
    provider: &dyn Provider, config: &AiConfig, api_key: &str,
    content: &str,
) -> Result<PromptDraft, AppError>;
```

返回的 `PromptDraft`：

- `content`：改写后的正文（嵌入了 `{{变量}}` 占位）；
- `variables`：`Vec<VariableDefinition>`，其中 `name/defaultValue/options/description` 由模型给出，后端用 `template::extract_variables(&draft.content)` 合并去重，`name` 以正文实际占位为准（模型自报不全时以提取结果补全）。

### 5.3 降级

- 若模型调用失败或返回非法 JSON，**不覆盖**：回退为规则版把「原正文当作正文」返回，并在前端提示「AI 提取失败，已退回规则模式」。

## 6. 本地模型接入（local provider）

### 6.1 现状

`OpenAiCompatibleProvider` 请求 `POST {baseUrl}/chat/completions`，而：

- **Ollama**：OpenAI 兼容端点 `http://localhost:11434/v1/chat/completions`（也提供原生 `/api/chat`）。
- **LM Studio**：内置 OpenAI 兼容服务 `http://localhost:1234/v1/chat/completions`。

二者均**开箱即用**，只需把 `baseUrl` 指向本地、`model` 填本地已加载模型即可。因此「本地模型接入」主要是**易用性**而非协议工作。

### 6.2 改动项

1. **Provider 预设**（`core::models::ai` 扩展 `AiConfig` 增加可空 `preset`，或前端硬编码预设下拉）：
   - `openai`：`https://api.openai.com/v1`
   - `deepseek`：`https://api.deepseek.com/v1`
   - `ollama`：`http://localhost:11434/v1`
   - `lmstudio`：`http://localhost:1234/v1`
2. **空 Key 跳鉴权**：`OpenAiCompatibleProvider` 在 `api_key` 为空时**不发送** `Authorization: Bearer` 头（当前 `bearer_auth("")` 会带空令牌）。本地服务通常忽略，但显式跳过更干净、避免部分实现误判。
3. **本地模型列表（可选 P3）**：Ollama 提供 `GET /api/tags` 列出已装模型，可在设置页「本地模型」下拉动态拉取；本期仅做手动填 `model`，列表发现留待后续。

### 6.3 契约

沿用一期 `AiConfig`（`provider/baseUrl/model/temperature/maxTokens/timeoutSecs`），新增可选 `apiKeyRequired: bool`（本地预设为 false）。`set_ai_api_key` 对本地预设允许留空。

## 7. 脱敏模式（desensitize）

### 7.1 目标

用户在把「需求描述 / 待调优文案」发送到**远端** API 前，自动把敏感内容替换为无损占位，回包后再还原，从而在享受云端模型能力时不泄露真实人名/公司/联系方式等。

### 7.2 识别策略（三层，降级叠加）

1. **内置正则**（零配置开箱）：手机号、邮箱、身份证号、银行卡号、IPv4。
2. **用户自定义敏感词表**（设置页维护，`Vec<String>`）：公司名、项目代号、人名等专有名词。
3. **AI 语义识别（P3，本期不做）**：用本地规则/小模型自动发现未列名词，仅作提示不自动替换。

### 7.3 替换与还原

- 占位符形如 `[[SENSITIVE_01]]`，命中项按出现顺序编号，**同名替换回同一编号**（保障一致性）。
- 处置管线（`crates/ai/src/desensitize.rs`）：
  - 出网前：`sanitize(text) -> (masked_text, Map<编号, 原文>)`，映射仅存**内存**，不落盘、不进日志、不随请求发出；
  - 回网后：`restore(output) -> 还原文本`，将输出中的占位符替换回原文。
- 仅作用于 `user` 侧文本与模型输出；`system` 提示词不含用户敏感数据。

### 7.4 配置

`AiConfig` 增加可选 `desensitize: bool` 与 `sensitiveTerms: Vec<String>`（非敏感项进 `ai_config.json`，与一期一致）。前端在设置页提供「脱敏开关 + 敏感词管理」。

## 8. 命令契约（前端 `invoke` 扩展）

| 命令 | 入参 | 出参 |
| --- | --- | --- |
| `ai_polish_prompt` | `{ content, options }` | `PromptDraft` |
| `ai_extract_variables` | `{ content }` | `PromptDraft` |
| `get_ai_config` / `set_ai_config`（扩展） | 增 `apiKeyRequired`/`desensitize`/`sensitiveTerms` 字段 | 同左 + `hasApiKey` |

> 一期已存在的 `get_ai_config/set_ai_config/set_ai_api_key/ai_generate_prompt` 不变，仅 `AiConfig` 结构体向后兼容地**新增可空字段**（老配置能正常反序列化，字段缺省回退默认）。

所有新命令与一期一致：命令层在 `src-tauri/src/ai_commands.rs` 下追加，装配层 `src-tauri/src/lib.rs` 注册。

## 9. 安全与隐私

- **API Key**：仍只落 OS 钥匙串，前端只写不回显；本地 Provider 允许留空。
- **敏感词表**：仅存本地 `ai_config.json`，与 Key 隔离，不随同步/导入导出。
- **脱敏映射**：仅存内存，请求/日志/返回体中不出现原文与映射。
- **数据出网显式化**：非本地 Provider 的对话框持续提示「内容将发送至 {baseUrl}」；脱敏开启时提示「已脱敏」。
- **本地优先**：当选择本地 Provider 且为本地地址时，可默认不脱敏、不出网（决策 D5）。

## 10. 决策记录（需评审拍板）

- **D1 调优方向**：内置「更专业 / 更简洁 / 更具体」三个固定片段 + 自定义指令；不实现「对话式多轮调优」，一轮完成（多轮/流式统一留到「对话调试」）。
- **D2 提取变量双轨**：AI 改写 + 规则版校验兜底；`VariableDefinition` 类型与模板引擎保持一致，不新增类型。
- **D3 本地模型即 OpenAI 兼容**：不单独实现 Ollama 原生 `/api/chat`，统一走 `/v1/chat/completions`；本地模型列表发现列为 P3。
- **D4 脱敏范围**：仅 `user` 侧文本与模型输出；不落盘；同名占位复用同一编号。AI 语义识别（第三层）本期不做。
- **D5 脱敏触发**：默认远程 Provider 可用可选开关；本地地址默认不脱敏。
- **D6 落库边界**：四能力产物均为「草稿」，用户审阅后手动保存，**绝不自动写库**（与一期 D5 一致）。

## 11. 验收要点（评审通过、进入开发后对照）

- [ ] `cargo check`（全工作区）通过。
- [ ] `cargo test -p ai`：调优 / 提取变量 / 脱敏（sanitize→restore 往返）/ 本地空 Key 跳鉴权 各新增 mock 用例均绿。
- [ ] `tsc --noEmit` 通过。
- [ ] 桌面端：调优、提取变量产出草稿并可从编辑器应用；切换 Ollama 预设后（本地已起服务）可生成；开启脱敏后请求不携带敏感原文、回包正确还原。
- [ ] 验收安全项：`get_ai_config` 与日志/报错中不含 Key 与脱敏映射明文。