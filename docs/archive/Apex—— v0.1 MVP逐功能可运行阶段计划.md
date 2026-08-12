# Apex—— v0.1 MVP 逐功能可运行阶段细化计划

> 文档状态：v0.1 MVP 执行级计划
> 编制日期：2026-08-08
> 上位计划：`Apex—— 项目开发计划（从最小粒度TUI闭环开始）.md`
> 适用范围：Apex v0.1 MVP，从日志基础设施到可发布的单机 TUI 产品
> 核心要求：本文件中的每一个阶段都必须能够编译、启动或执行一个真实验收动作；不得以“模块代码已写完”替代阶段完成。

---

## 1. 计划目标

v0.1 不是一次性开发“大功能集合”，而是由一组可独立运行、可观测、可回退的垂直阶段组成：

```text
日志可写
  → TUI 可启动
  → TUI 可输入
  → Command 可传输
  → Core 可产生事件
  → SQLite 可恢复
  → Provider 可对话
  → 单会话 Agent Loop 可推进
  → 每个 Tool 可独立审批和执行
  → Spec 每个阶段可单独确认
  → Rules 可阻断违规变更
  → Checkpoint/Snapshot 可恢复
  → 观测面板可查询
  → 安装、升级、备份、恢复可演练
```

### 1.1 v0.1 完成定义

只有同时满足以下条件，v0.1 才能标记为完成：

- 所有 P0 功能均通过对应的可运行阶段；
- 每个阶段均有一条可重复执行的命令或集成测试；
- 每个阶段都能生成 `run_id` 关联的本地日志；
- 失败、取消、重启和恢复路径经过验证；
- Secret、Provider 原文、Prompt 原文和敏感文件内容不进入日志；
- SQLite 事件是状态恢复依据，日志仅作为诊断数据；
- TUI、Core、Storage、Provider、Tool、Spec、Rules 之间仍保持 Port 边界。

### 1.2 阶段状态

```text
planned → ready → coding → runnable → verified → done
                         │          │
                         └──────────┴→ blocked
```

`runnable` 表示存在可执行验证入口；`verified` 表示代码、测试、日志、文档和安全验收全部通过。

### 1.3 当前执行状态（2026-08-08）

- `S00-001`～`S00-008`：`verified`。已完成 `apex-observability`、每次运行独立 Spring Boot 风格的单行位置式日志文件、时间/PID/OS 线程/源码文件名与行号/traceId、显式 Tokio TaskContext、进度字段、字段名脱敏、flush/shutdown、单元测试和 `apex-log-demo`。
- `S00-009`：`coding`。后续阶段接入日志门禁后才能改为 `verified`；当前尚未创建 `apex-domain`、`apex-protocol` 和 `apex-tui`。
- `S01`～`S24`：按依赖保持 `planned`/`blocked`，不得把已有 workspace 空壳误报为 TUI 或完整 Apex 功能完成。
- S00 的脱敏边界：日志库对结构化字段名执行基础脱敏；业务代码仍禁止把 Prompt、Provider 原文、Secret、完整工具参数和敏感文件正文传入日志；完整 Secret Scanner 与 safe view 属于后续 S22。

---

## 2. 统一阶段契约

每个阶段都必须提供以下六类内容：

| 内容 | 要求 |
|---|---|
| 输入 | 明确依赖的命令、事件、文件、配置或上游阶段 |
| 实现 | 只修改阶段声明的写入范围 |
| 运行入口 | `cargo run`、CLI 子命令、测试或脚本之一 |
| 可见结果 | 终端输出、日志文件、SQLite 查询、生成文件或状态变化 |
| 验收 | 可自动化断言，避免只靠人工观察 |
| 回滚 | 删除生成物、恢复 migration、关闭 runtime 或 restore snapshot |

### 2.1 统一运行环境

```text
Rust 2024
Tokio multi-thread runtime
TUI: ratatui + crossterm
Serialization: serde + serde_json
Logging: tracing + apex-observability
Storage: rusqlite bundled SQLite + WAL
Provider: trait + FakeProvider + 至少一个真实 Provider
```

### 2.2 统一日志字段

所有阶段在启动时初始化 `apex-observability`。每条记录至少包含：

```json
{
  "timestamp": "UTC RFC3339 微秒",
  "sequence": 1,
  "elapsed_ms": 12,
  "level": "INFO",
  "component": "apex-runtime",
  "run_id": "...",
  "pid": 1234,
  "thread": {"id": "ThreadId(1)", "name": "main"},
  "task": {"id": "task-...", "name": "agent-loop", "kind": "tokio"},
  "source": {"file": "真实绝对路径", "line": 123, "module": "..."},
  "message_code": "runtime.turn.started",
  "message": "...",
  "fields": {"progress_current": 1, "progress_total": 4}
}
```

日志消息必须带稳定的 `message_code`；阶段进度必须使用 `progress_stage`、`progress_current`、`progress_total` 或 `progress_percent`，禁止只打印无法查询的自由文本。

---

## 3. 阶段总览

