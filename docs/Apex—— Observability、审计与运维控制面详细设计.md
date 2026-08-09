# Apex—— Observability、审计与运维控制面详细设计

> 文档状态：详细设计基线（面向最终完整产品）  
> 适用版本：Apex v0.1 ～ v1.x（按 §交付阶段 分档启用；档位表以需求文档 §5.3 为准）  
> 上游文档：`Apex—— 需求分析文档.md`、`Apex—— 系统总体架构设计.md`、`Apex—— 领域模型与事件规范.md`、`Apex—— API与实时事件协议设计.md`、`Apex—— SQLite数据模型与迁移设计.md`、`Apex—— Agent Runtime与DAG调度器详细设计.md`、`Apex—— Tool Gateway与权限引擎详细设计.md`、`Apex—— Rules与Verification Gate详细设计.md`、`Apex—— MCP、Skill、Hook与Plugin扩展系统详细设计.md`、`Apex—— Credential与敏感数据治理详细设计.md`  
> 关键词：Observability、Audit、Event Store、Projection、Metric、Trace、Alert、Incident、Maintenance、Support Bundle、Recovery、Redaction

---

## 0. 文档目的与范围

本文定义 Apex 最终完整产品的可观测性、审计和运维控制面。目标不是简单增加几个日志页面，而是把“系统发生了什么、谁允许了什么、运行是否健康、出现故障后如何恢复”统一为一套可持久化、可重放、可查询、可脱敏和可操作的控制面。

本文覆盖：

- Domain Event、Realtime Event、Log、Metric、Trace、Audit Record、Incident、Maintenance Run 的边界；
- Event Store、Projection、Event Bus、Outbox、实时订阅和游标恢复；
- TUI、Desktop、Web 三类客户端共用的面板查询模型；
- Skill、MCP、SubAgent、Tool、Rule、Gate、Checkpoint、Workspace、Credential 和 Provider 的审计；
- 结构化单行日志、指标、分布式追踪、告警、事件（Incident）和支持诊断包；
- 数据库、投影、备份、迁移、恢复、隐私清除和磁盘水位等运维任务；
- 运维权限、双人确认、维护锁、进度、取消、失败恢复和审计闭环；
- 生产故障模式、测试策略、SLO 和分阶段交付路线。

本文不重新定义业务状态机、Tool 判权算法、Credential Store 内部加密实现或各 Provider 的专用协议。它们通过既有 Domain Event、Capability、Command、Query、Blob 和 Maintenance Port 接入本控制面。

### 0.1 设计对象

```text
业务命令 / Runtime / Adapter
            │
            ▼
      Domain Event commit
            │
            ├── Event Store（事实）
            ├── Projection（查询）
            ├── Event Bus / Outbox（实时与可靠投递）
            ├── Audit（责任与安全证据）
            ├── Metrics（聚合数值）
            ├── Traces（跨边界时序）
            ├── Logs（诊断上下文）
            └── Alerts / Incidents（处置闭环）
```

### 0.2 核心问题

Apex 需要同时满足以下看似冲突的要求：

1. 运行过程足够透明，用户可以看到 Agent、工具、MCP、Skill、规则和文件变化；
2. 事件可恢复，重启后不能依赖内存日志猜测之前发生了什么；
3. 诊断信息足够丰富，但 API key、Prompt 中的 Secret、敏感文件和外部响应不能泄漏；
4. 实时面板接近秒级更新，但高频 token delta 不应把 SQLite 写放大到不可用；
5. 维护操作可控可审计，投影重建、恢复、清理和备份不能被普通查询权限触发；
6. 故障发生后支持重试，但未知外部副作用不得被盲目重放。

---

## 1. 核心架构结论

### 1.1 Event Store 是业务事实源，Telemetry 不是业务事实源

只有已经通过 Core 事务提交的 Domain Event 才能作为业务事实、恢复依据和审计证据。日志、指标、Trace、实时文本流和客户端缓存都只能辅助诊断，不能被用来推断或覆盖业务状态。

```text
Domain Event       = 已提交、不可变、可重放的业务事实
Realtime Event     = 面向订阅者的短期更新，可丢失，可由游标恢复
Audit Record       = 责任、安全、合规和运维动作证据
Log                = 人类/机器诊断上下文，不承载状态权威
Metric             = 聚合数值，用于趋势、容量和 SLO
Trace              = 一次操作跨模块/进程的时序视图
Projection         = 从事实派生的可查询读模型
Incident           = 对异常的处置工作流，不是原始事实
Maintenance Run    = 受控运维命令的生命周期
```

### 1.2 Event Bus 是运行时事实广播的唯一入口

任何模块需要通知客户端、面板、告警计算器或异步消费者时，都必须从 Event Bus 订阅。禁止模块绕过 Core 直接向某个 WebSocket、TUI channel 或前端 store 写入业务事件。

```text
CommandHandler / Runtime / Recovery
                 │
                 ▼
       StorageWriter transaction
       ├─ append domain event
       ├─ update projection cursor
       └─ enqueue outbox
                 │ commit
                 ▼
              Event Bus
       ├─ Realtime broadcaster
       ├─ Audit indexer
       ├─ Alert evaluator
       ├─ Metric reducer
       └─ external adapter (optional, off by default)
```

事务提交前不可广播“成功”事实；提交失败不得发送成功事件。提交后广播失败不回滚业务事实，可靠消费者通过 Outbox 和 cursor 继续追赶。

### 1.3 可观测性不能改变业务状态

Observability 组件只能：

- 读取已提交事件和授权读模型；
- 生成诊断数据、告警和 Incident；
- 接收明确授权的运维 Command；
- 触发受策略保护的维护任务。

Observability 组件不能：

- 直接修改 Run、Workflow、Approval、Permission、Credential、Workspace 或文件状态；
- 通过“告警处理器”执行任意 Tool；
- 把前端看到的状态写回 Domain Store；
- 以“补日志”为理由写入未经定义的业务事件。

### 1.4 脱敏发生在数据离开可信边界之前

脱敏不是前端展示技巧。任何数据进入 Event Store、Log Sink、Metric Label、Trace Attribute、Diagnostic Bundle、Blob 下载或外部 telemetry 之前，必须经过分类和扫描：

```text
raw input
  → classify
  → secret scanner / sensitive-path policy
  → redact or reject
  → safe summary + digest + lineage
  → persist / broadcast / export
```

`secret_prohibited`、`credential_material`、`private_key` 等数据分类不得出现在普通事件、日志、指标标签或 Trace 属性中。必要时只记录 `secret_present=true`、数量、来源类型、策略结果和不可逆摘要。

### 1.5 三类客户端共享同一控制面

TUI、Desktop 和 Web 的页面布局可以不同，但不拥有不同的事实源或业务逻辑。它们使用同一套 Query、Event Subscription、Cursor、Capability 和 Redaction 语义：

```text
TUI (Rust/ratatui)      ┐
Desktop (Tauri/Vue/TS)  ├── Core Query + Event Subscription
Web (Actix/Vue/TS)      ┘
```

Web Gateway 仅做认证、协议转换、限流和连接管理；不能为了前端方便另造一套审计语义。

---

## 2. 术语与数据分类

### 2.1 TelemetryRecord 统一外壳

不同类型的可观测数据共享关联字段，但不共享生命周期和持久化等级：

```rust
pub struct TelemetryContext {
    pub project_id: ProjectId,
    pub session_id: Option<SessionId>,
    pub run_id: Option<RunId>,
    pub turn_id: Option<TurnId>,
    pub operation_id: Option<OperationId>,
    pub actor_id: Option<ActorId>,
    pub client_id: Option<ClientId>,
    pub correlation_id: CorrelationId,
    pub causation_id: Option<EventId>,
    pub trace_id: Option<TraceId>,
    pub span_id: Option<SpanId>,
}
```

所有记录都必须能够回答“属于哪个项目/会话/运行/操作”，但允许某些后台维护任务没有 Session 或 Run。

### 2.2 持久化等级

| 等级 | 数据 | 默认保留 | 能否作为恢复依据 | 典型用途 |
|---|---|---:|---|---|
| D0 | Domain Event | 项目生命周期内 | 是 | 状态、审计事实、重放 |
| D1 | Security/Operator Audit | 项目或策略定义 | 只能作为责任证据 | 权限、批准、维护、导出 |
| D2 | Projection | 可重建 | 否 | 面板和查询 |
| D3 | Outbox | 直到确认 | 否 | 可靠投递 |
| D4 | Metrics | 按窗口聚合 | 否 | SLO、容量、趋势 |
| D5 | Logs/Traces | 可配置短期 | 否 | 故障诊断 |
| D6 | Realtime Event | 内存或短期缓存 | 否 | token、进度、UI 刷新 |

任何文档、代码或 UI 都不得把 D4～D6 的数据当作 D0 事实使用。

### 2.3 数据分类

可观测性管道采用 Credential 治理文档 §3 定义的五级数据分级，并增加 Telemetry 约束：

```text
public              可公开展示的版本、计数和非敏感状态
internal            项目内部信息，可在授权项目范围查询
confidential        任务参数摘要、路径、模型用量、规则详情
sensitive           Prompt 片段、外部响应、环境变量名值、个人数据
secret_prohibited   Secret 原文、完整 Authorization、私钥、Cookie、口令
```

分类与 `redaction_level`、`taint` 和访问 Capability 一起决定是否存储、是否广播、是否可导出，而不是由某个客户端自行决定。

---

## 3. 端到端数据流与边界

### 3.1 主路径

```text
[Command / Provider / Tool / Runtime]
                │
                ▼
        Context + Policy Check
                │
                ▼
        Domain State Transition
                │
                ▼
      Canonical Event + Safe Payload
                │
                ▼
      SQLite StorageWriter transaction
        ├── domain_events
        ├── projections / projection_cursors
        └── outbox
                │ commit
                ▼
          Event Bus (global_seq)
          ├── clients/realtime
          ├── audit projector
          ├── metrics reducer
          ├── alert evaluator
          └── trace/log correlation
```

### 3.2 外部边界

外部 Provider、MCP Server、Shell 子进程、Hook、Plugin 和网络连接都被视为不可信或部分可信边界。它们只能通过对应 Adapter 返回规范化结果；Adapter 负责：

- 生成 `operation_id` 和 `external_operation_id`；
- 记录请求开始、结束、超时、取消和未知副作用；
- 扫描输出并标记 taint；
- 删除原始 Secret 和完整凭据头；
- 提供稳定的 `result_summary`、`error_code`、`duration_ms` 和 `external_effect_state`；
- 把原始大响应放入受策略保护的 Blob，而不是写入普通事件。

### 3.3 事务边界

`StorageWriter` 在一个 SQLite 写事务中完成：

