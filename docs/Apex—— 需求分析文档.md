# Apex —— 需求分析文档

> 版本：v0.1（需求分析）
> 日期：2026-08-07
> 状态：待评审
> 参考：本文档大量设计决策来源于对 8 个开源 AI 编码 Agent 的逆向分析（项目根目录上一级的 `../docs/` 目录下 9 份文档，共 10261 行），引用处注明出处。

---

## 一、产品定位

### 1.1 一句话定义

**Apex 是一款以 Spec 驱动开发（Spec-Driven Development）为核心、内置编码规范强制校验的开源 AI 编程 Agent**，用 Rust 实现，提供 TUI / 桌面端 / Web 端三种形态共享同一会话数据，运行过程全链路可观测。

### 1.2 要解决的问题

现有 AI 编码 Agent（Claude Code、opencode、CodeWhale 等）存在三个共性问题：

1. **"先斩后奏"式编码**：Agent 拿到需求直接改代码，缺少"先产出可评审的需求/设计/任务文档、确认后再动手"的强制流程。代码改完了，人还没对齐。Kiro 和 GitHub spec-kit 证明了 Spec 驱动流水线的价值，但它们是独立工具或 IDE 插件，与 Agent 本体割裂。

2. **编码规范形同虚设**：项目级规范文件（`rules/`、`.editorconfig`、lint 配置）对 Agent 没有强制力。模型可能"知道"规范但"忘记"遵守，且没有执行后的自动校验兜底。

3. **运行过程黑盒**：Agent 调了哪个 Skill、连了哪个 MCP 服务、派生了哪些子 Agent 在干什么，用户只能在消息流里翻找。opencode、CodeWhale 等虽有 TUI 状态栏，但没有统一的可观测面板。

### 1.3 与参考项目的差异化

| 维度 | 现有方案 | Apex |
|---|---|---|
| Spec 流水线 | Kiro/spec-kit 是独立工具，Agent 本体无此能力 | **内置为 Agent 一等公民**，与上下文管理、工作流引擎深度集成 |
| 规范校验 | 各 Agent 只有 PostToolUse hook 概念，无规范引擎 | **三层机制**：spec 内嵌 + PostToolUse 兜底 + 增量检查修复子任务 |
| 三端共享 | opencode/Reasonix 有多前端但无会话共享保证 | **本机常驻服务 + 统一 SQLite 存储**，TUI/桌面/Web 实时共享 |
| 可观测面板 | 各 Agent 只有消息流和状态栏 | **独立面板**：Skill/MCP/SubAgent 调用详情实时展示 |
| 上下文管理 | 分级摘要（Reasonix）或拒绝摘要（DeepSeek-TUI） | **Checkpoint-first + 分级摘要兜底**（MiMo 混合策略），spec 文档常驻 |

---

## 二、目标用户与使用场景

### 2.1 目标用户

开源产品，面向三类用户：

1. **个人开发者**：在日常编码中使用，追求代码质量和过程可控
2. **技术团队**：需要统一编码规范、可追溯的开发流程
3. **Agent 工具开发者**：参考架构设计，或基于插件机制二次开发

### 2.2 核心使用场景

**场景一：新功能开发（完整 spec 流水线）**

```
用户：帮我给用户系统加一个权限管理模块
Agent：
  1. 生成 requirements.md（需求文档）→ 用户确认/修改
  2. 生成 design.md（技术设计）→ 用户确认/修改
  3. 生成 tasks.md（任务拆解）→ 用户确认/修改
  4. 按 DAG 工作流并行执行任务（子 Agent 写路径互斥）
  5. 每次文件改动后自动增量规范检查，发现问题派修复子任务
  6. 全部完成 → 生成验收报告，对照 spec 逐条验证
```

**场景二：快速修复（逃生门）**

```
用户：/skip-spec 修复登录页的空指针异常
Agent：
  1. 记录 skip-spec 决策（留痕）
  2. 直接定位问题、修复、跑测试
  3. 修复后仍走增量规范检查兜底
```

**场景三：跨端协作**

```
上午在公司：TUI 里开始一个 spec 流水线，完成了需求文档和设计文档
下午在浏览器：打开 Web 端，同一个项目下看到会话进度，审批任务拆解
晚上在桌面端：Tauri 应用里查看子 Agent 执行详情，确认最终变更
```