| 阶段 | 可运行交付 | 版本门槛 |
|---|---|---|
| S00 | 本地 Spring Boot 风格的单行位置式日志、调用点、线程、PID、Tokio 任务身份 | 必须最先完成 |
| S01 | Cargo workspace、CLI、TUI 空壳 | v0.0.1 |
| S02 | TUI 静态布局和 terminal guard | v0.0.1 |
| S03 | TUI 输入、命令、Reducer | v0.0.1 |
| S04 | In-process Fake Application | v0.0.1 |
| S05 | Fake Chat 流式闭环 | v0.0.2 |
| S06 | approval/tool mock 和 TUI 面板 | v0.0.3 |
| S07 | apexd 启动、IPC、握手、重连 | v0.0.4 |
| S08 | SQLite migration、事件、projection、cursor | v0.0.5 |
| S09 | Project/Session/Run/Turn 生命周期 | v0.1-alpha |
| S10 | Provider 配置、凭据元数据、真实请求 | v0.1-alpha |
| S11 | Agent Loop 单会话可恢复闭环 | v0.1-alpha |
| S12 | Permission 引擎和 Approval | v0.1-beta |
| S13 | Read/Glob/Grep 工具 | v0.1-beta |
| S14 | Write/Edit 工具 | v0.1-beta |
| S15 | Bash 工具 | v0.1-beta |
| S16 | Task 工具（线性子任务版） | v0.1-beta |
| S17 | Spec requirements/design/tasks | v0.1-rc1 |
| S18 | Spec implementation/verification/skip | v0.1-rc1 |
| S19 | Rules 编译、增量检查、Gate | v0.1-rc1 |
| S20 | Context budget、Checkpoint、压缩 | v0.1-rc1 |
| S21 | Shadow Git Snapshot、restore、冲突 | v0.1-rc1 |
| S22 | Observability 查询和 TUI 面板 | v0.1-rc1 |
| S23 | MVP 集成、崩溃恢复、迁移、备份 | v0.1-rc2 |
| S24 | 安装、升级、性能、安全和发布验收 | v0.1.0 |

---

## 4. S00：日志基础设施（首先实现）

日志设计参考 `D:\参考项目\日志项目\FastLog` 的分层思想：

```text
tracing call site
  → ApexFileLayer（Record + 字段采集）
  → JSON formatter
  → bounded channel
  → 独立文件 writer thread
  → 每次运行一个 logs/<component>-<run_id>.log
```

FastLog 提供了源码位置、线程、PID、formatter、file sink、异步 sink 和 flush 等可借鉴能力；Apex 使用 Rust `tracing` 的 `file!()`/`line!()` 元数据，使用 `tokio::task_local!` + `spawn_logged` 补充协程逻辑身份，并增加 `run_id`、`message_code` 和进度字段。

### S00-001：日志 crate 可编译

- **目标**：创建 `crates/apex-observability`。
- **输入**：Rust workspace。
- **实现**：定义 `LogConfig`、`LogLevel`、`LogError`、`LogRuntime`。
- **运行**：`cargo check -p apex-observability`。
- **验收**：crate 编译成功；无 UI/Core/SQLite 依赖。
- **依赖**：无。

### S00-002：每次运行创建独立文件

- **目标**：初始化日志目录，并以 `component + run_id` 创建新文件。
- **运行**：`cargo test -p apex-observability creates_a_distinct_file_for_each_run`。
- **验收**：两次初始化产生两个不同文件；目录不存在时自动创建；文件名不包含 Secret。
- **依赖**：S00-001。

### S00-003：真实源码文件和行号

- **目标**：每条日志记录调用点的真实文件和行号。
- **运行**：`cargo test -p apex-observability records_run_thread_process_and_real_call_site`。
- **验收**：`source.file` 只记录实际调用文件的最后一级文件名；`source.line` 等于测试调用行；不是 logger 内部固定行。
- **依赖**：S00-002。

### S00-004：时间、PID、线程记录

- **目标**：记录 UTC 微秒时间、PID、OS 线程 ID 和线程名。
- **运行**：日志 demo 启动后读取最新 Spring Boot 风格的单行位置式日志文件。
- **验收**：`timestamp` 可解析；所有记录 PID 一致；主线程和后台线程可区分；sequence 单调递增。
- **依赖**：S00-003。

### S00-005：Tokio 协程身份

- **目标**：给每个运行期后台协程提供稳定逻辑任务 ID 和名称。
- **实现**：所有长期后台任务通过 `spawn_logged`，需要保留已有上下文的异步函数使用 `scope_task`。
- **运行**：`cargo test -p apex-observability records_tokio_coroutine_identity`。
- **验收**：日志中 `task.id/name/kind` 可关联任务生命周期；未注册的同步代码允许 `task=null`。
- **依赖**：S00-004。

### S00-006：阶段进度字段

- **目标**：支持查询执行过程和中间状态。
- **约定**：每个可运行阶段至少打印 started、progress、completed/failed 三类 message code。
- **运行**：`cargo run -p apex-log-demo`。
- **验收**：终端打印 run_id 和日志路径；文件包含 `progress_current/progress_total`；`dropped_records=0`。
- **依赖**：S00-005。

### S00-007：敏感字段脱敏