1. 事件 payload canonicalize 和安全扫描；
2. 插入 Domain Event；
3. 更新同步投影和投影游标；
4. 写入需要可靠投递的 Outbox；
5. 写入必要的审计索引；
6. 提交事务。

事务内禁止网络调用、Provider 调用、文件大规模扫描、外部进程启动和 telemetry exporter 远程发送。慢任务必须先提交“开始”事实，再异步执行并提交结果事实。

---

## 4. Domain Event、Audit、Log、Metric 与 Trace 的边界

### 4.1 Domain Event

Domain Event 是不可变业务事实，至少包含：

```json
{
  "event_id": "evt_01...",
  "global_seq": 1842,
  "project_id": "prj_...",
  "session_id": "ses_...",
  "run_id": "run_...",
  "actor_id": "actor_...",
  "event_type": "tool.call_finished",
  "occurred_at": "2026-08-08T10:00:00Z",
  "correlation_id": "corr_...",
  "causation_id": "evt_...",
  "schema_version": 1,
  "redaction_level": "safe_view",
  "payload": {}
}
```

事件 payload 只包含可安全持久化的业务字段。原始参数、Secret、Prompt 和响应正文必须被摘要化、脱敏或移入受保护 Blob。

### 4.2 Audit Record

Audit Record 记录“责任与控制”，重点回答：

- 谁（actor/client/principal）；
- 在哪个项目、哪个连接、哪个进程中；
- 对什么对象执行了什么动作；
- 动作由哪个命令、批准、策略或维护任务触发；
- 判定依据和策略版本是什么；
- 结果是允许、拒绝、跳过、撤销、失败还是未知；
- 证据是否完整、是否经过脱敏、是否可导出。

审计记录可以由一个或多个 Domain Event 派生，但不得只保留“前端点击了按钮”这种无业务结果的 UI 事件。

### 4.3 Log

Log 是诊断上下文，允许更短的保留期和采样，但必须是结构化数据。日志不能代替事件，也不能成为执行控制开关。日志中的 `message` 只应描述安全摘要，原始参数使用字段级脱敏后的结构化字段。

### 4.4 Metric

Metric 只保存可聚合的数值和受限标签，例如请求数、错误数、延迟、队列长度、数据库页数、磁盘水位。不得将 Prompt、路径全文、URL 查询参数、Credential ID 全量、用户输入或事件 payload 放入标签。

### 4.5 Trace

Trace 描述跨模块、跨进程和外部边界的时间关系。Trace 可以关联 Event ID 和 Operation ID，但不能通过 span attribute 保存 Secret 或原始外部响应。采样丢失不影响业务恢复。

---

## 5. Event Envelope、命名和完整性

### 5.1 强制字段

沿用领域事件规范，Observability 相关事件必须具备：

| 字段 | 约束 | 说明 |
|---|---|---|
| `event_id` | 全局唯一 | ULID/UUIDv7，便于时间排序 |
| `global_seq` | 单项目单调递增 | 实时游标和恢复边界 |
| `project_id` | 必填 | 多项目隔离边界 |
| `session_id` | 可选 | 后台任务可为空 |
| `run_id` | 可选 | 维护任务或系统事件可为空 |
| `actor_id` | 必填或显式 `system` | 责任主体 |
| `event_type` | 注册表内 | 事件命名空间和版本 |
| `occurred_at` | UTC | 业务发生时间 |
| `recorded_at` | UTC | Core 接收/提交时间 |
| `correlation_id` | 必填 | 一条用户意图或维护任务链 |
| `causation_id` | 可选 | 触发该事件的上游事件 |
| `operation_id` | 可选 | 外部操作或运维操作关联 |
| `schema_version` | 必填 | Upcaster 依据 |
| `redaction_level` | 必填 | `safe_view` 等 |
| `payload_digest` | 必填 | canonical payload 摘要 |
| `payload` | 安全 JSON | 禁止 secret_prohibited |

### 5.2 命名空间

事件名称使用小写点分隔并带版本：

```text
session.*
run.*
turn.*
provider.*
tool.*
mcp.*
skill.*
agent.*
workflow.*
permission.*
approval.*
rules.*
gate.*
checkpoint.*
workspace.*
credential.*
data.*
observability.*
audit.*
maintenance.*
incident.*
recovery.*
```

事件名称表达事实，不表达 UI 行为。例如使用 `approval.grant.completed`，而不是 `button.approve.clicked`。

### 5.3 链式完整性

对需要高可信审计的项目，事件表支持项目级哈希链：

```text
event_hash = SHA256(
    previous_event_hash ||
    canonical_envelope_without_event_hash
)
```

哈希链用于检测删除、插入和顺序篡改，不等于不可伪造的外部签名。需要跨机器或合规归档时，支持按时间窗口生成签名 manifest；签名私钥不放在 SQLite 中。

### 5.4 版本演进

- 新增可选字段：原版本保持可读；
- 修改语义或必填字段：创建新事件版本；
- 旧事件通过 Upcaster 映射到当前内部模型；
- Projection rebuild 必须记录使用的 schema registry revision；
- 未知事件类型不得静默丢弃，必须进入 dead-letter/unknown 计数并阻止受影响投影宣称最新。

---

## 6. Durable Event 与 Realtime Event 策略

### 6.1 必须持久化的事实

以下事实默认写入 Domain Event：

- Session、Run、Turn、Workflow、Agent 的状态转移；
- Provider 请求开始/完成/失败、usage 和模型标识摘要；
- Tool/MCP/Hook/Plugin 调用及其结果、超时、取消和外部副作用状态；
- Permission 判定、Approval 请求/批准/拒绝/过期和撤销；
- Rule Check、Verification Gate、Waiver、Skip Spec 和验证结论；
- Checkpoint 创建/恢复/失效；
- Workspace Snapshot、Write Claim、文件变化摘要；
- Credential 使用、撤销、轮换、Data Egress 判定和 Redaction；
- Recovery、Migration、Backup、Restore、Purge、Projection Rebuild 等维护事实；
- Incident 创建、确认、升级、缓解和关闭。

### 6.2 仅实时传输的高频数据

以下数据默认是 Realtime Event，可按需采样或合并：

- token delta、逐字输出；
- 高频 stdout/stderr 片段；
- Agent 进度百分比的连续变化；
- 连接心跳、鼠标移动、终端重绘；
- 尚未形成业务结论的中间 Trace span。

当实时流中断时，客户端必须通过 Query 获取最终状态。实时事件不承担“最后一次状态”的唯一职责。

### 6.3 降采样规则

- 进度事件以时间窗口和百分比变化双重限流；
- token delta 可合并为 50～200ms 批次；
- stdout/stderr 默认保留摘要，原始输出进入受策略保护的 Blob；
- 相同错误在单一 operation 内按 fingerprint 去重；
- 批量调用按批次记录统计，单项失败仍必须保留可审计索引。

---

## 7. Projection 与一致性模型

### 7.1 投影原则

Projection 是 Domain Event 的派生读模型，允许删除后重建。Projection 不能产生外部副作用，不能修改 Domain Event，不能把“修正显示”写回业务表。

每个投影注册：

```rust
pub struct ProjectionRegistration {
    pub projection_id: String,
    pub revision: String,
    pub min_event_schema: u32,
    pub subscriptions: Vec<EventPattern>,
    pub rebuild_strategy: RebuildStrategy,
    pub privacy_class: DataClass,
}
```

### 7.2 一致性等级

Query 支持三种一致性：

| 等级 | 语义 | 适用 |
|---|---|---|
| `eventual` | 返回当前已构建投影 | 历史趋势、低延迟列表 |
| `at_least_seq` | 等待投影达到指定 `global_seq` | 实时面板、操作后刷新 |
| `strong_current` | 在同一 Core 读事务中读取最新状态 | 维护命令前确认、权限/审批 |

响应必须返回：

```json
{
  "as_of_global_seq": 1842,
  "projection_revision": "observability-v3",
  "consistency": "at_least_seq",
  "is_complete": true,
  "items": []
}
```

### 7.3 游标和缺口恢复

客户端保存 `last_seen_global_seq` 和各订阅的 `subscription_cursor`。重新连接时：

```text
connect(last_seen_seq)
  → authenticate + capability negotiation
  → receive snapshot/as_of_seq
  → replay durable events > last_seen_seq
  → receive live events
```

如果服务器无法提供游标窗口、客户端检测到 seq gap、事件 schema 未知或投影 revision 不兼容，必须发送 `projection.refresh_required`，客户端重新 Query，不得拼接不完整状态。

### 7.4 投影延迟

Projection lag 是 `event_store_head_seq - projection_applied_seq`，分为：

- 正常：小于 100 个事件且小于 1 秒；
- 警告：连续 30 秒超过阈值；
- 严重：超过配置上限、发生 dead-letter 或超过客户端可接受的 `at_least_seq` 等待时间。

投影落后时 UI 显示明确的 `as_of_global_seq` 和“数据可能滞后”，不得显示为实时准确。

---

## 8. Event Bus、Outbox 与订阅

### 8.1 Event Bus 主题

内部主题以事件类型和项目范围过滤：

```text
project/{project_id}/events/{event_type}
project/{project_id}/audit
project/{project_id}/maintenance
project/{project_id}/realtime
system/health
system/incidents
```

主题只是路由概念，不改变 Domain Event 的全局排序和权限模型。

### 8.2 订阅请求

```json
{
  "subscription_id": "sub_01...",
  "project_id": "prj_01...",
  "from_global_seq": 1840,
  "event_types": ["run.*", "tool.*", "mcp.*"],
  "include_realtime": true,
  "max_batch_events": 100,
  "max_batch_bytes": 1048576,
  "projection_revision": "observability-v3"
}
```

服务端必须再次按 actor、client、project、capability 和 redaction policy 过滤；客户端传入的 event type 不是授权声明。

### 8.3 背压

订阅者有独立的队列、字节上限和发送超时：

1. 先合并可丢失的 realtime 进度；
2. 再暂停低优先级日志类事件；
3. durable event 不能静默丢失，发送失败转由 cursor replay；
4. 队列持续超限则断开连接并返回 `resync_required`；
5. 任何客户端断线不得阻塞 Core 写入。

### 8.4 广播失败

Event Store commit 成功而广播失败时：

- 业务事实保持成功；
- Outbox 保留待发送记录；
- 指标增加 `outbox_lag` 和 `broadcast_failures_total`；
- Alert evaluator 可创建运维 Incident；
- 客户端下一次连接通过 cursor replay 获取事实。

---

## 9. 客户端控制面与面板架构

### 9.1 面板统一协议

所有面板 Query 都返回：

- `project_id`；
- `as_of_global_seq`；
- `projection_revision`；
- `generated_at`；
- `consistency`；
- `redaction_level`；
- `items` 或 `summary`；
- `next_cursor`；
- `warnings`（如滞后、采样、权限裁剪）。