**场景四：运行过程观测**

```
用户打开可观测面板：
  - Skill 标签页：当前加载了哪些 Skill、哪个 Skill 正在被调用、消耗多少 token
  - MCP 标签页：连接了哪些 MCP 服务、每个服务的工具列表、调用耗时
  - SubAgent 标签页：当前活跃的子 Agent、各自的任务描述、执行进度、产出摘要
  - Memory 标签页：哪些记忆被召回、引用时机、可编辑/删除
```

---

## 三、功能需求

### 3.1 Spec 驱动流水线（P0 —— 核心差异化功能）

#### 3.1.1 流水线阶段

| 阶段 | 产出物 | 确认门 | 说明 |
|---|---|---|---|
| 需求分析 | `requirements.md` | 用户确认 | 结构化需求描述，含功能点、约束、验收标准 |
| 技术设计 | `design.md` | 用户确认 | 架构决策、模块划分、接口定义、编码规范约束内嵌 |
| 任务拆解 | `tasks.md` | 用户确认 | 任务列表 + 依赖关系 + 并行/串行标记 + 写路径声明 |
| 编码实现 | 代码变更 | 自动（增量检查兜底） | DAG 工作流引擎驱动，子 Agent 并行执行 |
| 验证交付 | `verification.md` | 用户确认 | 对照 spec 逐条验收，规范检查报告 |

#### 3.1.2 逃生门机制

- `/skip-spec <任务描述>`：跳过完整流水线，直接进入编码
- 逃生行为持久化记录到会话事件流，标记 `spec_skipped: true`
- 跳过 spec 的任务仍需通过增量规范检查
- 会话统计中展示 skip 率，辅助团队质量审计

#### 3.1.3 Spec 文档管理

- 存储位置：项目根目录 `apex/specs/<feature-name>/`
- 格式：Markdown，带 YAML frontmatter：`id`、`feature`、`kind`、`status`、`version`、`created_at`、`updated_at`、`content_sha256`、`format_version`（`version` 是人类可见的内容版本，`format_version` 是 frontmatter 结构版本，二者独立）
- 版本化：每次用户修改 spec 文档产生新版本，Agent 可 diff 感知变更
- 导出：spec 文档天然是 markdown，可直接提交 git 供团队评审

### 3.2 编码规范引擎（P0）

三层校验机制：

| 层级 | 触发时机 | 行为 |
|---|---|---|
| **Spec 内嵌** | 生成 design.md 时 | 将项目编码规范转化为设计约束和验收标准写入文档 |
| **PostToolUse 兜底** | 每次 Write/Edit 后 | 自动触发增量检查（仅本次变更文件），lint-staged 模式 |
| **修复子任务** | 检查发现问题时 | 派生修复子 Agent 自动修复；高危问题阻断流程，等待用户决策 |

规范来源（按优先级）：
1. 项目根 `apex/rules/` 目录
2. 项目根 `AGENTS.md` / `CLAUDE.md`（兼容生态）
3. 全局 `~/apex/rules/` 目录
4. 内置默认规则集（安全基线、不可变性、文件组织等）

### 3.3 三端架构与会话共享（P0）

#### 3.3.1 架构模型

```
┌─────────────────────────────────────────────┐
│              Apex Core (Rust)           │
│  ┌──────────┐ ┌──────────┐ ┌──────────────┐ │
│  │ Agent    │ │ Spec     │ │ Workflow     │ │
│  │ Engine   │ │ Engine   │ │ Engine (DAG) │ │
│  └──────────┘ └──────────┘ └──────────────┘ │
│  ┌──────────┐ ┌──────────┐ ┌──────────────┐ │
│  │ Permission│ │ Memory  │ │ MCP Client   │ │
│  │ Engine    │ │ System  │ │ Manager      │ │
│  └──────────┘ └──────────┘ └──────────────┘ │
│  ┌──────────────────────────────────────────┐│
│  │        Unified Storage (SQLite+WAL)      ││
│  └──────────────────────────────────────────┘│
│                   ▲                           │
│         gRPC / WebSocket / IPC                │
├───────────────────┼───────────────────────────┤
│    TUI (ratatui)  │  Desktop (Tauri+Vue)  │  Web (Actix+Vue) │
└─────────────────────────────────────────────┘
```