- **目标**：日志先于文件写入执行结构化字段名级脱敏，并为后续 Secret Scanner 保留接入点。
- **禁止**：API key、token、password、Cookie、Authorization、私钥、Prompt 原文、Provider 原文、敏感文件正文。
- **运行**：S00-003 测试中的 `api_key` 断言。
- **验收**：敏感字段值变为 `[REDACTED]`；日志 writer 出错不会把原始值打印到 stderr；业务调用点不得传入禁止记录的自由文本。
- **依赖**：S00-006。

### S00-008：flush、关闭和写入失败

- **目标**：支持有界队列、阻塞/丢弃策略、flush 和 writer join。
- **运行**：`cargo test -p apex-observability --lib`。
- **验收**：shutdown 前所有已提交记录可读；writer 线程异常不导致业务 panic；丢弃数量可查询。
- **依赖**：S00-007。

### S00-009：日志模块接入所有后续阶段

- **目标**：形成硬性工程门禁。
- **规则**：任何新阶段没有日志初始化、started/progress/completed/failed 记录，不得进入 `verified`。
- **运行**：`cargo run -p apex-log-demo --offline` 作为最小基线；之后各阶段提供自己的 demo。
- **验收**：计划中的每个阶段表都包含日志验收项。
- **依赖**：S00-008。

### S00 完成产物

- `crates/apex-observability/`；
- `apps/apex-log-demo/`；
- `logs/<component>-<run_id>.log` Spring Boot 风格的单行位置式日志；
- 日志字段与脱敏测试；
- 日志使用约定和故障排查说明。

---

## 5. S01～S08：TUI 到本机事件恢复的基础链路

### S01：Workspace 和 CLI 空壳

| 阶段 | 实现 | 可运行验收 | 依赖 |
|---|---|---|---|
| S01-001 | 根 workspace、Cargo.lock、toolchain | `cargo metadata --no-deps` | S00 |
| S01-002 | `apex-domain` 稳定 ID/Error/Result | `cargo test -p apex-domain` | S01-001 |
| S01-003 | `apex-protocol` Command/Query/Event DTO | serde round-trip 测试 | S01-002 |
| S01-004 | `apex-cli` 命令行入口 | `apex --help` | S01-003 |
| S01-005 | `apex-tui` binary | `apex tui --help` | S01-004 |
| S01-006 | 启动日志接入 | 生成 run log 并记录 `cli.started` | S01-005 |

### S02：TUI 终端生命周期和静态界面

| 阶段 | 实现 | 可运行验收 | 依赖 |
|---|---|---|---|
| S02-001 | terminal raw mode guard | panic/错误后终端恢复测试 | S01 |
| S02-002 | 80x24 三栏布局 | snapshot/golden 渲染测试 | S02-001 |
| S02-003 | Header/Conversation/Inspector/Input/Footer | 启动后全部区域出现 | S02-002 |
| S02-004 | resize、Unicode、CJK 宽度处理 | fixture 渲染无越界 | S02-003 |
| S02-005 | quit/EOF/Ctrl-C 优先通道 | 所有退出路径 flush 日志 | S02-004 |

### S03：输入、Command 和 Reducer

| 阶段 | 实现 | 可运行验收 | 依赖 |
|---|---|---|---|
| S03-001 | InputState 光标、插入、删除、清空 | action 序列单测 | S02 |
| S03-002 | Enter/Esc/Ctrl-C 事件映射 | scripted key test | S03-001 |
| S03-003 | slash command parser | `/help`、`/quit`、未知命令测试 | S03-002 |
| S03-004 | AppState 和 Reducer | 相同输入产生相同状态 | S03-003 |
| S03-005 | effect queue | Command/Query/Subscribe effect 可枚举 | S03-004 |
| S03-006 | input progress log | 每次 command 记录 command_id 和 source line | S03-005 |

### S04：In-process Fake Application

| 阶段 | 实现 | 可运行验收 | 依赖 |
|---|---|---|---|
| S04-001 | ApplicationClient Port | Mock/InProcess 实现共用 trait | S03 |
| S04-002 | Fake event bus | event sequence 单调递增 | S04-001 |
| S04-003 | Fake project/session | 创建、选择、恢复 session | S04-002 |
| S04-004 | Query projection | TUI 不直接读 domain | S04-003 |
| S04-005 | `apex tui --demo` | 输入后显示 event/query 结果 | S04-004 |
| S04-006 | Fake app failure/retry | 人工制造错误后 TUI 可重试 | S04-005 |

### S05：Fake Chat 流式闭环

| 阶段 | 实现 | 可运行验收 | 依赖 |
|---|---|---|---|
| S05-001 | FakeProvider script | 固定 response/delta/error/cancel fixture | S04 |
| S05-002 | Turn started/finished | 每次输入得到可查询 turn 状态 | S05-001 |
| S05-003 | streaming delta | delta 合并为最终 assistant message | S05-002 |
| S05-004 | cancel path | Ctrl-C/Esc 后 provider 停止且生成取消日志 | S05-003 |
| S05-005 | replay final query | 丢失 transient delta 后 Query 修正 TUI | S05-004 |
| S05-006 | `v0.0.2` demo | `cargo run -p apex-cli -- tui --demo` | S05-005 |