面板不得轮询整张事件表。首屏使用 Projection Query，后续使用事件推送和增量刷新。

### 9.2 总览与会话时间线

总览面板展示：

- Core、数据库、Event Store、Projector、Outbox、Provider、MCP 和 Credential Store 健康状态；
- 当前活跃 Session、Run、Agent、Tool operation；
- 当前告警、Incident 和维护任务；
- CPU、内存、磁盘、WAL、队列、事件追加延迟和投影延迟；
- 最近一段时间的成功率、失败率、取消率、跳过率和质量审计结果。

会话时间线以 `global_seq` 和时间组合排序，区分 Domain、Audit 和 Realtime 记录，支持按 Run、Turn、Actor、Tool、MCP、Rule、Gate 过滤。

### 9.3 Skill 面板

最小字段：

| 字段 | 说明 |
|---|---|
| `skill_id/name/version/digest` | 稳定身份和内容版本 |
| `source/layer` | builtin/project/user/extension |
| `load_status` | loaded/skipped/failed/blocked |
| `invocation_count` | 当前 Session/Run 统计 |
| `token_usage` | 输入、输出、缓存摘要 |
| `last_invocation_at` | 最近使用时间 |
| `policy_decision` | allow/deny/approval_required |
| `redaction_summary` | 是否发生脱敏、数量和分类 |

### 9.4 MCP 面板

展示：

- Server identity、transport、配置摘要、连接状态和 registry generation；
- 工具列表、声明能力、当前启停状态；
- 调用次数、成功率、延迟分位数、超时/取消/拒绝数；
- 参数安全摘要、结果摘要、taint、redaction 和外部副作用状态；
- Credential 使用引用和 Data Egress 判定结果，但不展示 Secret；
- 连接断开原因、重连退避和最近健康检查。

### 9.5 SubAgent 面板

展示 Agent/Node 的：

- 生命周期和父子关系；
- 当前 Task、状态、重试次数、截止时间和取消原因；
- Token、Tool、文件变化、Write Claim 和 Checkpoint 摘要；
- 当前等待原因（依赖、审批、资源、外部调用）；
- 失败分类和可重试性；
- 结束后的结果、验证状态和是否产生未解决 Incident。

### 9.6 Memory 面板

展示（对应需求文档 §3.4.4）：

- 记忆文件列表：路径、摘要、类型、创建时间、最后召回时间；
- 当前会话中被召回的记忆，高亮标记并给出**召回原因**（命中关键词、BM25 分值、来源 query）；
- 召回统计：召回次数、命中率、平均分值、被忽略的低分候选数；
- 记忆的编辑/删除/导出入口，以及外部文件修改导致的重建索引状态；
- FTS5 搜索框与查询结果预览。

投影来源为 `memories` 与 `recalls`（字段 `path/reason/score/last_recalled`）。Memory 的权威源是 Markdown 文件，FTS5 索引可删除重建，因此面板必须显示索引状态（`ready` / `rebuilding` / `stale`），避免用户把索引滞后误读为记忆丢失。

> ADR-0033（跨文档一致性审查）：原文只有 Skill、MCP、SubAgent 三个面板，缺需求文档 §3.4.4 与系统总体架构 §12 规定的 Memory 面板（四个面板投影之一）。

### 9.7 工具、规则与安全面板

工具面板展示调用时间线、耗时、状态、参数安全摘要、输出摘要、权限判定、Approval、taint、数据外发和文件影响。规则面板展示 Rule Check、Verification Gate、Waiver、Skip Spec、失败证据和重跑入口。安全面板展示权限拒绝、异常认证、Credential 使用、脱敏和可疑外发，但不暴露原文。

### 9.8 运维面板

运维面板分为只读健康视图和受保护操作区：

- 数据库/WAL/磁盘/Blob/Snapshot 水位；
- 事件追加、投影、Outbox、FTS 和 GC 状态；
- 备份、恢复、迁移、完整性检查、投影重建和隐私清除；
- 当前维护锁、运行人、开始时间、进度、预计剩余时间和取消状态；
- 健康检查、告警、Incident 和支持包；
- 所有操作都必须显示影响范围、风险、前置条件和审计编号。

---

## 10. Query API 设计

### 10.1 服务划分

```text
EventQueryService
EventSubscriptionService
AuditQueryService
ObservabilityQueryService
MaintenanceCommandService
SupportBundleService
IncidentService
```

这些服务位于 Core 内部 Port；Web Gateway 和 Desktop/TUI Adapter 仅做协议适配。

### 10.2 REST 映射

```text
GET  /api/v1/events
GET  /api/v1/events/{event_id}
GET  /api/v1/events/stream
GET  /api/v1/audit/records
GET  /api/v1/observability/overview
GET  /api/v1/panels/skills
GET  /api/v1/panels/mcp
GET  /api/v1/panels/subagents
GET  /api/v1/observability/metrics
GET  /api/v1/observability/traces/{trace_id}
GET  /api/v1/alerts
GET  /api/v1/incidents
GET  /api/v1/maintenance/runs
POST /api/v1/maintenance/runs
POST /api/v1/maintenance/runs/{run_id}:cancel
POST /api/v1/incidents/{incident_id}:ack
POST /api/v1/incidents/{incident_id}:resolve
POST /api/v1/support-bundles
GET  /api/v1/support-bundles/{bundle_id}
```

### 10.3 Event Query

支持的过滤条件：

- `project_id`、`session_id`、`run_id`、`turn_id`；
- `event_type` 前缀或注册模式；
- `actor_id`、`operation_id`、`correlation_id`；
- `global_seq_from/to`、`occurred_at_from/to`；
- `outcome`、`redaction_level`、`taint`；
- `limit`、`cursor`、`max_bytes`。

服务端先按授权裁剪字段，再分页；不可通过 `limit` 绕过最大响应字节限制。完整大对象只能返回一次性 Blob capability，且 capability 具有 scope、过期时间和下载审计。

### 10.4 事件详情的安全视图

事件详情包含：

```json
{
  "event_id": "evt_...",
  "event_type": "tool.call_finished",
  "safe_payload": {
    "tool_name": "git.diff",
    "duration_ms": 480,
    "result_status": "success",
    "output_summary": "42 lines changed",
    "redaction": {"applied": true, "count": 1}
  },
  "payload_capabilities": [],
  "audit_ref": "audit_...",
  "as_of_global_seq": 1842
}
```

`safe_payload` 是服务端生成的，不允许客户端依据原始参数自行拼装 `approval_summary`、风险级别或审计结论。

---

## 11. 审计模型与责任链

### 11.1 审计记录结构

```rust
pub struct AuditRecord {
    pub audit_id: AuditId,
    pub project_id: ProjectId,
    pub occurred_at: Timestamp,
    pub recorded_at: Timestamp,
    pub actor: AuditActor,
    pub client: AuditClient,
    pub action: String,
    pub target: AuditTarget,
    pub reason_code: String,
    pub policy_revision: Option<String>,
    pub approval_ref: Option<ApprovalId>,
    pub correlation_id: CorrelationId,
    pub causation_event_id: Option<EventId>,
    pub outcome: AuditOutcome,
    pub evidence: SafeEvidence,
    pub redaction_level: RedactionLevel,
    pub retention_class: RetentionClass,
}
```

`actor` 区分用户、服务身份、系统恢复器、定时任务和未知外部主体；`client` 记录 TUI、Desktop、Web、CLI、内部模块及版本；`target` 使用实体类型和稳定 ID，不使用原始路径、完整 URL 或 Secret。

### 11.2 审计类别

| 类别 | 典型动作 | 默认等级 |
|---|---|---|
| `auth` | 登录、握手、会话建立、失败 | D1 |
| `authorization` | 权限允许、拒绝、审批、撤销 | D0+D1 |
| `data_access` | Credential 使用、Blob 下载、敏感文件读取 | D0+D1 |
| `data_egress` | 向 Provider/MCP/网络目的地发送数据 | D0+D1 |
| `execution` | Tool、Shell、Plugin、Hook、MCP 调用 | D0+D1 |
| `quality` | Rule、Gate、Skip Spec、Waiver | D0+D1 |
| `workspace` | Snapshot、Write Claim、文件变更 | D0+D1 |
| `operator` | 维护、恢复、备份、导出、清除 | D0+D1 |
| `extension` | Skill/MCP/Plugin 注册、启停、升级 | D0+D1 |
| `privacy` | 脱敏、删除、保留策略和隐私清除 | D0+D1 |

### 11.3 必须审计的控制点

下列动作不论成功或失败都要产生审计：

- 认证失败、短期握手 token 拒绝、越权 Query；
- 权限判定、Approval 创建/批准/拒绝/过期/撤销；
- 跳过 Spec、绕过 Gate、使用 Waiver、修改策略；
- 读取或使用 Credential、Credential 轮换和撤销；
- Data Egress 允许、拒绝、脱敏后允许和疑似泄漏阻断；
- Tool/MCP/Hook/Plugin/Shell 执行及外部副作用未知；
- Snapshot、Checkpoint、Workspace Claim、恢复和强制释放；
- 迁移、备份、恢复、完整性检查、投影重建、GC 和隐私清除；
- 支持诊断包生成、下载、导出和分享；
- 扩展安装、注册、禁用、升级、回滚和 registry generation 变化。

### 11.4 审计不可抵赖边界

本地单机模式下，审计记录防止普通业务模块随意修改，但不能声称抵抗拥有数据库文件和操作系统管理员权限的攻击者。若用户启用归档签名，则支持：

1. 按项目和时间窗口冻结事件/审计范围；
2. 生成 canonical manifest、事件数量、seq 范围、哈希根；
3. 使用外部签名器或 OS keyring 中的签名密钥；
4. 记录签名算法、密钥引用和签名时间；
5. 下载归档包时记录审计。

---

## 12. 日志设计

### 12.1 单行位置式日志格式

Apex 文件日志使用与 Spring Boot Console Log Pattern 对齐的位置式文本格式。日志文件不写 ANSI 颜色控制码；`%clr(...)` 只在未来控制台 sink 启用颜色时生效。

```text
%d{yyyy-MM-dd HH:mm:ss.SSS} %5p ${PID} --[traceId: %X{traceId}]-- [%15.15t] %-40.40logger{39} %M:%L : %m %wEx
```

实际输出示例：

```text
2026-08-09 10:00:22.523  WARN 33744 --[traceId: 550e8400-e29b-41d4-a716-446655440000]-- [tokio-rt-worker] apex_runtime                             run.rs:142 : projection lag exceeded
```

映射规则如下：