核心原则（参考 opencode/Reasonix 的"单核多前端"）：
- Core 是 transport-agnostic 的常驻服务，独占 SQLite 写入
- 三个端都是薄客户端，通过 gRPC/WebSocket/IPC 连接 Core
- 会话数据集中存储，任一端创建/修改的会话，其他端实时可见

#### 3.3.2 会话存储

| 数据类型 | 存储方式 | 说明 |
|---|---|---|
| 会话元数据 | SQLite | id、title、project、created_at、status |
| 消息历史 | SQLite | 结构化存储，支持分页、搜索 |
| 事件流 | SQLite | 工具调用、权限审批、spec 阶段推进等结构化事件 |
| Spec 文档 | 文件系统 (markdown) | `apex/specs/` 目录，同步导出 |
| Checkpoint | 文件系统 (markdown) | `apex/checkpoints/` 目录 |
| 记忆文件 | 文件系统 (markdown) | `apex/memory/` 目录，FTS5 索引存 SQLite |
| 文件快照 | 影子 Git 仓库 | `~/apex/snapshots/<project_hash>/<worktree_hash>/.git` |

#### 3.3.3 三端职责

| 端 | 技术栈 | 定位 | 关键能力 |
|---|---|---|---|
| TUI | Rust + ratatui | 日常使用主力 | 完整 Agent 交互、面板展示、审批操作 |
| 桌面端 | Tauri + Vue + TS | 可视化增强 | 富文本 spec 编辑、DAG 工作流可视化、面板图表 |
| Web 端 | Actix Web + Vue + TS | 轻量访问 | 会话查看、spec 评审、面板只读、审批操作 |

### 3.4 可观测面板（P0 —— 用户明确要求）

面板在所有三端可用，实时展示：

#### 3.4.1 Skill 标签页

- 已加载 Skill 列表（名称、描述、来源路径、触发状态）
- 当前正在执行的 Skill（高亮标记）
- 每个 Skill 的调用次数、token 消耗统计
- Skill 的渐进式加载层级指示（metadata → body → resources）

#### 3.4.2 MCP 标签页

- 已发现的 MCP 服务列表（名称、状态：connected/disconnected/error）
- 每个服务的工具列表（工具名、描述、参数 schema）
- 调用日志（时间戳、工具名、参数摘要、耗时、结果状态）
- 启用/禁用开关

#### 3.4.3 SubAgent 标签页

- 活跃子 Agent 列表（名称、任务描述、状态：running/completed/failed）
- 每个子 Agent 的：
  - 任务描述（在干什么）
  - 声明的 write_paths（写路径互斥范围）
  - 执行进度（当前步骤 / 总步骤）
  - 已产生的文件变更列表
  - token 消耗
- DAG 工作流图（桌面端/Web 端可视化，TUI 用 ASCII 图）

#### 3.4.4 Memory 标签页

- 记忆文件列表（路径、摘要、创建时间、最后召回时间）
- 当前会话中被召回的记忆（高亮标记 + 召回原因）
- 记忆编辑/删除/导出操作
- FTS5 搜索框

### 3.5 上下文管理（P0）

采用 **Checkpoint-first + 分级摘要兜底**（参考 MiMo-Code 混合策略）：

#### 3.5.1 Checkpoint 机制

- **触发点**：每个 spec 阶段完成时 + token 使用率达到窗口 60%/75%/85% 时
- **内容**：结构化 checkpoint.md，包含：
  - 当前 spec 阶段与状态
  - 已完成任务清单与关键决策
  - 活跃文件列表（最近读写过的文件）
  - 待办事项
  - 编码规范要点提醒
- **存储**：`apex/checkpoints/<session_id>/checkpoint_<n>.md`

#### 3.5.2 溢出恢复

当上下文窗口溢出时：
1. **首选**：加载最近 checkpoint + spec 文档，无损重建上下文
2. **兜底**：对 checkpoint 之前的对话历史做分级摘要（参考 Reasonix 四档策略：软提示 → snip 裁短工具结果 → prune 占位化 → LLM 摘要）
3. **spec 文档始终常驻上下文**，不参与摘要

#### 3.5.3 Prefix Cache 优化

- Volatile-content-last prompt 布局：稳定内容（system prompt、spec 文档、工具定义）在前，易变内容（环境信息、最近消息）在后
- 工具目录按名称排序，保证字节稳定
- Anthropic：`cache_control: {"type": "ephemeral"}` 标记精确放置
- OpenAI：`prompt_cache_key` 按 session_id 穿透