### S06：Approval、Tool Mock 和面板

| 阶段 | 实现 | 可运行验收 | 依赖 |
|---|---|---|---|
| S06-001 | ToolCallView | 工具名、operation_id、状态可见 | S05 |
| S06-002 | ApprovalView | waiting/approved/denied/expired 可渲染 | S06-001 |
| S06-003 | Fake approval command | TUI 确认后收到唯一结果 | S06-002 |
| S06-004 | Inspector 面板路由 | Tab 切换不丢输入 | S06-003 |
| S06-005 | call log panel | 调用耗时、状态、摘要可查询 | S06-004 |
| S06-006 | `v0.0.3` demo | 完成一次 mock approval | S06-005 |

### S07：apexd 和 Native Transport

| 阶段 | 实现 | 可运行验收 | 依赖 |
|---|---|---|---|
| S07-001 | `apps/apexd` 启动根 | `apexd --help`/启动日志 | S06 |
| S07-002 | instance lock | 同一数据目录第二实例被拒绝 | S07-001 |
| S07-003 | Windows named pipe/native endpoint | discovery 文件和 endpoint 可用 | S07-002 |
| S07-004 | Hello/Challenge/Ready | 版本/token matrix 测试 | S07-003 |
| S07-005 | NativeTransport client | TUI command/query/event 流经 IPC | S07-004 |
| S07-006 | reconnect/cursor | kill/restart apexd 后 TUI 恢复 | S07-005 |
| S07-007 | `v0.0.4` demo | 单命令启动 core+TUI 并生成两端日志 | S07-006 |

### S08：SQLite 事件和投影

| 阶段 | 实现 | 可运行验收 | 依赖 |
|---|---|---|---|
| S08-001 | SQLite bootstrap/WAL/application_id | DB 打开并通过 pragma 断言 | S07 |
| S08-002 | schema_migrations | migration checksum 和 revision 可查询 | S08-001 |
| S08-003 | domain_events | seq、event_id、event_type、payload、redaction 字段存在 | S08-002 |
| S08-004 | StorageWriter 单写者 | 事务成功或整体回滚 | S08-003 |
| S08-005 | session/message projections | 删除投影后可重建 | S08-004 |
| S08-006 | outbox | commit 后投递，commit 前不可见 | S08-005 |
| S08-007 | event cursor/replay | after_seq/limit 边界正确 | S08-006 |
| S08-008 | crash recovery | 中断写入后 DB 可打开且不出现半事务 | S08-007 |
| S08-009 | `v0.0.5` demo | 重启后会话和消息恢复 | S08-008 |

---

## 6. S09～S11：Project、Provider 和 Agent Loop

### S09：Project/Session/Run/Turn 生命周期

| 阶段 | 实现 | 可运行验收 | 依赖 |
|---|---|---|---|
| S09-001 | Project 创建和 workspace 路径校验 | 非目录、越界路径被拒绝 | S08 |
| S09-002 | Session 创建/选择/关闭 | TUI 可列出并选择项目会话 | S09-001 |
| S09-003 | Run/Turn 状态机 | queued/running/waiting/completed/failed/interrupted 转换单测 | S09-002 |
| S09-004 | command_id 幂等 | 重复提交不重复产生业务结果 | S09-003 |
| S09-005 | lifecycle projection | 重启后恢复 active run 和 cursor | S09-004 |
| S09-006 | `v0.1-project-session` demo | 创建项目→会话→Run→Turn→恢复 | S09-005 |

### S10：Provider 层

| 阶段 | 实现 | 可运行验收 | 依赖 |
|---|---|---|---|
| S10-001 | `Provider` trait | FakeProvider 和测试 adapter 共用 contract | S09 |
| S10-002 | provider config schema | model/base_url/timeout/retry 可验证 | S10-001 |
| S10-003 | credential metadata | 仅引用 credential_id，不读取原文到日志 | S10-002 |
| S10-004 | FakeProvider stream/error/timeout | fixture 可重复模拟 | S10-003 |
| S10-005 | OpenAI-compatible adapter | 真实请求、流式 delta、usage、错误映射 | S10-004 |
| S10-006 | 第二 Provider adapter | 切换不破坏历史和 session | S10-005 |
| S10-007 | retry/backoff/cancel | 只重试 provider 请求，不重试已执行工具 | S10-006 |
| S10-008 | `v0.1-provider-alpha` demo | 配置 provider 后完成一次真实或 fake turn | S10-007 |

### S11：单会话 Agent Loop

| 阶段 | 实现 | 可运行验收 | 依赖 |
|---|---|---|---|
| S11-001 | RunContext | project/session/run/turn/operation/correlation 全量关联 | S09、S10 |
| S11-002 | prompt/context builder | system、rules、spec、history、user input 分层 | S11-001 |
| S11-003 | provider response parser | text/tool_call/finish/usage/error 分流 | S11-002 |
| S11-004 | linear Agent Loop | user→provider→assistant 可完成 | S11-003 |
| S11-005 | tool-call continuation | provider tool call→approval/tool result→继续推理 | S11-004 |
| S11-006 | cancel/timeout/failure | 任意 turn 可中断并保存最终状态 | S11-005 |
| S11-007 | restart/recover run | Core 重启后不重复已完成 operation | S11-006 |
| S11-008 | `v0.1-agent-alpha` demo | 一次 Agent turn 全链路可查询 | S11-007 |