| Pattern 段 | Apex 实现 |
|---|---|
| `%d{yyyy-MM-dd HH:mm:ss.SSS}` | 进程所在时区的本地时间，毫秒精度 |
| `%5p` | 右对齐、宽度 5 的日志级别 |
| `${PID}` | 操作系统进程 PID |
| `%X{traceId}` | 日志事件的 `traceId` 或 `trace_id` 字段；缺失时使用任务上下文生成的链路 UUID |
| `%15.15t` | OS 线程名；无名称时显示线程 ID；超过 15 字符时保留末尾 15 字符 |
| `%-40.40logger{39}` | `tracing` target，左对齐、宽度 40；超过宽度时保留末尾 |
| `%M:%L` | Rust 不提供 JVM 式稳定方法名，因此映射为真实源码文件名和代码行号 |
| `%m` | TRACE/DEBUG 级别先输出 `messageCode`、`runId`、协程身份和业务诊断字段，再输出主消息；INFO/WARN/ERROR 级别只输出主消息 |
| `%wEx` | TRACE/DEBUG 级别将 `exception`、`error` 或 `error.message` 字段转义后放在主消息之前；INFO/WARN/ERROR 级别不输出该诊断上下文 |

`messageCode` 是稳定可检索编号，消息文本不能成为机器解析的唯一依据。日志不是 `key=value` 格式，也不是 JSON；TRACE/DEBUG 级别使用方括号前缀承载诊断上下文，随后才是主消息；INFO/WARN/ERROR 级别不输出这些方括号诊断字段。日志中的换行、回车、制表符和控制字符必须转义，确保一条事件只占一个物理行。

### 12.1.1 v0.1 进程执行诊断信息

v0.1 首先实现本地文件日志模块，每次进程运行生成一个独立的单行文本文件。文件名使用 `<component>-<run_id>.log`，`run_id` 在进程内唯一并贯穿一次启动到退出的全部日志。

TRACE/DEBUG 级别除主 Pattern 的时间、级别、PID、任务链路上下文、线程、logger 和源码位置外，主消息之前必须输出以下诊断上下文；INFO/WARN/ERROR 级别隐藏以下诊断上下文：

- `[runId: ...]`：本次进程运行标识；
- `[messageCode: ...]`：稳定事件编码；
- `[coroutine: <task-id>/<task-name>]`：显式注册的 Tokio 协程身份；
- `[spans: ...]`：当前 `tracing` span 路径；
- `[progressCurrent: ...]`、`[progressTotal: ...]` 等业务诊断字段；
- 敏感字段只允许输出 `[REDACTED]`。

默认采用有界队列和独立写线程；默认每条记录 flush，优先保证查询执行过程时日志不滞留。队列满时默认阻塞生产者，只有明确配置 `DropNewest` 才允许丢弃，并必须累计 `dropped_records`。日志库错误返回给启动方，不能通过 panic 终止业务进程。

协程身份不依赖 Tokio 未承诺的内部 task ID，而由运行时通过 `TaskContext`/`spawn_logged` 显式注册；因此所有需要查询“哪个协程执行”的异步工作都必须从统一包装器创建。日志只用于诊断，SQLite Event Store 仍是业务事实和恢复依据。

Pattern 中的源码路径是 Rust 编译器 callsite 的**代码位置元数据**，用于定位“哪一行代码产生日志”，与工具参数、项目文件路径、用户数据路径不是同一类字段。用户/项目路径仍必须按本节安全规则记录为分类、相对路径或 digest，不得因为 source callsite 要求而放宽敏感路径治理。
### 12.2 日志级别

- `ERROR`：需要人工或自动处置，必须包含 error code、操作引用和下一步建议；
- `WARN`：违反软阈值、重试、降级、采样或数据不完整；
- `INFO`：生命周期和重要里程碑；
- `DEBUG`：开发诊断，生产默认关闭或采样；
- `TRACE`：短期实验级信息，禁止默认写盘。

### 12.3 日志安全规则

禁止记录：

- API key、OAuth token、Cookie、私钥、密码和完整 Authorization；
- Prompt、外部响应、环境变量值和敏感文件正文；
- 未脱敏的工具参数、完整 URL 查询串和命令行 Secret；
- 通过拼接形成的巨大字符串和无法控制大小的 stdout。

允许记录：

- 工具名、Provider/MCP identity、参数 schema 摘要、字段名集合；
- 数量、字节数、耗时、状态、错误码、fingerprint、内容 digest；
- 路径分类（workspace-relative、outside-workspace、sensitive-path），而非未经裁剪的绝对路径；
- `redaction_applied`、`redaction_count`、`taint` 和 `external_effect_state`。

### 12.4 本地日志与远程 telemetry

Apex 默认只写本地受权限保护的诊断日志。外部 telemetry 默认关闭，开启时必须：

- 由项目级或实例级明确配置；
- 显示目的地、数据分类、保留期限和开关状态；
- 经过 `TelemetryExportPolicy` 和 Secret Scanner；
- 采用批量、限速、失败丢弃或本地缓冲策略，不阻塞主业务；
- 产生 `observability.telemetry.configured` 和 `data.egress.evaluated` 审计事实。

---

## 13. Metrics 设计

### 13.1 指标类型

- Counter：累计请求、错误、拒绝、重试、脱敏和丢弃数量；
- Gauge：当前连接数、队列长度、磁盘使用、活跃 Run、投影 lag；
- Histogram：启动、事件提交、Query、Tool、MCP、Provider、备份和恢复时延；
- Info：版本、构建 digest、数据库 schema revision、projection revision。

### 13.2 命名规范

```text
apex_core_commands_total
apex_core_command_duration_ms
apex_event_append_total
apex_event_append_duration_ms
apex_event_bus_realtime_dropped_total
apex_projection_lag_events
apex_projection_apply_duration_ms
apex_outbox_pending_events
apex_tool_calls_total
apex_mcp_calls_total
apex_provider_requests_total
apex_permission_decisions_total
apex_redactions_total
apex_data_egress_denied_total
apex_db_writer_queue_depth
apex_db_busy_total
apex_db_size_bytes
apex_wal_size_bytes
apex_disk_free_bytes
apex_maintenance_runs_total
apex_incidents_open
```

### 13.3 Label 白名单

允许的 label 只能来自有限集合：

```text
component, command_type, event_family, outcome, error_code,
provider_kind, tool_namespace, mcp_transport, projection_id,
maintenance_kind, severity, client_kind, retryable
```

禁止将 `prompt_hash`、完整 `tool_name`（若来源可任意扩张）、路径、用户 ID、Credential ID、URL host 未经注册或任意输入直接作为 label。高基数实体使用日志字段或聚合 Top-N，而不是指标 label。

### 13.4 业务质量指标

为满足会话统计和质量审计，Core 提供以下聚合：

- Turn 完成率、失败率、取消率和超时率；
- Tool/MCP/Provider 成功率、重试率、拒绝率；
- Rule Check、Verification Gate 通过率和失败类型分布；
- Spec skip rate、Waiver rate 和未验证交付率；
- Approval 等待时间、拒绝率和过期率；
- 文件修改后验证覆盖率、回滚率和冲突率；
- Memory recall 命中率和人工采纳率；
- Token、调用次数、估算成本和上下文压缩次数。

这些指标展示的是聚合结果；原始责任链仍由事件和审计记录提供。

### 13.5 SLO 建议

| SLO | 目标基线 | 备注 |
|---|---:|---|
| Command commit latency | p95 < 250ms | 不含外部 Provider 时间 |
| Durable event availability | 99.9% | Core 可写且未进入只读保护 |
| Panel refresh latency | p95 < 1s | 需求目标，Projection 正常时 |
| Projection lag | p95 < 1s | 低负载基线 |
| Outbox delivery lag | p95 < 5s | 本地订阅 |
| Reconnect replay success | > 99.9% | 无数据库损坏时 |
| Audit completeness | 100% | 对已定义控制点 |
| Secret leakage in telemetry | 0 | 任何已知路径均 fail-closed |

阈值必须在配置中版本化，修改阈值本身产生运维审计。

---

## 14. Trace 设计

### 14.1 Span 层次

```text
command.handle
  ├── policy.evaluate
  ├── context.build
  ├── runtime.schedule
  ├── provider.request
  │     ├── credential.acquire
  │     └── result.normalize
  ├── tool.execute
  │     ├── permission.evaluate
  │     ├── subprocess.spawn / mcp.call
  │     └── output.scan
  ├── checkpoint.create
  └── event.commit
```

维护任务使用独立根 Span：

```text
maintenance.run
  ├── maintenance.preflight
  ├── backup.snapshot
  ├── projection.rebuild
  ├── integrity.check
  └── maintenance.finalize
```

### 14.2 Span 属性

强制关联：`project_id/session_id/run_id/turn_id/operation_id/event_id`（存在时）。允许记录耗时、状态、重试次数、错误码、数据字节数、工具/Provider/MCP 的注册 identity 和策略 revision。

不允许记录 Prompt、Secret、完整参数、响应原文、绝对敏感路径和未经批准的外部地址。Trace error event 只保留安全错误分类和内部诊断引用。

### 14.3 采样

- Command、Approval、Credential、Data Egress、Maintenance 和 Incident 相关 Span 默认保留；
- 高频 token、心跳和成功健康检查可采样；
- 错误、超时、重试和未知副作用触发尾采样保留；
- 采样策略不影响 Domain Event 和 Audit Record 的完整性。

---

## 15. Alert、Incident 与处置闭环

### 15.1 Alert 与 Incident 的区别

`Alert` 是规则或异常检测产生的信号，可自动恢复、合并和抑制；`Incident` 是需要跟踪责任、处置步骤和最终结论的工作对象。Alert 不一定创建 Incident，但安全、数据完整性和持久化故障必须创建或关联 Incident。

```text
metric/event condition
        │
        ▼
      Alert
        ├── deduplicate / suppress / route
        ├── auto-resolve
        └── create or update Incident
                         │
                         ▼
             acknowledge → mitigate → resolve
```

### 15.2 告警规则

规则以声明式 JSON/YAML 存储并版本化：

```yaml
rule_id: projection_lag_critical
source: metric
expression: apex_projection_lag_events > 10000 for 60s
severity: critical
scope: project
cooldown_seconds: 300
action: create_incident
redaction_level: safe_view
```

告警动作只允许：创建/更新 Incident、发送本地通知、暂停某类非关键任务、请求人工确认。禁止规则直接调用 Shell、Tool、MCP 或 Provider。

### 15.3 关键告警

默认规则包括：