### 3.6 权限引擎（P0）

采用 **AST 静态解析**（参考 opencode/CodeWhale 方案）：

#### 3.6.1 权限模式

| 模式 | 行为 |
|---|---|
| `plan` | 只读工具放行，一切写操作需审批 |
| `ask` | 读操作放行，写操作/Bash 每次询问 |
| `allow` | 已批准规则放行，新操作询问 |
| `bypass` | 全部放行（仅限受信任项目，需显式开启） |

#### 3.6.2 Bash 命令解析

- 使用 tree-sitter 解析 Bash 命令 AST
- Decompose 出所有子命令（`$(...)`、管道、`&&`、`||`、`;`）
- 每个子命令独立过策略检查
- Arity 表：将 `git checkout main` 归一化为 `git checkout *` 语义规则
- 用户点击"总是允许"时存储语义化通用规则而非精确命令串
- 高危命令硬编码拦截（`rm -rf /`、`git push --force` 等），不可被"总是允许"覆盖

#### 3.6.3 文件操作权限

- 路径白名单/黑名单（glob 模式）
- 默认限制在项目根目录内，越界写入需显式审批
- 敏感文件保护（`.env`、`*.key`、`*.pem`、`credentials*`）

### 3.7 多 Agent 与工作流引擎（P1）

#### 3.7.1 子 Agent 系统

- 主 Agent 可通过 Task 工具 spawn 子 Agent
- 子 Agent 定义：markdown frontmatter（name、description、tools、model、write_paths）
- 上下文隔离：子 Agent 有独立消息历史，结果回传主会话
- 写路径互斥：可写子 Agent 必须声明 `write_paths`，调度器做路径互斥检查
- 并发限流：全局信号量 `min(16, 2 * cpu_cores)`

#### 3.7.2 DAG 工作流引擎

- 声明式 DAG：spec 的任务拆解阶段自动生成 DAG（任务 = 节点，依赖 = 边）
- 确定性重放：工作流状态持久化到 SQLite，crash 后可精确恢复
- 并行执行：无依赖关系的任务并行分派给子 Agent
- 暂停/恢复/部分回滚：用户可在 DAG 任意节点暂停、审批、回滚

### 3.8 Skills 系统（P1）

#### 3.8.1 格式兼容

- 基础格式完全兼容生态标准（YAML frontmatter + Markdown body）
- 发现路径（按优先级）：
  1. 项目级 `apex/skills/`
  2. 用户级 `~/apex/skills/`
  3. 兼容目录 `~/.claude/skills/`、`~/.codex/skills/`、`~/.agents/skills/`

#### 3.8.2 扩展字段

在标准 frontmatter 基础上支持可选扩展：

```yaml
---
name: rust-testing
description: Rust testing patterns with cargo test
allowed-tools: [Bash, Read, Write]
# --- 以下为 Apex 扩展字段 ---
spec-phase: implementation      # 绑定流水线阶段
requires-tools: [Bash]           # 声明依赖的工具
version: 1.2.0                   # 版本号
---
```

#### 3.8.3 渐进式加载

- 三层加载：metadata（常驻系统提示）→ body（触发时加载）→ resources（按需读取）
- 系统提示只注入 `name + description` 三元组，正文靠模型 `read` 加载

### 3.9 MCP 集成（P1）

#### 3.9.1 本地自动发现

- 扫描已知配置文件位置：
  - `~/.config/claude/claude_desktop_config.json`（Claude Desktop 格式兼容）
  - `apex/mcp.json`（项目级）
  - `~/apex/mcp.json`（用户级）
- 发现的 MCP 服务出现在面板中，一键启用/禁用

#### 3.9.2 连接管理

- 传输方式：stdio（本地子进程）、SSE/HTTP（远程）
- 工具命名空间：`mcp__<server>__<tool>`
- 进程树清理：MCP stdio 子进程退出时清理所有子孙进程（`pgrep -P` 策略）
- 连接状态实时监控，断线自动重连

### 3.10 记忆系统（P1）

#### 3.10.1 存储

- 目录：项目根 `apex/memory/`
- 格式：markdown 文件，带 YAML frontmatter（name、description、type、created_at）
- 索引：SQLite FTS5 全文索引

