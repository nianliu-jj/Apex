# Apex—— 项目开发计划（从最小粒度 TUI 闭环开始）

> 文档状态：开发基线（可执行任务拆解）  
> 适用范围：Apex v0.0.1 ～ v1.x 最终完整产品  
> 编制日期：2026-08-08  
> 依据文档：`Apex—— 需求分析文档.md`、`Apex—— 系统总体架构设计.md`、`Apex—— 领域模型与事件规范.md`、`Apex—— API与实时事件协议设计.md`、`Apex—— SQLite数据模型与迁移设计.md`、`Apex—— Agent Runtime与DAG调度器详细设计.md`、`Apex—— Tool Gateway与权限引擎详细设计.md`、`Apex—— Rules与Verification Gate详细设计.md`、`Apex—— MCP、Skill、Hook与Plugin扩展系统详细设计.md`、`Apex—— Credential与敏感数据治理详细设计.md`、`Apex—— Observability、审计与运维控制面详细设计.md`、`Apex—— Deployment、升级与灾备详细设计.md`  
> 参考实现：`DeepSeek-TUI`、`CodeWhale`、`codex/codex-rs`、`claude-code` 目录中的模块化、TUI、协议、状态、工具和扩展实现  
> 关键词：Development Plan、Vertical Slice、TUI、Fake Application、apexd、MVP、Task、Acceptance、Critical Path

---

## 0. 计划目的

本文把 Apex 的架构设计转换为可执行的开发计划，特别强调：

1. **从最小粒度 TUI 端开始**，先得到一个可以编译、启动、响应输入、渲染状态并优雅退出的最小闭环；
2. TUI 的每一步都使用未来 Core 的协议、Command、Query、Event 和 Port 边界，避免先写一个无法演进的“大而全单体 TUI”；
3. 使用垂直切片逐步增加真实能力：`TUI Mock → In-process Application → apexd IPC → SQLite Event Store → Provider → Tool → Spec → Rules → Runtime → 多端 → 扩展 → 运维发布`；
4. 每个任务都具备明确的输入、输出、依赖、验收标准和测试要求；
5. 在 v0.1 只实现必要的单会话闭环，但从第一天保留 v0.3/v0.5/v1.x 所需的架构边界。

本文是工程执行计划，不替代领域模型、协议和详细设计文档。若计划与详细设计冲突，应先记录冲突并更新 ADR，再修改任务。

---

## 1. 开发总原则

### 1.1 先垂直闭环，后横向扩展

每个里程碑优先交付“用户能操作、系统有事实、状态能恢复”的闭环，而不是只完成孤立的库：

```text
可启动
  → 可输入
  → 可产生 Command
  → 可得到 Domain Event
  → 可更新 Projection
  → 可渲染
  → 可重启恢复
  → 可测试
```

### 1.2 TUI 是客户端，不是业务核心

即使第一个版本只有 TUI，也不得把 Agent Loop、权限判定、SQLite 写入、文件写入和 Provider 调用直接写进 ratatui 页面模块。第一阶段可以使用进程内 Fake Application，但接口必须与未来 `apexd` 客户端相同：

```text
TUI View / Input
        ↓
TUI Controller
        ↓
ApplicationClient Port
        ↓
InProcessTransport（v0.0.x）或 NativeTransport（v0.1）
        ↓
Application/Core
```

### 1.3 事实优先于显示

TUI 的消息列表、状态栏、工具面板和审批提示都是 Query Projection 或事件缓存。禁止通过“追加一行文本”冒充业务状态；所有持久状态都必须通过 Command Handler 产生事件。

### 1.4 安全先于功能

即使在 Fake Provider 阶段，也要保留：

- ToolGateway；
- Permission Decision；
- `redaction_level`；
- `taint`；
- Command idempotency；
- `expected_revision`；
- 不可信外部内容标记；
- 高危动作确认门。

禁止先实现一个绕过权限和审计的“临时执行通道”，再承诺以后重构。

### 1.5 质量门不可延期

Rust 代码从第一个 commit 开始执行：

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
cargo deny / dependency audit（可用时）
markdown/document consistency check
```

生产代码避免 `unwrap`、`expect`、`panic`、`process::exit`；错误必须通过稳定错误类型传递到边界。

---

## 2. 交付范围与版本目标

### 2.1 版本路线

| 版本 | 用户可见目标 | 核心范围 |
|---|---|---|
| v0.0.1 | TUI 能启动和交互 | 静态页面、输入模型、Fake Application、事件渲染 |
| v0.0.2 | TUI 最小聊天闭环 | in-process Command/Query/Event、Fake Provider、会话状态 |
| v0.0.3 | TUI 连接 Core | `apexd`、本机 IPC、握手、游标和断线恢复 |
| v0.1.0 | 单机 MVP | SQLite、Provider、Tool、权限、Spec 流水线、Checkpoint、Snapshot、基础面板 |
| v0.2.x | MVP 稳定化 | 可靠性、性能、错误体验、跨平台打包、备份恢复 |
| v0.3.0 | 三端共享 | Tauri Desktop、Actix Web、WebSocket、DeepSeek/Kimi、统一面板 |
| v0.5.0 | 编排增强 | DAG、Write Claim、MCP、Skills、Memory、SubAgent 面板 |
| v0.7.0 | 可靠性增强 | Deterministic Replay、Hook、增强权限、恢复对账 |
| v1.0.0 | 最终完整产品 | Plugin API、稳定协议、完整 Observability、发布/升级/灾备 |
| v1.x | 远程与企业增强 | 远程部署、组织身份、外部备份/KMS、可选多实例/多租户 |

### 2.2 v0.0.x 明确不做

在 TUI 最小闭环阶段暂不实现：

- 真实 Provider 网络请求；
- 任意 Shell、文件写入和 Git 修改；
- SQLite 持久化；
- MCP 自动发现；
- DAG 和并行 SubAgent；
- Tauri、Web、远程服务；
- Credential 原文管理；
- 自动更新和远程 telemetry。

Fake Provider、Fake Tool 和 InMemory Event Store 只用于验证 UI/API 边界，不能称作产品功能已完成。

### 2.3 v0.1 MVP 必须实现

依据需求文档，v0.1 至少包含：

- TUI 完整交互界面；
- 单会话 Agent Loop；
- 双 Provider 抽象和至少一个真实 Provider；
- Read、Write、Edit、Bash、Glob、Grep、Task 基础工具；
- AST/规则驱动权限引擎和高危命令拦截；
- Spec 四阶段流水线、确认门和 `/skip-spec` 留痕；
- PostToolUse 增量规范检查与修复子任务基础版；
- SQLite + Markdown 导出；
- Checkpoint-first 上下文策略；
- Shadow Git 快照与基础回滚；
- Skill/MCP/SubAgent/调用日志面板的基础投影；
- 审计、脱敏、错误恢复和基础备份。

---

## 3. 角色、工作方式与任务粒度

### 3.1 角色

建议至少划分以下工程角色；早期可由同一人承担，但写入边界保持分离：

| 角色 | 负责 |
|---|---|
| TUI | ratatui、输入、渲染、客户端状态、交互测试 |
| Protocol | DTO、Command/Query/Event、版本协商、错误码 |
| Core/Application | 用例编排、领域命令、事件提交和投影 |
| Storage | SQLite、迁移、事务、备份和恢复 |
| Runtime | Session/Run/Turn/Agent/Tool 生命周期、取消和恢复 |
| Security | Permission、Credential、Redaction、Data Egress |
| Quality | Rules、Verification Gate、测试、CI、发布门禁 |
| Release/Ops | 打包、升级、灾备、Observability、Runbook |

### 3.2 最小任务定义

每个任务尽量满足：

- 单一目标；
- 单一主要写入范围；
- 1 个可验证输出；
- 0.25～2 个工程日完成，复杂任务拆为子任务；
- 可独立 review；
- 失败时能明确阻断下游任务。

### 3.3 任务状态

```text
planned → ready → in_progress → review → verified → done
                         │            │
                         ├────────────┴→ blocked
                         └──────────────→ cancelled
```

`done` 必须满足代码、测试、文档、审计和验收条件；模型输出、开发者口头说明或“本地能跑”不能替代验证。

### 3.4 任务字段模板

```yaml
task_id: TUI-001
title: 初始化 Cargo workspace
priority: P0
estimate: 0.5d
depends_on: []
write_scope:
  - Cargo.toml
  - crates/apex-tui/
output:
  - 可编译 workspace
acceptance:
  - cargo check --workspace 通过
  - cargo run -p apex-tui -- --help 可执行
verification:
  - fmt
  - clippy
  - unit test
```

---

## 4. 总体依赖图

```text
TUI-001 Workspace
   ↓
TUI-002 Domain IDs / Error / Result
   ↓
TUI-003 Protocol DTO ───────────────┐
   ↓                               │
TUI-004 Client Port                │
   ↓                               │
TUI-005 App State / Reducer         │
   ↓                               │
TUI-006 Static Renderer              │
   ↓                               │
TUI-007 Input / Command              │
   ↓                               │
TUI-008 InMemory Event Bus           │
   ↓                               │
TUI-009 Fake Application ────────────┘
   ↓
TUI-010 Fake Provider / Streaming
   ↓
TUI-011 Chat vertical slice
   ↓
TUI-012 Approval / Tool mock
   ↓
TUI-013 Snapshot tests / CI
   ↓
CORE-001 apexd bootstrap
   ↓
CORE-002 Native transport / Hello
   ↓
CORE-003 Event Store + SQLite
   ↓
CORE-004 Query projection / cursor
   ↓
CORE-005 TUI reconnect and recovery
   ↓
MVP-001 Real Provider
   ↓
MVP-002 Tool Gateway + Permission
   ↓
MVP-003 Spec pipeline
   ↓
MVP-004 Checkpoint / Snapshot
   ↓
MVP-005 Observability / Backup
   ↓
V03-001 Tauri / Web
   ↓
V05-001 DAG / MCP / Skills / Memory
   ↓
V07-001 Replay / Hooks / Advanced Recovery
   ↓