- 数据库 `FULL`、`IOERR`、`CORRUPT`、`BUSY` 持续超阈值；
- 事件写入失败、事件哈希链断裂、序列不连续；
- Projection dead-letter、projection lag、Outbox lag；
- 磁盘接近保护水位、WAL 无法 checkpoint、Blob 超额；
- Provider/MCP 连续超时、认证失败、断线重连失败；
- Credential Store 不可用、Credential 使用被大量拒绝或撤销后仍请求；
- 未知外部副作用、恢复过程中存在 active operation；
- Secret scanner 检出或 telemetry export 被阻断；
- 规则跳过率、验证失败率、人工 Waiver 率异常升高；
- 维护任务超时、锁丢失、进度停滞或校验不匹配。

### 15.4 Incident 状态机

```text
open → acknowledged → investigating → mitigated → resolved
  │          │              │             │
  └──────────┴──────────────┴─────────────┴→ reopened
open/acknowledged/investigating → suppressed（仅授权）
```

状态转移必须由 Incident Command 产生 Domain Event 和 Audit Record。`resolved` 必须带 `resolution_code`、证据引用和是否需要后续行动；重新出现的同一 fingerprint 可以 reopen 原 Incident 或创建关联 Incident。

### 15.5 告警去重

去重键由 `rule_id + project_id + normalized_target + error_fingerprint` 组成。不得使用完整参数或 Secret 参与 fingerprint；fingerprint 只使用 canonical safe fields。

---

## 16. 健康检查与运行时状态

### 16.1 Health Check 层级

```text
liveness   Core 进程是否可响应
readiness  能否接受业务命令
degraded   某依赖异常但仍可执行部分能力
safe_mode  只读或限制性运行
failed     不能提供核心服务
```

### 16.2 组件健康

健康投影至少覆盖：

- Core 主循环、Command Router、Event Store；
- SQLite Writer、WAL、Checkpoint、FTS；
- Projection Worker、Event Bus、Outbox；
- Agent Supervisor、DAG Scheduler、子进程回收器；
- Provider Adapter、MCP Registry/Connection、Tool Gateway；
- Credential Store/Broker；
- Workspace Claim、Snapshot、Blob Store；
- Rule Engine、Verification Gate、Memory；
- 磁盘、配额、备份和恢复可用性。

每项健康结果包括 `status`、`checked_at`、`latency_ms`、`error_code`、`degraded_capabilities` 和安全摘要。健康检查失败不得把底层异常全文返回给普通客户端。

### 16.3 Readiness 规则

Apex 在以下条件下拒绝新建需要写入的 Run，但允许查询和恢复：

- Schema migration 未完成；
- Event Store integrity check 未通过；
- 磁盘低于 `stop_writes` 水位；
- SQLite writer 永久失败或数据库只读；
- Credential Store 被策略标记为不可用且任务需要 Credential；
- Projection 不完整不影响写入，但强依赖实时面板的操作必须提示滞后；
- 恢复过程正在处理未知外部副作用。

---

## 17. 维护控制面模型

### 17.1 MaintenanceRun

沿用 SQLite 设计中的 `maintenance_runs`，维护任务是受授权、可观察、可取消和可恢复的命令，而不是脚本直接改文件：

```rust
pub struct MaintenanceRun {
    pub maintenance_run_id: MaintenanceRunId,
    pub project_id: Option<ProjectId>,
    pub kind: MaintenanceKind,
    pub requested_by: ActorId,
    pub client_id: ClientId,
    pub state: MaintenanceState,
    pub target: SafeTarget,
    pub progress: Progress,
    pub lock_scope: LockScope,
    pub started_at: Option<Timestamp>,
    pub finished_at: Option<Timestamp>,
    pub report_ref: Option<BlobRef>,
    pub error_code: Option<String>,
}
```

### 17.2 任务类型

```text
migration
backfill
backup
restore
quick_check
integrity_check
projection_rebuild
fts_rebuild
gc
privacy_purge
health_repair
support_bundle
```

### 17.3 状态机

```text
requested → preflight → queued → running → finalizing → completed
     │          │          │        │            │
     ├──────────┴──────────┴────────┴────────────┴→ failed
     └────────────────────────────────────────────→ cancelled
requested/queued → blocked
```

- `preflight` 检查版本、权限、磁盘、锁和影响范围；
- `blocked` 表示等待条件，不是失败；
- `running` 周期性提交进度事实；
- `finalizing` 写报告、校验摘要和释放锁；
- 任何终态不得回到运行态；需要再次执行必须创建新任务并关联旧任务。

### 17.4 维护锁

维护锁按照作用域分层：

```text
db_write
project_events
projection:{id}
fts
blob_gc
workspace:{id}
credential_metadata
privacy:{project_id}
```

锁必须具备 owner、lease、fence token 和过期处理。锁丢失时任务进入 `failed` 或 `blocked`，不得继续写入。高风险任务（restore、privacy_purge、强制 GC）需要显式确认和二次身份验证。

### 17.5 命令前置确认

客户端提交维护命令时，Core 返回预检：

```json
{
  "maintenance_kind": "projection_rebuild",
  "impact": "panel_queries_may_be_stale",
  "requires_quiesce": false,
  "estimated_bytes": 52428800,
  "required_capabilities": ["maintenance.projection_rebuild.v1"],
  "warnings": [],
  "confirmation_token": "short_lived_one_time_token"
}
```

确认 token 绑定 actor、目标、参数摘要和短期过期时间，不能被改写参数后复用。

---

## 18. 维护任务详细策略

### 18.1 Migration

迁移顺序：备份/确认 → 获取 `db_write` 锁 → 检查 schema revision → 单步迁移 → quick check → 更新 schema metadata → 发布 `maintenance.migration.completed`。迁移必须可检测、可重入或有明确 rollback/restore 路径。

### 18.2 Backup

备份必须记录：

- 数据库 schema revision、事件 seq 范围和 projection revision；
- SQLite checkpoint/WAL 状态；
- Blob、Snapshot 和 Credential metadata 的清单；
- 加密、压缩、校验和、创建者、策略版本及保留期限；
- 是否包含敏感数据以及恢复所需的外部 Credential Store 引用。

备份不应包含未授权 Secret Store 原文。恢复后 Credential Store 通过引用重新绑定或进入不可用状态，不能凭备份里的普通 JSON 自动复活 Secret。

### 18.3 Restore

Restore 是破坏性高风险任务：

1. 预检目标版本、备份签名和 schema；
2. 停止受影响的写入能力；
3. 新建临时数据库或隔离目录；
4. 导入并校验事件序列、哈希链、Blob 清单；
5. 重建 Projection、FTS 和安全索引；
6. 对未知外部操作标记 `interrupted/unknown`；
7. 恢复 Workspace/Claim 时重新核对 fence token；
8. 切换数据库并发布 `recovery.completed`；
9. 旧数据库保留为回滚证据，直到保留期结束。

### 18.4 Projection Rebuild

投影重建使用临时表或临时数据库，以 `last_event_seq` 分批读取事件。重建期间旧投影继续服务查询并标记 revision；切换时使用原子 rename 或事务更新 registry。任何未知事件、upcaster 错误或安全扫描错误都必须使任务失败或进入 blocked，不得产出“看似完整”的错误投影。

### 18.5 FTS Rebuild

FTS 重建只处理允许被搜索的安全文本。Prompt、Secret、敏感文件正文和被策略禁止索引的外部响应不会进入 FTS。重建前后比较文档数量、token 数和抽样查询结果，差异写入 maintenance report。

### 18.6 GC 与隐私清除

GC 只删除没有引用且超过保留期的 Blob、Trace、Log、Snapshot 和临时目录。Privacy Purge 是更高等级任务：

- 需要明确主体/项目/时间范围和法规或用户请求理由；
- 记录删除范围、计数、失败项和不可删除原因；
- Domain Event 本身通常不物理改写，而是通过 tombstone、payload purge 或加密密钥销毁实现；
- 审计必须保留“发生过清除”这一最小事实，但不得保留被清除的内容；
- 备份、缓存、诊断包和外部 telemetry 的清除责任必须分别报告。

### 18.7 Quick Check 与 Integrity Check

`quick_check` 用于启动和低成本健康检查；`integrity_check` 用于人工或故障处置。检查内容包括 SQLite integrity、事件 seq、哈希链、Projection cursor、Outbox 引用、Blob 引用、Snapshot manifest 和 FTS 一致性。

---

## 19. SQLite 数据模型补充

### 19.1 设计原则

现有 `domain_events`、`event_consumers`、`outbox`、`projection_registry`、`projection_cursors`、`metric_samples` 和 `maintenance_runs` 作为基础表继续使用。本节只增加 Observability、Audit、Alert、Incident 和支持包所需的表；不把高频日志和 Trace 强行塞进业务事件表。

### 19.2 审计表

```sql
CREATE TABLE IF NOT EXISTS audit_records (
    audit_id              TEXT PRIMARY KEY,
    project_id            TEXT,
    occurred_at           TEXT NOT NULL,
    recorded_at           TEXT NOT NULL,
    actor_type            TEXT NOT NULL,
    actor_id              TEXT NOT NULL,
    client_kind           TEXT NOT NULL,
    client_version        TEXT,
    action                TEXT NOT NULL,
    target_type           TEXT NOT NULL,
    target_id             TEXT,
    reason_code           TEXT NOT NULL,
    policy_revision       TEXT,
    approval_id           TEXT,
    correlation_id        TEXT NOT NULL,
    causation_event_id    TEXT,
    outcome               TEXT NOT NULL,
    evidence_json         TEXT NOT NULL,
    redaction_level       TEXT NOT NULL,
    retention_class       TEXT NOT NULL,
    event_id              TEXT,
    created_at            TEXT NOT NULL,
    FOREIGN KEY (event_id) REFERENCES domain_events(event_id)
);

CREATE INDEX IF NOT EXISTS idx_audit_project_time
    ON audit_records(project_id, occurred_at, audit_id);
CREATE INDEX IF NOT EXISTS idx_audit_actor_time
    ON audit_records(actor_id, occurred_at);
CREATE INDEX IF NOT EXISTS idx_audit_target_time
    ON audit_records(target_type, target_id, occurred_at);
```

### Metric 采样表

`metric_samples` 是系统总体架构 §10.2 声明的 observability 表族成员，此前无 DDL 定义，现补齐：