#### 3.10.2 智能检索（参考 Reasonix auto-recall）

- 关键词匹配：从当前用户消息和上下文中提取关键词
- FTS5 全文搜索召回最相关的记忆条目
- 召回的记忆注入到当前 user turn 尾部（不破坏前缀缓存）
- 记忆面板展示召回详情

#### 3.10.3 记忆生命周期

- Agent 自动创建：spec 流程中的关键决策、踩过的坑、用户纠正
- 用户手动创建：通过面板或 `/memory add` 命令
- 编辑/删除/导出：面板操作 + 文件系统直接编辑

### 3.11 LLM Provider 层（P0）

#### 3.11.1 架构

```
┌─────────────────────────────────┐
│      Provider Trait (统一抽象)    │
│  chat() / stream() / count()    │
├─────────────────────────────────┤
│  适配器层 (基于现有 Rust SDK)     │
│  ├─ AnthropicAdapter            │
│  ├─ OpenAIAdapter               │
│  ├─ DeepSeekAdapter (二期)      │
│  └─ KimiAdapter (二期)          │
├─────────────────────────────────┤
│  自研通道 (预留，二期)            │
│  └─ 针对特定模型的深度优化       │
└─────────────────────────────────┘
```

#### 3.11.2 MVP 首发

- **Anthropic**：`async-anthropic` 或 `reqwest` 自研（ephemeral cache_control 标记）
- **OpenAI**：`async-openai` crate（`prompt_cache_key` 穿透 + OpenAI 兼容端点）

#### 3.11.3 二期扩展

- DeepSeek：前缀缓存 24h TTL 优化
- Kimi：长上下文窗口适配
- 自研通道：针对特定模型绕过 SDK，直接 HTTP 调用实现深度优化

### 3.12 文件快照与回滚（P1）

- 影子 Git 仓库：`~/apex/snapshots/<project_hash>/<worktree_hash>/.git`
- 每回合前后自动快照
- `objects/info/alternates` 与真实仓库共享 object，不占双倍磁盘
- 支持文件级回滚、按 patch 部分回滚
- 不污染用户 `.git` 目录

---

## 四、非功能需求

### 4.1 性能

| 指标 | 目标 |
|---|---|
| TUI 启动时间 | < 500ms |
| 首次 LLM 响应延迟 | < 2s（网络正常） |
| 面板刷新频率 | 1s（可配置） |
| SQLite 查询延迟 | < 10ms（会话列表加载） |
| Checkpoint 生成 | < 5s（不打断用户流程） |
| 内存占用 | 常驻服务 < 200MB |

### 4.2 安全

| 维度 | 要求 |
|---|---|
| 命令注入防护 | tree-sitter AST 解析，decompose 所有子命令 |
| 路径穿越防护 | 文件操作限制在项目根目录内，越界需审批 |
| 敏感文件保护 | `.env`、`*.key`、`*.pem` 等默认只读 |
| 密钥管理 | API key 存 `~/apex/auth.json`（0600 权限），不入日志 |
| 审计日志 | 所有权限审批、skip-spec 决策、文件变更记录到 SQLite 事件流 |
| Prompt Injection 防护 | 外部内容（web fetch、MCP 结果）标记为不可信，不直接注入系统提示 |

### 4.3 可靠性

| 维度 | 要求 |
|---|---|
| 崩溃恢复 | 会话状态、工作流状态持久化到 SQLite，crash 后精确恢复 |
| 原子写入 | 会话文件、spec 文档使用原子写入（write to tmp + rename） |
| MCP 容错 | MCP 服务断连自动重连，工具调用超时 30s |
| LLM 容错 | API 调用指数退避重试（3 次），流式断线自动续传 |

### 4.4 可维护性

| 维度 | 要求 |
|---|---|
| 代码规范 | `Cargo.toml` 显式 deny `unwrap/expect/panic/exit`（参考 claude-code-rust） |
| 测试覆盖率 | 核心模块 ≥ 80% |
| 文档 | rustdoc 全覆盖公共 API，架构决策记录（ADR） |
| CI/CD | GitHub Actions：lint + test + build（TUI/Desktop/Web）+ release |

### 4.5 可扩展性