V10-001 Plugin / Stable Release / DR
```

### 4.1 关键原则

`TUI-003 Protocol DTO` 和 `TUI-004 Client Port` 必须早于真实 `apexd`，确保 TUI 不与 InMemory 实现耦合。`CORE-003 SQLite` 不能提前侵入页面组件；页面只依赖 Query View。

---

## 5. 推荐 Repository 结构

第一天创建完整边界，但可以只实现最少代码：

```text
apex/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── deny.toml
├── LICENSE
├── README.md
├── AGENTS.md
├── docs/
├── proto/
│   ├── apex/v1/common.proto
│   ├── apex/v1/commands.proto
│   ├── apex/v1/queries.proto
│   └── apex/v1/events.proto
├── migrations/
├── crates/
│   ├── apex-domain/
│   ├── apex-protocol/
│   ├── apex-application/
│   ├── apex-storage/
│   ├── apex-runtime/
│   ├── apex-policy/
│   ├── apex-tools/
│   ├── apex-provider/
│   ├── apex-observability/
│   ├── apex-recovery/
│   ├── apex-cli/
│   └── apex-tui/
├── apps/
│   ├── apexd/
│   ├── desktop/
│   │   ├── src-tauri/
│   │   └── ui/
│   └── web/
│       ├── server/
│       └── ui/
├── tests/
│   ├── contract/
│   ├── integration/
│   ├── fixtures/
│   └── golden/
└── scripts/
```

### 5.1 v0.0.x 的实现收缩

初始阶段可暂时把 `apex-application`、`apex-domain` 和 `apex-protocol` 合并为少量模块，但必须保持公共类型、Port 和模块依赖方向；不得把 Core 逻辑写进 `crates/apex-tui/src/ui.rs`。

### 5.2 参考项目的取舍

- 参考 `DeepSeek-TUI` 的 `tui` 与 `tui-core` 分离，保留渲染/状态边界；
- 参考 `CodeWhale` 的 `core`、`protocol`、`workflow`、`lane`、`release` 分层，但 Apex 不提前引入完整 Workflow VM；
- 参考 `codex-rs` 的 app-server/client/protocol 分离，确定 TUI 是协议客户端；
- 参考多个 Agent 的双 lane 设计，把取消、审批和安全阻断保留高优先级通道；
- 不复制任何参考项目的业务状态、命令格式或许可证不兼容代码。

---

## 6. 开发环境与固定基线

### 6.1 工具链

推荐固定：

```text
Rust edition: 2024
MSRV: 1.85+（若依赖实际要求更高，以发布 ADR 为准）
Tokio
Ratatui
Crossterm
Serde / serde_json
thiserror
tracing
rusqlite bundled SQLite
Prost/Tonic（Native protocol 阶段）
```

Cargo.lock 必须提交。SQLite bundled 版本、Protocol generator、Rust toolchain 和前端 Node/pnpm 版本写入构建文档。

### 6.2 第一批工程文件

- `../../../Cargo.toml` workspace；
- `../../../rust-toolchain.toml`；
- `rustfmt.toml`；
- `clippy.toml`；
- `deny.toml`；
- `.editorconfig`；
- `AGENTS.md`；
- CI workflow；
- `docs/ADR/`；
- `scripts/check-docs.ps1` 或跨平台等价脚本。

### 6.3 工程硬规则

```toml
[lints.rust]
unsafe_code = "deny"

[lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
```

具体 lint 需根据依赖和实现验证，不能在未编译前盲目加入造成无效配置；规则应通过 CI 逐步收紧。

### 6.4 本地验证命令

```text
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo run -p apex-tui -- --help
cargo run -p apexd -- --doctor
```

### 6.5 v0.1 执行级细化与日志基础设施先行

v0.1 的实际执行顺序、每个可运行阶段的输入/输出/验收/回滚，以及阶段之间的依赖，以 `docs/Apex—— v0.1 MVP逐功能可运行阶段计划.md` 为准。本计划保留总体架构和里程碑；执行时不得跳过该文件中的 `S00`～`S24` 阶段。

日志基础设施是 v0.1 的 `S00`，必须早于 TUI、Core、Storage、Provider 和 Tool 功能进入集成。任何后续阶段都必须能够在一次运行内生成 `run_id` 关联的本地日志，并在日志中保留：UTC 时间、日志序号、进程 PID、操作系统线程 ID/名称、显式注册的 Tokio 协程/任务 ID/名称、真实源码文件绝对路径、源码行号、模块、级别、稳定 `message_code` 和进度字段。

当前已落地的最小实现为：

- `crates/apex-observability`：基于 `tracing` 的结构化文件日志库；
- `apps/apex-log-demo`：可重复运行的日志验收程序；
- `logs/<component>-<run_id>.log`：每次运行独立文件，Spring Boot 风格的单行位置式格式；
- 默认阻塞式有界队列、独立写线程、逐条 flush、敏感字段名脱敏和显式 TaskContext；
- `cargo test --workspace --lib --bins` 与日志产物字段验收作为 S00 的第一道门。

日志属于诊断数据，不替代 SQLite Event Store 中的业务事实；日志字段、脱敏和查询能力后续由 Observability 阶段继续扩展。

---

## 7. 第一阶段：TUI 最小可运行闭环

这一阶段的目标不是聊天，而是构建可持续演进的 TUI 外壳。每个任务都可以单独完成和验证。

### TUI-001：初始化 Workspace

- **目标**：创建 Cargo workspace、`apex-tui` binary 和最小 `apex-domain`/`apex-protocol` library。
- **依赖**：无。
- **输出**：可执行 `apex-tui`、workspace metadata、锁定工具链。
- **验收**：`cargo check --workspace` 和 `cargo run -p apex-tui -- --version` 通过。
- **测试**：CI 中加入 fmt/check。

### TUI-002：建立错误和 ID 基础类型

- **目标**：定义 `ProjectId`、`SessionId`、`RunId`、`TurnId`、`CommandId`、`EventId`、`CorrelationId` 和统一错误层级。
- **依赖**：TUI-001。
- **输出**：类型可序列化、可显示、不可混用。
- **验收**：不同 ID 在 Rust 类型层面不能直接互相赋值；错误可映射为安全用户提示。
- **测试**：serde roundtrip、Display、错误分类单测。

### TUI-003：定义最小协议 DTO

- **目标**：实现 `Hello`、`Ready`、`CommandEnvelope`、`QueryRequest`、`QueryResponse`、`EventEnvelope`。
- **依赖**：TUI-002。
- **最小命令**：`session.open`、`turn.submit`、`session.cancel`、`app.shutdown`。
- **最小查询**：`session.current`、`session.timeline`、`app.health`。
- **最小事件**：`session.opened`、`turn.submitted`、`turn.completed`、`error.occurred`。
- **验收**：DTO canonical JSON roundtrip；事件带 schema/version/correlation。
- **测试**：固定 JSON fixtures 和 unknown-field 兼容测试。

### TUI-004：定义 ApplicationClient Port

- **目标**：TUI 只依赖客户端 Port，不依赖 Fake Application 或未来 `apexd` 实现。
- **依赖**：TUI-003。
- **接口**：`hello`、`execute_command`、`query`、`subscribe`、`close`。
- **验收**：TUI crate 的依赖图中不出现 `rusqlite`、Provider SDK、文件写入库。
- **测试**：使用 mock client 验证命令调用参数和查询请求。

### TUI-005：建立 AppState

- **目标**：定义最小不可变状态：连接状态、当前 Session、消息、输入框、错误提示、退出状态、游标。
- **依赖**：TUI-003、TUI-004。
- **输出**：`AppState`、`AppAction`、`AppEffect`、`ConnectionState`。
- **验收**：State 更新只能通过 Reducer/Action，不在 Renderer 中修改状态。
- **测试**：纯函数 reducer 单测，覆盖重复事件、乱序事件和未知事件。

### TUI-006：实现终端生命周期

- **目标**：进入 raw mode、alternate screen、panic/错误清理和优雅退出。
- **依赖**：TUI-001。
- **验收**：Ctrl-C、`q`、启动异常和 panic 后终端恢复；退出码稳定。
- **测试**：TerminalGuard 单测；Unix/Windows CI 至少执行非交互启动检查。

### TUI-007：实现静态三栏布局

- **目标**：渲染 Header、Conversation、Status/Side panel 三个区域。
- **依赖**：TUI-005、TUI-006。
- **验收**：80x24、120x40、窄终端下不 panic；中文/emoji 不破坏主布局。
- **测试**：渲染 snapshot/golden；空状态、长文本、错误状态。

### TUI-008：实现输入编辑器

- **目标**：支持字符输入、退格、左右移动、Enter 提交、Esc 清空/取消、Ctrl-C 退出。
- **依赖**：TUI-007。
- **验收**：输入状态与渲染一致；提交后输入清空；组合键不产生乱码。
- **测试**：键盘事件序列 property tests；中文输入以终端事件为准进行集成验证。

### TUI-009：实现事件驱动渲染循环

- **目标**：将键盘事件、Core Event、tick 和异步错误合并到统一事件循环。
- **依赖**：TUI-005、TUI-006、TUI-007、TUI-008。
- **验收**：一个事件不会重复渲染或重复提交；tick 可配置；退出时所有 task 被取消。
- **测试**：事件队列边界、慢消费者、关闭竞态。

### TUI-010：实现 Fake Application

- **目标**：提供进程内的最小 ApplicationClient，实现 Session/Turn 的确定性状态转移。
- **依赖**：TUI-003、TUI-004、TUI-005。
- **行为**：`session.open` 返回 Session；`turn.submit` 返回用户消息、assistant delta、turn completed。
- **验收**：无需网络和文件即可完整启动、输入、提交、看到响应和退出。
- **测试**：command idempotency、expected revision、event order。

### TUI-011：完成第一个垂直切片

- **目标**：交付 `apex tui --demo`。
- **依赖**：TUI-006～TUI-010。
- **用户流程**：启动 → 打开 Demo Session → 输入 `hello` → 看到 assistant 回复 → `/quit` 退出。
- **验收**：在干净环境运行；不读用户项目文件；不产生外部网络请求；错误可恢复。
- **发布物**：内部 `v0.0.1` binary。

---

## 8. 第二阶段：TUI Mock Chat 与协议稳定化

目标：让 TUI 从“静态 Demo”变成可测试的 Chat Client，同时不引入真实网络和危险工具。

### TUI-012：消息模型与渲染分层

- **目标**：区分 UserMessage、AssistantMessage、SystemMessage、ToolSummary、ApprovalSummary、ErrorNotice。
- **依赖**：TUI-005、TUI-007。
- **验收**：消息不会把原始事件 JSON 直接渲染；每类消息有安全摘要和状态。
- **测试**：不同状态、长内容截断、taint/redaction 标记。

### TUI-013：流式 Assistant Delta

- **目标**：支持 transient delta 合并为一条 assistant message。
- **依赖**：TUI-009、TUI-010、TUI-012。
- **验收**：delta 丢失不会破坏最终 `turn.completed`；完成事件到达后以持久结果为准。
- **测试**：乱序 delta、重复 delta、连接中断、完成事件先到。

### TUI-014：命令历史和本地输入导航

- **目标**：上下键历史、可配置历史长度、敏感输入不落盘。
- **依赖**：TUI-008。
- **验收**：普通命令可以回看；含 Credential/Secret 标记的输入不写日志和历史文件。
- **测试**：历史去重、容量限制、敏感内容过滤。

### TUI-015：Slash Command 路由

- **目标**：实现 `/help`、`/quit`、`/clear`、`/status`、`/session`、`/cancel`、`/skip-spec` 的解析外壳。
- **依赖**：TUI-008、TUI-003。
- **验收**：Slash Command 与自然语言输入在类型层面分离；未知命令返回安全帮助。
- **测试**：空参数、引号、Unicode、未知命令和参数过多。

### TUI-016：协议错误映射

- **目标**：将 Core/Transport 错误映射为用户可理解的错误卡片。
- **依赖**：TUI-003、TUI-012。
- **覆盖**：版本不兼容、权限拒绝、超时、取消、数据库只读、Projection 滞后、未知外部副作用。
- **验收**：不向用户显示 Secret、完整堆栈或底层 SQL/HTTP body。

### TUI-017：Mock Provider 行为脚本

- **目标**：为 Fake Application 增加确定性 Provider 脚本：立即完成、流式输出、工具请求、超时、错误、取消。
- **依赖**：TUI-010、TUI-013。
- **验收**：测试可通过 fixture 重现同一事件序列；随机延迟由可控时钟驱动。
- **测试**：每种 Provider outcome 一组 golden event fixture。

### TUI-018：Chat Vertical Slice v0.0.2

- **目标**：交付可演示的 Mock Chat。
- **依赖**：TUI-012～TUI-017。
- **用户流程**：打开 Session → 输入消息 → 流式回复 → 查看状态/用量摘要 → 取消或退出。
- **验收**：核心 UI 逻辑可在无终端模式下通过 reducer 测试；交互演示不依赖网络。
- **发布物**：`v0.0.2` 内部版本和协议 fixture。

---

## 9. 第三阶段：TUI 面板与审批 Mock

目标：先验证需求中明确要求的可观测面板和审批体验，再接入真实 Runtime。

### TUI-019：Panel Model 基础

- **目标**：定义 `OverviewPanel`、`SkillPanel`、`McpPanel`、`SubAgentPanel`、`CallLogPanel`。
- **依赖**：TUI-005、TUI-012。
- **字段**：状态、时间、操作 ID、耗时、结果、摘要、token/call count、redaction、as_of_seq。
- **验收**：Panel 只消费 projection view，不直接读取 Domain Event 原始 payload。

### TUI-020：面板切换和焦点管理

- **目标**：实现 Tab/快捷键切换、焦点区域、滚动和返回会话。
- **依赖**：TUI-007、TUI-019。
- **验收**：面板切换不会丢失输入；窄屏下退化为单面板；快捷键可在帮助中发现。
- **测试**：焦点状态机和渲染 snapshot。

### TUI-021：实时刷新和 `as_of_global_seq`

- **目标**：面板由事件触发刷新，显示投影水位和滞后提示。
- **依赖**：TUI-019、TUI-020、TUI-003。
- **验收**：不做全表轮询；事件缺口时显示 refresh required；高频进度合并。

### TUI-022：Approval View

- **目标**：展示 Tool/Permission/Spec/Gate 的审批卡片。
- **依赖**：TUI-012、TUI-019。
- **字段**：风险等级、工具名、目标摘要、参数摘要、原因、过期时间、approval ID。
- **安全要求**：由 Core 生成 `approval_summary`；TUI 不从原始参数推导安全结论。

### TUI-023：Approval Command Mock

- **目标**：实现 Approve、Deny、Later、Cancel 的 Command 调用和状态更新。
- **依赖**：TUI-003、TUI-010、TUI-022。
- **验收**：重复批准、过期批准、版本冲突和无权限均显示确定性结果；不允许 UI 本地伪造批准成功。

### TUI-024：高优先级取消通道

- **目标**：将取消、安全阻断和审批响应与普通输出分离。
- **依赖**：TUI-009、TUI-023。
- **验收**：大量 delta 或日志压力下，取消仍能在可接受时间内到达；取消不会丢失审计。

### TUI-025：TUI Panel Vertical Slice v0.0.3

- **目标**：交付会话、调用日志、审批和基础面板演示。
- **依赖**：TUI-019～TUI-024。
- **验收**：演示脚本覆盖成功、拒绝、超时、取消、滞后和未知副作用六类状态。

---

## 10. 第四阶段：apexd 最小服务与本机连接

目标：把 TUI 从进程内 Fake Application 切换到真实 `apexd`，同时保留 Fake Transport 作为测试后端。

### CORE-001：创建 `apexd` binary

- **目标**：增加 `apps/apexd`（实际目录统一为 `apps/apexd`）组装根。
- **依赖**：TUI-003、总体 Workspace 结构。
- **验收**：`apexd --version`、`apexd --doctor`、`apexd --home <path>` 可执行。
- **约束**：启动器不直接处理业务命令。

### CORE-002：实例身份与单实例锁

- **目标**：生成 `instance_id`，对 Apex Home 建立 single-instance lock。
- **依赖**：CORE-001、Deployment §4/§5。
- **验收**：第二个 daemon 不能打开同一 writer；锁异常有安全恢复提示。
- **测试**：并发启动、异常退出、过期锁诊断。

### CORE-003：Native endpoint discovery

- **目标**：实现 Unix socket/Windows named pipe 抽象和非 Secret endpoint discovery。
- **依赖**：CORE-001、CORE-002。
- **验收**：TUI 能发现同机 daemon；无法使用 Native IPC 时明确回退 loopback。
- **测试**：endpoint 失效、PID 不存在、协议版本不兼容。

### CORE-004：Hello/Challenge/Ready

- **目标**：实现连接握手、短期 token、协议范围协商和连接 Capability。
- **依赖**：CORE-003、TUI-003。
- **验收**：无效 token、过期 token、错误 nonce、无版本交集都 fail-closed。
- **测试**：握手状态机、重放 token、客户端版本矩阵。

### CORE-005：NativeTransport Client

- **目标**：TUI 的 `ApplicationClient` 增加 NativeTransport 实现。
- **依赖**：CORE-004、TUI-004。
- **验收**：同一 TUI Controller 可切换 InProcessTransport/NativeTransport，不修改业务 UI。
- **测试**：Transport contract tests 复用同一组 fixture。

### CORE-006：apexd 进程生命周期

- **目标**：实现 booting、ready、degraded、quiescing、stopped 和 graceful shutdown。
- **依赖**：CORE-001～CORE-005。
- **验收**：TUI 收到 `server_restart` 后显示重连提示；daemon 关闭不产生虚假成功事件。

### CORE-007：连接失败与重连体验

- **目标**：TUI 支持 daemon 未启动、启动中、重启、不可达和版本不兼容。
- **依赖**：TUI-016、CORE-005、CORE-006。
- **验收**：认证失败/版本不兼容不无限重试；临时网络/进程重启采用有界退避。

### CORE-008：apexd/TUI Vertical Slice v0.0.4

- **目标**：交付真实进程边界下的 Mock Chat。
- **用户流程**：启动 `apexd` → TUI 连接 → 打开 Session → Mock Chat → 断开/重启 daemon → 重新连接。
- **依赖**：CORE-001～CORE-007。
- **验收**：TUI 不依赖进程内 Fake Application 才能运行；FakeTransport 仍保留。

---

## 11. 第五阶段：领域状态与 SQLite 事件闭环

目标：把内存状态换成可恢复的 SQLite Event Store 和可重建 Projection。

### STORAGE-001：SQLite Bootstrap

- **目标**：按 SQLite 设计创建 `apex.db`、WAL、FTS5/JSON1 能力和 application_id。
- **依赖**：CORE-001、Deployment §4、SQLite 设计 §2/§3。
- **验收**：数据库目录预检通过；读连接 query_only；writer 只有一个。
- **测试**：跨平台 bootstrap、权限、共享目录拒绝。

### STORAGE-002：Migration Runner

- **目标**：实现 schema revision、checksum、forward-only migration 和 maintenance 状态。
- **依赖**：STORAGE-001。
- **验收**：空库初始化、重复迁移、checksum 改变、半迁移恢复行为确定。

### STORAGE-003：Domain Event 表与 Writer

- **目标**：实现 event envelope、canonical JSON、global_seq、aggregate version 和单事务提交。
- **依赖**：TUI-003、STORAGE-001、领域事件规范。
- **验收**：commit 前不广播；提交后事件不可变；重复 command 不产生重复事实。
- **测试**：事务回滚、并发 command、版本冲突、payload digest。

### STORAGE-004：Session/Message/Run Projection

- **目标**：实现当前 TUI 查询所需的最小投影。
- **依赖**：STORAGE-003。
- **验收**：可从事件删除并重建；查询返回 `as_of_global_seq` 和 projection revision。

### STORAGE-005：Outbox 与 Event Consumer

- **目标**：为广播和可靠消费者写入 outbox/cursor。
- **依赖**：STORAGE-003、Observability 设计。
- **验收**：广播失败不回滚事实；重启后 outbox 可继续发送；消费者幂等。

### STORAGE-006：Event Query

- **目标**：实现按 project/session/run/type/seq/time/cursor 过滤的安全查询。
- **依赖**：STORAGE-004、STORAGE-005、权限 Port。
- **验收**：响应有最大 rows/bytes；原始 Secret/Prompt 不出现在 safe view。

### STORAGE-007：TUI Cursor Replay

- **目标**：TUI 保存最后消费游标，重连时 snapshot → replay → live。
- **依赖**：CORE-007、STORAGE-004～STORAGE-006。
- **验收**：断线期间持久事件不丢；transient delta 可丢并从最终 Query 修正。

### STORAGE-008：SQLite TUI Vertical Slice v0.0.5

- **目标**：真实数据库保存和恢复 TUI Session/Message/Turn。
- **用户流程**：输入消息 → daemon 提交事件 → daemon 重启 → TUI 重连 → 历史和当前状态恢复。
- **依赖**：STORAGE-001～STORAGE-007。
- **验收**：符合 I1～I5 不变量；无直接 UI 写表。

---

## 12. 第六阶段：Provider 抽象与单会话 Agent Loop

目标：在真实 Core/SQLite/协议基础上接入 Provider，但先限制为串行单会话。

### PROVIDER-001：Provider Port

- **目标**：定义统一 `Provider` trait、请求快照、响应流、usage、错误和取消。
- **依赖**：TUI-003、STORAGE-003。
- **验收**：Provider adapter 不直接修改 Session/Run；所有结果由 Application Handler 收敛。

### PROVIDER-002：Fake Provider Contract

- **目标**：把 TUI-017 Mock Provider 迁移为共享测试 adapter。
- **依赖**：PROVIDER-001。
- **验收**：真实 Provider 与 Fake Provider 复用同一 contract test。

### PROVIDER-003：Provider 配置与 CredentialRef

- **目标**：保存 endpoint/model/capability/timeout 和 CredentialRef metadata。
- **依赖**：PROVIDER-001、Credential 设计。
- **验收**：配置和事件不含 Secret；Credential Store 不可用时依赖操作 fail-closed。

### PROVIDER-004：第一个真实 Provider

- **目标**：接入一个目标 Provider，完成请求、流式 delta、usage、错误和取消。
- **依赖**：PROVIDER-001～PROVIDER-003。
- **验收**：网络失败、超时、限流、认证失败、空响应均映射稳定错误码。
- **测试**：wiremock/fake HTTP 或官方测试 endpoint，CI 默认不使用真实密钥。

### PROVIDER-005：第二个 Provider

- **目标**：接入第二个 Provider，验证 adapter 可替换性。
- **依赖**：PROVIDER-004。
- **验收**：TUI/Core 不增加 Provider-specific 分支；usage 和模型摘要统一。

### RUNTIME-001：Session/Run/Turn 状态机

- **目标**：实现单会话串行生命周期：created → running → awaiting_approval/blocked → completed/failed/cancelled。
- **依赖**：STORAGE-003、PROVIDER-001。
- **验收**：终态不可回退；迟到 Provider 结果不能覆盖终态。

### RUNTIME-002：ApplicationService

- **目标**：实现 `session.open`、`turn.submit`、`turn.cancel`、`run.resume` 用例。
- **依赖**：RUNTIME-001、PROVIDER-004、STORAGE-006。
- **验收**：Command 幂等、expected revision 校验、事件/投影/outbox 顺序正确。

### RUNTIME-003：Context Builder 最小版

- **目标**：组合系统指令、项目规则、Session 历史、当前用户消息和安全摘要。
- **依赖**：RUNTIME-002、Context 设计。
- **验收**：稳定前缀与易变后缀分离；外部内容标记 untrusted；不把敏感数据直接写入日志。

### RUNTIME-004：单会话 Agent Loop

- **目标**：实现 `input → context → provider → delta → final → usage → event`。
- **依赖**：RUNTIME-001～RUNTIME-003。
- **验收**：TUI 可使用真实 Provider 完成一次回答；取消和超时有可恢复状态。

### RUNTIME-005：Provider Vertical Slice v0.1-alpha

- **目标**：交付第一个真实 Agent Loop。
- **依赖**：PROVIDER-005、RUNTIME-004。
- **验收**：Fake Provider 全量测试通过；真实 Provider 手工验收；CI 不依赖 Secret。

---

## 13. 第七阶段：Tool Gateway 与权限引擎

目标：在 Agent Loop 中接入受控工具执行，先实现只读工具，再逐步开放写入和 Shell。

### TOOL-001：Tool 定义与 Operation ID

- **目标**：定义 ToolSpec、ToolCall、ToolResult、OperationJournal 和稳定 operation_id。
- **依赖**：TUI-003、RUNTIME-004。
- **验收**：每次工具调用都可关联 Session/Run/Turn/Command/causation；结果包含状态、耗时、taint、redaction、external effect state。

### TOOL-002：ToolGateway Port

- **目标**：所有工具通过统一 Gateway，UI/Agent 不能直接执行。
- **依赖**：TOOL-001、RUNTIME-002。
- **验收**：不存在公开 execute-tool backdoor；调用前必须经过 capability、policy、version 和 lease 检查。

### TOOL-003：Read/Glob/Grep

- **目标**：实现工作区边界内的只读工具。
- **依赖**：TOOL-002、Workspace 设计。
- **安全要求**：canonical path、符号链接、大小限制、敏感路径分类和输出截断。
- **验收**：不能读取 workspace 外路径、Secret 文件或超额内容；返回安全摘要和可选 Blob capability。

### TOOL-004：Permission AST 解析

- **目标**：实现 Bash 命令分词/AST、规则匹配、风险等级和默认 deny。
- **依赖**：TOOL-002、需求文档权限约束。
- **验收**：管道、重定向、命令替换、链式命令、引号、平台差异均有测试；无法证明安全时拒绝或要求审批。

### TOOL-005：Write/Edit

- **目标**：实现写入意图、临时文件、digest、原子 rename 和 Write Claim 接入前的单任务保护。
- **依赖**：TOOL-003、TOOL-002、Storage write intent。
- **验收**：不能静默覆盖冲突；文件变更有 before/after digest 和事件；敏感文件默认阻断。

### TOOL-006：Bash Supervisor

- **目标**：以受监督子进程运行 Bash，限制环境、cwd、超时、输出、取消和退出码。
- **依赖**：TOOL-004、TOOL-005、Deployment 权限设计。
- **验收**：子进程崩溃不损坏 Core；未知副作用标记 unknown；完整 Secret 不进日志/事件。

### TOOL-007：Approval 集成

- **目标**：将 Permission Decision、Approval Request、Grant/Deny/Expiry 接入 Domain Event 和 TUI。
- **依赖**：TOOL-002、TUI-022～TUI-024。
- **验收**：Approval 绑定 tool、scope、risk、args digest、policy revision 和 expiry；参数改变必须重新判权。

### TOOL-008：Tool Result Normalizer

- **目标**：统一成功、失败、超时、取消、拒绝、截断和未知外部效果。
- **依赖**：TOOL-003、TOOL-006。
- **验收**：面板可展示安全摘要；原始输出只通过受保护 Blob capability 访问并审计。

### TOOL-009：Tool Vertical Slice v0.1-beta

- **目标**：TUI 中完成 Read → Agent 决策 → Permission → Tool → Result → Rule/usage 事件闭环。
- **依赖**：TOOL-001～TOOL-008。
- **验收**：只读工具成功、越权拒绝、高危命令审批、取消、超时、重启恢复均可演示和测试。

---

## 14. 第八阶段：Spec-Driven Development 流水线

目标：实现 Apex 的核心差异化，不把 Spec 作为 TUI 文本模板，而是作为有状态、有确认门、有事件和可验证产物的领域能力。

### SPEC-001：Spec/Artifact 领域模型

- **目标**：定义 Spec、ArtifactRevision、Stage、ConfirmationGate、AcceptanceCriterion。
- **依赖**：STORAGE-003、RUNTIME-001。
- **阶段**：requirements、design、tasks、implementation、verification。
- **验收**：上游修订会使依赖下游失效；阶段终态有明确事件和版本。

### SPEC-002：Markdown Artifact Port

- **目标**：实现 `apex/specs/`、`apex/checkpoints/`、`apex/memory/` 的安全读写镜像。
- **依赖**：TOOL-005、Storage write intent。
- **验收**：文件 rename 与 DB revision 有 Recovery 对账；冲突保留两份，禁止静默覆盖。

### SPEC-003：Requirements Stage

- **目标**：TUI 中创建、编辑、查看需求文档和验收标准。
- **依赖**：SPEC-001、SPEC-002、TUI-019。
- **验收**：用户确认前不能进入 design；确认产生 Approval/Domain Event。

### SPEC-004：Design Stage

- **目标**：生成/编辑 design artifact，并加载已编译 Rules/约束摘要。
- **依赖**：SPEC-003、Rules 设计。
- **验收**：设计文档包含来源、约束、接口、验收标准和 policy revision；模型自报完成不算通过。

### SPEC-005：Tasks Stage

- **目标**：生成任务列表、依赖、串并行标记、write path 和验收条件。
- **依赖**：SPEC-004。
- **v0.1 限制**：只支持线性或简单顺序任务，不实现完整 DAG。
- **验收**：任务可逐项开始/完成/阻断，后续任务受上游状态约束。

### SPEC-006：Implementation Stage

- **目标**：将任务映射到 Agent/Tool 操作，记录代码变更、测试和规则检查。
- **依赖**：SPEC-005、TOOL-009。
- **验收**：每项任务有 command/run/operation/commit 关联；写入路径和结果可追踪。

### SPEC-007：Verification Stage

- **目标**：生成 verification artifact，汇总需求、设计、任务、规则、测试和例外。
- **依赖**：SPEC-006、RULE-001～RULE-005（见下一阶段）。
- **验收**：状态仅允许 passed/failed/blocked/not_run；不接受模型自报完成。

### SPEC-008：Skip Spec 逃生门

- **目标**：支持 `/skip-spec`，但强制记录理由、范围、风险、actor、policy revision 和后续补偿项。
- **依赖**：SPEC-003～SPEC-005、TUI-015。
- **验收**：跳过不是删除阶段；后续仍触发增量规则检查和审计面板。

### SPEC-009：Spec TUI Vertical Slice v0.1-rc1

- **目标**：完成需求 → 设计 → 任务 → 实现 → 验证的串行 MVP。
- **依赖**：SPEC-001～SPEC-008、TOOL-009、RULE-001～RULE-005。
- **用户验收**：TUI 能创建项目、确认需求、查看设计、批准任务、执行最小修改并生成验证报告。

---

## 15. 第九阶段：Rules、Verification Gate 与质量闭环

### RULE-001：规则来源加载

- **目标**：按优先级加载项目 `apex/rules/`、`AGENTS.md`/`CLAUDE.md`、全局规则。
- **依赖**：STORAGE-001、SPEC-001。
- **验收**：来源、层级、digest、优先级和冲突信息可查询；外部内容不能覆盖安全硬规则。

### RULE-002：规则编译器

- **目标**：将规则编译为 prompt constraints、checker 配置和 severity。
- **依赖**：RULE-001。
- **验收**：编译结果有 revision/digest；无效规则进入 blocked/diagnostic，不静默忽略。

### RULE-003：Incremental Checker

- **目标**：只对本次变更文件执行 formatter/lint/test/AST 检查。
- **依赖**：TOOL-005、RULE-002。
- **验收**：warning 默认不阻断；error/high-risk 按规则阻断；输出经过 Redactor。

### RULE-004：PostToolUse Hook 基础内核

- **目标**：Tool 完成后触发规则检查，结果通过事件/Command 返回。
- **依赖**：TOOL-008、RULE-003、Observability 事件规范。
- **验收**：检查失败可派修复子任务或等待用户；不得在 Hook 内直接绕过 Permission。

### RULE-005：Verification Gate

- **目标**：实现 Gate 的输入快照、检查项、证据、结论和 Waiver。
- **依赖**：RULE-003、SPEC-007。
- **验收**：Gate 终态不可回退；Waiver/Skip Spec 必须高等级审计；证据引用可追踪。

### RULE-006：质量面板

- **目标**：TUI 展示规则命中、失败、修复建议、Gate、Waiver、Skip rate。
- **依赖**：RULE-005、TUI-019～TUI-021。
- **验收**：用户能从失败项定位到任务/文件/operation，但不泄漏敏感文件内容。

### RULE-007：Rules Vertical Slice

- **目标**：一次文件修改后自动检查，失败后阻断或派修复，最终生成验证结论。
- **依赖**：SPEC-009、RULE-001～RULE-006。
- **验收**：通过、warning、error、skip、waiver、checker crash 六种路径都有明确结果。

---

## 16. 第十阶段：Context、Checkpoint 与 Snapshot

### CONTEXT-001：消息与 Context 输入边界

- **目标**：建立 system/project/spec/session/tool/external 各类 Context 来源及可信级别。
- **依赖**：RUNTIME-003、SPEC-001。
- **验收**：外部 Provider/MCP/Web 内容标记 untrusted；不能修改权限语义或注入系统规则。

### CONTEXT-002：Context Budget 与摘要策略

- **目标**：按稳定前缀、易变后缀和预算裁剪上下文。
- **依赖**：CONTEXT-001、PROVIDER-001。
- **验收**：预算不足时先压缩/引用，不静默删除用户确认的 Spec 或安全约束。

### CONTEXT-003：Checkpoint Schema

- **目标**：保存 Run/Turn/Agent 的可恢复摘要、输入范围、digest、生成版本和 Blob 引用。
- **依赖**：STORAGE-003、CONTEXT-002。
- **验收**：Checkpoint 是恢复线索，不替代 Domain Event；敏感内容遵守独立存储策略。

### CONTEXT-004：Checkpoint Materializer

- **目标**：异步生成 Markdown/Blob Checkpoint，不阻塞主事件提交。
- **依赖**：CONTEXT-003、SPEC-002、Observability。
- **验收**：崩溃后 pending materializer 可恢复；内容 digest 与事件一致。

### SNAPSHOT-001：Shadow Git 初始化

- **目标**：为项目创建独立 Shadow Git，不污染用户 `.git`。
- **依赖**：SPEC-002、Deployment 路径规则。
- **验收**：项目工作区、Shadow Repo、Snapshot metadata 有稳定关联。

### SNAPSHOT-002：Snapshot Create/Restore

- **目标**：实现创建快照、比较、恢复前预览和恢复后校验。
- **依赖**：SNAPSHOT-001、TOOL-005、Workspace 设计。
- **验收**：恢复不会静默覆盖；冲突、缺对象、路径越界和 claim 占用有明确处理。

### SNAPSHOT-003：Context/Snapshot Vertical Slice

- **目标**：完成“任务前 Checkpoint/Snapshot → 修改 → 验证失败 → 恢复”的 MVP。
- **依赖**：CONTEXT-004、SNAPSHOT-002、RULE-007。
- **验收**：至少一条故障恢复演示可重复；所有恢复动作有事件和审计。

---

## 17. 第十一阶段：Observability、审计与运维最小闭环

目标：让每个已实现能力都可解释、可查询、可恢复，再扩展高级运维。

### OBS-001：统一 TelemetryContext

- **目标**：贯通 project/session/run/turn/operation/actor/client/correlation/trace。
- **依赖**：STORAGE-003、RUNTIME-001。
- **验收**：核心事件、日志、指标和 Trace 在存在对应对象时可关联；不记录 Secret。

### OBS-002：Event Registry

- **目标**：注册事件的 schema、durability、redaction、projection、audit 和 realtime 策略。
- **依赖**：STORAGE-003、领域事件规范。
- **验收**：未注册事件不能静默进入持久 Event Store；未知事件计数和阻断行为可测试。

### OBS-003：Projection Registry

- **目标**：实现 overview/session/skill/mcp/subagent/security/ops 投影登记和 cursor。
- **依赖**：STORAGE-004、OBS-002。
- **验收**：投影可删除重建；返回 revision/as_of_seq/is_complete。

### OBS-004：Safe View/Redaction Pipeline

- **目标**：所有事件、日志、诊断包和面板字段进入持久化/广播前扫描和脱敏。
- **依赖**：Credential 设计、Tool Gateway、STORAGE-003。
- **验收**：Secret、完整 header、敏感文件正文和 Prompt 原文不会进入普通 telemetry；扫描失败 fail-closed。

### OBS-005：单行位置式日志

- **目标**：实现 message_code、level、component、correlation、error_code 和安全字段。
- **依赖**：OBS-001、OBS-004。
- **验收**：日志限长、滚动、保留期和磁盘水位有效；日志不是业务事实源。

### OBS-006：Metrics/Health

- **目标**：实现 writer queue、commit latency、projection lag、outbox lag、DB/WAL/disk、Provider/Tool 成功率。
- **依赖**：OBS-001、STORAGE-005。
- **验收**：高基数 label 被拒绝；面板能显示健康和滞后。

### OBS-007：Audit Record

- **目标**：记录权限、Approval、Credential、Data Egress、Gate、Snapshot、Maintenance、Support Bundle。
- **依赖**：OBS-002、Permission、Credential、Rules、Snapshot。
- **验收**：控制点 100% 有审计；Audit 查询按 capability 裁剪。

### OBS-008：Alert/Incident 基础

- **目标**：实现 projection lag、outbox lag、DB 错误、磁盘水位和未知副作用告警。
- **依赖**：OBS-006、OBS-007。
- **验收**：告警去重、Incident 状态机和处置审计可用；规则不能直接执行任意 Tool。

### OBS-009：MaintenanceRun

- **目标**：实现 quick_check、backup、projection_rebuild、fts_rebuild、GC 的受控任务模型。
- **依赖**：Deployment 设计、STORAGE-002、OBS-007。
- **验收**：预检、确认、锁、lease、progress、cancel、failed/completed 状态完整。

### OBS-010：TUI 运维面板

- **目标**：在 TUI 展示 Overview、Health、Audit、Maintenance、Incident 和 Panel lag。
- **依赖**：OBS-003、OBS-006～OBS-009。
- **验收**：只读与高风险操作区分；高风险操作显示影响范围、确认 token 和审计编号。

### OBS-011：Support Bundle

- **目标**：生成范围可见、脱敏、短期过期的本地诊断包。
- **依赖**：OBS-004、OBS-005、OBS-006、OBS-007。
- **验收**：包中无 Secret；生成/下载/过期均审计；扫描失败不降级导出原文。

### OBS-012：Observability Vertical Slice

- **目标**：故意制造一个 Projector lag 或 DB warning，TUI 能显示、创建 Incident、查询安全证据并执行修复任务。
- **依赖**：OBS-001～OBS-011。
- **验收**：故障→告警→Incident→Maintenance→恢复→关闭的全链路可演练。

---

## 18. 第十二阶段：v0.1 MVP 集成与发布

### MVP-001：项目选择与会话管理

- **目标**：TUI 支持选择/创建 Project、Session 列表、恢复最近会话。
- **依赖**：STORAGE-004、RUNTIME-002。
- **验收**：项目边界、路径校验、会话游标和权限范围正确。

### MVP-002：双 Provider 配置

- **目标**：支持两个 Provider Adapter 的配置、切换、usage 和错误展示。
- **依赖**：PROVIDER-005、Credential Store。
- **验收**：切换不破坏历史；Provider 事件包含 adapter/model/config revision。

### MVP-003：基础工具集完成

- **目标**：Read、Write、Edit、Bash、Glob、Grep、Task 简单版全部通过 ToolGateway。
- **依赖**：TOOL-009、RUNTIME-004。
- **验收**：每个工具都有 schema、权限、超时、取消、结果规范化、审计和集成测试。

### MVP-004：Spec 流水线完成

- **目标**：requirements/design/tasks/implementation/verification 和 `/skip-spec` 可用。
- **依赖**：SPEC-009、RULE-007。
- **验收**：确认门、下游失效、verification 结论和证据完整。

### MVP-005：Checkpoint/Snapshot/Recovery

- **目标**：最小可恢复编码任务闭环。
- **依赖**：SNAPSHOT-003、STORAGE-007、OBS-012。
- **验收**：Core 崩溃、Provider 超时、Tool 取消和文件冲突可恢复或明确阻断。

### MVP-006：TUI 可用性打磨

- **目标**：完善帮助、快捷键、错误重试、分页、搜索、长文本、主题和终端兼容。
- **依赖**：MVP-001～MVP-005。
- **验收**：真实终端手工测试清单通过；80x24 下核心操作可完成。

### MVP-007：v0.1 Release Candidate

- **目标**：冻结协议/事件/migration，生成 v0.1 RC。
- **依赖**：MVP-001～MVP-006、发布门禁。
- **验收**：全量质量门、安装/升级/恢复演练、Secret 扫描、性能基线通过。

### MVP-008：v0.1 正式发布

- **目标**：发布 TUI/CLI/Core artifact 和文档。
- **依赖**：MVP-007。
- **发布物**：二进制、Release Manifest、SBOM、迁移说明、备份/恢复说明、故障 Runbook、已知限制。

---

## 19. 第十三阶段：v0.2 稳定化与工程化

### STAB-001：跨平台 TUI 回归

- Windows Terminal、PowerShell、cmd、macOS Terminal/iTerm2、Linux 常见终端；
- Unicode/CJK/emoji/宽度、颜色、鼠标、resize、clipboard；
- 无 tty、重定向 stdout、TERM 异常、SSH 终端。

### STAB-002：性能基线

- TUI cold start < 500ms（需求目标，基准机定义在测试文档）；
- SQLite 会话列表查询 p95 < 10ms；
- 面板刷新接近 1s；
- Core 默认内存目标 < 200MB（不含外部子进程/缓存特例）；
- 流式输出有界，不因长响应无限增长内存。

### STAB-003：错误体验

- 每个稳定错误码对应用户提示、恢复建议和诊断引用；
- retryable 与 non-retryable 清晰区分；
- 网络错误、权限拒绝、Rule 阻断、数据库只读和未知副作用不混淆；
- TUI 支持从错误卡片跳转到安全详情/Incident。

### STAB-004：数据和日志保留

- 配置事件/日志/Trace/Blob/Snapshot/Backup 保留策略；
- GC、WAL checkpoint、日志滚动和磁盘水位压测；
- 隐私清除和诊断包扫描。

### STAB-005：开发者体验

- `cargo xtask check` 或等价统一检查入口；
- fake Provider、fake Tool、fake Clock、fake Transport、SQLite temp DB fixture；
- 一键启动 TUI demo、daemon demo、恢复演练；
- ADR、变更日志和 protocol fixture 自动校验。

---

## 20. 第十四阶段：v0.3 三端共享

### V03-001：Protocol 代码生成

- **目标**：从 `proto/` 生成 Rust client/server DTO 和版本校验。
- **依赖**：TUI-003、CORE-004。
- **验收**：Native gRPC/IPC 和 REST/WS 使用同一业务语义；Gateway 无第二状态机。

### V03-002：Tauri Rust Shell

- **目标**：复用 `ApplicationClient`，实现窗口、连接、通知和安全存储引用。
- **依赖**：CORE-005、MVP-008。
- **验收**：WebView 不直接读本地敏感数据；Core 重启后 shell 能重连。

### V03-003：Vue Desktop UI

- **目标**：实现 Spec 编辑、DAG 预览、面板图表、审批和会话查看。
- **依赖**：V03-002、Projection Query。
- **验收**：与 TUI 共享事件/Query 语义；富文本只作为展示/编辑层。

### V03-004：Actix Web Gateway

- **目标**：REST、WebSocket、认证、CSRF/CORS、限流、静态资源。
- **依赖**：CORE-004、OBS-007、Deployment 远程预检。
- **验收**：默认 loopback；远程监听无 TLS 时启动阻断；无直接 DB/Tool 访问。

### V03-005：Vue Web UI

- **目标**：会话查看、Spec 评审、审批、只读面板和轻量操作。
- **依赖**：V03-004。
- **验收**：连接恢复、projection refresh、权限裁剪和审计下载能力正确。

### V03-006：DeepSeek/Kimi Provider

- **目标**：接入需求文档中的第二批 Provider。
- **依赖**：PROVIDER-005、V03-004。
- **验收**：Provider 选择、CredentialRef、usage、错误和模型能力统一。

### V03-007：三端会话共享

- **目标**：TUI、Desktop、Web 同时打开同一 Project/Session。
- **依赖**：V03-001～V03-006。
- **验收**：一个端提交 Command，其他端按事件游标更新；冲突由 Core 处理。

---

## 21. 第十五阶段：v0.5 编排增强

### V05-001：DAG 声明模型与编译器

- **目标**：任务依赖、节点类型、并发限制、失败策略、重试策略和版本化编译结果。
- **依赖**：SPEC-005、RUNTIME-001。
- **验收**：DAG 是声明式数据；环、未满足依赖、写路径不确定和权限扩大在编译期拒绝。

### V05-002：Workflow Runtime

- **目标**：节点生命周期、ready queue、依赖收敛、暂停/恢复和取消。
- **依赖**：V05-001、RUNTIME-004。
- **验收**：同一 node/attempt 幂等；终态不可回退；恢复不盲重放外部副作用。

### V05-003：Write Claim

- **目标**：路径交集判定、租约、fence、释放和冲突恢复。
- **依赖**：TOOL-005、V05-002、Workspace 设计。
- **验收**：文件与父目录冲突、符号链接、glob 不确定性有测试；子 Agent 不能扩大 claim。

### V05-004：SubAgent Supervisor

- **目标**：父子 Agent、profile、权限继承、上下文隔离、结果回传和子进程回收。
- **依赖**：V05-002、V05-003、RUNTIME-004。
- **验收**：子 Agent 无法扩大父权限；崩溃、取消、超时、迟到结果可恢复。

### V05-005：MCP Registry/Supervisor

- **目标**：配置发现、连接生命周期、工具 registry、超时、重连和停用。
- **依赖**：TOOL-002、Credential、V05-004。
- **验收**：MCP 返回 untrusted；调用、Credential、Data Egress、延迟和错误进入面板/审计。

### V05-006：Skills Loader

- **目标**：兼容项目/user/builtin/extension 分层 Skills，支持 metadata 渐进披露和 digest。
- **依赖**：SPEC-001、Context、Extension registry。
- **验收**：Skill 不能扩大权限；加载/跳过/失败/调用有事件和面板摘要。

### V05-007：Memory 基础版

- **目标**：Markdown Memory、FTS5 索引、recall 记录和敏感内容策略。
- **依赖**：STORAGE-001、CONTEXT-002、OBS-003。
- **验收**：Memory 可重建；recall 有 reason/score/来源；不把外部不可信文本变为系统约束。

### V05-008：编排面板

- **目标**：DAG、SubAgent、MCP、Skill、Memory 面板。
- **依赖**：V05-002～V05-007、OBS-010。
- **验收**：TUI/未来 Desktop/Web 使用同一 projection；用户能定位阻断和资源占用。

---

## 22. 第十六阶段：v0.7 可靠性、Replay 与 Hook

### V07-001：Operation Journal 完整化

- **目标**：记录外部操作 intent、开始、结果未知、查询和收敛。
- **依赖**：TOOL-001、V05-005、OBS-007。
- **验收**：进程崩溃后恢复器能枚举 active operation 并标记 unknown。

### V07-002：Deterministic Replay

- **目标**：仅重放纯逻辑事件、Provider fixture、Tool normalized result 和 projection reducer。
- **依赖**：STORAGE-003、V07-001、OBS-003。
- **验收**：重放不执行真实网络、Shell、MCP、文件写入或 Credential；状态结果可比较。

### V07-003：Recovery Reconciler

- **目标**：启动恢复、事件对账、Write Claim 释放、子进程回收和终态收敛。
- **依赖**：V07-001、V05-003、Deployment Restore。
- **验收**：恢复顺序和 unknown external effect 语义符合设计；每次修正都有 causation/recovery ID。

### V07-004：Hook Contract

- **目标**：PreToolUse、PostToolUse、PermissionRequest、SpecStageChanged、AgentStop、SessionStop。
- **依赖**：TOOL-002、RULE-004、Extension registry。
- **验收**：Hook 只能返回版本化 `continue` / `deny` / `request_approval` / `block_completion` / `diagnostic_only` / `propose_rewrite` / `async_check_scheduled`（7 值，权威定义见扩展系统详细设计 §19.4 `HookDecision`）；不能绕过 Gateway。

### V07-005：增强权限和 Break-glass

- **目标**：Capability、范围、策略版本、二次确认、过期和审计。
- **依赖**：V07-004、Credential、Observability。
- **验收**：break-glass 自动创建 Incident；权限撤销对新 operation 生效。

### V07-006：Replay/Recovery 面板

- **目标**：展示恢复水位、事件缺口、未知副作用、重放范围和维护报告。
- **依赖**：V07-002、V07-003、OBS-010。
- **验收**：用户能理解“已恢复/待确认/不可重放”的区别。

---

## 23. 第十七阶段：v1.0 扩展、发布与灾备

### V10-001：Plugin API

- **目标**：稳定 Plugin manifest、能力声明、版本、签名、隔离和生命周期。
- **依赖**：V05-006、V07-004、Protocol 版本稳定。
- **验收**：插件不能链接 Core 私有类型；崩溃不影响核心；能力不能静默扩大。

### V10-002：Extension Marketplace/Offline Import

- **目标**：支持签名扩展导入、禁用、升级、回滚和 registry generation。
- **依赖**：V10-001、Deployment 供应链。
- **验收**：manifest/digest/签名/兼容性/撤销全部验证；失败 fail-closed。

### V10-003：Backup/Restore Productization

- **目标**：将备份、恢复、Portable Import、验证和 Credential rebind 做成正式控制面。
- **依赖**：OBS-009、Deployment Backup/Restore。
- **验收**：普通用户可安全备份；管理员可执行高风险恢复；全过程可审计。

### V10-004：Upgrade/Rollback Productization

- **目标**：安装器/更新器、灰度、迁移、回滚、旧客户端兼容和发布观察。
- **依赖**：MVP-007、V03-007、V07-003。
- **验收**：升级失败不丢唯一恢复点；回滚后的 Core、DB、Blob、Projection 一致。

### V10-005：Release Candidate

- **目标**：完成 v1.0 RC 的全量测试和文档冻结。
- **依赖**：V10-001～V10-004。
- **验收**：三端、扩展、权限、审计、备份、恢复、升级、灾备演练全部通过。

### V10-006：v1.0 正式发布

- **目标**：发布 Stable channel。
- **依赖**：V10-005、发布门禁。
- **输出**：安装包、容器/服务器包（如承诺支持）、迁移矩阵、恢复手册、SBOM、签名清单、已知限制。

---

## 24. TUI 端详细开发顺序

本节是最重要的执行顺序。任何第一批开发者都可以只阅读本节开始工作。

### 24.1 TUI 最小垂直切片顺序

```text
1. cargo workspace 能编译
2. apex-tui 能启动和退出
3. 终端进入/退出安全
4. 画出 Header/Conversation/Input/Status
5. 输入 hello 并在本地显示
6. 输入 Enter 生成 typed Command
7. Fake Application 返回 typed Event
8. Reducer 应用 Event
9. Renderer 显示 assistant 回复
10. /quit 优雅退出
11. 测试无终端模式的 reducer
12. 测试完整 demo 脚本
```

### 24.2 第一批文件建议

```text
crates/apex-domain/src/
  ids.rs
  errors.rs
  session.rs
  turn.rs

crates/apex-protocol/src/
  envelope.rs
  commands.rs
  queries.rs
  events.rs
  version.rs

crates/apex-tui/src/
  main.rs
  app.rs
  state.rs
  action.rs
  effect.rs
  input.rs
  events.rs
  keymap.rs
  terminal.rs
  render/
    mod.rs
    layout.rs
    header.rs
    conversation.rs
    input.rs
    status.rs
  transport/
    mod.rs
    in_process.rs
    mock.rs
  test_support/
    fixtures.rs
    scripted_provider.rs
```

### 24.3 TUI 状态最小模型

```rust
pub struct AppState {
    pub mode: AppMode,
    pub connection: ConnectionState,
    pub session: Option<SessionView>,
    pub messages: Vec<MessageView>,
    pub input: InputState,
    pub focused_pane: FocusedPane,
    pub pending_approval: Option<ApprovalView>,
    pub notification: Option<NotificationView>,
    pub last_seen_global_seq: u64,
    pub should_quit: bool,
}
```

### 24.4 TUI 事件循环优先级

```text
Priority 0: quit / panic cleanup / fatal transport error
Priority 1: cancel / security block / approval response
Priority 2: durable domain event / query completion
Priority 3: user input
Priority 4: realtime delta / progress
Priority 5: tick / redraw / health refresh
```

优先级不是业务事实顺序。Domain Event 仍以 Core 的 `global_seq` 为准；它只控制客户端在高负载下的响应性。

### 24.5 TUI 不变量

```text
T1: Renderer 不修改 AppState
T2: Input 不直接调用 Provider/Tool/DB
T3: 一个 command_id 最多产生一个业务结果
T4: 一个 durable event 不能被同一 cursor 应用两次
T5: transient delta 丢失后最终 Query 能修正视图
T6: terminal cleanup 在所有退出路径执行
T7: 未授权 payload 不展示原文
T8: TUI 不把本地显示状态写回 Core
```

---

## 25. 最小粒度任务清单（可直接建 issue）

### 25.1 Bootstrap 类

| ID | 任务 | 依赖 | 预计 | 验收 |
|---|---|---|---:|---|
| BOOT-001 | 创建根 Cargo.toml | 无 | 0.25d | workspace 可解析 |
| BOOT-002 | 固定 Rust toolchain | BOOT-001 | 0.25d | `rustc --version` 符合 |
| BOOT-003 | 创建 apex-domain crate | BOOT-001 | 0.25d | crate 可编译 |
| BOOT-004 | 创建 apex-protocol crate | BOOT-001 | 0.25d | serde DTO 可编译 |
| BOOT-005 | 创建 apex-tui binary | BOOT-001 | 0.25d | `--help` 可用 |
| BOOT-006 | 加入 fmt/clippy/test CI | BOOT-001 | 0.5d | PR 门禁执行 |
| BOOT-007 | 加入 README 与本地启动说明 | BOOT-005 | 0.25d | 新人可按说明运行 |
| BOOT-008 | 添加 ADR 模板 | BOOT-001 | 0.25d | ADR 可校验 |

### 25.2 TUI 核心类

| ID | 任务 | 依赖 | 预计 | 验收 |
|---|---|---|---:|---|
| TUI-101 | 定义 AppMode | TUI-005 | 0.25d | 模式转换有单测 |
| TUI-102 | 定义 ConnectionState | TUI-003 | 0.25d | ready/degraded/reconnecting 可渲染 |
| TUI-103 | 定义 MessageView | TUI-012 | 0.5d | 多类型消息可序列化 |
| TUI-104 | 定义 InputState | TUI-008 | 0.25d | 光标/选择/清空可测 |
| TUI-105 | 定义 FocusedPane | TUI-020 | 0.25d | Tab/快捷键转换 |
| TUI-106 | 实现 Reducer | TUI-005 | 0.75d | action 序列得到确定状态 |
| TUI-107 | 实现 Effect 列表 | TUI-004 | 0.5d | command/query/subscribe 可调度 |
| TUI-108 | 实现 tick 合并 | TUI-009 | 0.5d | redraw 有界 |
| TUI-109 | 实现终端 guard | TUI-006 | 0.5d | 异常后终端恢复 |
| TUI-110 | 实现主布局 | TUI-007 | 0.5d | 80x24 可用 |
| TUI-111 | 实现消息列表 | TUI-012 | 0.75d | 滚动/截断/状态图标 |
| TUI-112 | 实现输入框 | TUI-008 | 0.75d | Enter/Esc/Ctrl-C |
| TUI-113 | 实现状态栏 | TUI-007 | 0.5d | 连接/seq/usage 状态 |
| TUI-114 | 实现帮助弹窗 | TUI-015 | 0.5d | 快捷键可发现 |
| TUI-115 | 实现面板路由 | TUI-020 | 0.75d | 面板切换不丢输入 |

### 25.3 Transport/Mock 类

| ID | 任务 | 依赖 | 预计 | 验收 |
|---|---|---|---:|---|
| TR-101 | MockClient | TUI-004 | 0.5d | 可注入错误/延迟 |
| TR-102 | InProcessClient | TUI-004 | 0.75d | 连接 Application Port |
| TR-103 | Scripted Event Source | TR-101 | 0.5d | fixture 顺序可复现 |
| TR-104 | Fake Clock | TR-101 | 0.25d | 时间可控 |
| TR-105 | Fake Provider | PROVIDER-002 | 0.75d | stream/error/cancel |
| TR-106 | Transport contract test | TR-101/TR-102 | 0.75d | 两种 transport 共用测试 |
| TR-107 | Native endpoint DTO | CORE-003 | 0.5d | discovery schema 固定 |
| TR-108 | Hello client/server test | CORE-004 | 0.75d | version/token matrix |

### 25.4 Storage 类

| ID | 任务 | 依赖 | 预计 | 验收 |
|---|---|---|---:|---|
| DB-101 | SQLite bootstrap | STORAGE-001 | 0.75d | WAL/application_id |
| DB-102 | schema_migrations | STORAGE-002 | 0.75d | checksum/状态 |
| DB-103 | domain_events 表 | STORAGE-003 | 0.75d | seq/版本/摘要 |
| DB-104 | StorageWriter actor | DB-103 | 1.0d | 单写者/事务 |
| DB-105 | session/message projection | DB-104 | 0.75d | 可查询 |
| DB-106 | event cursor | DB-104 | 0.5d | replay 边界 |
| DB-107 | outbox 表 | DB-104 | 0.5d | commit 后投递 |
| DB-108 | projection rebuild | DB-105 | 1.0d | 删除可重建 |
| DB-109 | SQLite crash tests | DB-104 | 1.0d | 中断后可恢复 |
| DB-110 | backup manifest | DB-101 | 0.75d | digest/seq/Blob 引用 |

---

## 26. 任务优先级、并行边界与写入范围

### 26.1 P0 关键路径

```text
S00 日志基础设施 → BOOT → TUI core → Fake Application → Native transport → apexd
→ SQLite Event Store → Provider → Tool Gateway → Spec → Rules
→ Checkpoint/Snapshot → Observability → v0.1 release
```

P0 任务失败会阻断用户闭环，不应被低优先级 UI 美化或扩展功能抢占。

### 26.2 可并行任务

以下任务可以在不修改同一主要文件的情况下并行：

| 轨道 A | 轨道 B | 轨道 C |
|---|---|---|
| TUI Renderer | Protocol DTO | Fake Provider |
| AppState/Reducer | Event Registry | Storage migration |
| TUI snapshot tests | Domain state machine | CI/release scripts |
| Panel views | Audit safe view | Backup manifest |
| Tauri shell | Actix Gateway | Provider adapters |

并行开发必须明确 write scope；两个任务不能同时修改同一状态机、协议文件或 migration 文件而没有 owner。

### 26.3 不应并行的任务

以下工作必须串行或由同一 owner 负责：

- Event Envelope 与所有事件注册表的基础字段；
- Command 幂等和 aggregate version；
- SQLite schema migration 的同一 revision；
- Permission decision 与 Tool execute 的边界；
- Spec stage transition 与 Verification Gate；
- Backup/Restore 与数据库格式变化；
- Protocol major version 和客户端兼容矩阵。

### 26.4 推荐分支/提交策略

```text
feat/bootstrap-*       → BOOT
feat/tui-core-*        → TUI-101～TUI-115
feat/protocol-*        → TUI-003/CORE-004
feat/storage-*         → DB-101～DB-110
feat/provider-*        → PROVIDER
feat/tools-*           → TOOL
feat/spec-*            → SPEC
feat/observability-*   → OBS
release/v0.0.x         → 内部版本
release/v0.1.x         → RC/Stable
```

一个提交尽量对应一个可审查任务；跨层任务应拆成“类型/接口 → 实现 → 集成测试”三个提交。

---

## 27. 每个阶段的 Definition of Done

### 27.1 TUI 阶段

- [ ] 终端所有退出路径清理；
- [ ] Renderer、Reducer、Effect、Transport 分离；
- [ ] 无 UI 直接写数据库/调用工具/provider；
- [ ] reducer、输入、渲染至少有单测/golden；
- [ ] 断线、错误、取消、长文本和窄终端可处理；
- [ ] 至少一个端到端演示脚本可重复执行。

### 27.2 Core/Storage 阶段

- [ ] 单 writer、WAL、migration、event seq、cursor；
- [ ] Command 幂等、expected revision、终态不可回退；
- [ ] Projection 可删除重建；
- [ ] commit 前不广播，广播失败不回滚事实；
- [ ] Crash/restart/reconnect 有测试；
- [ ] 备份 manifest 可验证。

### 27.3 Provider/Tool 阶段

- [ ] Provider/Tool 都通过 Port/Adapter；
- [ ] operation_id、timeout、cancel、retry 和 error code；
- [ ] Permission/Approval/Redaction/Taint；
- [ ] unknown external effect 不盲重放；
- [ ] Fake adapter 与真实 adapter contract test 共用。

### 27.4 Spec/Rules 阶段

- [ ] 需求、设计、任务、实现、验证阶段均有状态和事件；
- [ ] 确认门和下游失效规则有效；
- [ ] Rule 编译、增量检查、Gate、Waiver、Skip Spec 可审计；
- [ ] verification 结论不是模型自报；
- [ ] 文件变更有 digest、snapshot、恢复路径。

### 27.5 发布阶段

- [ ] manifest、签名、SBOM、迁移矩阵；
- [ ] 安装、升级、回滚和恢复演练；
- [ ] Secret telemetry scan 为零泄漏；
- [ ] 磁盘水位和只读恢复；
- [ ] Runbook、错误码、已知限制和用户文档；
- [ ] 所有 P0 任务为 verified/done。

---

## 28. 测试金字塔与自动化计划

### 28.1 单元测试

优先覆盖纯逻辑：

- ID/错误/序列化；
- Reducer、Keymap、InputState；
- Event Envelope canonicalization；
- 状态机和终态；
- 路径 canonicalization/相交；
- Bash AST/Permission；
- Redaction/Taint；
- Rule compiler/Verification Gate；
- Projection reducer；
- Cursor 和 replay；
- Migration plan validation。

### 28.2 Contract Tests

为每个 Port 保持 contract suite：

```text
ApplicationClient: InProcess / Native / Mock
Provider: Fake / Provider-A / Provider-B
Tool: Read / Write / Bash / MCP
Storage: InMemory fixture / SQLite
EventBus: direct / outbox replay
```

同一套业务语义测试不能因为换 transport 或 adapter 就重新定义验收标准。

### 28.3 集成测试

- `apexd` + temporary Apex Home + TUI client；
- SQLite migration → command → event → projection → query；
- Provider stream → cancel → final state；
- Tool permission → approval → execution → audit；
- Spec stage → Rule → Gate → verification；
- Snapshot → file change → restore；
- restart → recovery → cursor replay。

### 28.4 Golden/Snapshot 测试

TUI 使用稳定宽度和固定 terminal backend 做 golden：

```text
empty_state.snap
chat_user_message.snap
assistant_streaming.snap
approval_pending.snap
tool_denied.snap
panel_lagging.snap
incident_open.snap
maintenance_running.snap
reconnecting.snap
```

Golden 只验证布局和安全展示，不验证时间戳、随机 ID 或平台颜色差异；这些字段必须通过 normalization 处理。

### 28.5 端到端脚本

```text
scripts/e2e/
  tui_demo.ps1
  tui_demo.sh
  restart_replay.ps1
  spec_pipeline.ps1
  tool_approval.ps1
  backup_restore.ps1
  provider_failure.ps1
```

脚本使用 Fake Provider、temporary directory、固定 Clock 和可控 event seed，不依赖生产账户。

### 28.6 性能测试

- TUI 渲染 1k/10k 消息；
- 事件 replay 1k/10k/100k rows；
- Projection rebuild；
- 1s panel refresh；
- 64KB/1MB/16MB 输出截断；
- 多客户端订阅和慢消费者；
- SQLite WAL/checkpoint/backup；
- Provider stream cancellation latency。

---

## 29. CI/CD 与发布流程

### 29.1 Pull Request 门禁

```text
changed-files scope check
→ cargo fmt --check
→ cargo check --workspace
→ cargo test --workspace
→ cargo clippy --all-targets --all-features -- -D warnings
→ protocol/event fixture compatibility
→ migration checksum check
→ documentation link/heading check
→ secret scan
```

涉及 `proto/`、`events/`、`migrations/`、`permissions/`、`credentials/`、`restore/` 的 PR 必须要求领域/安全/存储 owner review。

### 29.2 Nightly

- 跨平台编译；
- 长时 TUI/daemon 运行；
- 事件 replay/property tests；
- migration upgrade matrix；
- backup/restore；
- chaos/crash tests；
- dependency audit；
- coverage trend。

### 29.3 Release Candidate

RC pipeline：

```text
version bump proposal
→ changelog + migration/event registry generation
→ build signed artifacts
→ install clean machine
→ upgrade representative database
→ restore backup
→ run E2E/chaos subset
→ generate SBOM/release manifest
→ manual approval
→ publish preview channel
```

### 29.4 Stable 发布

Stable 只允许从通过 RC 门禁的 commit 发布。发布后执行观察窗口，发现严重数据库、权限、数据泄漏或恢复故障时停止灰度并进入回滚/Incident 流程。

---

## 30. 计划风险与缓解

| 风险 | 影响 | 缓解 |
|---|---|---|
| TUI 先写成业务单体 | 后续三端/daemon 重构 | 第一天引入 Client Port 和 Protocol DTO |
| 过早实现 DAG | MVP 延迟、状态复杂 | v0.1 只做线性任务，v0.5 再做 DAG |
| 真实 Provider 先于安全边界 | Secret/工具失控 | Fake Provider 先过 contract，Provider 经 Broker/Gateway |
| SQLite 直接被 UI 使用 | 并发、恢复和审计失效 | TUI 只读 Query View，Core 独占 Writer |
| 规则误报阻断开发 | 用户绕过系统 | warning/error 分层、修复子任务、留痕 escape hatch |
| 三端同步过早 | 版本和冲突爆炸 | v0.1 单 TUI，v0.3 统一协议后再上三端 |
| Provider SDK 变动 | adapter 破坏核心 | Provider trait + contract tests + capability snapshot |
| TUI 渲染性能不足 | 用户体验差 | 有界事件队列、delta 合并、Projection 查询、snapshot benchmark |
| 事件 schema 过早不稳定 | migration/客户端成本 | Event Registry、版本化事件、upcaster 和 ADR |
| 备份不可恢复 | 数据丢失 | manifest + 定期真实 restore 演练 |
| 任务过大无法 review | 质量下降 | 0.25～2d 粒度、单一写入范围、vertical slice |
| 安全文档与实现脱节 | 生产泄漏 | Secret scan、security owner、fail-closed 测试 |

---

## 31. 开发节奏与里程碑建议

这里使用“工程日”而不是日历日期，避免在团队规模、并行度和依赖未明确前制造虚假精确度。

| 里程碑 | 目标 | 主要任务范围 | 建议规模 |
|---|---|---|---:|
| M0 | 日志与工程骨架 | S00-001～009 + BOOT-001～008 | 3～5 人日 |
| M1 | TUI 最小闭环 | TUI-001～011 | 5～10 人日 |
| M2 | Mock Chat | TUI-012～018 | 4～8 人日 |
| M3 | 面板/审批 Mock | TUI-019～025 | 5～10 人日 |
| M4 | apexd 连接 | CORE-001～008 | 6～12 人日 |
| M5 | SQLite 恢复闭环 | STORAGE-001～008 | 10～20 人日 |
| M6 | 真实 Provider/Agent | PROVIDER/RUNTIME | 10～20 人日 |
| M7 | Tool/Permission | TOOL-001～009 | 15～30 人日 |
| M8 | Spec/Rules | SPEC/RULE | 20～40 人日 |
| M9 | Checkpoint/Snapshot | CONTEXT/SNAPSHOT | 10～20 人日 |
| M10 | Observability/Ops | OBS-001～012 | 12～25 人日 |
| M11 | v0.1 发布 | MVP/STAB | 15～30 人日 |
| M12 | 三端/编排/可靠性 | v0.3～v0.7 | 按模块拆分 |
| M13 | v1.0 完整产品 | V10 + 发布/灾备 | 按 RC 结果拆分 |

### 31.1 第一周建议节奏

如果只有一名开发者，第一周只做：

```text
Day 1: S00-001～S00-008（日志库、demo、字段和测试）
Day 2: S00-009 + BOOT-001～008（日志门禁与 TUI workspace 骨架）
Day 3: TUI-001～003（TerminalGuard、空布局、日志初始化）
Day 4: TUI-004～007（输入、Reducer、键盘事件）
Day 5: TUI-008～011（主循环、Mock 闭环、snapshot/CI/README）
```

第一周结束必须得到带 `run_id` 日志的 `apex-tui --demo`，而不是半成品 Provider、数据库或权限引擎。

### 31.2 第一轮 Review 问题

- TUI 是否仍然可以在没有 Core/Provider/DB 时运行测试？
- Renderer 是否完全没有业务副作用？
- 命令、查询和事件是否已区分？
- 退出、取消、错误和重连是否有明确状态？
- 未来替换 InProcessTransport 为 NativeTransport 是否无需改 UI？
- 中文宽度、长文本、窄终端是否通过测试？
- 是否有任何 Secret、原始参数或任意 payload 进入日志/golden？

---

## 32. 第一批实际执行清单

按 `docs/Apex—— v0.1 MVP逐功能可运行阶段计划.md` 的阶段顺序执行；先完成日志基础设施，再进入 TUI，不要跨阶段提前开发真实工具：

```text
[x] 创建项目根 Cargo.toml
[x] 创建 rust-toolchain.toml
[x] 创建 apex-observability
[x] 实现每次运行独立日志文件
[x] 记录 UTC 时间、sequence、PID、OS 线程、真实源码文件和行号
[x] 记录显式注册的 Tokio 协程/任务 ID 与名称
[x] 实现敏感字段名脱敏、队列背压、flush 和 shutdown
[x] 创建 apex-log-demo 并通过 S00 单元/运行验收
[ ] 补齐 apex-domain
[ ] 补齐 apex-protocol
[ ] 创建 apex-tui
[ ] 加入 fmt/check/test/clippy CI
[ ] 定义强类型 ID
[ ] 定义统一错误
[ ] 定义最小 Command/Query/Event
[ ] 定义 ApplicationClient Port
[ ] 创建 AppState/Action/Effect
[ ] 创建 TerminalGuard
[ ] 渲染三栏布局
[ ] 实现输入框
[ ] 实现主事件循环
[ ] 实现 Mock/InProcess Application
[ ] 实现 fake session.open
[ ] 实现 fake turn.submit
[ ] 实现 assistant response event
[ ] 实现 /quit
[ ] 写 reducer tests
[ ] 写 terminal snapshot tests
[ ] 写 demo e2e script
[ ] 发布内部 v0.0.1
```

### 32.1 第一批禁止事项

```text
[ ] 不直接调用 OpenAI/DeepSeek/Kimi 网络 API
[ ] 不直接打开 SQLite
[ ] 不直接执行 Shell
[ ] 不直接写项目文件
[ ] 不把业务状态塞进 ratatui Widget
[ ] 不使用 String 代替强类型 ID
[ ] 不把事件 JSON 直接显示给用户
[ ] 不在 UI 中实现 Permission 判定
[ ] 不为未来 DAG 提前引入复杂调度器
[ ] 不为了演示跳过错误/取消/退出清理
```

---

## 33. 交付物清单

### M0/M1

- Cargo workspace；
- `apex-tui`；
- Domain/Protocol 最小类型；
- TUI 状态机和渲染；
- Mock Application；
- 单元/golden/e2e 测试；
- README 与运行截图/录屏（可选）；
- v0.0.1 changelog。

### v0.1

- `apexd`；
- Native protocol；
- SQLite migrations/Event Store/Projection/Outbox；
- Provider/Tool/Permission；
- Spec/Rules/Verification；
- Checkpoint/Snapshot；
- 基础 Observability、Audit、Backup；
- TUI MVP release artifact。

### v0.3/v0.5/v0.7/v1.0

按前述阶段生成：

- Tauri/Web 客户端；
- DAG/Write Claim/SubAgent；
- MCP/Skill/Memory；
- Replay/Recovery/Hook；
- Plugin API；
- Upgrade/Rollback/Disaster Recovery；
- 稳定协议、安装包、SBOM、签名和完整文档。

---

## 34. 计划与架构文档的追踪矩阵

| 架构文档 | 计划任务 |
|---|---|
| 总体架构 | BOOT、CORE、V03、V10 |
| 领域模型与事件规范 | TUI-002/003、STORAGE-003、RUNTIME、OBS |
| API 与实时事件协议 | TUI-003、CORE-004/005、STORAGE-007、V03 |
| SQLite 数据模型与迁移 | STORAGE、DB、MVP、V10 |
| Agent Runtime/DAG | RUNTIME、V05、V07 |
| Tool Gateway/权限 | TOOL、PROVIDER、V07 |
| Context/Checkpoint | CONTEXT、SNAPSHOT |
| Workspace/Snapshot/Claim | TOOL-005、SNAPSHOT、V05-003 |
| Rules/Verification Gate | RULE、SPEC、MVP |
| MCP/Skill/Hook/Plugin | V05、V07、V10 |
| Credential 治理 | PROVIDER-003、TOOL、OBS、V10-003 |
| Observability/审计/运维 | OBS、STAB、V10 |
| Deployment/升级/灾备 | CORE、STORAGE、STAB、V10 |

---

## 35. 最终验收标准

### 35.1 最小 TUI 验收

- `apex-tui --demo` 可以启动、渲染、接受输入、返回 Fake Event、显示回复并退出；
- 80x24 和 CJK 文本不会 panic 或破坏核心布局；
- 终端 raw mode/alternate screen 在正常、错误、Ctrl-C、panic 后均恢复；
- TUI 无 Provider、SQLite、文件系统和工具执行副作用；
- AppState 可通过纯 reducer 测试重放。

### 35.2 v0.0.x 验收

- TUI 通过同一 Client Port 使用 Mock/InProcess 两种后端；
- 流式输出、审批、取消、错误、重连和面板状态可演示；
- 协议 DTO 有版本、schema、correlation 和安全错误映射；
- 无敏感内容进入日志、golden、fixture 和事件摘要。

### 35.3 v0.1 验收

- TUI 连接真实 `apexd`，SQLite Event Store 支持恢复；
- 真实 Provider、基础 Tool、Permission、Spec、Rules、Checkpoint、Snapshot 可完成一个编码任务；
- Tool/Approval/Rule/文件变更都有事件和审计；
- Crash/Cancel/Timeout/Unknown 外部副作用有明确恢复行为；
- 基础备份和恢复演练通过。

### 35.4 v1.0 验收

- TUI、Desktop、Web 从同一 Core 事实和 Projection 工作；
- DAG、MCP、Skill、Memory、SubAgent、Hook、Plugin 的能力、权限、审计和恢复闭环；
- Observability、Maintenance、Upgrade、Rollback、Backup、Restore、Disaster Recovery 可操作；
- 安装包、签名、SBOM、版本矩阵、迁移和供应链测试通过；
- 核心不变量、P0 质量门和 Secret 防泄漏标准全部通过。

---

## 附录 A：首个 Issue 模板

```markdown
## 任务

- ID：TUI-001
- 标题：
- 优先级：P0/P1/P2
- 预估：
- Owner：

## 依赖

- 

## 写入范围

- 

## 实现要求

- 

## 验收标准

- [ ]
- [ ]

## 测试

- [ ] unit
- [ ] integration
- [ ] golden/e2e

## 安全/兼容影响

- 

## 关联 ADR/文档

- 
```

---

## 附录 B：首个提交序列建议

```text
commit 1: chore(workspace): initialize Apex Rust workspace and pinned toolchain
commit 2: feat(observability): add Spring Boot-style pattern file logging, run_id, call-site metadata and writer thread
commit 3: test(observability): verify PID/thread/task/source-line/redaction and demo output
commit 4: feat(domain): add typed IDs and error taxonomy
commit 5: feat(protocol): add minimal command/query/event envelopes
commit 6: feat(tui): add terminal guard and empty layout
commit 7: feat(tui): add app state reducer and input editor
commit 8: feat(tui): add in-process fake application client
commit 9: feat(tui): render session and assistant response
commit 10: test(tui): add reducer and terminal golden tests
commit 11: ci: add fmt check clippy and workspace tests
commit 12: docs: add v0.0.1 runbook and development plan
```

每个提交都应该能说明“新增了哪条可验证能力”，不要把第一周工作压成一个无法 review 的巨型提交。

---

## 附录 C：下一步执行命令

Apex workspace 已直接建立在当前仓库根目录，绝不能修改 `DeepSeek-TUI`、`CodeWhale`、`codex` 等参考项目目录。下一步先验证并运行已落地的 S00 日志基线：

```powershell
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo run -p apex-log-demo
```

确认 `logs/apex-log-demo-<run_id>.log` 中包含 PID、线程、协程、真实源码文件和行号后，再创建 `apex-domain`、`apex-protocol` 与 `apex-tui`。所有后续 crate 均从启动入口初始化 `apex-observability`，并按 S00-009 记录 started/progress/completed/failed。

---

## 附录 D：执行状态记录

本计划生成时的建议初始状态：

```text
S00-001～S00-008    verified（日志库、TaskContext、demo、字段/脱敏测试）
S00-009            coding（接入后续每个阶段并补齐失败路径）
BOOT-001～BOOT-008  coding（根 workspace 已建立，domain/protocol/tui 尚未齐备）
TUI-001～TUI-011    planned（等待 S00-009 与 TUI workspace 骨架）
TUI-012 以后       blocked by previous milestone
CORE-001 以后      blocked by TUI protocol/client boundary
```

每完成一个里程碑，更新本文件中的任务状态、实际耗时、发现的架构冲突和对应 ADR，不以计划文档替代真实 issue tracker。