```sql
CREATE TABLE IF NOT EXISTS metric_samples (
    sample_id      TEXT PRIMARY KEY,
    project_id     TEXT,                       -- 全局指标可为 NULL
    metric_name    TEXT NOT NULL,
    metric_kind    TEXT NOT NULL
        CHECK (metric_kind IN ('counter','gauge','histogram')),
    -- 标签集合的 canonical JSON；仅允许低基数、已登记的标签键
    labels_json    TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(labels_json)),
    labels_digest  TEXT NOT NULL,               -- sha256:<hex>，用于聚合去重
    value_num      REAL,                        -- counter/gauge
    histogram_json TEXT CHECK (histogram_json IS NULL OR json_valid(histogram_json)),
    window_start_us INTEGER NOT NULL,
    window_end_us   INTEGER NOT NULL,
    created_at_us   INTEGER NOT NULL,
    CHECK (window_end_us >= window_start_us),
    CHECK ((metric_kind = 'histogram') = (histogram_json IS NOT NULL)),
    CHECK ((metric_kind IN ('counter','gauge')) = (value_num IS NOT NULL))
);

CREATE INDEX IF NOT EXISTS idx_metric_name_window
    ON metric_samples(metric_name, window_start_us DESC);
CREATE INDEX IF NOT EXISTS idx_metric_project_window
    ON metric_samples(project_id, metric_name, window_start_us DESC);
CREATE UNIQUE INDEX IF NOT EXISTS uq_metric_sample_window
    ON metric_samples(metric_name, labels_digest, window_start_us);
```

约束说明：

- 指标是 D4 等级数据，**不是业务事实源**，可按窗口聚合与过期删除，不参与恢复；
- 标签键必须来自已登记的低基数集合。`session_id`、`run_id`、`tool_call_id`、文件路径、用户输入等高基数值**不得**作为标签，只能进入 Trace 或事件；违规标签由采集侧拒绝并计入 `METRIC_LABEL_REJECTED`；
- 标签值不得包含 `sensitive` 及以上分级的数据（见 §2.3）；
- `uq_metric_sample_window` 保证同一指标同一标签集在同一窗口只有一行，使重复上报幂等。

> ADR-0011（跨文档一致性审查）：`metric_samples` 在系统总体架构 §10.2 中被列为 observability 表族成员，但全库无 DDL 定义。现补齐，并随 `0014_observability_ops.sql` 迁移。

`evidence_json` 只能保存 SafeEvidence，例如字段 digest、数量、状态和策略结果；禁止保存 Secret 和未经扫描的原文。

### 19.3 指标表

```sql
CREATE TABLE IF NOT EXISTS metric_samples (
    sample_id             INTEGER PRIMARY KEY AUTOINCREMENT,
    metric_name           TEXT NOT NULL,
    recorded_at           TEXT NOT NULL,
    project_id            TEXT,
    value                 REAL NOT NULL,
    aggregation           TEXT NOT NULL,
    labels_json           TEXT NOT NULL,
    window_start          TEXT,
    window_end            TEXT,
    source_revision       TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_metric_name_time
    ON metric_samples(metric_name, recorded_at);
```

生产实现应按时间窗口或保留策略聚合，避免每个请求产生不可控的单点样本。标签经过注册表校验后 canonicalize。

### 19.4 Alert 与 Incident

```sql
CREATE TABLE IF NOT EXISTS alerts (
    alert_id              TEXT PRIMARY KEY,
    project_id            TEXT,
    rule_id               TEXT NOT NULL,
    fingerprint            TEXT NOT NULL,
    severity              TEXT NOT NULL,
    state                 TEXT NOT NULL,
    first_seen_at         TEXT NOT NULL,
    last_seen_at          TEXT NOT NULL,
    resolved_at           TEXT,
    summary_safe          TEXT NOT NULL,
    evidence_json         TEXT NOT NULL,
    incident_id            TEXT,
    event_id              TEXT,
    UNIQUE(project_id, fingerprint, state)
);

CREATE TABLE IF NOT EXISTS incidents (
    incident_id           TEXT PRIMARY KEY,
    project_id            TEXT,
    severity              TEXT NOT NULL,
    state                 TEXT NOT NULL,
    title_safe            TEXT NOT NULL,
    description_safe      TEXT NOT NULL,
    fingerprint           TEXT NOT NULL,
    opened_at             TEXT NOT NULL,
    acknowledged_at       TEXT,
    mitigated_at          TEXT,
    resolved_at           TEXT,
    owner_actor_id        TEXT,
    resolution_code       TEXT,
    evidence_json         TEXT NOT NULL,
    created_event_id      TEXT,
    updated_at            TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_incidents_state_time
    ON incidents(state, severity, opened_at);
```

### 19.5 维护步骤与健康检查

```sql
CREATE TABLE IF NOT EXISTS maintenance_run_steps (
    step_id               TEXT PRIMARY KEY,
    maintenance_run_id    TEXT NOT NULL,
    step_order             INTEGER NOT NULL,
    step_key               TEXT NOT NULL,
    state                  TEXT NOT NULL,
    progress_json          TEXT NOT NULL,
    started_at             TEXT,
    finished_at            TEXT,
    error_code             TEXT,
    report_ref             TEXT,
    FOREIGN KEY (maintenance_run_id)
      REFERENCES maintenance_runs(maintenance_run_id)
);

CREATE TABLE IF NOT EXISTS health_checks (
    health_check_id        TEXT PRIMARY KEY,
    component              TEXT NOT NULL,
    scope_type             TEXT,
    scope_id               TEXT,
    status                 TEXT NOT NULL,
    checked_at             TEXT NOT NULL,
    latency_ms             INTEGER,
    error_code             TEXT,
    safe_details_json      TEXT NOT NULL,
    source_revision        TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_health_component_time
    ON health_checks(component, checked_at);
```

健康检查可以只保留最近窗口；发生 failed、degraded 或恢复时，必须同时产生事件或审计索引，不能只覆盖旧的 gauge 值。

### 19.6 支持包与操作记录

```sql
CREATE TABLE IF NOT EXISTS support_bundles (
    bundle_id             TEXT PRIMARY KEY,
    project_id            TEXT,
    requested_by          TEXT NOT NULL,
    requested_at          TEXT NOT NULL,
    expires_at             TEXT NOT NULL,
    scope_json             TEXT NOT NULL,
    manifest_json          TEXT NOT NULL,
    status                 TEXT NOT NULL,
    blob_ref               TEXT,
    redaction_report_json  TEXT NOT NULL,
    download_count         INTEGER NOT NULL DEFAULT 0,
    completed_at           TEXT,
    error_code             TEXT
);

CREATE TABLE IF NOT EXISTS operator_actions (
    operator_action_id     TEXT PRIMARY KEY,
    maintenance_run_id     TEXT,
    incident_id             TEXT,
    actor_id                TEXT NOT NULL,
    action                  TEXT NOT NULL,
    parameters_digest       TEXT NOT NULL,
    confirmation_ref        TEXT,
    occurred_at             TEXT NOT NULL,
    outcome                 TEXT NOT NULL,
    event_id                TEXT
);
```

### 19.7 日志和 Trace 的存储策略

默认日志写入本地滚动文件或受控本地 sink，Trace 使用短期 ring buffer 或可选 OTLP exporter。若产品需要在 SQLite 内查询最近诊断数据，只保存已脱敏、限长、可过期的索引和 Blob 引用，不保存无限增长的原始日志表。

---

## 20. 事件注册表

Observability 需要给每个事件声明 durability、敏感性、投影、审计和广播策略：

```rust
pub struct EventDescriptor {
    pub event_type: &'static str,
    pub schema_version: u32,
    pub durability: Durability,
    pub default_redaction: RedactionLevel,
    pub audit_class: Option<AuditClass>,
    pub projection_targets: &'static [&'static str],
    pub realtime_policy: RealtimePolicy,
    pub retention_class: RetentionClass,
    pub upcaster: Option<UpcasterId>,
}
```

示例：

| 事件 | Durable | Audit | Realtime | Projection |
|---|---|---|---|---|
| `run.started` | 是 | 否 | 是 | session/run |
| `tool.call_finished` | 是 | 是 | 是 | tools |
| `tool.output.delta` | 否 | 否 | 是 | 无/临时 |
| `permission.decision.completed` | 是 | 是 | 是 | security |
| `credential.use.completed` | 是 | 是 | 可选 | credential |
| `data.egress.denied` | 是 | 是 | 是 | security |
| `projection.rebuild.completed` | 是 | 是 | 是 | ops |
| `health.check.sampled` | 否或聚合 | 否 | 可选 | health |
| `incident.opened` | 是 | 是 | 是 | incidents |

注册表是代码和文档的共同契约。未注册事件不得进入持久 Event Store；开发模式可以拒绝启动，生产模式至少创建 `unknown_event_type` 故障并阻止相关投影前进。

---

## 21. 审计与权限集成

### 21.1 查询权限

控制面 Capability 最小化为：

```text
observability.read.v1
observability.read_sensitive_summary.v1
events.read.v1
events.read_payload_capability.v1
audit.read.v1
audit.export.v1
metrics.read.v1
traces.read.v1
alerts.read.v1
incidents.read.v1
incidents.manage.v1
maintenance.read.v1
maintenance.migration.v1
maintenance.backup.v1
maintenance.restore.v1
maintenance.projection_rebuild.v1
maintenance.privacy_purge.v1
support_bundle.create.v1
support_bundle.download.v1
```

`events.read_payload_capability.v1` 不等于读取 Secret；它只允许在 Blob capability 和 Data Policy 同时允许时下载特定安全对象。

### 21.2 维护权限

高风险动作使用“命令能力 + 目标范围 + 二次确认 + 当前版本”四元组：

```text
effective_maintenance =
  actor capability
  ∩ project scope
  ∩ maintenance kind
  ∩ preflight confirmation
  ∩ current policy revision
  ∩ lock/fence validity
```

前端不能把按钮隐藏当作安全控制。每个 Command 都由 Core 重新判权并记录拒绝原因。

### 21.3 Break-glass

紧急绕过只允许在显式配置的 break-glass 流程中使用：

- 需要理由、事件/Incident 引用、过期时间和最小范围；
- 默认禁止 Credential 原文和 data egress；
- 所有动作强制写入高严重级别审计；
- 自动创建 Incident；
- 到期自动撤销；
- 禁止用于删除 Domain Event 或抹除审计痕迹。

---

## 22. 支持诊断包

### 22.1 目标

支持包帮助用户或开发者提交问题，默认服务于本地诊断，不默认上传。它必须可重复生成、范围可见、内容可审查、短期可下载并自动过期。

### 22.2 包内容

默认包括：

- Apex 版本、构建 digest、OS/架构摘要；
- schema、event registry、projection revision；
- 最近一段时间的安全日志、错误日志和聚合指标；
- 健康检查、告警、Incident 和维护报告；
- 事件 seq 范围、投影 lag、Outbox 状态；
- Tool/MCP/Provider/Skill/Agent 的安全摘要；
- 脱敏报告、采样说明和 manifest 校验值。

默认不包括：

- Prompt、Secret、完整外部响应、Cookie、私钥、环境变量值；
- 完整工具参数、任意项目文件和敏感绝对路径；
- Credential Store 原文和未授权 Blob。

### 22.3 生成流程

```text
create support bundle command
  → preflight + scope preview
  → user confirmation
  → snapshot event/cursors/logs/metrics
  → scan and redact
  → manifest + checksum
  → encrypted temporary Blob
  → download capability with TTL
  → expiry GC
```