| 维度 | 要求 |
|---|---|
| Provider 插件化 | 统一 Provider trait，新 provider 只需实现 trait |
| Skill 插件化 | 文件系统约定，无需编译 |
| MCP 服务热插拔 | 运行时启用/禁用，不重启核心服务 |
| Hook 系统（二期） | PreToolUse / PostToolUse / Stop 等事件钩子 |

---

## 五、MVP 范围（v0.1）

### 5.1 MVP 包含

| 模块 | 范围 |
|---|---|
| LLM 层 | Anthropic + OpenAI 双 provider，基于现有 Rust SDK |
| Agent 核心 | 单会话 Agent Loop（输入 → prompt 组装 → LLM → 工具调用 → 结果回填） |
| Spec 流水线 | 完整四阶段 + 确认门 + `/skip-spec` 逃生门 |
| 工具集 | Read、Write、Edit、Bash、Glob、Grep、Task（简单子 Agent） |
| 权限引擎 | AST 解析 + 权限模式 + 高危命令拦截 |
| 规范校验 | spec 内嵌 + PostToolUse 增量检查 + 修复子任务 |
| TUI | 完整交互界面 + 可观测面板（Skill/MCP/SubAgent 标签页） |
| 会话存储 | SQLite + markdown 导出（spec/checkpoint/memory） |
| 上下文管理 | Checkpoint-first + 分级摘要兜底 |
| 文件快照 | 影子 Git 仓库 + 基础回滚 |

### 5.2 MVP 不包含（二期及以后）

| 模块 | 计划版本 |
|---|---|
| Tauri 桌面端 | v0.3 |
| Actix Web 端 | v0.3 |
| 三端会话共享 | v0.3 |
| DeepSeek / Kimi provider | v0.3 |
| DAG 工作流引擎 | v0.5 |
| 写路径互斥调度 | v0.5（并行调度）；**Write Claim 机制本身属 v0.1** |
| MCP 本地自动发现 | v0.5 |
| Skills 系统 | v0.5 |
| 记忆系统（FTS5 + 面板） | v0.5 |
| 确定性重放 | v0.7 |
| Hook 系统 | v0.7 |
| 插件 API | v1.0 |

### 5.3 版本路线图

```
v0.1  核心闭环：TUI + 双 provider + spec 流水线 + 基础工具 + 会话存储
v0.3  三端骨架：Tauri + Web + 会话共享 + DeepSeek/Kimi
v0.5  编排增强：DAG 工作流 + MCP + Skills + 记忆系统
v0.7  可靠性：确定性重放 + Hook 系统 + 高级权限
v1.0  完整发布：插件 API + 完整文档 + 稳定 API
```

---

## 六、技术选型汇总

| 层 | 选型 | 理由 |
|---|---|---|
| 语言 | Rust 2024（MSRV 1.85+） | 单二进制分发、性能、安全性 |
| 异步运行时 | tokio | Rust 异步事实标准 |
| TUI | ratatui | 最成熟的 Rust TUI 框架，CodeWhale/DeepSeek-TUI 验证 |
| 桌面端 | Tauri + Vue + TS | 用户前端技术栈偏好 + Rust 复用 |
| Web 端 | Actix Web + Vue + TS | 用户指定 + Rust 原生 Web 框架 |
| 数据库 | rusqlite (SQLite + WAL + FTS5) | 嵌入式、零依赖、事务、全文索引 |
| LLM SDK | async-openai + reqwest(Anthropic) | MVP 快速接入，预留自研通道 |
| Bash 解析 | tree-sitter-bash | AST 级命令解析，权限引擎基础 |
| 序列化 | serde + serde_json | Rust 序列化标准 |
| gRPC | tonic | 端间通信（三端共享会话） |
| Git 操作 | shell out 到系统 git | 避免 LGPL 依赖（参考 CodeWhale 决策） |
| 配置 | TOML + serde | 人类可读，Rust 生态标准 |

---

## 七、风险与缓解