---

## 7. S12～S16：权限与基础工具逐个可运行

### S12：Permission 和 Approval

| 阶段 | 实现 | 可运行验收 | 依赖 |
|---|---|---|---|
| S12-001 | PermissionRule schema | allow/deny/ask/always/never 可解析 | S11 |
| S12-002 | path classification | workspace-relative/outside/sensitive 正确分类 | S12-001 |
| S12-003 | Bash AST command classification | `rm -rf`、网络、重定向、管道等分类 | S12-002 |
| S12-004 | policy evaluator | 同样输入得到确定 decision | S12-003 |
| S12-005 | Approval lifecycle | create/approve/deny/expire/revoke 均产生事件 | S12-004 |
| S12-006 | priority control lane | cancel/security/approval 不被普通 delta 阻塞 | S12-005 |
| S12-007 | denial/approval TUI | TUI 可显示理由和下一步动作 | S12-006 |
| S12-008 | `v0.1-permission-beta` demo | 安全命令被阻断，用户批准后才继续 | S12-007 |

### S13：Read/Glob/Grep

| 阶段 | 实现 | 可运行验收 | 依赖 |
|---|---|---|---|
| S13-001 | ToolDefinition/operation_id | schema、版本、超时、取消字段可序列化 | S12 |
| S13-002 | Read tool | 读取 workspace 文件，输出限长、行号和 digest | S13-001 |
| S13-003 | Glob tool | glob 结果排序、数量/字节上限和越界阻断 | S13-002 |
| S13-004 | Grep tool | 搜索结果脱敏、截断、文件类型过滤 | S13-003 |
| S13-005 | result normalizer | success/denied/timeout/cancelled/error 统一格式 | S13-004 |
| S13-006 | tool audit projection | 工具调用耗时、状态、参数摘要可查 | S13-005 |
| S13-007 | `v0.1-read-tools-beta` demo | TUI 输入查询并看到结果和日志 | S13-006 |

### S14：Write/Edit

| 阶段 | 实现 | 可运行验收 | 依赖 |
|---|---|---|---|
| S14-001 | write preflight | workspace、符号链接、大小、编码检查 | S13 |
| S14-002 | Write tool | 仅经 Permission + ToolGateway 写文件 | S14-001 |
| S14-003 | Edit patch parser | 单文件 patch、上下文匹配和冲突检测 | S14-002 |
| S14-004 | Edit tool | 成功修改、匹配失败和冲突均可恢复 | S14-003 |
| S14-005 | PostToolUse event | 文件变更产生 digest、size、diff summary | S14-004 |
| S14-006 | rollback-on-failure | 写入后 gate 失败可回到变更前状态 | S14-005 |
| S14-007 | `v0.1-write-tools-beta` demo | 创建文件→编辑→失败回滚 | S14-006 |

### S15：Bash

| 阶段 | 实现 | 可运行验收 | 依赖 |
|---|---|---|---|
| S15-001 | process supervisor | 子进程生命周期、stdout/stderr、退出码可回收 | S14 |
| S15-002 | command allowlist | `cargo test` 等安全命令可执行 | S15-001 |
| S15-003 | dangerous command block | 删除、提权、网络外传、后台驻留被阻断 | S15-002 |
| S15-004 | timeout/cancel/process-tree cleanup | Ctrl-C 后子孙进程结束 | S15-003 |
| S15-005 | output truncation | stdout/stderr 限长，digest 和 byte count 保留 | S15-004 |
| S15-006 | shell audit | command fingerprint、decision、exit class 可查询 | S15-005 |
| S15-007 | `v0.1-bash-beta` demo | 执行安全测试命令并拒绝危险命令 | S15-006 |

### S16：Task（线性子任务版）

| 阶段 | 实现 | 可运行验收 | 依赖 |
|---|---|---|---|
| S16-001 | Task schema | parent_run、task_id、write_scope、status 可持久化 | S15 |
| S16-002 | sequential task runner | 依赖顺序执行，不实现并行 DAG | S16-001 |
| S16-003 | child context | 子任务继承安全上下文但拥有独立 operation | S16-002 |
| S16-004 | child failure handling | 失败暂停父任务并显示原因 | S16-003 |
| S16-005 | Task TUI panel | 子任务状态、进度、输出摘要 | S16-004 |
| S16-006 | `v0.1-task-beta` demo | 父任务运行两个线性子任务并恢复 | S16-005 |

---

## 8. S17～S19：Spec 和 Rules 完整闭环

### S17：Requirements、Design、Tasks