如果扫描失败，包生成失败并记录 `support_bundle.redaction_failed`；不能为了“帮助诊断”降级为原文导出。

---

## 23. 故障模式与恢复

### 23.1 Event Commit 失败

- 若事务未提交：不广播成功事件，命令返回可分类错误；
- 若提交结果未知：先执行事务结果核对，不能直接重试可能产生重复副作用的命令；
- 使用 `idempotency_key`、command digest 和 aggregate version 识别已提交结果；
- 持久化错误码和恢复建议，不写入原始数据库异常中的敏感参数。

### 23.2 Event Store 与 Projection 不一致

以事件 seq、projection cursor、event hash 和投影 registry 对账。发现投影多写、少写或跨项目污染时，投影进入 `rebuild_required`，旧投影标记不可信，重建到临时结构后再切换。

### 23.3 Outbox 堵塞

Outbox consumer 采用 lease、重试次数、指数退避和 dead-letter。外部目的地不可用时，不能阻塞 Domain Event commit；本地客户端订阅优先于可选外部 exporter。

### 23.4 Broadcaster 或客户端过慢

广播器只负责发送，不持有业务写锁。慢客户端被断开并要求 resync；实时 delta 可丢弃，持久事件通过 cursor replay。单个客户端的队列、CPU 或内存异常必须可见但不能拖垮 Core。

### 23.5 Projection 失败或未知事件

投影 worker 记录失败事件、错误码、schema revision 和安全上下文，暂停受影响投影而不吞错。修复 upcaster 或投影代码后运行 `projection_rebuild`。如果事件 payload 被发现含有违规 Secret，需执行隐私事故流程，而不是简单覆盖事件。

### 23.6 数据库满或损坏

磁盘水位分为：

```text
normal → warn → stop_nonessential → stop_writes → read_only_recovery
```

进入 `stop_nonessential` 时停止日志扩张、可选 Snapshot 和外部上传；进入 `stop_writes` 时拒绝新 Run/Tool，保留查询、备份和恢复；发现 `CORRUPT` 时进入只读恢复，优先复制文件、导出可读事件并创建 Incident。

### 23.7 Clock Drift

所有持久事件以 Core 记录时间和 UTC 为准，同时保留 Adapter 提供的外部时间作为不可信字段。检测到时钟明显漂移时，告警并避免用墙上时间推断事件顺序；顺序以 `global_seq` 为权威。

### 23.8 未知外部副作用

进程崩溃、网络断开或 Adapter 无法确认结果时，操作状态必须为 `interrupted/unknown`：

- 不自动宣称成功或失败；
- 不盲目重放；
- 由 Recovery Reconciler、人工确认或 Provider 查询进行收敛；
- 用户在面板看到影响范围、外部操作 ID 和推荐动作；
- 最终收敛产生新的 Domain Event 和 Audit Record。

### 23.9 Credential Store 不可用

需要 Secret 的操作 fail-closed；不应回退到日志、环境变量或 SQLite 明文。只读查询和不需要 Credential 的操作仍可用，健康面板显示依赖降级。

---

## 24. Recovery 与启动顺序

Apex 启动时按以下顺序恢复。本序列是领域模型 §10.2 十二步恢复的运维视角展开，编号后括注对应的基线步骤：

```text
 1. 获取 daemon 单实例锁                                  [基线 ①]
 2. 读取配置和本地身份
 3. 打开 SQLite WAL                                       [基线 ②]
 4. 执行受控迁移或进入 migration required                 [基线 ②]
 5. 验证 schema、event registry、事件 seq 与 projection cursor  [基线 ③]
 6. 检查完整性（quick_check；hash chain 若启用）
 7. 恢复未投递的 outbox                                   [基线 ④]
 8. reconcile Markdown write intent 与外部编辑             [基线 ⑤]
 9. reconcile Snapshot 的 creating/restoring 操作          [基线 ⑥]
10. 检查 Provider/MCP/Tool operation journal              [基线 ⑦]
11. 回收或登记孤儿子进程、维护 lease                      [基线 ⑧]
12. 标记运行中但未完成的维护任务
13. 扫描 active Run/Workflow/Agent/Node，分类为可恢复/blocked/interrupted  [基线 ⑨]
14. 将无法确认的外部操作置为 interrupted/unknown          [基线 ⑨]
15. 将过期 Write Claim 置为 suspect 并 reconcile owner 进程，
    确认 owner 确已消失后才释放                          [基线 ⑩]
16. 重建必要的 projection / FTS 索引，执行 catch-up       [基线 ⑪]
17. 启动 Event Bus、Projector、Outbox 和 Health Worker
18. 发布 recovery.completed 并开放写流量                  [基线 ⑫]
19. 更新 readiness
```

完整性检查（第 5–6 步）失败时只开放诊断、导出和显式修复，不开放 Agent 执行。

> ADR-0027（跨文档一致性审查）：原序列缺 daemon 单实例锁、outbox 恢复、Markdown write intent reconcile、Snapshot reconcile 和 FTS 重建五个基线步骤。原第 10 步"释放过期 Write Claim"与领域模型 §4.9 冲突——lease 过期不等于 owner 已死，直接释放可能让新写者与仍在运行的旧 owner 并发写同一路径；已改为先置 `suspect` 再 reconcile。

启动过程中不执行未经确认的外部 Tool、MCP、Provider 或 Credential 使用。恢复器产生的每个状态修正都带 `causation_id` 和 `recovery_id`。

---

## 25. Rust 模块和 Port 设计

建议在 Workspace 中拆出以下模块：

```text
crates/
  apex-observability/
    src/
      lib.rs
      context.rs
      event_registry.rs
      event_store.rs
      event_query.rs
      event_bus.rs
      outbox.rs
      projector.rs
      projection_registry.rs
      audit.rs
      redaction.rs
      metrics.rs
      tracing.rs
      health.rs
      alerts.rs
      incidents.rs
      maintenance.rs
      support_bundle.rs
      retention.rs
      error.rs
  apex-core/
    src/ports/observability.rs
    src/ports/maintenance.rs
    src/ports/audit.rs
  apex-storage/
    migrations/
    src/observability_tables.rs
```

### 25.1 Port

```rust
#[async_trait]
pub trait ObservabilityPort {
    async fn query_overview(
        &self,
        request: OverviewQuery,
        auth: AuthContext,
    ) -> Result<OverviewView, ObservabilityError>;

    async fn subscribe(
        &self,
        request: SubscriptionRequest,
        auth: AuthContext,
    ) -> Result<EventSubscription, ObservabilityError>;

    async fn create_support_bundle(
        &self,
        request: SupportBundleRequest,
        auth: AuthContext,
    ) -> Result<SupportBundleView, ObservabilityError>;
}

#[async_trait]
pub trait MaintenancePort {
    async fn preflight(
        &self,
        command: MaintenancePreflight,
        auth: AuthContext,
    ) -> Result<MaintenancePlan, MaintenanceError>;

    async fn start(
        &self,
        command: MaintenanceStart,
        confirmation: ConfirmationToken,
        auth: AuthContext,
    ) -> Result<MaintenanceRunView, MaintenanceError>;

    async fn cancel(
        &self,
        run_id: MaintenanceRunId,
        auth: AuthContext,
    ) -> Result<MaintenanceRunView, MaintenanceError>;
}
```

### 25.2 依赖规则

- `apex-observability` 可以依赖领域事件 Port、Storage Port、Policy Port 和 Blob Port；
- 不得依赖 UI、Web Framework、Provider SDK 或具体 MCP 实现；
- Alert action 只依赖 Incident Command Port，不依赖 Tool Gateway；
- Projection 只能调用纯查询/派生逻辑；
- Maintenance worker 通过隔离的 job Port 调用数据库、文件系统或子进程能力，并提交结果事件。

---

## 26. 配置与默认值

配置分为不可变启动配置、实例运维配置、项目级策略和会话级临时选项。敏感值仍遵守 Credential 文档，不直接写入普通配置。

```toml
[observability]
enabled = true
local_log_level = "info"
realtime_batch_ms = 100
realtime_max_queue_bytes = 1048576
projection_lag_warn_ms = 1000
projection_lag_critical_ms = 10000
event_replay_max_rows = 10000
event_replay_max_bytes = 16777216
external_telemetry = false

[observability.retention]
metrics_days = 30
logs_days = 7
traces_days = 3
realtime_minutes = 10
support_bundle_ttl_hours = 24

[maintenance]
allow_restore = false
allow_privacy_purge = false
require_confirmation = true
max_parallel_jobs = 1
```

所有配置变更都通过 Configuration Command，产生版本、差异摘要、actor、原因和生效时间审计。环境变量只用于启动时提供非敏感覆盖或 Credential Store 定位信息，不能将 Secret 值写入日志。

---

## 27. 测试、验证与混沌演练

### 27.1 单元测试

- Event envelope canonicalization、schema registry 和 upcaster；
- Redaction、taint、secret scanner 和数据分类；
- Audit safe evidence 生成；
- Metric label 白名单和高基数拒绝；
- Projection reducer 的确定性和幂等性；
- Alert fingerprint、去重、抑制和 Incident 状态机；
- Maintenance preflight、confirmation、lease、fence 和取消。

### 27.2 集成测试

- Command commit 后事件、投影、Outbox 和客户端通知顺序；
- 重启后 cursor replay、projection catch-up 和 unknown operation 恢复；
- Web/TUI/Desktop 使用相同 Query 得到相同安全语义；
- MCP/Provider/Tool 失败、超时、取消和未知副作用的审计闭环；
- Credential 使用、Data Egress、Redaction 和诊断包无 Secret 泄漏；
- 迁移、备份、恢复、FTS 重建和隐私清除。

### 27.3 混沌场景

注入以下故障并验证可观测性本身不会放大故障：

- SQLite `BUSY/LOCKED/FULL/IOERR/CORRUPT`；
- commit 前崩溃、commit 后广播前崩溃、Outbox 发送中断；
- Projector 随机崩溃、未知事件、upcaster 抛错；
- WebSocket 慢消费、网络断开、cursor 过期；
- Provider/MCP 响应延迟、重复响应、连接半开；
- 子进程被杀、操作结果未知、Write Claim lease 过期；
- Secret Scanner 误报、扫描超时、Blob 写入失败；
- 磁盘水位跨越保护阈值和维护锁丢失。

每个演练至少验证：事实是否可恢复、审计是否完整、是否 fail-closed、用户是否得到安全可理解的结果、是否创建或关闭对应 Incident。

### 27.4 属性与不变量