| 风险 | 影响 | 缓解 |
|---|---|---|
| Spec 流水线体感重 | 用户觉得流程繁琐，弃用 | `/skip-spec` 逃生门 + 自适应提示（简单任务建议跳过） |
| Rust LLM SDK 生态不成熟 | 某些 provider 功能缺失 | 统一 Provider trait 预留自研通道，可逐 provider 替换 |
| 三端会话同步复杂度 | 状态不一致、冲突 | MVP 只做 TUI 单端，v0.3 再上三端；Core 独占写入避免冲突 |
| DAG 工作流过度设计 | 简单任务不需要 DAG | MVP 不含 DAG，v0.5 再引入；spec 任务拆解默认线性执行 |
| 开源竞品追赶 | 功能被抄袭 | 核心差异化在 spec 引擎与规范校验的深度集成，非单点功能 |
| 规范检查误报 | 阻断正常开发流程 | 增量检查只报错误级别，warning 不阻断；用户可配置规则集 |

---

## 八、关键设计决策记录（ADR 摘要）

| # | 决策 | 理由 | 参考 |
|---|---|---|---|
| 1 | Checkpoint-first 上下文管理 | spec 文档天然是结构化 checkpoint，无损重建优于 LLM 摘要 | MiMo-Code `checkpoint.ts` |
| 2 | AST 静态解析权限 | 确定性、可审计、不耗 token | opencode `permission/arity.ts`、CodeWhale `execpolicy` |
| 3 | 影子 Git 仓库做文件快照 | 三家独立项目收敛到同一方案，验证有效性 | opencode/DeepSeek-TUI/CodeWhale |
| 4 | SQLite 为主 + markdown 导出 | 性能与用户可审计性兼顾 | opencode `SQLite+Drizzle` |
| 5 | Volatile-content-last prompt | 保 prefix cache 命中，降低 API 成本 | CodeWhale/DeepSeek-TUI/Reasonix |
| 6 | 单核多前端 | 核心逻辑集中，UI 只是可替换前端 | opencode/Reasonix |
| 7 | 逃生门留痕 | 质量审计需要，跳过 spec 是技术债务信号 | — |
| 8 | 兼容生态 Skills 格式 | 零成本迁移用户已有 skill 资产 | pi/opencode/claude-code |
| 9 | 本地自动发现 MCP | 降低配置门槛，复用用户已有 MCP 配置 | claude-code `.mcp.json` |
| 10 | Rust SDK 先行 + 自研通道预留 | MVP 速度优先，长期优化不受限 | pi 的 Provider 抽象 |

---

## 九、附录

### 9.1 参考项目文档索引

| 项目 | 文档 | 核心参考价值 |
|---|---|---|
| opencode | `../docs/opencode 实现原理分析.md` | System Context 代数、arity 表、影子 Git、单核多前端 |
| CodeWhale | `../docs/CodeWhale 实现原理分析.md` | Fleet 编排、execpolicy、prefix cache、影子 Git |
| MiMo-Code | `../docs/MiMo-Code 实现原理分析.md` | Checkpoint-first、DAG 工作流、规范校验 |
| DeepSeek-TUI | `../docs/DeepSeek-TUI 实现原理分析.md` | 拒绝摘要、RLM、prefix cache pin tests |
| DeepSeek-Reasonix | `../docs/DeepSeek-Reasonix 实现原理分析.md` | Cache-first、四级裁剪、auto-recall、Bash AST |
| pi | `../docs/pi 实现原理分析.md` | Skills 兼容、Provider 抽象、JSONL 会话树 |
| claude-code 官方 | `../docs/claude-code（官方）实现原理分析.md` | Hook 协议、Skills 契约、权限模式、扩展生态 |
| claude-code-rust | `../docs/claude-code-rust 实现原理分析.md` | Rust TUI 前端架构、双 lane 命令通道 |

### 9.2 术语表

| 术语 | 定义 |
|---|---|
| Spec 流水线 | 需求 → 设计 → 任务 → 实现 → 验证 的结构化开发流程 |
| Checkpoint | 结构化上下文快照，用于溢出时无损重建 |
| Arity 表 | Bash 命令语义化归一规则表（`git checkout main` → `git checkout *`） |
| 渐进式加载 | Skill 的三层加载策略：metadata → body → resources |
| 写路径互斥 | 子 Agent 声明写路径，调度器保证路径不冲突 |
| 影子 Git 仓库 | 独立于用户 .git 的快照仓库，用于文件级回滚 |
| Prefix Cache | LLM provider 的前缀缓存机制，命中可大幅降低延迟和成本 |
| FTS5 | SQLite 的全文搜索扩展 |

---

> 本文档为需求分析阶段产出，后续将基于此文档生成：系统架构设计文档、模块详细设计文档、任务拆解与排期。