| 阶段 | 实现 | 可运行验收 | 依赖 |
|---|---|---|---|
| S17-001 | Spec aggregate/schema | feature、revision、stage、status 可持久化 | S16 |
| S17-002 | requirements artifact | 生成带 frontmatter 的 requirements.md | S17-001 |
| S17-003 | requirements review gate | 用户 approve/reject/edit 后状态正确 | S17-002 |
| S17-004 | design artifact | 生成 design.md 并嵌入 rules/acceptance | S17-003 |
| S17-005 | design review gate | 修改后产生 revision、下游失效 | S17-004 |
| S17-006 | tasks artifact | 生成 tasks.md、依赖和写路径声明 | S17-005 |
| S17-007 | tasks review gate | 确认后才能进入 implementation | S17-006 |
| S17-008 | `v0.1-spec-plan-rc1` demo | 需求→设计→任务三道门逐一确认 | S17-007 |

### S18：Implementation、Verification 和 Skip-Spec

| 阶段 | 实现 | 可运行验收 | 依赖 |
|---|---|---|---|
| S18-001 | implementation runner | 按 tasks 顺序调用 Agent Loop/Tool | S17 |
| S18-002 | artifact versioning | 用户编辑/Agent 修改均有 revision/diff | S18-001 |
| S18-003 | `/skip-spec` command | 跳过决策带理由和 spec_skipped=true | S18-002 |
| S18-004 | verification artifact | 生成 verification.md 和逐条证据 | S18-003 |
| S18-005 | verification review gate | 用户确认后 feature completed | S18-004 |
| S18-006 | invalidation | 上游 requirements/design 变更使下游失效 | S18-005 |
| S18-007 | `v0.1-spec-full-rc1` demo | 正常流水线和 skip-spec 各完成一次 | S18-006 |

### S19：Rules、PostToolUse 和 Verification Gate

| 阶段 | 实现 | 可运行验收 | 依赖 |
|---|---|---|---|
| S19-001 | rule source loader | 加载 `apex/rules`、AGENTS/CLAUDE、全局规则 | S18 |
| S19-002 | rule compiler | 规则解析、优先级、revision、diagnostic code | S19-001 |
| S19-003 | changed-file detector | 只检查本次变更文件 | S19-002 |
| S19-004 | format/lint/test adapter | 运行规则命令并收集结果 | S19-003 |
| S19-005 | PostToolUse hook | 每次 Write/Edit 后自动触发增量检查 | S19-004 |
| S19-006 | repair task | error 级问题生成修复子任务 | S19-005 |
| S19-007 | verification gate | gate 阻断或放行有事件和安全理由 | S19-006 |
| S19-008 | rules quality panel | TUI 显示通过、警告、错误、修复中 | S19-007 |
| S19-009 | `v0.1-rules-rc1` demo | 故意引入违规代码，流程自动阻断并修复 | S19-008 |

---

## 9. S20～S22：上下文、快照和可观测性

### S20：Context Budget 和 Checkpoint

| 阶段 | 实现 | 可运行验收 | 依赖 |
|---|---|---|---|
| S20-001 | context item 分类 | system/rule/spec/history/tool/external 分层 | S19 |
| S20-002 | token budget calculator | 预算、保留区、超限原因可查询 | S20-001 |
| S20-003 | Checkpoint schema | 输入、稳定前缀、摘要、usage、revision 可保存 | S20-002 |
| S20-004 | checkpoint materializer | turn 完成、cancel、tool 后生成 checkpoint | S20-003 |
| S20-005 | compaction strategy | checkpoint-first，分级摘要兜底 | S20-004 |
| S20-006 | prefix cache layout | 稳定前缀和易变后缀分离 | S20-005 |
| S20-007 | restore context | 重启后根据 checkpoint 恢复下一 turn | S20-006 |
| S20-008 | `v0.1-context-rc1` demo | 超长历史压缩后继续完成任务 | S20-007 |

### S21：Shadow Git Snapshot 和恢复

| 阶段 | 实现 | 可运行验收 | 依赖 |
|---|---|---|---|
| S21-001 | shadow repository bootstrap | 项目目录外建立隔离 Git metadata | S20 |
| S21-002 | clean baseline snapshot | run 开始前记录基线 commit/digest | S21-001 |
| S21-003 | post-tool snapshot | 每个写操作后生成文件级快照 | S21-002 |
| S21-004 | diff/patch query | TUI 显示变更文件、增删行、摘要 | S21-003 |
| S21-005 | restore snapshot | 用户确认后恢复到指定 checkpoint | S21-004 |
| S21-006 | conflict detection | 外部修改导致 restore 时明确阻断 | S21-005 |
| S21-007 | `v0.1-snapshot-rc1` demo | 写入→查看 diff→恢复→验证文件内容 | S21-006 |

### S22：Observability 和 TUI 运行面板