```text
I1: global_seq 在项目内严格递增且不重复
I2: 事务未提交的事实不可被客户端观察为 committed
I3: projection_applied_seq 不超过 event_store_head_seq
I4: projection 可由事件重建得到同一 revision 结果
I5: terminal aggregate 不被迟到结果回退
I6: secret_prohibited 不进入普通事件、日志、指标标签、Trace 或支持包
I7: 单个慢订阅者不阻塞 Event Store commit
I8: unknown external effect 不自动重放
I9: maintenance 终态不可回到运行态
I10: 每个高风险 operator action 有 actor、reason、scope 和 outcome
```

---

## 28. 交付阶段

### 28.1 P0：可恢复事实与基本面板

- 统一 Event Envelope、Event Registry、Event Store 和 global_seq；
- Session/Run/Tool/MCP/Agent/Rule/Checkpoint/Workspace 基础事件；
- Projection Registry、cursor、Outbox 和客户端 reconnect；
- Skill、MCP、SubAgent、Tool 调用面板；
- 基础结构化单行日志、健康检查、SQLite 指标；
- Secret scanner 和 safe view 强制接入。

### 28.2 P1：安全审计与运维任务

- Audit Record、安全/权限/凭据/Data Egress 审计；
- Approval、Gate、Waiver、Skip Spec 和恢复审计；
- MaintenanceRun、备份、完整性检查、投影重建、FTS、GC；
- Alert/Incident 基础状态机；
- 支持诊断包和本地下载 capability；
- TUI/Desktop/Web 运维面板。

### 28.3 P2：高级运营能力

- Trace 聚合与尾采样；
- 项目级 SLO、趋势和容量预测；
- 签名审计归档；
- 多实例或远程 Event Relay；
- 可选外部 telemetry exporter；
- 自动化但受限的低风险修复动作；
- 更细粒度的数据保留和隐私清除编排。

---

## 29. ADR 清单

实现前建议冻结以下 ADR：

1. `ADR-OBS-001`：SQLite Event Store 与全局序列边界；
2. `ADR-OBS-002`：Domain Event、Realtime Event 和 Outbox 的持久化等级；
3. `ADR-OBS-003`：Projection revision、upcaster 和 rebuild 切换；
4. `ADR-OBS-004`：统一 redaction、taint 和 safe evidence 管道；
5. `ADR-OBS-005`：本地日志/Trace 与外部 telemetry 默认关闭；
6. `ADR-OBS-006`：Audit Record 与 Domain Event 的关联方式；
7. `ADR-OBS-007`：Alert/Incident 去重、抑制和处置权限；
8. `ADR-OBS-008`：MaintenanceRun、维护锁和 fence token；
9. `ADR-OBS-009`：Backup/Restore 与 Credential Store 的分离；
10. `ADR-OBS-010`：支持诊断包的内容白名单和 TTL；
11. `ADR-OBS-011`：高基数指标和本地容量保护；
12. `ADR-OBS-012`：审计归档哈希链与可选签名。

---

## 30. 验收标准

### 30.1 事实与恢复

- 任一已提交 Run、Tool、MCP、Approval、Rule、Gate、Snapshot 和维护任务都能通过事件或安全投影解释；
- Core 重启后能从 `last_seen_global_seq` 恢复客户端；
- Projection 删除后可重建，且结果与同一事件范围一致；
- 未知外部副作用不会被自动伪装成成功或再次执行。

### 30.2 面板与体验

- TUI、Desktop、Web 使用相同核心 Query 和事件语义；
- Skill、MCP、SubAgent 和通用调用日志面板能展示需求文档规定的字段；
- 面板刷新目标接近 1 秒，并明确显示 `as_of_global_seq` 和滞后告警；
- 连接断开、事件缺口、schema 未知时能自动请求 projection refresh。

### 30.3 安全与审计

- Secret、完整凭据头、Prompt 原文和敏感文件正文不会进入普通 telemetry；
- 权限、Approval、Credential、Data Egress、Redaction、Gate、维护和导出都有审计；
- 普通客户端不能直接执行维护或改变审计；
- 支持包经过扫描、范围确认、TTL 和下载审计；
- break-glass 使用能自动创建高严重级别 Incident。

### 30.4 运维与容量

- Migration、Backup、Restore、Integrity Check、Projection Rebuild、FTS、GC 和 Purge 均可查询状态；
- 数据库错误、磁盘水位、Projection/Outbox lag 和外部依赖异常能产生可解释 Alert/Incident；
- 日志、Metric、Trace 和 Realtime 流具备限速、采样、背压和过期机制；
- 维护锁丢失、任务取消和进程崩溃后不会留下不可见的后台写入。

---

## 附录 A：推荐事件目录

```text
observability.health.status_changed
observability.telemetry.configured
observability.realtime.batch_dropped
observability.export.blocked

audit.record.created
audit.export.requested
audit.export.completed

audit.export.denied

maintenance.requested
maintenance.preflight.completed
maintenance.started
maintenance.progressed
maintenance.blocked
maintenance.cancel_requested
maintenance.cancelled
maintenance.completed
maintenance.failed
maintenance.lock_acquired
maintenance.lock_lost
maintenance.migration.completed
maintenance.backup.completed
maintenance.restore.completed
maintenance.integrity_check.completed
maintenance.projection_rebuild.completed
maintenance.fts_rebuild.completed
maintenance.gc.completed
maintenance.privacy_purge.completed

alert.opened
alert.updated
alert.resolved
alert.suppressed
incident.opened
incident.acknowledged
incident.mitigated
incident.resolved
incident.reopened
incident.suppressed

support_bundle.requested
support_bundle.redaction_started
support_bundle.redaction_failed
support_bundle.completed
support_bundle.downloaded
support_bundle.expired
support_bundle.deleted

recovery.started
recovery.unknown_operation_detected
recovery.claim_released
recovery.completed
```

---

## 附录 B：推荐错误码

> ADR-0010（跨文档一致性审查）：本清单原使用 `OBS_*` 模块前缀，是全库唯一自建前缀族，且 `OBS_PROJECTION_LAGGING`、`OBS_CURSOR_EXPIRED` 与领域模型 §6.2 的通用码语义重复。现按全局规范去除前缀；与通用码同名者直接复用领域模型定义，不再单列。

```text
EVENT_UNKNOWN_TYPE
EVENT_SCHEMA_INVALID
EVENT_REDACTION_FAILED
EVENT_PAYLOAD_TOO_LARGE
EVENT_SEQUENCE_GAP
EVENT_HASH_CHAIN_BROKEN
PROJECTION_LAGGING
PROJECTION_REBUILD_REQUIRED
PROJECTION_DEAD_LETTER
OUTBOX_LAGGING
SUBSCRIBER_BACKPRESSURE
CURSOR_EXPIRED
CAPABILITY_REDACTED
TELEMETRY_DISABLED
TELEMETRY_EXPORT_BLOCKED
METRIC_LABEL_REJECTED
SUPPORT_BUNDLE_SCOPE_REQUIRED
SUPPORT_BUNDLE_REDACTION_FAILED
MAINTENANCE_CONFIRMATION_REQUIRED
MAINTENANCE_LOCK_BUSY
MAINTENANCE_FENCE_LOST
MAINTENANCE_CANCELLED
MAINTENANCE_PRECONDITION_FAILED
DB_DISK_WATERMARK
DB_INTEGRITY_FAILED
DB_READ_ONLY_RECOVERY
UNKNOWN_EXTERNAL_EFFECT
CREDENTIAL_STORE_UNAVAILABLE
```

错误码面向客户端和自动化处理稳定，具体数据库错误、堆栈和内部文件位置只能进入受保护诊断日志。

---

## 附录 C：面板事件刷新策略

| 面板 | 首次加载 | 实时事件 | 缺口处理 | 默认刷新 |
|---|---|---|---|---:|
| Overview | projection query | health/incident/run | 全量 overview refresh | 1s |
| Session Timeline | cursor query | durable events + sampled progress | seq replay/query | 500ms～1s |
| Skill | skill projection | skill load/invocation | projection refresh | 1s |
| MCP | MCP projection | connection/call | projection refresh | 1s |
| SubAgent | agent/node projection | lifecycle/progress | projection refresh | 1s |
| Security | audit/security projection | approval/deny/egress | strong query | 1s |
| Operations | maintenance/health projection | maintenance/incident | strong query | 1s |

实时事件只触发刷新意图；客户端不能仅凭一个 `tool.progress` 事件自行修改调用状态。

---

## 附录 D：最小审计验收样例

```json
{
  "audit_id": "audit_01H...",
  "action": "data_egress.evaluate",
  "target": {
    "type": "mcp_server",
    "id": "mcp_server_01H..."
  },
  "actor": {
    "type": "agent",
    "id": "agent_01H..."
  },
  "reason_code": "tool_requested_external_call",
  "policy_revision": "policy-v12",
  "approval_ref": "approval_01H...",
  "evidence": {
    "classification": "confidential",
    "fields_considered": ["workspace_diff", "credential_ref"],
    "bytes": 1820,
    "taint": "external_destination",
    "redaction_applied": true,
    "content_digest": "sha256:..."
  },
  "outcome": "allowed_after_redaction",
  "redaction_level": "safe_view"
}
```

这里不会出现 token、完整请求头、Prompt 原文、工作区文件正文或 MCP 返回正文；若用户具有额外 Blob capability，也必须另行下载并产生 `blob.downloaded` 审计。

---

## 附录 E：与其他详细设计的交叉约束

| 上游设计 | Observability 必须遵守 |
|---|---|
| 总体架构 | Core 是唯一事实源；Web Gateway 不复制业务逻辑；native IPC 优先 |
| 领域模型与事件规范 | 事件不可变、命令幂等、投影可重建、终态不可回退 |
| API 与实时协议 | Query/Command/Event 分离；global_seq、cursor、projection refresh |
| SQLite 数据模型 | 单写者、WAL、事务内无外部副作用、维护表和容量水位 |
| Agent Runtime | Run/Turn/Node/Tool 生命周期和未知副作用可恢复 |
| Tool Gateway | operation_id、safe arguments、result normalization、capability redaction |
| Rules/Gate | Verification、Waiver、Skip Spec 和证据进入审计 |
| Context/Checkpoint | checkpoint 摘要、恢复原因和敏感数据隔离 |
| Workspace/Snapshot | snapshot、claim、fence、文件摘要和恢复对账 |
| Extension System | registry generation、digest、invocation、taint 和扩展审计 |
| Credential Governance | Secret 不进普通 telemetry；credential.use 与 data.egress 分离 |

---

## 附录 F：后续设计

Observability、审计和运维控制面完成后，下一份建议文档为：

> `Apex—— Deployment、升级与灾备详细设计.md`

该文档将把当前控制面落到单机开发、桌面发行、Web 服务部署、数据目录布局、版本升级、跨平台打包、灾备和发布回滚上，并明确 v0.5～v1.x 的最终交付拓扑。