| 阶段 | 实现 | 可运行验收 | 依赖 |
|---|---|---|---|
| S22-001 | TelemetryContext | project/session/run/turn/operation/correlation 可关联 | S21 |
| S22-002 | structured log query | 可按 run_id、level、message_code、时间查询本地日志 | S22-001 |
| S22-003 | call log projection | Provider/Tool/MCP/Skill/SubAgent 调用摘要可查 | S22-002 |
| S22-004 | health metrics | latency、error、queue depth、projection lag 可查 | S22-003 |
| S22-005 | audit projection | approval、skip、write、restore 有安全证据 | S22-004 |
| S22-006 | TUI Overview panel | 当前 run、进度、耗时、错误、连接状态 | S22-005 |
| S22-007 | TUI Skill/MCP/SubAgent/Memory panel | v0.1 展示基础投影；能力缺失显示明确 unavailable | S22-006 |
| S22-008 | support bundle | 日志脱敏、事件摘要、版本、health 打包 | S22-007 |
| S22-009 | `v0.1-observability-rc1` demo | 运行失败→查询证据→导出安全诊断包 | S22-008 |

---

## 10. S23～S24：集成、恢复与发布

### S23：单机 MVP 集成

| 阶段 | 实现 | 可运行验收 | 依赖 |
|---|---|---|---|
| S23-001 | Project/Session 选择 | TUI 创建项目并恢复最近会话 | S09、S22 |
| S23-002 | 双 Provider 切换 | 两 adapter 完成相同任务，历史连续 | S10、S11 |
| S23-003 | 基础工具全量接入 | Read/Write/Edit/Bash/Glob/Grep/Task 都走 Gateway | S12～S16 |
| S23-004 | Spec 全量接入 | 正常五阶段和 skip-spec 均可完成 | S17、S18 |
| S23-005 | Rules Gate 接入 | Write/Edit 后检查并阻断违规变更 | S19 |
| S23-006 | Checkpoint/Snapshot 接入 | cancel/crash/timeout/restore 可恢复 | S20、S21 |
| S23-007 | TUI 全面状态投影 | session/run/tool/approval/spec/rules/snapshot 可切换 | S22 |
| S23-008 | `v0.1-mvp-integration` demo | 从输入需求到变更验证的完整单机流程 | S23-001～S23-007 |

### S24：质量、安装、升级和正式发布

| 阶段 | 实现 | 可运行验收 | 依赖 |
|---|---|---|---|
| S24-001 | core integration suite | fake provider 全量回归 | S23 |
| S24-002 | crash/restart matrix | Core、TUI、Provider、Tool、DB 各类中断可恢复 | S24-001 |
| S24-003 | Secret scan | 仓库、日志、fixture、support bundle 扫描 | S24-002 |
| S24-004 | performance baseline | TUI 启动、事件提交、SQLite query、日志吞吐达标 | S24-003 |
| S24-005 | migration upgrade | 空库、旧库、失败 migration、回滚演练 | S24-004 |
| S24-006 | backup/restore | 备份 manifest、digest、seq 和恢复结果正确 | S24-005 |
| S24-007 | package install | Windows/macOS/Linux 本机安装后 `apex tui` 可用 | S24-006 |
| S24-008 | release candidate | `cargo fmt/clippy/test/audit` 全部通过 | S24-007 |
| S24-009 | v0.1.0 release | 二进制、SBOM、迁移说明、Runbook、已知限制齐全 | S24-008 |

---

## 11. 每个可运行阶段的日志最小模板

阶段启动：

```rust
tracing::info!(
    message_code = "stage.started",
    progress_stage = "S13-002-read-tool",
    progress_current = 0_u64,
    progress_total = 1_u64,
    "stage started"
);
```

中间状态：

```rust
tracing::debug!(
    message_code = "tool.read.progress",
    progress_stage = "S13-002-read-tool",
    progress_current = 512_u64,
    progress_total = 2048_u64,
    bytes_read = 512_u64,
    "read progress"
);
```

完成或失败：

```rust
tracing::info!(
    message_code = "stage.completed",
    progress_stage = "S13-002-read-tool",
    progress_current = 1_u64,
    progress_total = 1_u64,
    duration_ms = 42_u64,
    "stage completed"
);
```

```rust
tracing::error!(
    message_code = "stage.failed",
    progress_stage = "S13-002-read-tool",
    error_code = "tool.read.denied",
    retryable = false,
    "stage failed"
);
```

禁止：

- 只写 `println!("doing...")` 而没有结构化字段；
- 在日志中写入完整工具参数、Prompt、Provider 响应或 Shell 输出；
- 把日志记录当作 Session/Run 事实状态；
- 使用无界 channel；
- 在 TUI 绘制模块内自行创建日志文件；
- 在业务代码中直接调用 `process::exit` 绕过日志 flush。

---

## 12. P0 关键路径和并行边界

### 12.1 严格串行关键路径

```text
S00 日志
 → S01 workspace
 → S02 TUI shell
 → S03 input/reducer
 → S04 fake application
 → S05 fake chat
 → S07 apexd
 → S08 SQLite
 → S09 生命周期
 → S10 Provider
 → S11 Agent Loop
 → S12 Permission
 → S13 Read tools
 → S14 Write/Edit
 → S15 Bash
 → S17/S18 Spec
 → S19 Rules
 → S20 Checkpoint
 → S21 Snapshot
 → S22 Observability
 → S23 集成
 → S24 发布
```

### 12.2 可并行但不能提前合并

- S00 完成后，日志字段测试、文档、demo 可并行；
- S02 后，TUI renderer snapshot 和键盘 fixture 可并行；
- S08 后，projection rebuild、backup manifest、crash test 可并行；
- S10 后，两个 Provider adapter 可以并行，但必须共用 provider contract；
- S13 的 Read/Glob/Grep 可并行，但必须共用 ToolGateway 和 result normalizer；
- S17 的 markdown artifact、frontmatter、review gate 可并行；
- S22 的日志查询、metrics、audit、TUI panel 可并行；
- 不允许在 S12 Permission 未完成前实现真实 Write/Edit/Bash；
- 不允许在 S19 Rules 未完成前将写工具标记为 MVP 完成；
- 不允许在 S20/S21 未完成前宣称 Agent Loop 支持可靠恢复。

---

## 13. 每个里程碑的命令入口

| 里程碑 | 推荐命令 | 结果 |
|---|---|---|
| S00 | `cargo run -p apex-log-demo` | 生成一份带 PID/线程/协程/源码行的 Spring Boot 风格的单行位置式日志 |
| S01 | `cargo run -p apex -- --help` | CLI 帮助可见 |
| S02 | `cargo run -p apex -- tui --demo-layout` | 80x24 TUI 静态布局 |
| S03 | `cargo test -p apex-tui reducer` | 输入状态确定性测试 |
| S04 | `cargo run -p apex -- tui --demo-app` | fake command/event/query 闭环 |
| S05 | `cargo run -p apex -- tui --demo-chat` | streaming chat 闭环 |
| S07 | `cargo run -p apexd` + `cargo run -p apex -- tui` | TUI 通过本机连接 Core |
| S08 | `cargo test -p apex-storage crash_recovery` | SQLite 事件恢复 |
| S10 | `cargo run -p apex -- provider probe` | provider 配置和连接检查 |
| S11 | `cargo run -p apex -- run --fake-provider` | 单会话 Agent Loop |
| S12 | `cargo run -p apex -- permission explain` | 权限决策解释 |
| S13～S16 | `cargo test -p apex-tools integration` | 每个工具独立验收 |
| S17～S19 | `cargo run -p apex -- spec demo` | Spec + Rules gate |
| S20～S21 | `cargo run -p apex -- recovery demo` | checkpoint/snapshot restore |
| S22 | `cargo run -p apex -- diagnostics export` | 脱敏诊断包 |
| S24 | `cargo test --workspace` | 发布前总回归 |

命令名称在实际 crate 落地时可以调整，但每个阶段必须保留一个等价的稳定入口，并更新本表。

---

## 14. 第一轮实施顺序

第一轮只做以下阶段，不提前实现 Provider、SQLite 或真实工具：

1. S00-001～S00-009：日志模块和 demo；
2. S01-001～S01-006：workspace 与 CLI/TUI 空壳；
3. S02-001～S02-005：终端生命周期和静态布局；
4. S03-001～S03-006：输入、Reducer、命令和日志；
5. S04-001～S04-006：In-process Fake Application；
6. S05-001～S05-006：Fake Chat；
7. 运行一次完整质量门后，再进入 S07。

### 14.1 第一轮禁止事项

- 不接真实 API；
- 不读取或保存 credential 原文；
- 不执行真实 Shell；
- 不修改用户 workspace 文件；
- 不在日志中记录 Prompt 或模型原始输出；
- 不以 UI 文本代替 Domain Event；
- 不为了“先跑起来”删除 Port、错误类型或日志字段。

---

## 15. v0.1 最终验收清单

### 功能

- [ ] TUI 能启动、输入、取消、退出并恢复终端；
- [ ] 项目和会话可创建、选择、重启恢复；
- [ ] 双 Provider 可配置、切换、流式输出、超时和取消；
- [ ] 单会话 Agent Loop 可调用 Tool 并继续推理；
- [ ] Read/Write/Edit/Bash/Glob/Grep/Task 全部走 Gateway；
- [ ] 高危动作有确定性拒绝或审批；
- [ ] Spec 五阶段和 `/skip-spec` 可审计；
- [ ] PostToolUse 增量 Rules 检查和 Verification Gate 可阻断；
- [ ] Checkpoint、Shadow Git Snapshot、restore 可运行；
- [ ] 日志、调用记录、审批、审计、健康状态可查询。

### 日志

- [ ] 每次进程运行生成独立日志文件；
- [ ] 每条日志含时间、run_id、PID、线程和真实源码行；
- [ ] Tokio 后台任务含逻辑协程 ID/名称；
- [ ] 每个阶段含 started/progress/completed 或 failed；
- [ ] 日志队列有界，flush/shutdown 可验证；
- [ ] 日志落盘失败不会导致核心业务 panic；
- [ ] Secret、Prompt、Provider 原文和敏感文件内容不会进入日志；
- [ ] 日志不是业务恢复事实源。

### 质量

- [ ] `cargo fmt --all -- --check`；
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`；
- [ ] `cargo test --workspace`；
- [ ] 依赖审计和 Secret 扫描；
- [ ] Windows/macOS/Linux TUI 手工验收；
- [ ] Core 崩溃、Provider 超时、Tool 取消、DB 中断、Snapshot 冲突均有明确结果。




