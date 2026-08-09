# Apex—— SQLite 数据模型与迁移设计

> 文档状态：总体架构详细设计（面向最终完整产品）  
> 上游依据：`Apex—— 需求分析文档.md`、`Apex—— 系统总体架构设计.md`、`Apex—— 领域模型与事件规范.md`、`Apex—— API与实时事件协议设计.md`  
> 设计日期：2026-08-08  
> 目标读者：Core、Storage、Runtime、Protocol、Desktop/Web/TUI、测试与运维开发者

---

## 0. 目的、范围与权威关系

本文定义 Apex 最终完整产品的 SQLite 持久化架构，包括：

- 数据库拓扑、连接和 PRAGMA；
- 类型、命名、主键、时间、版本与软删除约定；
- Command、Domain Event、Current State、Projection、Outbox 的事务关系；
- Project、Session、Run、Spec、Workflow、Tool、Permission、Checkpoint、Snapshot、Memory、MCP 等表族；
- FTS5、Blob、Markdown 镜像、影子 Git 与数据库之间的一致性协议；
- Schema migration、数据回填、备份、恢复、损坏处理、归档与 GC；
- Rust `apex-storage` 的接口边界、测试和实施路线。

本文不重新定义领域状态机或对外 Wire Model：

1. **领域事实与不变式**以《领域模型与事件规范》为权威；
2. **外部 Command/Query/Event 字段**以《API 与实时事件协议设计》为权威；
3. 本文决定这些语义如何映射为 SQLite schema、事务和恢复行为；
4. 若数据库结构与领域语义冲突，优先修改数据库映射，不得通过 SQL 偷改领域含义。

---

## 1. 存储架构结论

### 1.1 核心结论

Apex 采用以下持久化模型：

```text
本机单用户数据库 <APEX_HOME>/apex.db
            │
            ├─ apex-core / StorageWriter：唯一业务写者
            ├─ Current State：强一致聚合当前态
            ├─ Domain Event Store：不可变事实与全局游标
            ├─ Projection：面向 Query/UI 的可重建读模型
            ├─ Command Dedup：Command 幂等结果
            ├─ Operation Journal：外部副作用恢复依据
            └─ Outbox：数据库提交后执行文件/MCP/广播等动作

项目工作区
  ├─ <project>/apex/specs/、rules/、skills/、memory/、checkpoints/：团队可提交资产
  └─ 源代码：最终文件事实

<APEX_HOME>/blobs/：内容寻址大对象
<APEX_HOME>/snapshots/：影子 Git 文件快照
```

关键决定：

1. `apexd` 内的 Core 是 SQLite **唯一业务写者**；TUI、Tauri、Web Gateway、Plugin、Hook、Worker 不直接打开数据库写入；
2. 使用 **Current State + Domain Event + Projection** 混合模型，不采用“每次 Query 都完整事件回放”的纯 Event Sourcing；
3. Command 接受结果、聚合变化、Domain Event、幂等记录和必要 Outbox 必须在一个数据库事务中提交；
4. 长时间 Provider、Bash、MCP、文件和 Git I/O 不得持有 SQLite transaction；先提交 intent，再执行，再以新 Command/结果事务落库；
5. Domain Event 和权威 Artifact Revision 不依赖 Projection 存活；Projection、FTS5 和缓存允许删除重建；
6. 大正文和二进制放入内容寻址 Blob Store，SQLite 只保存元数据、digest、引用和授权 scope；
7. Spec Markdown、Memory Markdown、Checkpoint 文件和影子 Git 是不同权威层级，不能把“文件存在”直接等同于“业务事务已提交”；
8. migration 只前进，不支持旧二进制直接打开新 schema；降级依赖备份恢复或导入导出；
9. 数据库发生完整性异常时进入只读维护模式，不继续执行 Tool；
10. 初始技术选型使用 `rusqlite` + bundled SQLite，启用 WAL、FTS5、JSON1、RETURNING、backup API，并在构建时固定最低 SQLite 版本。

### 1.2 为什么选 `rusqlite`

需求文档已经指定 `rusqlite (SQLite + WAL + FTS5)`。最终产品继续采用该路线，原因是：

- Core 使用单写者 actor，天然适合一个受控的同步 SQLite writer connection；
- 能精细控制 `sqlite3_authorizer`、busy handler、backup、WAL checkpoint、update hook 和扩展可用性；
- 避免异步连接池隐藏长事务或跨 `await` 持锁；
- bundled SQLite 能固定安全修复和 FTS5/JSON1 能力，减少系统 SQLite 差异；
- Tokio 侧通过专用 StorageWriter 线程/actor 与 bounded channel 交互，不在异步 executor 上执行阻塞 SQL。

`sqlx` 可用于独立工具或未来远程数据库适配器，但不得形成第二套业务写路径。

### 1.3 非目标与禁止模式

明确禁止：

- Client 或 Plugin 直接执行 SQL；
- 把 JSON 文件与 SQLite 同时作为 Session 权威源；
- 在事务内等待用户审批、Provider token、子进程或网络；
- 通过 UPDATE/DELETE 修改历史 Domain Event；
- 用 `occurred_at`、ULID 或客户端到达顺序代替 `global_seq`；
- 把 Provider key、浏览器 cookie、OS credential 明文写入数据库；
- 用 `INSERT OR REPLACE` 更新聚合；它可能隐式 DELETE 并破坏 FK、审计和版本语义；
- 用 offset pagination 承担长期稳定分页；
- 依靠 SQLite trigger 实现完整领域状态机；
- migration 中静默丢弃未知枚举、失败行或旧事件字段。

---

## 2. 数据库拓扑、文件与生命周期

### 2.1 文件布局

用户级 Apex Home 使用平台原生目录（权威定义见 Deployment §4.1）：

```text
Windows: %APPDATA%\Apex\
macOS:   ~/Library/Application Support/Apex/
Linux:   ${XDG_STATE_HOME:-~/.local/state}/apex/
```

本文档统一以 `<APEX_HOME>` 指代该目录。存储相关子项：

```text
<APEX_HOME>/
├── apex.db
├── apex.db-wal                  # SQLite 管理，不单独移动
├── apex.db-shm                  # SQLite 管理，不单独移动
├── backups/
│   ├── pre-migration-<version>-<timestamp>.db
│   └── scheduled-<timestamp>.db
├── quarantine/                  # 损坏库与恢复报告
├── blobs/
│   ├── objects/sha256/ab/cd/<digest>
│   ├── tmp/
│   └── staging/<upload-id>.part
├── snapshots/                   # <project_hash>/<worktree_hash>/.git
├── config/
├── rules/                       # 用户级规则
├── skills/                      # 用户级 Skill
├── mcp.json                     # 用户级 MCP 配置
├── runtime/daemon.json
├── logs/
└── diagnostics/
```

数据库必须位于本机文件系统。禁止把 WAL 数据库直接放在 SMB/NFS/云同步目录；若用户配置的 `APEX_HOME` 不满足本地锁语义，启动 preflight 必须拒绝或退回受支持路径。

Credential 不落盘于此：默认后端是 OS Credential Store，`~/apex/auth.json` 仅作为历史版本的**一次性导入路径**（Credential 治理 §5.3），不属于 `<APEX_HOME>` 结构。

> ADR-0002（跨文档一致性审查）：原布局用 `~/apex/state/apex.db`，与本文档 §1 概览及 Deployment §4.1 三方互斥。现统一为平台原生 Home + 扁平化 DB 路径（去掉 `state/` 层），并补回 `rules/`、`skills/`、`mcp.json`。项目级可提交资产（specs/rules/skills/memory/checkpoints）见 Deployment §4.1.1。

### 2.2 一个库还是多个库

v1.0 使用**每个 OS 用户一个主数据库**，所有 Project 通过 `project_id` 隔离。优点：

- 全局 `global_seq` 与三端统一重连简单；
- 跨项目搜索、最近会话、统一审批和审计容易实现；
- migration、备份和 daemon ownership 单一；
- 无需跨数据库事务。

日志、遥测或高吞吐 transient delta 可以未来拆分独立库，但拆分库不得承载领域事实。若未来引入多个 Core 实例，则每库独立 `event_store_id` 和序列，不能伪造跨库全局顺序。

### 2.3 数据库身份

`db_metadata.event_store_id` 是数据库事件历史的稳定身份：

- 正常迁移、VACUUM、在线备份恢复保持不变；
- 创建全新数据库时生成新 ID；
- 只导入 Current State 而没有完整事件历史时必须生成新 ID；
- 客户端发现 `event_store_id` 改变后，丢弃 durable event cursor 并重新 Query；
- 不允许复制库后两个独立 daemon 同时继续写入同一 `event_store_id`。首次写入前必须检查 instance ownership。

### 2.4 打开与关闭顺序

启动顺序：

```text
锁定 daemon singleton
  → 检查路径与权限
  → 只读读取 SQLite header/application_id/user_version
  → 打开 migration connection
  → PRAGMA integrity preflight（按策略 quick_check）
  → 创建 pre-migration backup（若需要）
  → 执行 migration / backfill gate
  → 打开 writer connection
  → 打开 read pool
  → 校验事件、投影、outbox、operation journal
  → reconcile 文件/快照/外部操作
  → 接受 Command
```

正常关闭：停止接受新 Command → drain StorageWriter → 将可安全中断 operation 记为 stopping/interrupted → checkpoint WAL → 关闭 read pool → 关闭 writer。超时强制退出时不以“WAL 文件消失”为成功条件，恢复依赖下次 SQLite recovery。

---

## 3. 连接模型与 PRAGMA

### 3.1 连接角色

| 连接 | 数量 | 权限 | 用途 |
|---|---:|---|---|
| MigrationConnection | 启动期 1 | schema write | migration、校验、rebuild；运行期关闭 |
| WriterConnection | 运行期 1 | data write | 所有 Unit of Work |
| ReadConnection | 2～8，可配置 | query_only | Query、Projection 读取、导出 |
| MaintenanceConnection | 显式维护期 0/1 | restricted | backup、integrity、VACUUM、受控修复 |

WriterConnection 由 `StorageWriter` 独占，任务之间不得泄漏 connection 或 transaction。ReadConnection 打开后执行 `PRAGMA query_only=ON`；即使应用误用，也不能写入。

### 3.2 基础 PRAGMA

每个 connection 需要设置并验证：

```sql
PRAGMA foreign_keys = ON;
PRAGMA busy_timeout = 5000;
PRAGMA temp_store = MEMORY;
PRAGMA trusted_schema = OFF;
PRAGMA recursive_triggers = OFF;
```

数据库级设置：

```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = FULL;
PRAGMA wal_autocheckpoint = 0;
PRAGMA auto_vacuum = INCREMENTAL;
PRAGMA application_id = 1095779672; -- 0x41504558，ASCII "APEX"
```

说明：

- 默认 `synchronous=FULL`，因为 Command、审批、Spec 决策和工具审计不能为少量吞吐牺牲断电耐久性；
- 可提供 `balanced` profile 使用 `NORMAL`，但 UI 必须标明耐久性变化，安全审计事件仍可强制 FULL checkpoint；
- 禁用 SQLite 自动 checkpoint，由 Apex 根据 WAL 大小、空闲时间和 shutdown 主动执行；
- `mmap_size`、`cache_size` 由基准测试决定，不写死为领域约束；
- `auto_vacuum=INCREMENTAL` 必须在 bootstrap 建表前设置；既有数据库变更该模式需要受控 `VACUUM`，不能仅执行 PRAGMA 就假定已生效；
- 每次借出连接时抽查 `foreign_keys/query_only`，避免第三方调用改变连接状态。

### 3.3 WAL checkpoint 策略

- WAL 达到 64 MiB：请求 `PASSIVE` checkpoint；
- WAL 达到 256 MiB 或磁盘压力高：暂停新低优先级写入，等待旧 reader，执行 `RESTART`；
- 正常关闭：执行 `TRUNCATE`，但失败不影响数据库正确性；
- 长 Query 必须有 deadline，防止 reader snapshot 长期阻塞 checkpoint；
- 导出/备份使用 SQLite backup API，不复制正在变化的 `.db/.wal/.shm` 三个文件；
- 记录 `wal_bytes`、checkpoint latency、busy reader count 和失败原因。

### 3.4 事务模式

业务写事务默认使用 `BEGIN IMMEDIATE`：

- 进入 Unit of Work 时尽早获得 reserved lock，避免完成大量领域计算后才遇到 writer 竞争；
- 单写者下锁竞争应极少，出现频繁 `SQLITE_BUSY` 视为连接泄漏或外部进程误开库；
- 事务内只执行 CPU 有界的校验、SQL 和小 payload 序列化；
- 禁止事务跨 `.await`；
- 目标 p95 提交时间低于 20 ms，超 100 ms 记录 slow transaction，超 2 s 触发健康降级。

---

## 4. 类型、命名与通用列约定

### 4.1 命名

- 表、列、索引使用 `snake_case`；
- 表名使用复数名词；
- FK 列命名 `<entity>_id`；
- 时间列使用 `*_at_us`；
- digest 列使用 `*_digest`，格式 `sha256:<hex>`；
- JSON 列使用 `*_json`；
- 字节数使用 `*_bytes`；
- 计数和顺序使用 `*_seq`、`*_version`、`ordinal`；
- partial index 以 `idx_<table>__<purpose>` 命名；
- unique index 以 `uq_<table>__<purpose>` 命名。

### 4.2 ID

领域 ID 使用带前缀的 Typed ULID，SQLite 存为 `TEXT`。完整前缀注册表见 `Apex—— 领域模型与事件规范.md` §2.1；常见示例：

```text
prj_（Project）   wt_（Worktree）   ses_（Session）  msg_（Message）
run_（Run）       turn_（Turn）     agt_（Agent）    evt_（Event）
spc_（Spec）      art_（Artifact）  arv_（ArtifactRevision）  rev_（Review）
wfl_（Workflow）  wfn_（WorkflowNode）  clm_（WriteClaim）
tol_（ToolCall）  per_（PermissionRequest）  rck_（RuleCheck）
ckp_（Checkpoint） snp_（Snapshot）  cmd_（Command）  op_（Operation）
cor_（Correlation）
```

**注意 `arv_` 与 `rev_` 不可混用**：`arv_` 是不可变的 Artifact Revision，`rev_` 是用户对某个 revision 作出的 Review 决定。二者在 Spec 审批链中同时出现，混用会导致批准绑定到错误对象。

约束主要由 Rust newtype/parser 执行。数据库仅对核心表使用长度和非空 CHECK，不在每个表复制复杂 glob，以免 migration 难以维护。

> ADR-0025（跨文档一致性审查）：原示例中 `rev_` 紧随 `art_` 出现，易被读作 artifact revision，与领域模型 §2.1 的 `arv_`=ArtifactRevision、`rev_`=Review 冲突。现补全前缀清单并显式标注二者区别。

### 4.3 时间

数据库统一使用 UTC Unix epoch **微秒**的 signed `INTEGER`：

```sql
created_at_us INTEGER NOT NULL CHECK (created_at_us >= 0)
```

协议层转换为 RFC3339/protobuf Timestamp。禁止在排序和恢复中依赖本机时区。客户端提供的时间存入单独 `client_observed_at_us`，不得覆盖 Core 的 `committed_at_us`。

### 4.4 顺序和整数范围

SQLite `INTEGER` 是 signed 64-bit，因此：

- `global_seq`、`aggregate_version`、`message_seq`、ordinal 范围为 `1..=i64::MAX`；
- Wire Model 虽使用 `uint64`，Core 对超出 `i64::MAX` 的输入拒绝；
- REST/WS 仍用十进制字符串，不能因为数据库是 signed integer 而改成 JavaScript number；
- `global_seq` 必须严格递增。是否必须无 gap：Apex 定义为**提交后的事件序列连续**；批量事务回滚不能消耗已提交序号。

### 4.5 枚举、布尔与状态

- 枚举存 `TEXT`，关键稳定枚举加 CHECK；高演进频率枚举由应用校验，避免每次新增值都重建表；
- 布尔存 `INTEGER NOT NULL CHECK (value IN (0,1))`；
- 状态迁移由 Domain/Application 层执行，SQL CHECK 只阻止明显非法值；
- 未知历史枚举通过 `unknown/legacy` 显式承载，不映射为空字符串。

### 4.6 JSON 与大内容

- 小型结构化 payload 使用 UTF-8 canonical JSON `TEXT`；
- 写入前在 Rust 侧 canonicalize，数据库可加 `CHECK(json_valid(...))`；
- Event 保存 `payload_json`、`payload_digest` 和 `schema_version`；
- 超过 64 KiB 的正文、stdout、diff、附件使用 `blob_id`；
- JSON 字段不能用于高频过滤的核心条件；需要过滤的字段必须提升为普通列；
- secret scanner 在 payload 进入 Event/diagnostic 前执行，redaction 不是 UI 临时处理。

### 4.7 通用版本列

权威 Current State 表一般包含：

```sql
version          INTEGER NOT NULL CHECK (version >= 0),
created_at_us    INTEGER NOT NULL,
updated_at_us    INTEGER NOT NULL,
last_event_seq   INTEGER NOT NULL CHECK (last_event_seq >= 0)
```

`version` 是聚合版本；`last_event_seq` 是最后影响该行的全局事件水位，两者不能混用。Projection 使用 `as_of_global_seq` 和独立 `projection_revision`。

---

## 5. Unit of Work、序列与提交协议

### 5.1 标准 Command 事务

```text
StorageWriter.handle(command)
  1. BEGIN IMMEDIATE
  2. 查询 command_dedup
  3. 校验 payload_digest / actor / expected_version
  4. 读取并构建聚合
  5. Domain 决策，产生 state changes + events + outbox intents
  6. 预留连续 global_seq 区间
  7. CAS 更新 Current State
  8. append domain_events
  9. 同步更新 strong projection
 10. 写 command_dedup/result、operation_journal、outbox
 11. COMMIT
 12. commit 后通知 EventBroadcaster 和 OutboxWorker
```

步骤 12 失败不能回滚已提交事务。Broadcaster 可从 Event Store 补发，OutboxWorker 可扫描 pending 行恢复。

### 5.2 全局序列分配

使用单行 counter，在事务中按事件数量预留区间：

```sql
CREATE TABLE sequence_counters (
    name  TEXT PRIMARY KEY,
    value INTEGER NOT NULL CHECK (value >= 0)
) WITHOUT ROWID;

INSERT INTO sequence_counters(name, value)
VALUES ('global_event_seq', 0)
ON CONFLICT(name) DO NOTHING;
```

事务内：

```sql
SELECT value FROM sequence_counters WHERE name='global_event_seq';
UPDATE sequence_counters
SET value = value + :event_count
WHERE name='global_event_seq' AND value = :old_value;
```

单写者 + `BEGIN IMMEDIATE` 下得到 `[old+1, old+event_count]`。counter 更新与事件插入同事务，回滚不消耗序号。提交前断言实际插入事件数与预留数一致。

### 5.3 乐观并发

聚合写使用 CAS：

```sql
UPDATE sessions
SET state = :new_state,
    version = version + 1,
    updated_at_us = :now,
    last_event_seq = :last_seq
WHERE session_id = :session_id
  AND version = :expected_version;
```

`changes() != 1` 返回领域冲突，不自动覆盖。Session Actor 仍需版本检查，因为跨 Session Command、恢复任务和迟到 Worker 结果可能并发。

### 5.4 强投影与异步投影

- `session_summary`、`pending_approvals`、Spec 当前阶段等安全/交互关键视图与事件同事务更新；
- `audit_timeline`、skill/mcp 面板、统计聚合等可由 projection worker 异步消费；
- 每个 projection 有 cursor，批次提交“读模型变化 + cursor”必须同事务；
- Query `at_least_seq` 等待目标 projection cursor，而不是等待事件广播；
- Projection rebuild 禁止产生 Domain Event、Outbox 或任何外部副作用。

### 5.5 提交后可见性

CommandResponse 只能在 COMMIT 成功后返回 Accepted/Completed。返回字段：

- `committed_event_ids` 来自已提交行；
- `as_of_global_seq` 是该事务最后事件序号；
- `aggregate_version` 是 CAS 后版本；
- Outbox 尚未执行时，response 返回 operation/status query ref，不谎报外部动作完成。

---
## 6. 元数据、身份与配置表族

### 6.1 元数据表

| 表 | 作用 | 权威性 |
|---|---|---|
| `db_metadata` | event_store_id、format、创建信息、维护状态 | 数据库权威 |
| `schema_migrations` | 已执行 migration、checksum、执行耗时 | 数据库权威 |
| `sequence_counters` | global event sequence 与内部序列 | 数据库权威 |
| `runtime_instances` | daemon lease、进程/协议版本、启动与心跳 | 运行诊断 |
| `projection_registry` | projection 版本、cursor、状态和错误 | 投影控制面 |
| `maintenance_runs` | integrity、backup、rebuild、GC 等维护操作 | 审计/恢复 |

```sql
CREATE TABLE db_metadata (
    singleton_id INTEGER PRIMARY KEY CHECK (singleton_id = 1),
    event_store_id TEXT NOT NULL UNIQUE,
    database_format_version INTEGER NOT NULL,
    application_id INTEGER NOT NULL,
    created_at_us INTEGER NOT NULL,
    last_opened_at_us INTEGER NOT NULL,
    maintenance_mode TEXT NOT NULL DEFAULT 'normal',
    maintenance_reason TEXT,
    last_clean_shutdown_at_us INTEGER,
    last_integrity_check_at_us INTEGER,
    CHECK (maintenance_mode IN ('normal','starting','migrating','readonly_recovery','quarantine'))
);

CREATE TABLE schema_migrations (
    migration_id TEXT PRIMARY KEY,
    version INTEGER NOT NULL UNIQUE,
    name TEXT NOT NULL,
    checksum TEXT NOT NULL,
    applied_at_us INTEGER NOT NULL,
    duration_ms INTEGER NOT NULL,
    tool_version TEXT NOT NULL,
    execution_mode TEXT NOT NULL CHECK (execution_mode IN ('transactional','online','backfill'))
);

CREATE TABLE runtime_instances (
    instance_id TEXT PRIMARY KEY,
    process_id INTEGER NOT NULL,
    binary_version TEXT NOT NULL,
    protocol_version INTEGER NOT NULL,
    endpoint TEXT NOT NULL,
    lease_token TEXT NOT NULL,
    started_at_us INTEGER NOT NULL,
    heartbeat_at_us INTEGER NOT NULL,
    lease_until_us INTEGER NOT NULL,
    stopped_at_us INTEGER,
    stop_reason TEXT
);
CREATE INDEX idx_runtime_instances__lease
    ON runtime_instances(lease_until_us, heartbeat_at_us);

CREATE TABLE maintenance_runs (
    maintenance_run_id TEXT PRIMARY KEY,
    kind TEXT NOT NULL CHECK (kind IN ('migration','backfill','backup','restore','quick_check','integrity_check','projection_rebuild','fts_rebuild','gc','privacy_purge')),
    state TEXT NOT NULL CHECK (state IN ('requested','running','completed','failed','cancelled','blocked')),
    requested_by_actor_id TEXT,
    target_json TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(target_json)),
    progress_json TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(progress_json)),
    report_blob_id TEXT,
    started_at_us INTEGER,
    finished_at_us INTEGER,
    created_at_us INTEGER NOT NULL,
    error_json TEXT
);
CREATE INDEX idx_maintenance_runs__state_time
    ON maintenance_runs(state, created_at_us DESC);
```

`PRAGMA user_version` 镜像当前 `database_format_version`，便于工具快速预检；`schema_migrations` 才是 migration 历史与 checksum 的权威。两者不一致时禁止业务写入并进入 migration/recovery 诊断。

### 6.2 Project、Worktree 与信任

```sql
CREATE TABLE projects (
    project_id TEXT PRIMARY KEY,
    canonical_path TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    trust_state TEXT NOT NULL,
    config_digest TEXT,
    config_revision INTEGER NOT NULL DEFAULT 0,
    created_at_us INTEGER NOT NULL,
    updated_at_us INTEGER NOT NULL,
    last_event_seq INTEGER NOT NULL DEFAULT 0,
    version INTEGER NOT NULL DEFAULT 0,
    archived_at_us INTEGER,
    CHECK (trust_state IN ('unknown','trusted','restricted','revoked'))
);

CREATE TABLE worktrees (
    worktree_id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(project_id),
    path TEXT NOT NULL,
    canonical_path TEXT NOT NULL,
    source TEXT NOT NULL CHECK (source IN ('main','git','apex_isolated','external')),
    branch TEXT,
    base_commit TEXT,
    is_primary INTEGER NOT NULL DEFAULT 0 CHECK (is_primary IN (0,1)),
    status TEXT NOT NULL DEFAULT 'active',
    created_at_us INTEGER NOT NULL,
    updated_at_us INTEGER NOT NULL,
    UNIQUE(project_id, canonical_path)
);
CREATE UNIQUE INDEX uq_worktrees__one_primary
    ON worktrees(project_id) WHERE is_primary = 1 AND status = 'active';
CREATE INDEX idx_worktrees__project_status
    ON worktrees(project_id, status, updated_at_us DESC);

CREATE TABLE project_trust_revisions (
    trust_revision_id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(project_id),
    state TEXT NOT NULL,
    actor_id TEXT NOT NULL,
    reason TEXT,
    policy_digest TEXT,
    created_at_us INTEGER NOT NULL,
    event_seq INTEGER NOT NULL
);

CREATE TABLE config_revisions (
    config_revision_id TEXT PRIMARY KEY,
    scope_type TEXT NOT NULL CHECK (scope_type IN ('user','project','session')),
    scope_id TEXT,
    format_version INTEGER NOT NULL,
    source_path TEXT,
    raw_json TEXT NOT NULL CHECK (json_valid(raw_json)),
    effective_json TEXT NOT NULL CHECK (json_valid(effective_json)),
    digest TEXT NOT NULL,
    created_at_us INTEGER NOT NULL,
    supersedes_id TEXT REFERENCES config_revisions(config_revision_id)
);
CREATE INDEX idx_config_revisions__scope_time
    ON config_revisions(scope_type, scope_id, created_at_us DESC);
```

项目级配置文件仍是团队可提交资产；数据库保存解析后的版本、digest、诊断和生效关系，不允许“数据库配置覆盖文件但不留下 revision”。用户 TOML、项目 config、Session 临时设置按总体架构规定的优先级合并。

### 6.3 Actor、Client 与访问审计

```sql
CREATE TABLE actors (
    actor_id TEXT PRIMARY KEY,
    actor_type TEXT NOT NULL CHECK (actor_type IN ('user','apex_core','agent','plugin','hook','system')),
    display_name TEXT NOT NULL,
    principal_ref TEXT,
    parent_actor_id TEXT REFERENCES actors(actor_id),
    capabilities_json TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(capabilities_json)),
    created_at_us INTEGER NOT NULL,
    revoked_at_us INTEGER
);

CREATE TABLE clients (
    client_id TEXT PRIMARY KEY,
    client_type TEXT NOT NULL CHECK (client_type IN ('tui','tauri','web','plugin','internal')),
    label TEXT NOT NULL,
    protocol_min INTEGER NOT NULL,
    protocol_max INTEGER NOT NULL,
    first_seen_at_us INTEGER NOT NULL,
    last_seen_at_us INTEGER NOT NULL,
    revoked_at_us INTEGER
);

CREATE TABLE access_audit (
    access_audit_id TEXT PRIMARY KEY,
    actor_id TEXT NOT NULL REFERENCES actors(actor_id),
    client_id TEXT REFERENCES clients(client_id),
    action TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id TEXT,
    outcome TEXT NOT NULL,
    redaction_level TEXT NOT NULL,
    request_id TEXT,
    created_at_us INTEGER NOT NULL,
    detail_json TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(detail_json))
);
CREATE INDEX idx_access_audit__resource_time
    ON access_audit(resource_type, resource_id, created_at_us DESC);
```

Actor、principal、client 和 transport 身份由认证连接注入；Command body 中同名字段不能覆盖它们。访问审计不得保存 secret 原文。

---

## 7. Session、Conversation、Run 与 Provider 表族

### 7.1 Session 与分支

```sql
CREATE TABLE sessions (
    session_id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(project_id),
    worktree_id TEXT REFERENCES worktrees(worktree_id),
    state TEXT NOT NULL,
    title TEXT NOT NULL DEFAULT '',
    current_branch_id TEXT,
    active_run_id TEXT,
    default_provider TEXT,
    default_model TEXT,
    created_by_actor_id TEXT NOT NULL REFERENCES actors(actor_id),
    created_at_us INTEGER NOT NULL,
    updated_at_us INTEGER NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    last_event_seq INTEGER NOT NULL DEFAULT 0,
    archived_at_us INTEGER,
    CHECK (state IN ('idle','awaiting_input','running','awaiting_approval','blocked','completed','archived'))
);

CREATE TABLE session_branches (
    branch_id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES sessions(session_id),
    parent_branch_id TEXT REFERENCES session_branches(branch_id),
    fork_message_id TEXT,
    name TEXT NOT NULL,
    head_message_seq INTEGER NOT NULL DEFAULT 0,
    head_message_id TEXT,
    created_at_us INTEGER NOT NULL,
    created_by_actor_id TEXT NOT NULL REFERENCES actors(actor_id),
    is_current INTEGER NOT NULL DEFAULT 0 CHECK (is_current IN (0,1))
);
CREATE UNIQUE INDEX uq_session_branches__current
    ON session_branches(session_id) WHERE is_current = 1;
CREATE INDEX idx_session_branches__session_created
    ON session_branches(session_id, created_at_us, branch_id);
```

SQLite 不能直接表达跨表的 `sessions.current_branch_id -> session_branches.branch_id` 复合一致性，因此由 Session aggregate 在同一事务中校验；实现层可在后续 migration 加 deferred trigger，但不以 trigger 替代聚合决策。

### 7.2 Message、Turn 与 Run

```sql
CREATE TABLE messages (
    message_id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES sessions(session_id),
    branch_id TEXT NOT NULL REFERENCES session_branches(branch_id),
    parent_message_id TEXT REFERENCES messages(message_id),
    message_seq INTEGER NOT NULL CHECK (message_seq > 0),
    role TEXT NOT NULL CHECK (role IN ('system','user','assistant','tool','developer','event')),
    author_actor_id TEXT REFERENCES actors(actor_id),
    content_inline TEXT,
    content_blob_id TEXT,
    content_digest TEXT,
    content_size_bytes INTEGER,
    content_format TEXT NOT NULL DEFAULT 'markdown',
    visibility TEXT NOT NULL DEFAULT 'normal',
    created_at_us INTEGER NOT NULL,
    committed_at_us INTEGER NOT NULL,
    event_seq INTEGER NOT NULL,
    UNIQUE(branch_id, message_seq),
    CHECK ((content_inline IS NOT NULL) OR (content_blob_id IS NOT NULL) OR role = 'event')
);
CREATE INDEX idx_messages__branch_seq
    ON messages(branch_id, message_seq);
CREATE INDEX idx_messages__session_created
    ON messages(session_id, created_at_us, message_id);

CREATE TABLE message_parts (
    message_part_id TEXT PRIMARY KEY,
    message_id TEXT NOT NULL REFERENCES messages(message_id) ON DELETE CASCADE,
    ordinal INTEGER NOT NULL CHECK (ordinal >= 0),
    part_type TEXT NOT NULL CHECK (part_type IN ('text','attachment','tool_call','tool_result','artifact_ref','citation')),
    inline_text TEXT,
    blob_id TEXT REFERENCES blobs(blob_id),
    artifact_revision_id TEXT,
    tool_call_id TEXT,
    metadata_json TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(metadata_json)),
    content_digest TEXT,
    created_at_us INTEGER NOT NULL,
    UNIQUE(message_id, ordinal),
    CHECK ((inline_text IS NOT NULL) OR (blob_id IS NOT NULL) OR (artifact_revision_id IS NOT NULL) OR (tool_call_id IS NOT NULL) OR part_type = 'citation')
);
CREATE INDEX idx_message_parts__message_ordinal
    ON message_parts(message_id, ordinal);

CREATE TABLE runs (
    run_id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES sessions(session_id),
    project_id TEXT NOT NULL REFERENCES projects(project_id),
    branch_id TEXT REFERENCES session_branches(branch_id),
    state TEXT NOT NULL,
    trigger TEXT NOT NULL,
    spec_id TEXT,
    workflow_id TEXT,
    current_turn_ordinal INTEGER NOT NULL DEFAULT 0,
    outcome_code TEXT,
    block_reason_json TEXT,
    started_at_us INTEGER,
    finished_at_us INTEGER,
    created_at_us INTEGER NOT NULL,
    updated_at_us INTEGER NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    last_event_seq INTEGER NOT NULL DEFAULT 0,
    CHECK (state IN ('queued','running','waiting_approval','waiting_user','paused','cancel_requested','blocked','completed','failed','cancelled','interrupted'))
);
-- ADR-0017（跨文档一致性审查）：原 CHECK 仅 8 值且写作 awaiting_approval，
-- 缺 waiting_user/paused/cancel_requested，导致 API 的 PauseRun 命令无法落库。
-- 现与领域模型 §5.4 的 11 态对齐。
CREATE INDEX idx_runs__session_time ON runs(session_id, created_at_us DESC, run_id DESC);
CREATE INDEX idx_runs__project_state ON runs(project_id, state, updated_at_us DESC);
CREATE INDEX idx_runs__active ON runs(session_id, updated_at_us DESC)
    WHERE state IN ('queued','running','awaiting_approval','blocked');

CREATE TABLE turns (
    turn_id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL REFERENCES runs(run_id),
    ordinal INTEGER NOT NULL CHECK (ordinal > 0),
    state TEXT NOT NULL,
    input_message_id TEXT REFERENCES messages(message_id),
    assistant_message_id TEXT REFERENCES messages(message_id),
    checkpoint_id TEXT,
    provider_call_id TEXT,
    started_at_us INTEGER,
    finished_at_us INTEGER,
    outcome_code TEXT,
    usage_json TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(usage_json)),
    created_at_us INTEGER NOT NULL,
    updated_at_us INTEGER NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    UNIQUE(run_id, ordinal),
    CHECK (state IN ('queued','running','awaiting_tool','awaiting_approval','completed','failed','cancelled','interrupted','blocked'))
);
CREATE INDEX idx_turns__run_ordinal ON turns(run_id, ordinal);

CREATE TABLE provider_calls (
    provider_call_id TEXT PRIMARY KEY,
    turn_id TEXT NOT NULL REFERENCES turns(turn_id),
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    request_blob_id TEXT,
    response_blob_id TEXT,
    request_digest TEXT,
    response_digest TEXT,
    state TEXT NOT NULL,
    attempt INTEGER NOT NULL DEFAULT 1,
    input_tokens INTEGER,
    output_tokens INTEGER,
    cached_tokens INTEGER,
    started_at_us INTEGER,
    finished_at_us INTEGER,
    error_json TEXT,
    operation_id TEXT,
    created_at_us INTEGER NOT NULL,
    CHECK (state IN ('requested','streaming','completed','failed','cancelled','interrupted','unknown'))
);
CREATE INDEX idx_provider_calls__turn ON provider_calls(turn_id, attempt);
CREATE INDEX idx_provider_calls__provider_time ON provider_calls(provider, created_at_us DESC);
```

`messages.content_inline/content_blob_id` 保存主文本的快速读取副本；`message_parts` 是附件、多模态内容、Tool 引用和 Artifact 引用的规范化顺序结构。两者由同一 Message repository 事务写入，主文本副本必须与 ordinal=0 的 text part digest 一致。`message_seq` 只在单 Session/branch 内解释；`turn.ordinal` 只在 Run 内解释；任何跨域 Query 都返回对应的 `event_seq` 或 `as_of_global_seq`，不把它们拼成一个“万能 seq”。

### 7.3 Operation Journal

```sql
CREATE TABLE operation_journal (
    operation_id TEXT PRIMARY KEY,
    operation_type TEXT NOT NULL,
    project_id TEXT REFERENCES projects(project_id),
    session_id TEXT REFERENCES sessions(session_id),
    run_id TEXT REFERENCES runs(run_id),
    actor_id TEXT NOT NULL REFERENCES actors(actor_id),
    state TEXT NOT NULL,
    idempotency_key TEXT,
    intent_json TEXT NOT NULL CHECK (json_valid(intent_json)),
    result_json TEXT,
    result_blob_id TEXT,
    external_ref TEXT,
    retry_count INTEGER NOT NULL DEFAULT 0,
    next_retry_at_us INTEGER,
    started_at_us INTEGER,
    completed_at_us INTEGER,
    last_error_json TEXT,
    created_at_us INTEGER NOT NULL,
    updated_at_us INTEGER NOT NULL,
    CHECK (state IN ('intent','leased','running','succeeded','failed','cancelled','interrupted','unknown','compensating','compensated'))
);
CREATE UNIQUE INDEX uq_operation_journal__idempotency
    ON operation_journal(operation_type, idempotency_key)
    WHERE idempotency_key IS NOT NULL;
CREATE INDEX idx_operation_journal__recovery
    ON operation_journal(state, next_retry_at_us, updated_at_us);
```

Operation Journal 是外部副作用的恢复事实，不等于 Domain Event。Bash 等无法可靠探测的操作在进程崩溃后必须进入 `unknown`，由 reconcile 或用户决定，禁止自动重复执行。

---

## 8. Domain Event、幂等与 Outbox 表族

### 8.1 Domain Event Store

```sql
CREATE TABLE domain_events (
    global_seq INTEGER PRIMARY KEY CHECK (global_seq > 0),
    event_id TEXT NOT NULL UNIQUE,
    event_store_id TEXT NOT NULL,
    project_id TEXT REFERENCES projects(project_id),
    session_id TEXT REFERENCES sessions(session_id),
    run_id TEXT REFERENCES runs(run_id),
    aggregate_type TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    aggregate_version INTEGER NOT NULL CHECK (aggregate_version > 0),
    actor_id TEXT NOT NULL REFERENCES actors(actor_id),
    event_type TEXT NOT NULL,
    schema_version INTEGER NOT NULL CHECK (schema_version > 0),
    occurred_at_us INTEGER NOT NULL,
    committed_at_us INTEGER NOT NULL,
    correlation_id TEXT NOT NULL,
    causation_event_id TEXT REFERENCES domain_events(event_id),
    operation_id TEXT,
    redaction_level TEXT NOT NULL DEFAULT 'none',
    payload_json TEXT NOT NULL CHECK (json_valid(payload_json)),
    payload_digest TEXT NOT NULL,
    blob_id TEXT REFERENCES blobs(blob_id),
    UNIQUE(aggregate_type, aggregate_id, aggregate_version)
);
CREATE INDEX idx_domain_events__project_seq
    ON domain_events(project_id, global_seq);
CREATE INDEX idx_domain_events__aggregate_seq
    ON domain_events(aggregate_type, aggregate_id, global_seq);
CREATE INDEX idx_domain_events__correlation_seq
    ON domain_events(correlation_id, global_seq);
CREATE INDEX idx_domain_events__type_seq
    ON domain_events(event_type, global_seq);
CREATE INDEX idx_domain_events__session_seq
    ON domain_events(session_id, global_seq)
    WHERE session_id IS NOT NULL;
```

约束：

- `global_seq` 由 Core 分配，不接受客户端指定；
- `event_store_id` 必须等于 `db_metadata.event_store_id`；
- `event_id`、`global_seq`、`aggregate_type+aggregate_id+aggregate_version` 唯一；
- 行不可 UPDATE/DELETE；更正使用新事件；
- 事件 payload 原文保留，读取时通过纯 upcaster 转成当前内存类型；
- `spec.skipped` 的 wire event type 与 Rust 领域事件 `SpecSkipped` 的映射在 Event Registry 中固定，数据库保存规范化 wire 名称。

在 Rust 实现中，Event Store repository 只暴露 append/read/verify；生产运行时启用 authorizer 或受控触发器阻止普通连接 UPDATE/DELETE。Migration/maintenance 使用明确的维护上下文，不通过“隐藏 SQL”绕过审计。

### 8.2 Command Dedup 与结果

```sql
CREATE TABLE command_dedup (
    command_id TEXT PRIMARY KEY,
    command_type TEXT NOT NULL,
    actor_id TEXT NOT NULL REFERENCES actors(actor_id),
    client_id TEXT REFERENCES clients(client_id),
    payload_digest TEXT NOT NULL,
    status TEXT NOT NULL,
    response_json TEXT,
    operation_id TEXT,
    first_seen_at_us INTEGER NOT NULL,
    committed_at_us INTEGER,
    expires_at_us INTEGER,
    CHECK (status IN ('processing','accepted','completed','rejected','in_progress','unknown'))
);
CREATE INDEX idx_command_dedup__operation ON command_dedup(operation_id);
CREATE INDEX idx_command_dedup__expiry ON command_dedup(expires_at_us)
    WHERE expires_at_us IS NOT NULL;
```

同 `command_id` + 同 digest + 同 actor 返回第一次结果；digest 或 actor 不同返回 `IDEMPOTENCY_KEY_REUSED`。`rejected` 也必须保留结果，因为 API 协议允许 Duplicate 返回原始 Rejected。安全关键 Command 的 dedup 不因普通 TTL 自动删除。

### 8.3 Outbox

```sql
CREATE TABLE outbox (
    outbox_id TEXT PRIMARY KEY,
    event_id TEXT REFERENCES domain_events(event_id),
    operation_id TEXT REFERENCES operation_journal(operation_id),
    kind TEXT NOT NULL,
    destination TEXT NOT NULL,
    payload_json TEXT NOT NULL CHECK (json_valid(payload_json)),
    dedup_key TEXT NOT NULL,
    state TEXT NOT NULL DEFAULT 'pending',
    available_at_us INTEGER NOT NULL,
    lease_owner TEXT,
    lease_until_us INTEGER,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    last_error_json TEXT,
    created_at_us INTEGER NOT NULL,
    updated_at_us INTEGER NOT NULL,
    completed_at_us INTEGER,
    CHECK (state IN ('pending','leased','running','succeeded','failed','dead_letter','cancelled'))
);
CREATE UNIQUE INDEX uq_outbox__dedup ON outbox(destination, dedup_key);
CREATE INDEX idx_outbox__claim ON outbox(state, available_at_us, lease_until_us, outbox_id);

CREATE TABLE outbox_attempts (
    outbox_attempt_id TEXT PRIMARY KEY,
    outbox_id TEXT NOT NULL REFERENCES outbox(outbox_id),
    attempt INTEGER NOT NULL,
    worker_id TEXT NOT NULL,
    started_at_us INTEGER NOT NULL,
    finished_at_us INTEGER,
    outcome TEXT,
    error_json TEXT,
    UNIQUE(outbox_id, attempt)
);
```

Outbox 目标包括 `event_broadcast`、`artifact_materialize`、`checkpoint_materialize`、`markdown_reconcile`、`snapshot_index`、`diagnostic_export` 等。事件广播允许重复，客户端按 event_id/global_seq 幂等；文件 materialize 必须按 digest 检查并可重试。

### 8.4 Event Consumer 与游标

```sql
CREATE TABLE event_consumers (
    consumer_name TEXT PRIMARY KEY,
    consumer_version TEXT NOT NULL,
    cursor_global_seq INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL,
    last_success_at_us INTEGER,
    last_error_json TEXT,
    updated_at_us INTEGER NOT NULL,
    CHECK (status IN ('active','paused','rebuilding','failed','disabled'))
);
CREATE INDEX idx_event_consumers__status ON event_consumers(status, cursor_global_seq);
```

客户端 durable cursor 不存进服务端的 `event_consumers`，除非用户显式启用跨设备/重连书签；普通客户端自行保存 `event_store_id + global_seq`。服务端只保存 projection、outbox 和内部 consumer cursor。

---
## 9. Spec、Artifact 与 Markdown 镜像表族

### 9.1 语义边界

Spec aggregate 管理阶段、head、review、gate 和 skip 决策；Artifact Revision 是不可变内容版本。`apex/specs/` 是可编辑 Markdown 镜像，不是绕过 Command 的第二数据库。

### 9.2 表设计

```sql
CREATE TABLE specs (
    spec_id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(project_id),
    session_id TEXT REFERENCES sessions(session_id),
    feature_key TEXT NOT NULL,
    title TEXT NOT NULL,
    stage TEXT NOT NULL,
    state TEXT NOT NULL,
    skipped INTEGER NOT NULL DEFAULT 0 CHECK (skipped IN (0,1)),
    skip_reason TEXT,
    current_artifact_kind TEXT,
    created_by_actor_id TEXT NOT NULL REFERENCES actors(actor_id),
    created_at_us INTEGER NOT NULL,
    updated_at_us INTEGER NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    last_event_seq INTEGER NOT NULL DEFAULT 0,
    archived_at_us INTEGER,
    UNIQUE(project_id, feature_key),
    CHECK (stage IN ('requirements','design','tasks','implementation','verification','completed')),
    CHECK (state IN ('draft','awaiting_review','approved','implementing','verifying','completed','blocked','cancelled')),
    CHECK ((skipped = 0) OR (skip_reason IS NOT NULL))
);

CREATE TABLE artifacts (
    artifact_id TEXT PRIMARY KEY,
    spec_id TEXT NOT NULL REFERENCES specs(spec_id),
    kind TEXT NOT NULL,
    logical_path TEXT NOT NULL,
    head_revision_id TEXT,
    status TEXT NOT NULL,
    created_at_us INTEGER NOT NULL,
    updated_at_us INTEGER NOT NULL,
    UNIQUE(spec_id, kind),
    UNIQUE(spec_id, logical_path)
);

CREATE TABLE artifact_revisions (
    artifact_revision_id TEXT PRIMARY KEY,
    artifact_id TEXT NOT NULL REFERENCES artifacts(artifact_id),
    revision_number INTEGER NOT NULL CHECK (revision_number > 0),
    parent_revision_id TEXT REFERENCES artifact_revisions(artifact_revision_id),
    merge_parent_revision_id TEXT REFERENCES artifact_revisions(artifact_revision_id),
    format_version INTEGER NOT NULL,
    content_inline TEXT,
    content_blob_id TEXT REFERENCES blobs(blob_id),
    content_digest TEXT NOT NULL,
    content_size_bytes INTEGER NOT NULL,
    source TEXT NOT NULL CHECK (source IN ('agent','user','external_edit','migration','merge','recovery')),
    source_path TEXT,
    migration_source_revision_id TEXT,
    created_by_actor_id TEXT NOT NULL REFERENCES actors(actor_id),
    created_at_us INTEGER NOT NULL,
    event_seq INTEGER NOT NULL,
    UNIQUE(artifact_id, revision_number),
    UNIQUE(artifact_id, content_digest),
    CHECK ((content_inline IS NOT NULL) <> (content_blob_id IS NOT NULL))
);
CREATE INDEX idx_artifact_revisions__artifact_time
    ON artifact_revisions(artifact_id, revision_number DESC);

CREATE TABLE artifact_reviews (
    review_id TEXT PRIMARY KEY,
    spec_id TEXT NOT NULL REFERENCES specs(spec_id),
    artifact_revision_id TEXT NOT NULL REFERENCES artifact_revisions(artifact_revision_id),
    gate TEXT NOT NULL,
    reviewer_actor_id TEXT NOT NULL REFERENCES actors(actor_id),
    decision TEXT NOT NULL CHECK (decision IN ('approved','changes_requested','rejected','superseded')),
    comment_inline TEXT,
    comment_blob_id TEXT REFERENCES blobs(blob_id),
    decided_at_us INTEGER NOT NULL,
    event_seq INTEGER NOT NULL
);
CREATE INDEX idx_artifact_reviews__revision
    ON artifact_reviews(artifact_revision_id, decided_at_us DESC);

CREATE TABLE spec_invalidations (
    invalidation_id TEXT PRIMARY KEY,
    spec_id TEXT NOT NULL REFERENCES specs(spec_id),
    source_revision_id TEXT NOT NULL REFERENCES artifact_revisions(artifact_revision_id),
    invalidated_kind TEXT NOT NULL,
    reason TEXT NOT NULL,
    created_at_us INTEGER NOT NULL,
    resolved_at_us INTEGER,
    event_seq INTEGER NOT NULL
);
```

`artifacts.head_revision_id` 和 revision 插入在同一事务更新。由于创建 artifact 时 head 可为空，FK 可在 migration 完成后通过 rebuild 增加，或由 repository 执行强校验；任何 Query 不得返回不存在的 head。

### 9.3 Materialization Intent

```sql
CREATE TABLE materialization_intents (
    materialization_id TEXT PRIMARY KEY,
    resource_type TEXT NOT NULL CHECK (resource_type IN ('artifact','checkpoint','memory','export')),
    resource_id TEXT NOT NULL,
    revision_id TEXT,
    target_path TEXT NOT NULL,
    expected_digest TEXT NOT NULL,
    observed_digest TEXT,
    state TEXT NOT NULL,
    temp_path TEXT,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    last_error_json TEXT,
    created_at_us INTEGER NOT NULL,
    updated_at_us INTEGER NOT NULL,
    completed_at_us INTEGER,
    CHECK (state IN ('pending','writing_temp','renamed','verified','failed','conflict','orphaned'))
);
CREATE UNIQUE INDEX uq_materialization__target_pending
    ON materialization_intents(target_path)
    WHERE state IN ('pending','writing_temp','renamed');
CREATE INDEX idx_materialization__recovery
    ON materialization_intents(state, updated_at_us);
```

镜像协议：

1. Command 事务创建 revision、更新 head、追加事件并写 materialization intent/outbox；
2. Worker 在目标目录创建同文件系统临时文件，写入后 flush + fsync；
3. 通过原子 rename 替换目标；必要时 fsync 父目录；
4. 重新读取或 stat/校验 digest；
5. 新事务将 intent 标为 verified；
6. watcher 看到与 pending intent 相同 digest 时识别为 Core 自写，不重复导入；
7. watcher 看到未知 digest 时，通过 `ImportExternalArtifactEdit` Command 创建新 revision；
8. DB 与文件都发生不同修改时生成冲突 revision/诊断，禁止 last-writer-wins。

---

## 10. Workflow、Agent、Attempt 与写路径声明

### 10.1 Workflow 图

```sql
CREATE TABLE workflows (
    workflow_id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(project_id),
    session_id TEXT REFERENCES sessions(session_id),
    run_id TEXT REFERENCES runs(run_id),
    spec_id TEXT REFERENCES specs(spec_id),
    state TEXT NOT NULL,
    graph_version INTEGER NOT NULL DEFAULT 1,
    scheduler_version TEXT NOT NULL,
    created_at_us INTEGER NOT NULL,
    updated_at_us INTEGER NOT NULL,
    started_at_us INTEGER,
    finished_at_us INTEGER,
    version INTEGER NOT NULL DEFAULT 0,
    last_event_seq INTEGER NOT NULL DEFAULT 0,
    CHECK (state IN ('draft','ready','running','paused','blocked','completed','failed','cancelled','interrupted'))
);

CREATE TABLE workflow_nodes (
    node_id TEXT PRIMARY KEY,
    workflow_id TEXT NOT NULL REFERENCES workflows(workflow_id),
    node_key TEXT NOT NULL,
    node_type TEXT NOT NULL,
    title TEXT NOT NULL,
    state TEXT NOT NULL,
    priority INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 1,
    timeout_ms INTEGER,
    agent_profile TEXT,
    input_json TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(input_json)),
    output_json TEXT,
    current_attempt INTEGER NOT NULL DEFAULT 0,
    created_at_us INTEGER NOT NULL,
    updated_at_us INTEGER NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    UNIQUE(workflow_id, node_key),
    CHECK (state IN ('pending','ready','claiming','queued','running','verifying','completed','blocked','failed','cancelled','interrupted','invalidated'))
);
CREATE INDEX idx_workflow_nodes__sched
    ON workflow_nodes(workflow_id, state, priority DESC, node_id);

CREATE TABLE workflow_edges (
    workflow_id TEXT NOT NULL REFERENCES workflows(workflow_id),
    from_node_id TEXT NOT NULL REFERENCES workflow_nodes(node_id),
    to_node_id TEXT NOT NULL REFERENCES workflow_nodes(node_id),
    edge_type TEXT NOT NULL DEFAULT 'requires',
    condition_json TEXT,
    PRIMARY KEY(workflow_id, from_node_id, to_node_id),
    CHECK (from_node_id <> to_node_id)
) WITHOUT ROWID;
CREATE INDEX idx_workflow_edges__to ON workflow_edges(workflow_id, to_node_id);

CREATE TABLE node_attempts (
    node_attempt_id TEXT PRIMARY KEY,
    node_id TEXT NOT NULL REFERENCES workflow_nodes(node_id),
    attempt INTEGER NOT NULL CHECK (attempt > 0),
    run_id TEXT REFERENCES runs(run_id),
    agent_id TEXT,
    operation_id TEXT REFERENCES operation_journal(operation_id),
    state TEXT NOT NULL,
    baseline_snapshot_id TEXT,
    result_snapshot_id TEXT,
    started_at_us INTEGER,
    finished_at_us INTEGER,
    outcome_json TEXT,
    created_at_us INTEGER NOT NULL,
    UNIQUE(node_id, attempt),
    CHECK (state IN ('queued','leased','running','blocked','completed','failed','cancelled','interrupted'))
);
```

DAG 无环性、edge 两端属于同 workflow、ready 条件、重试创建新 attempt 等复杂约束由 Workflow aggregate + scheduler transaction 验证。加载图后必须运行 cycle detection；migration/import 也不能跳过。

### 10.2 Agent

```sql
CREATE TABLE agents (
    agent_id TEXT PRIMARY KEY,
    parent_agent_id TEXT REFERENCES agents(agent_id),
    project_id TEXT NOT NULL REFERENCES projects(project_id),
    session_id TEXT REFERENCES sessions(session_id),
    run_id TEXT REFERENCES runs(run_id),
    node_attempt_id TEXT REFERENCES node_attempts(node_attempt_id),
    actor_id TEXT NOT NULL REFERENCES actors(actor_id),
    profile TEXT NOT NULL,
    isolation_mode TEXT NOT NULL,
    state TEXT NOT NULL,
    worktree_id TEXT REFERENCES worktrees(worktree_id),
    context_checkpoint_id TEXT,
    spawned_at_us INTEGER NOT NULL,
    finished_at_us INTEGER,
    version INTEGER NOT NULL DEFAULT 0,
    last_event_seq INTEGER NOT NULL DEFAULT 0,
    CHECK (isolation_mode IN ('shared_readonly','path_claim','worktree')),
    CHECK (state IN ('created','running','waiting','blocked','completed','failed','cancelled','interrupted'))
);
CREATE INDEX idx_agents__parent ON agents(parent_agent_id, spawned_at_us);
CREATE INDEX idx_agents__run_state ON agents(run_id, state, spawned_at_us);
```

### 10.3 Write Claim

```sql
CREATE TABLE write_claims (
    claim_id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(project_id),
    worktree_id TEXT NOT NULL REFERENCES worktrees(worktree_id),
    owner_agent_id TEXT NOT NULL REFERENCES agents(agent_id),
    owner_attempt_id TEXT REFERENCES node_attempts(node_attempt_id),
    state TEXT NOT NULL,
    lease_token TEXT NOT NULL,
    acquired_at_us INTEGER NOT NULL,
    lease_until_us INTEGER NOT NULL,
    released_at_us INTEGER,
    version INTEGER NOT NULL DEFAULT 0,
    CHECK (state IN ('requested','active','releasing','released','expired','revoked'))
);

CREATE TABLE write_claim_scopes (
    claim_id TEXT NOT NULL REFERENCES write_claims(claim_id) ON DELETE CASCADE,
    ordinal INTEGER NOT NULL,
    canonical_path TEXT NOT NULL,
    scope_kind TEXT NOT NULL CHECK (scope_kind IN ('file','directory','glob')),
    recursive INTEGER NOT NULL DEFAULT 0 CHECK (recursive IN (0,1)),
    path_key TEXT NOT NULL,
    PRIMARY KEY(claim_id, ordinal)
) WITHOUT ROWID;
CREATE INDEX idx_write_claim_scopes__lookup
    ON write_claim_scopes(path_key, scope_kind, claim_id);
CREATE INDEX idx_write_claims__active_lease
    ON write_claims(worktree_id, lease_until_us)
    WHERE state = 'active';
```

路径先经过平台适配器 canonicalize，记录 separator、case sensitivity 和 symlink resolution 结果。SQLite 索引只能加速候选集合，目录包含、glob 相交、大小写和 symlink 冲突必须由应用层在 `BEGIN IMMEDIATE` 内判定。精确同路径可增加 partial unique 辅助约束，但不能据此声称不存在父子目录重叠。

---

## 11. Tool、Permission、Rule 与诊断表族

### 11.1 Tool Call

```sql
CREATE TABLE tool_calls (
    tool_call_id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL REFERENCES runs(run_id),
    turn_id TEXT REFERENCES turns(turn_id),
    agent_id TEXT REFERENCES agents(agent_id),
    operation_id TEXT REFERENCES operation_journal(operation_id),
    tool_name TEXT NOT NULL,
    tool_version TEXT,
    state TEXT NOT NULL,
    risk_level TEXT NOT NULL,
    arguments_json TEXT NOT NULL CHECK (json_valid(arguments_json)),
    arguments_digest TEXT NOT NULL,
    redacted_arguments_json TEXT NOT NULL CHECK (json_valid(redacted_arguments_json)),
    stdout_blob_id TEXT REFERENCES blobs(blob_id),
    stderr_blob_id TEXT REFERENCES blobs(blob_id),
    result_json TEXT,
    exit_code INTEGER,
    started_at_us INTEGER,
    finished_at_us INTEGER,
    created_at_us INTEGER NOT NULL,
    updated_at_us INTEGER NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    last_event_seq INTEGER NOT NULL DEFAULT 0,
    CHECK (state IN ('requested','validating','awaiting_permission','denied','awaiting_claim','preflight','executing','postflight','succeeded','succeeded_with_violations','failed','interrupted','reconcile_required','cancelled'))
);
CREATE INDEX idx_tool_calls__run_time ON tool_calls(run_id, created_at_us, tool_call_id);
CREATE INDEX idx_tool_calls__pending ON tool_calls(state, created_at_us)
    WHERE state IN ('requested','validating','awaiting_permission','awaiting_claim','preflight','executing','postflight');
```

原始 arguments 如果包含 secret，不直接持久化；保存安全 canonical 结构和 redacted view。执行所需 secret 由 CredentialStore 在 operation 执行时按 capability 注入。

### 11.2 Permission

```sql
CREATE TABLE permission_requests (
    permission_request_id TEXT PRIMARY KEY,
    tool_call_id TEXT NOT NULL REFERENCES tool_calls(tool_call_id),
    project_id TEXT NOT NULL REFERENCES projects(project_id),
    session_id TEXT REFERENCES sessions(session_id),
    requested_by_actor_id TEXT NOT NULL REFERENCES actors(actor_id),
    permission_kind TEXT NOT NULL,
    resource_json TEXT NOT NULL CHECK (json_valid(resource_json)),
    risk_level TEXT NOT NULL,
    reason TEXT NOT NULL,
    state TEXT NOT NULL,
    expires_at_us INTEGER,
    decided_at_us INTEGER,
    decision_id TEXT,
    created_at_us INTEGER NOT NULL,
    updated_at_us INTEGER NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    last_event_seq INTEGER NOT NULL DEFAULT 0,
    CHECK (state IN ('pending','approved','denied','expired','cancelled','superseded'))
);
CREATE INDEX idx_permission_requests__pending
    ON permission_requests(project_id, created_at_us)
    WHERE state = 'pending';

CREATE TABLE permission_decisions (
    permission_decision_id TEXT PRIMARY KEY,
    permission_request_id TEXT NOT NULL UNIQUE REFERENCES permission_requests(permission_request_id),
    decided_by_actor_id TEXT NOT NULL REFERENCES actors(actor_id),
    decision TEXT NOT NULL CHECK (decision IN ('allow_once','allow_session','allow_project','deny')),
    scope_json TEXT NOT NULL CHECK (json_valid(scope_json)),
    reason TEXT,
    decided_at_us INTEGER NOT NULL,
    event_seq INTEGER NOT NULL
);

CREATE TABLE permission_rules (
    permission_rule_id TEXT PRIMARY KEY,
    scope_type TEXT NOT NULL CHECK (scope_type IN ('builtin','user','project','session')),
    scope_id TEXT,
    priority INTEGER NOT NULL,
    effect TEXT NOT NULL CHECK (effect IN ('allow','deny','ask')),
    matcher_json TEXT NOT NULL CHECK (json_valid(matcher_json)),
    source_revision_id TEXT,
    enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0,1)),
    created_at_us INTEGER NOT NULL,
    updated_at_us INTEGER NOT NULL
);
CREATE INDEX idx_permission_rules__eval
    ON permission_rules(scope_type, scope_id, enabled, priority DESC);
```

Pending approval Query 必须读取强一致表，不依赖异步 audit projection。`permission_decisions` 不更新历史决定；规则型授权另建 `permission_rules` revision。

### 11.3 Rule、Check 与 Diagnostic

```sql
CREATE TABLE rulesets (
    ruleset_id TEXT PRIMARY KEY,
    project_id TEXT REFERENCES projects(project_id),
    scope TEXT NOT NULL,
    source_path TEXT,
    source_digest TEXT NOT NULL,
    compiled_blob_id TEXT REFERENCES blobs(blob_id),
    compiler_version TEXT NOT NULL,
    status TEXT NOT NULL,
    diagnostics_count INTEGER NOT NULL DEFAULT 0,
    created_at_us INTEGER NOT NULL,
    superseded_at_us INTEGER
);

CREATE TABLE rule_checks (
    rule_check_id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(project_id),
    run_id TEXT REFERENCES runs(run_id),
    tool_call_id TEXT REFERENCES tool_calls(tool_call_id),
    snapshot_id TEXT,
    trigger TEXT NOT NULL,
    state TEXT NOT NULL,
    verdict TEXT,
    failure_kind TEXT,
    ruleset_digest TEXT NOT NULL,
    files_digest TEXT,
    started_at_us INTEGER,
    finished_at_us INTEGER,
    created_at_us INTEGER NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    last_event_seq INTEGER NOT NULL DEFAULT 0,
    CHECK (state IN ('queued','running','completed','cancelled','interrupted','unknown')),
    CHECK (verdict IS NULL OR verdict IN ('pass','fail','inconclusive','stale','skipped','waived')),
    CHECK (failure_kind IS NULL OR failure_kind IN (
        'violations_found','checker_failed','checker_timeout','runner_unavailable',
        'input_missing','input_unstable','workspace_drift','ruleset_invalid',
        'output_invalid','permission_denied','sandbox_denied','external_unknown',
        'cancelled_by_user','interrupted_by_crash','legacy_ambiguous')),
    -- state=completed 必须有 verdict；未完成不得有 verdict
    CHECK ((state = 'completed') = (verdict IS NOT NULL)),
    -- verdict=pass 不允许携带 failure_kind
    CHECK (NOT (verdict = 'pass' AND failure_kind IS NOT NULL))
);

CREATE TABLE diagnostics (
    diagnostic_id TEXT PRIMARY KEY,
    project_id TEXT REFERENCES projects(project_id),
    rule_check_id TEXT REFERENCES rule_checks(rule_check_id),
    source_type TEXT NOT NULL,
    source_id TEXT,
    severity TEXT NOT NULL,
    code TEXT NOT NULL,
    message TEXT NOT NULL,
    canonical_path TEXT,
    range_json TEXT,
    fingerprint TEXT NOT NULL,
    data_json TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(data_json)),
    created_at_us INTEGER NOT NULL,
    resolved_at_us INTEGER,
    UNIQUE(source_type, source_id, fingerprint)
);
CREATE INDEX idx_diagnostics__project_open
    ON diagnostics(project_id, severity, canonical_path)
    WHERE resolved_at_us IS NULL;
```

Rule 编译缓存可以删除重建，但 `rule_checks` outcome 和与交付相关的 diagnostics 需要按审计策略保留。

---
## 12. Checkpoint、Snapshot、Blob 与文件引用

### 12.1 Checkpoint

```sql
CREATE TABLE checkpoints (
    checkpoint_id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(project_id),
    session_id TEXT NOT NULL REFERENCES sessions(session_id),
    run_id TEXT REFERENCES runs(run_id),
    turn_id TEXT REFERENCES turns(turn_id),
    kind TEXT NOT NULL,
    format_version INTEGER NOT NULL,
    state TEXT NOT NULL,
    baseline_event_seq INTEGER NOT NULL,
    context_manifest_json TEXT NOT NULL CHECK (json_valid(context_manifest_json)),
    content_inline TEXT,
    content_blob_id TEXT REFERENCES blobs(blob_id),
    content_digest TEXT NOT NULL,
    token_estimate INTEGER,
    source_path TEXT,
    materialization_state TEXT NOT NULL DEFAULT 'pending',
    created_at_us INTEGER NOT NULL,
    event_seq INTEGER NOT NULL,
    CHECK (kind IN ('turn_start','turn_end','compaction','manual','recovery','workflow_node')),
    CHECK (state IN ('building','ready','invalid','superseded')),
    CHECK ((content_inline IS NOT NULL) <> (content_blob_id IS NOT NULL))
);
CREATE INDEX idx_checkpoints__session_time
    ON checkpoints(session_id, created_at_us DESC, checkpoint_id DESC);
CREATE INDEX idx_checkpoints__run_turn
    ON checkpoints(run_id, turn_id);
```

Checkpoint 是上下文结构化快照，不是文件系统 rollback。`baseline_event_seq` 表示构建时的业务水位；prompt assembler 必须校验 format/version、digest 和引用可访问性。

### 12.2 Snapshot

```sql
CREATE TABLE snapshots (
    snapshot_id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(project_id),
    worktree_id TEXT NOT NULL REFERENCES worktrees(worktree_id),
    run_id TEXT REFERENCES runs(run_id),
    turn_id TEXT REFERENCES turns(turn_id),
    node_attempt_id TEXT REFERENCES node_attempts(node_attempt_id),
    kind TEXT NOT NULL,
    state TEXT NOT NULL,
    content_digest TEXT,
    git_object_id TEXT,
    manifest_blob_id TEXT REFERENCES blobs(blob_id),
    base_snapshot_id TEXT REFERENCES snapshots(snapshot_id),
    retained_until_us INTEGER,
    reference_count INTEGER NOT NULL DEFAULT 0,
    created_at_us INTEGER NOT NULL,
    completed_at_us INTEGER,
    version INTEGER NOT NULL DEFAULT 0,
    last_event_seq INTEGER NOT NULL DEFAULT 0,
    CHECK (kind IN ('turn_before','turn_after','node_before','node_after','manual','pre_rollback','import')),
    CHECK (state IN ('intent','creating','ready','failed','deleting','deleted','unknown'))
);
CREATE INDEX idx_snapshots__worktree_time
    ON snapshots(worktree_id, created_at_us DESC, snapshot_id DESC);
CREATE INDEX idx_snapshots__retention
    ON snapshots(retained_until_us, reference_count)
    WHERE state = 'ready';

CREATE TABLE snapshot_restores (
    restore_id TEXT PRIMARY KEY,
    snapshot_id TEXT NOT NULL REFERENCES snapshots(snapshot_id),
    project_id TEXT NOT NULL REFERENCES projects(project_id),
    worktree_id TEXT NOT NULL REFERENCES worktrees(worktree_id),
    baseline_snapshot_id TEXT REFERENCES snapshots(snapshot_id),
    operation_id TEXT NOT NULL UNIQUE REFERENCES operation_journal(operation_id),
    state TEXT NOT NULL,
    path_scope_json TEXT NOT NULL CHECK (json_valid(path_scope_json)),
    pre_restore_digest TEXT,
    post_restore_digest TEXT,
    created_by_actor_id TEXT NOT NULL REFERENCES actors(actor_id),
    created_at_us INTEGER NOT NULL,
    completed_at_us INTEGER,
    CHECK (state IN ('requested','approved','running','completed','conflicted','failed','cancelled','unknown'))
);
```

影子 Git object 是文件内容权威，SQLite 保存 snapshot lifecycle、引用、授权和关联事件。GC 只能删除 `reference_count=0`、超过 retention、无 active restore/outbox 的 snapshot。

### 12.3 Blob 元数据与引用

```sql
CREATE TABLE blobs (
    blob_id TEXT PRIMARY KEY,
    digest TEXT NOT NULL UNIQUE,
    algorithm TEXT NOT NULL DEFAULT 'sha256',
    size_bytes INTEGER NOT NULL CHECK (size_bytes >= 0),
    media_type TEXT NOT NULL,
    encoding TEXT,
    storage_path TEXT NOT NULL UNIQUE,
    state TEXT NOT NULL,
    redaction_level TEXT NOT NULL DEFAULT 'none',
    created_at_us INTEGER NOT NULL,
    committed_at_us INTEGER,
    last_verified_at_us INTEGER,
    CHECK (state IN ('staging','committed','quarantined','deleting','deleted'))
);

CREATE TABLE blob_uploads (
    upload_id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(project_id),
    session_id TEXT REFERENCES sessions(session_id),
    owner_actor_id TEXT NOT NULL REFERENCES actors(actor_id),
    purpose TEXT NOT NULL,
    expected_digest TEXT NOT NULL,
    expected_size_bytes INTEGER NOT NULL,
    received_size_bytes INTEGER NOT NULL DEFAULT 0,
    media_type TEXT NOT NULL,
    staging_path TEXT NOT NULL UNIQUE,
    state TEXT NOT NULL,
    expires_at_us INTEGER NOT NULL,
    created_at_us INTEGER NOT NULL,
    updated_at_us INTEGER NOT NULL,
    CHECK (state IN ('open','uploading','verifying','committed','failed','expired','deleted'))
);
CREATE INDEX idx_blob_uploads__gc ON blob_uploads(state, expires_at_us);

CREATE TABLE blob_refs (
    blob_id TEXT NOT NULL REFERENCES blobs(blob_id),
    owner_type TEXT NOT NULL,
    owner_id TEXT NOT NULL,
    project_id TEXT REFERENCES projects(project_id),
    purpose TEXT NOT NULL,
    created_at_us INTEGER NOT NULL,
    PRIMARY KEY(blob_id, owner_type, owner_id, purpose)
) WITHOUT ROWID;
CREATE INDEX idx_blob_refs__owner ON blob_refs(owner_type, owner_id);
CREATE INDEX idx_blob_refs__project ON blob_refs(project_id, blob_id);
```

Blob 提交和业务引用是两阶段：上传 commit 后 blob 为不可变但未必被业务引用；随后 Command 事务插入 `blob_refs`。GC 只清理超过 TTL 且无引用的 committed blob，删除采用 `deleting` intent → 文件删除 → `deleted` finalize。知道 blob ID 不代表有读权限，Query/Download 必须通过 `blob_refs.project_id/owner` 授权。

---

## 13. Skills、MCP、Memory、Hook 与 Plugin 表族

### 13.1 Skills

```sql
CREATE TABLE skills (
    skill_id TEXT PRIMARY KEY,
    scope_type TEXT NOT NULL CHECK (scope_type IN ('builtin','user','project','plugin')),
    scope_id TEXT,
    name TEXT NOT NULL,
    source_path TEXT,
    source_digest TEXT NOT NULL,
    manifest_json TEXT NOT NULL CHECK (json_valid(manifest_json)),
    status TEXT NOT NULL,
    discovered_at_us INTEGER NOT NULL,
    updated_at_us INTEGER NOT NULL,
    UNIQUE(scope_type, scope_id, name)
);

CREATE TABLE skill_loads (
    skill_load_id TEXT PRIMARY KEY,
    skill_id TEXT NOT NULL REFERENCES skills(skill_id),
    run_id TEXT REFERENCES runs(run_id),
    agent_id TEXT REFERENCES agents(agent_id),
    checkpoint_id TEXT REFERENCES checkpoints(checkpoint_id),
    token_estimate INTEGER,
    state TEXT NOT NULL,
    loaded_at_us INTEGER NOT NULL,
    error_json TEXT
);
```

Skill 文件/manifest 是权威资产；数据库保存 discovery、解析结果、诊断和调用观测。source digest 变化触发新解析，不直接 UPDATE 掩盖历史运行所使用的 digest。

### 13.2 MCP

```sql
CREATE TABLE mcp_servers (
    mcp_server_id TEXT PRIMARY KEY,
    scope_type TEXT NOT NULL,
    scope_id TEXT,
    name TEXT NOT NULL,
    transport TEXT NOT NULL,
    config_digest TEXT NOT NULL,
    safe_config_json TEXT NOT NULL CHECK (json_valid(safe_config_json)),
    state TEXT NOT NULL,
    capability_revision INTEGER NOT NULL DEFAULT 0,
    last_health_at_us INTEGER,
    last_error_json TEXT,
    created_at_us INTEGER NOT NULL,
    updated_at_us INTEGER NOT NULL,
    UNIQUE(scope_type, scope_id, name)
);

CREATE TABLE mcp_tools (
    mcp_server_id TEXT NOT NULL REFERENCES mcp_servers(mcp_server_id),
    tool_name TEXT NOT NULL,
    schema_digest TEXT NOT NULL,
    safe_schema_json TEXT NOT NULL CHECK (json_valid(safe_schema_json)),
    discovered_at_us INTEGER NOT NULL,
    retired_at_us INTEGER,
    PRIMARY KEY(mcp_server_id, tool_name)
) WITHOUT ROWID;

CREATE TABLE mcp_calls (
    mcp_call_id TEXT PRIMARY KEY,
    mcp_server_id TEXT NOT NULL REFERENCES mcp_servers(mcp_server_id),
    tool_call_id TEXT REFERENCES tool_calls(tool_call_id),
    operation_id TEXT REFERENCES operation_journal(operation_id),
    tool_name TEXT NOT NULL,
    state TEXT NOT NULL,
    request_digest TEXT NOT NULL,
    response_blob_id TEXT REFERENCES blobs(blob_id),
    started_at_us INTEGER,
    finished_at_us INTEGER,
    error_json TEXT,
    created_at_us INTEGER NOT NULL
);
CREATE INDEX idx_mcp_calls__server_time ON mcp_calls(mcp_server_id, created_at_us DESC);
```

MCP token、API key 和 auth header 只存 CredentialStore；`safe_config_json` 必须经过 redaction。

### 13.3 Memory 与 FTS5

Memory Markdown 为内容权威，SQLite 保存 revision 元数据与可重建 FTS5：

```sql
CREATE TABLE memories (
    memory_id TEXT PRIMARY KEY,
    project_id TEXT REFERENCES projects(project_id),
    scope_type TEXT NOT NULL CHECK (scope_type IN ('user','project','session')),
    scope_id TEXT,
    logical_path TEXT NOT NULL,
    title TEXT NOT NULL,
    head_revision_id TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    created_at_us INTEGER NOT NULL,
    updated_at_us INTEGER NOT NULL,
    UNIQUE(scope_type, scope_id, logical_path)
);

CREATE TABLE memory_revisions (
    memory_revision_id TEXT PRIMARY KEY,
    memory_id TEXT NOT NULL REFERENCES memories(memory_id),
    revision_number INTEGER NOT NULL,
    content_inline TEXT,
    content_blob_id TEXT REFERENCES blobs(blob_id),
    content_digest TEXT NOT NULL,
    source_path TEXT,
    source TEXT NOT NULL,
    created_at_us INTEGER NOT NULL,
    UNIQUE(memory_id, revision_number),
    UNIQUE(memory_id, content_digest),
    CHECK ((content_inline IS NOT NULL) <> (content_blob_id IS NOT NULL))
);

CREATE TABLE memory_documents (
    rowid INTEGER PRIMARY KEY,
    memory_revision_id TEXT NOT NULL UNIQUE REFERENCES memory_revisions(memory_revision_id),
    memory_id TEXT NOT NULL REFERENCES memories(memory_id),
    project_id TEXT,
    scope_type TEXT NOT NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    tags TEXT NOT NULL DEFAULT '',
    source_path TEXT NOT NULL,
    indexed_at_us INTEGER NOT NULL
);

CREATE VIRTUAL TABLE memory_fts USING fts5(
    title,
    body,
    tags,
    source_path,
    content='memory_documents',
    content_rowid='rowid',
    tokenize='unicode61 remove_diacritics 2',
    prefix='2 3 4'
);
```

FTS 同步通过显式 repository，而不是依赖复杂 trigger：在同一 projection transaction 更新 `memory_documents` 与 `memory_fts`。重建时：清空 FTS → 从当前有效 memory heads 重新生成 documents → `INSERT INTO memory_fts(memory_fts) VALUES('rebuild')` 或逐批插入 → 更新 projection cursor。中文检索质量不足时可增加由应用生成的分词列，但不得改变 Markdown 权威关系；向量索引不是 v1.0 必需项。

### 13.4 Hook 与 Plugin

```sql
CREATE TABLE plugins (
    plugin_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    source_path TEXT NOT NULL,
    manifest_digest TEXT NOT NULL,
    manifest_json TEXT NOT NULL CHECK (json_valid(manifest_json)),
    granted_capabilities_json TEXT NOT NULL CHECK (json_valid(granted_capabilities_json)),
    state TEXT NOT NULL,
    installed_at_us INTEGER NOT NULL,
    updated_at_us INTEGER NOT NULL,
    UNIQUE(name, version)
);

CREATE TABLE hook_invocations (
    hook_invocation_id TEXT PRIMARY KEY,
    plugin_id TEXT REFERENCES plugins(plugin_id),
    hook_name TEXT NOT NULL,
    event_id TEXT REFERENCES domain_events(event_id),
    operation_id TEXT REFERENCES operation_journal(operation_id),
    state TEXT NOT NULL,
    input_digest TEXT NOT NULL,
    output_blob_id TEXT REFERENCES blobs(blob_id),
    started_at_us INTEGER,
    finished_at_us INTEGER,
    error_json TEXT,
    created_at_us INTEGER NOT NULL
);
CREATE UNIQUE INDEX uq_hook_invocations__event_hook
    ON hook_invocations(event_id, plugin_id, hook_name)
    WHERE event_id IS NOT NULL;
```

Hook/Plugin 只能通过 capability 限定的 Application API 工作；数据库表用于注册、授权、调用审计和去重，不提供任意 SQL 扩展口。

---

## 14. Projection 数据模型

### 14.1 Projection Registry

```sql
CREATE TABLE projection_registry (
    projection_name TEXT PRIMARY KEY,
    projection_revision TEXT NOT NULL,
    cursor_global_seq INTEGER NOT NULL DEFAULT 0,
    target_global_seq INTEGER NOT NULL DEFAULT 0,
    state TEXT NOT NULL,
    rebuild_generation INTEGER NOT NULL DEFAULT 0,
    last_started_at_us INTEGER,
    last_success_at_us INTEGER,
    last_error_json TEXT,
    updated_at_us INTEGER NOT NULL,
    CHECK (state IN ('ready','catching_up','rebuilding','failed','disabled'))
);
```

### 14.2 强一致 Projection

建议物化：

- `project_overview_projection`：trust、effective config revision、主 worktree、active session count；
- `session_summary_projection`：state、active run、current branch、unread/last message、last seq；
- `pending_approvals_projection`：permission 与 spec review 统一队列；
- `spec_view_projection`：stage、heads、review gate、invalidations；
- `run_detail_projection`：outcome、usage、tools、blocks；
- `conversation_items_projection`：Message、Tool summary、Turn boundary 的统一时间线。

示例：

```sql
CREATE TABLE session_summary_projection (
    session_id TEXT PRIMARY KEY REFERENCES sessions(session_id),
    project_id TEXT NOT NULL,
    title TEXT NOT NULL,
    state TEXT NOT NULL,
    current_branch_id TEXT,
    active_run_id TEXT,
    last_message_id TEXT,
    last_message_preview TEXT,
    last_activity_at_us INTEGER NOT NULL,
    unread_count INTEGER NOT NULL DEFAULT 0,
    spec_stage TEXT,
    pending_approval_count INTEGER NOT NULL DEFAULT 0,
    as_of_global_seq INTEGER NOT NULL,
    projection_revision TEXT NOT NULL
);
CREATE INDEX idx_session_summary__project_activity
    ON session_summary_projection(project_id, last_activity_at_us DESC, session_id DESC);
CREATE INDEX idx_session_summary__active
    ON session_summary_projection(project_id, state, last_activity_at_us DESC)
    WHERE state <> 'archived';
```

所有客户端的“最近会话”和 auto-resume 必须复用同一个 Query/Projection 语义，不能 TUI 扫一套表、Web 再拼另一套。

### 14.3 异步 Projection 与 rebuild

异步投影使用 generation swap，避免重建期间暴露半成品：

```text
创建 <projection>_build_<generation>
  → 从 event 0 或权威 current tables 批量构建
  → 校验 row count / digest / target seq
  → 短事务交换 active generation 元数据
  → catch up target+1..current
  → 标记 ready
  → 延迟删除旧 generation
```

对于固定 schema 表，可在同库使用 shadow table rename；对于大表，也可用 `generation` 列 + active generation 元数据。Query 始终返回 `projection_revision` 和 `as_of_global_seq`。

### 14.4 `at_least_seq` 等待

Storage 层为 projection cursor 提供 watch channel：

1. Query 读取 cursor；
2. 若 `cursor >= min_global_seq`，直接执行；
3. 否则等待 watch，最长默认 2 秒；
4. 到期返回 `PROJECTION_LAGGING { current_seq, requested_seq }`；
5. 禁止通过伪造 `as_of_global_seq` 或直接把 Event Store seq 填进旧 projection 响应。

---

## 15. 索引与 Query 映射

### 15.1 索引原则

1. 每个 FK 的高频 join 方向建立索引；
2. 时间线索引使用稳定 tie-breaker，如 `(created_at_us DESC, id DESC)`；
3. active/pending/未归档使用 partial index，减小热索引；
4. 不为低选择性布尔列单独建全量索引；
5. JSON expression index 仅用于已经稳定且有基准数据证明的字段；
6. 每个公开 Query 必须有 `EXPLAIN QUERY PLAN` golden 测试；
7. 写放大敏感表（events/messages/tool calls）避免“为了可能会查”无限加索引；
8. 索引 migration 与数据 backfill 分离，避免长时间阻塞启动。

### 15.2 公开 Query 映射

| Query | 主表/Projection | 关键索引/游标 |
|---|---|---|
| ListProjects | project overview | `(archived, updated_at, project_id)` |
| ListSessions | session summary | `(project_id,last_activity,id)` keyset |
| GetConversation | conversation items/messages | `(branch_id,message_seq)` |
| GetRun | run detail | `run_id` + child `(run_id,ordinal/time)` |
| GetSpec | spec view/artifact heads | `spec_id`、`artifact_id,revision desc` |
| GetWorkflow | workflow graph | `workflow_id`、node/edge composite |
| PendingApprovals | strong pending projection | `(project_id,kind,created_at,id)` |
| EventQuery | domain events | `(project/session/type,global_seq)` |
| SearchMemory | memory_fts | FTS MATCH + scope filter |
| OperationQuery | operation journal | PK、`state,next_retry` |

分页使用 keyset cursor。以 ListSessions 为例：

```sql
SELECT ...
FROM session_summary_projection
WHERE project_id = :project_id
  AND (last_activity_at_us < :last_time
       OR (last_activity_at_us = :last_time AND session_id < :last_id))
ORDER BY last_activity_at_us DESC, session_id DESC
LIMIT :page_size_plus_one;
```

### 15.3 查询预算

- 常规面板 Query p95 < 50 ms；
- Event replay 单批 100～1000 行或按 1 MiB payload 限制；
- 单次 REST/gRPC page 默认 50、最大 500；
- 不允许 `SELECT *` 读取大 blob/json 列用于列表；
- 导出、FTS rebuild、projection rebuild 使用批次和 cooperative yield；
- 超时 Query 调用 `sqlite3_interrupt`，释放 read snapshot。

---
## 16. Schema Migration 设计

### 16.1 迁移原则

Apex migration 必须满足：

- forward-only；
- 每个 migration 有稳定 ID、递增 version、源码 checksum；
- 启动时发现数据库版本高于当前 binary：拒绝写入，进入兼容错误，不猜测读取；
- migration 失败：事务性步骤整体回滚；非事务性步骤通过状态机可恢复；
- 破坏性变更先 expand，再 backfill，再 contract；
- 不在 migration 中执行 Provider、Bash、MCP、Git 或普通文件写入；
- 大数据回填可暂停、可恢复、可观测，不阻塞所有启动超过产品预算；
- schema version、event schema version、protocol version、artifact format version 分开管理。

### 16.2 Migration 文件布局

```text
crates/apex-storage/
├── migrations/
│   ├── 0001_bootstrap.sql
│   ├── 0002_identity.sql
│   ├── 0003_conversation.sql
│   ├── 0004_event_store.sql
│   ├── 0005_spec_artifact.sql
│   ├── 0006_workflow_agent.sql
│   ├── 0007_tool_safety.sql
│   ├── 0008_checkpoint_snapshot_blob.sql
│   ├── 0009_extensions_memory.sql
│   ├── 0010_projections.sql
│   ├── 0011_rules_gate.sql              # rulesets、ruleset_revisions、rule_checks、
│   │                                    # diagnostics、gate_definitions、gate_attempts、
│   │                                    # gate_evidence、verification_results、waivers
│   ├── 0012_credential.sql              # credentials、credential_versions、credential_grants、
│   │                                    # credential_leases、credential_usages、
│   │                                    # redaction_records、data_lineage_edges
│   ├── 0013_extension_registry.sql      # extensions、extension_revisions、extension_instances、
│   │                                    # extension_grants、extension_generations、
│   │                                    # skill_revisions、skill_resource_loads、
│   │                                    # mcp_schema_revisions、hook_subscriptions
│   ├── 0014_observability_ops.sql       # audit_records、metric_samples、alerts、incidents、
│   │                                    # health_checks、operator_actions、
│   │                                    # maintenance_run_steps、support_bundles
│   ├── 0015_workspace_detail.sql        # snapshot_files、workspace_baselines、
│   │                                    # restore_plans、restore_conflicts
│   ├── 0016_context_build.sql           # context_blocks、context_builds
│   └── ...
├── src/migration.rs
└── tests/migrations/
    ├── fixtures/
    ├── upgrade_tests.rs
    ├── checksum_tests.rs
    └── failure_injection.rs
```

> ADR-0011（跨文档一致性审查）：`0011`–`0016` 收纳原先只存在于各详细设计文档、未进入本 schema 权威的 36 张表。DDL 正文仍由对应详细设计维护（避免同一份 DDL 双写漂移），但**迁移号、依赖顺序、外键与索引一致性由本文档统一把关**。其中 `audit_records` 承载审计链，属 P0 安全能力，不得游离于 schema 权威之外。
>
> 各迁移对应的能力档位：`0011` 属 v0.1（规则校验），`0012` 属 v0.1（Provider 密钥）～v0.5（完整 Broker），`0013` 属 v0.5，`0014` 属 v0.1（基础审计）～v0.7（完整运维面），`0015` 属 v0.1（快照）～v0.5（隔离工作区），`0016` 属 v0.1。

文件名的数字只用于排序，`schema_migrations.migration_id` 是不可变身份。已发布 migration 文件禁止编辑；修正必须创建新 migration。应用启动时计算源码 canonical bytes 的 SHA-256，并与已存 checksum 比较。

### 16.3 迁移状态机

```text
discovered
  → preflight
  → backup_created
  → applying
  → applied
  → backfill_pending
  → backfill_running
  → completed
```

失败分支：

```text
applying → rolled_back
backup_created → blocked_restore_required
backfill_running → resumable_failed → backfill_running
```

migration metadata 可保存在 `schema_migrations`；跨事务长 backfill 另存 `backfill_jobs`：

```sql
CREATE TABLE backfill_jobs (
    job_name TEXT PRIMARY KEY,
    migration_id TEXT NOT NULL,
    status TEXT NOT NULL,
    last_key TEXT,
    processed_rows INTEGER NOT NULL DEFAULT 0,
    failed_rows INTEGER NOT NULL DEFAULT 0,
    target_watermark TEXT,
    started_at_us INTEGER,
    last_progress_at_us INTEGER,
    finished_at_us INTEGER,
    error_json TEXT,
    CHECK (status IN ('pending','running','paused','completed','failed','blocked'))
);
```

### 16.4 Transactional migration

适合：CREATE TABLE、CREATE INDEX、ALTER TABLE ADD COLUMN、元数据更新和小规模 backfill。

```text
BEGIN EXCLUSIVE（migration 期间）
  → 校验 from_version
  → 应用 SQL
  → 写 schema_migrations
  → 更新 db_metadata.database_format_version
  → COMMIT
```

迁移连接独占期间，daemon 不接受业务 Command；只读 Query 可根据产品策略返回 `DATABASE_MIGRATING`，不能读取半迁移结构。

### 16.5 Expand / Backfill / Contract

大表变更采用三阶段：

**Expand**

- 新增 nullable 列/新表/新 projection；
- 代码同时支持 old+new；
- 不删除旧列、不改变旧语义；
- 新写入双写时必须在同一事务内完成。

**Backfill**

- 以稳定主键 keyset 分批，每批 100～1000 行；
- 每批短事务，更新 `backfill_jobs.last_key` 与统计；
- batch 可重试，写入必须幂等；
- 期间保留 dual-read 或校验 old/new digest；
- 发现异常行不跳过：记录诊断并按 migration 策略 pause/block。

**Contract**

- 达到全量 watermark 后运行一致性检查；
- 新代码停止读取旧列；
- 下一次受控 release 再删除旧列或旧表；
- 删除前先生成 backup，并记录破坏性 migration ADR。

### 16.6 迁移兼容矩阵

| 组件/数据 | 版本字段 | 兼容策略 |
|---|---|---|
| SQLite schema | `database_format_version` | binary 支持区间；高版本拒绝写 |
| Migration | `schema_migrations.version` | forward-only + checksum |
| Domain Event | `schema_version` | pure upcaster |
| Protocol | Hello range | 协商交集最高版本 |
| Artifact | `format_version` | 导入时新 revision |
| Projection | `projection_revision` | rebuild/swap |
| Config | raw/effective format | strict default + 显式诊断 |
| Blob | media/encoding | metadata compatibility + scanner |

### 16.7 降级与导入导出

不支持旧二进制直接降级打开新 schema。降级路径：

1. 使用升级前的数据库 backup 恢复到旧版本；或
2. 新版本导出受支持的 portable archive（Project、Spec revisions、messages、events、blobs manifest）；
3. 旧版本通过 importer 创建新 `event_store_id`，为不可迁移字段生成 warning；
4. 不把新事件伪装成旧事件类型。

---

## 17. 备份、完整性与恢复

### 17.1 备份策略

备份分级：

| 类型 | 触发 | 内容 | 目标 |
|---|---|---|---|
| Pre-migration | 破坏性/大 migration 前 | SQLite backup API + blob manifest | 本地 state/backups |
| Scheduled | 默认每日或累计写入阈值 | 在线 backup + digest | 用户指定目录 |
| User export | 用户 Command | 可移植事件/资产包 | 归档/团队共享 |
| Crash quarantine | integrity 失败 | 原始库、WAL 保留、报告 | quarantine |

SQLite backup API 读取一致性快照，不复制活动 `.db-wal` 文件。备份完成后计算数据库文件 digest、schema version、event_store_id、last global seq、blob manifest digest，并写 sidecar manifest。敏感备份按 OS 权限和用户明确策略加密；不把密钥写在 manifest。

### 17.2 恢复点

恢复必须明确两种语义：

- **同库恢复**：从 backup 完整恢复数据库和对应 blob/snapshot manifest，保留 `event_store_id`；客户端 cursor 只能回退到恢复库的 current seq，恢复事件通过新 `DatabaseRestored` 审计事件记录（若恢复库可写）；
- **导入恢复**：将可移植 archive 导入全新数据库，生成新 `event_store_id`，通过 import command 建立新事件链。

不能只恢复 SQLite 而不恢复外部 Blob/影子 Git，然后声称数据完整。manifest 缺 blob 的项目进入 degraded/readonly，不能执行引用该 blob 的 Run。

### 17.3 启动完整性检查

分层执行：

```sql
PRAGMA quick_check(1);
PRAGMA foreign_key_check;
```

周期性维护窗口执行：

```sql
PRAGMA integrity_check;
```

应用层校验：

- `global_seq` 从 1 到 current 无重复、无 gap、event_store_id 一致；
- 每个 aggregate 的 version 连续且 event type/schema 可 upcast；
- payload digest 与 canonical payload 匹配；
- projection cursor 不大于 event current seq；
- outbox pending 与引用的 event/operation 存在；
- operation journal 中 active lease 的 owner 是否存活；
- blob metadata、文件 size、digest、refs 一致；
- artifact head、revision parent 和 materialization digest 一致；
- write claim lease、path scope 与 active Agent/attempt 一致。

发现事件或聚合不连续时：

```text
停止 Tool/Provider/写文件
  → db_metadata.maintenance_mode = readonly_recovery
  → 生成诊断与备份
  → 尝试从 backup / export 恢复
  → 通过补偿事件或人工修复，不删除事实
```

### 17.4 SQLite 损坏处理

把错误分为：

- `SQLITE_BUSY/LOCKED`：连接/读者问题，退避重试，达到阈值报警；
- `SQLITE_FULL/IOERR`：磁盘或文件系统问题，停止写入并提示空间；
- `SQLITE_CORRUPT/NOTADB`：立即只读隔离，保留原始文件与 WAL，不尝试原地“修复后继续”；
- `SQLITE_CONSTRAINT`：通常是应用不变式、版本冲突或 migration 缺陷，按 Command error/启动阻断分类；
- `SQLITE_INTERRUPT`：Query 被 deadline 取消，不当作数据库损坏。

修复优先级：验证最近 backup → 恢复并校验 blob manifest → 重新构建 projection/FTS → reconcile 外部文件 → 恢复 daemon。每一步产生 `maintenance_runs` 记录。

---

## 18. 归档、保留、删除与 GC

### 18.1 权威事实保留

默认不可删除：

- Domain Event；
- Spec skip、审批、权限决定和安全审计；
- Artifact Revision（除非用户显式执行符合政策的 privacy purge）；
- 影响交付的 RuleCheck、Snapshot restore 和 unknown operation 记录。

可归档但不默认删除：

- 已结束 Session/Run/Turn；
- Provider request/response 大 Blob；
- 高频日志、Transient stream 合并结果；
- 已 superseded 的 Projection generation。

### 18.2 默认策略（可配置上限）

以下是建议默认值而非不可变领域规则：

| 数据 | 默认在线保留 | 归档/GC |
|---|---:|---|
| Domain Event | 永久 | 仅显式隐私清除/导出后政策清除 |
| Command dedup 安全关键 | 永久 | 不自动清理 |
| 普通 Command dedup | 180 天 | digest/审计策略允许才清理 |
| Event broadcast outbox | 成功后 7 天 | 保留重放窗口，之后归档 |
| Provider raw payload Blob | 30 天或按项目策略 | 无 ref + TTL GC |
| Tool stdout/stderr | 90 天 | 仅摘要可保留 |
| Checkpoint | Session 生命周期 + 30 天 | 引用计数/TTL GC |
| Snapshot | 最近每 Turn + 用户 pin | retain/release 后 GC |
| Projection | 当前 generation | 旧 generation 延迟删除 |
| FTS5 | 可重建 | 损坏可 drop/rebuild |
| 审计/权限/skip | 永久 | 合规政策另行处理 |

具体 TTL 由 user/project policy 解析到 effective config，并在 GC operation 中固化当次决策。配置变更不会追溯改变已发生事件含义。

### 18.3 Privacy purge

用户请求删除敏感内容时使用独立受控 Command：

1. 创建 purge operation 和范围快照；
2. 校验 Actor、项目边界、备份和确认；
3. 将不可删除事实中的内容替换为不可逆 redaction marker，保留 event_id、seq、type、digest lineage 和审计原因；
4. 删除/重加密 Blob 与 FTS 内容；
5. 更新 redaction manifest；
6. 追加 `PrivacyPurged` 补偿事件；
7. 重新构建受影响 Projection；
8. 导出报告，说明哪些事实保留、哪些内容不可恢复。

SQLite 物理 `DELETE` 可能留下 freelist/WAL 副本，因此高敏内容清除需 checkpoint、secure delete 策略、备份清除和文件系统政策配合；不能承诺普通 DELETE 等于密码学擦除。

### 18.4 Blob 与 Snapshot GC

GC 采用 mark-and-sweep：

```text
收集 roots：current heads、events、messages、checkpoints、operations、outbox、pins、active runs
  → 扫描 blob_refs / snapshot refs
  → 生成 candidate
  → 二次确认（candidate 仍无 ref 且超过 grace period）
  → 写 delete intent
  → 删除外部文件/Git object
  → finalize DB metadata
```

GC 与写入并发时不能仅依赖 reference_count；最终删除前必须在短事务中重新确认 refs、lease、pending materialization 和 active run。

---

## 19. 安全、密钥与本地权限

### 19.1 数据分级

| 分级 | 示例 | SQLite 策略 |
|---|---|---|
| public | Project 标题、非敏感状态 | 普通存储 |
| internal | 事件 payload、工具摘要 | scope 授权 + redaction |
| sensitive | prompt、源码片段、MCP 返回 | Blob/inline 皆须授权与审计 |
| secret | Provider key、cookie、token | 不入 SQLite；OS Credential Store |

数据库文件权限默认为当前 OS 用户私有。Web Gateway 即使监听 loopback，也不得把文件路径、secret 或未授权项目内容直接返回浏览器。

### 19.2 加密

v1.0 优先依赖 OS 磁盘加密和 Credential Store，不引入未经验证的 SQLCipher 分支作为默认依赖。若未来需要库级加密：

- 作为 storage backend capability，不改变领域/API schema；
- 密钥从 OS Keychain 获取，不由 config.toml 保存；
- rotation 必须在线 backup → re-encrypt → integrity check → atomic replace；
- crash 中间态保留旧库，不能删除唯一可恢复副本。

### 19.3 Secret 防泄漏门

以下路径必须经过统一 redaction：

- Provider request/response；
- Tool arguments/stdout/stderr；
- MCP resource/tool result；
- diagnostics/export；
- Event payload 与 `command_dedup.response_json`。

redaction 后保存 `redaction_level`、规则版本和 digest lineage，便于解释“为何看不到内容”，但不保存被遮盖值。

---

## 20. 并发、租约与故障恢复

### 20.1 Writer 队列

```text
Command/API/worker callback
  → bounded StorageCommand channel
  → StorageWriter actor
  → one rusqlite writer connection
  → commit result
```

队列满时：

- Query 仍可读取；
- 普通低优先级后台任务返回 `STORAGE_BACKPRESSURED` 并延迟；
- 用户 Command 返回明确 retry/operation ref，不在内存无限排队；
- 安全审批、取消和恢复 Command 有保留容量/高优先级通道，但仍遵守领域版本。

### 20.2 Lease

Outbox、Operation、Claim 和 backfill 都使用 owner token + lease expiry：

```text
BEGIN IMMEDIATE
  → 查找 pending 或 lease_until < now
  → 生成随机 lease_token
  → CAS 更新 owner/lease/state
  → COMMIT
  → 执行外部工作
  → 用 lease_token + operation version 提交结果
```

迟到 worker 结果如果 lease 已变更或 operation 已终态，必须丢弃/记录 stale completion，不能覆盖新 attempt。

### 20.3 崩溃恢复顺序

```text
1. SQLite 自身 WAL recovery
2. migration/backfill 状态恢复
3. event/global seq、aggregate version、payload digest 校验
4. projection cursor 与重放/重建
5. materialization intent 与 Markdown reconcile
6. blob upload staging 清理
7. active operation/worker/claim reconcile
8. 影子 Git snapshot lease/reachability 检查
9. 发布 RecoveryCompleted
```

恢复只重放确定性数据库状态，不自动重放不可逆 Bash/MCP/远端副作用。未知操作必须显示为 `interrupted/unknown`，提供 reconcile、retry-new-attempt 或用户确认动作。

### 20.4 取消语义

取消 Command 只改变可取消 operation 的状态并追加事件。取消数据库事务不能杀死已经运行的任意子进程；ProcessSupervisor 另行发送终止信号并记录是否确认退出。若不能确认，Run 为 `interrupted` 而不是 `cancelled`。

---

## 21. 观测、性能与容量预算

### 21.1 数据库指标

至少记录：

- open/migration/backup/integrity 成功失败计数；
- writer queue depth、transaction count、commit latency p50/p95/p99；
- `SQLITE_BUSY/LOCKED/FULL/IOERR/CORRUPT` 分类；
- WAL size、checkpoint duration、reader age；
- event append rate、projection lag、outbox lag；
- FTS query latency/rebuild duration；
- database bytes、blob bytes、snapshot bytes、freelist pages；
- GC candidates、deleted bytes、failed deletes；
- migration/backfill processed/failed rows 与 ETA。

指标 payload 不应携带 prompt、源码或 secret。慢 SQL 记录 query fingerprint、table/index、耗时和行数，不记录完整参数。

### 21.2 容量边界

启动时读取并校验用户/项目 resource limits：

- 单 inline content 64 KiB 默认上限；
- 单 Event payload 256 KiB 默认上限，超出转 Blob；
- 单 SQL transaction 事件/行/字节数有界；
- Event replay 按行数 + bytes 双重限额；
- 单 session message、单 project blob、单 workflow node 数量为 policy 限制；
- 达到磁盘水位时先停止新上传/非关键 snapshot，再停止普通 Run，最后进入只读恢复。

### 21.3 VACUUM 与空间回收

- WAL checkpoint 与 `incremental_vacuum` 在空闲维护窗口执行；
- 不在用户正在运行 Tool 的关键路径执行全量 VACUUM；
- projection/FTS 可能 rebuild 后产生 freelist，记录空间预算；
- 备份、恢复和 GC 前后报告文件 bytes 与逻辑 bytes，避免误判“文件变小即数据正确”。

---

## 22. 测试与质量门

### 22.1 Schema 静态测试

- 所有 migration 从空库顺序执行；
- 每个已发布 migration checksum golden；
- `PRAGMA foreign_key_check` 无错误；
- 每个表的 PK、FK、unique、partial index 经过 schema snapshot 比较；
- JSON 列 fixture 通过 `json_valid`；
- FTS external-content 的 insert/update/delete/rebuild fixture 完整；
- SQLite bundled 版本满足 FTS5、JSON1、backup API 与 WAL 需要。

### 22.2 事务性质测试

- Command 成功时 current state、event、dedup、outbox 全部出现；
- 任意 commit 前故障注入后四者要么都不出现，要么都可恢复；
- Event global_seq 无 gap、无重复；aggregate version 无 gap；
- 同 command_id 同 payload 返回同结果；不同 payload/actor 必须拒绝；
- projection 重建与增量消费结果 digest 相同；
- outbox 重复 delivery 不产生重复文件、广播或 hook side effect；
- stale worker 结果不能覆盖新版本。

### 22.3 故障注入

注入点包括：

- WAL 写入、fsync、commit 前后；
- sequence counter 更新与 Event insert 之间；
- materializer temp write、rename、parent fsync 前后；
- Blob digest verify、ref commit、GC delete 前后；
- lease claim、外部进程启动、进程退出、回调提交；
- migration 每个 statement、backfill 每个 batch；
- projection rebuild swap 前后；
- reader 长快照期间 checkpoint。

验收目标是业务语义可确定恢复，不是声称所有外部副作用可自动重放。

### 22.4 性能测试

基准场景：

- 10 万、100 万、1000 万 Domain Event 的按 seq replay；
- 10 万 Session 的 List/auto-resume Query；
- 100 万 Memory revision 的 FTS5 search/rebuild；
- 100 个并行 Agent 申请 overlapping claims；
- 1000 个 outbox item 的 lease/retry；
- 大 Blob 上传、Range 下载、GC；
- migration/backfill 在 WAL、磁盘接近满和旧 reader 存在时的行为。

### 22.5 安全测试

- 越权读取其它 project/session/blob；
- 命令/事件 body 伪造 actor、project、causation；
- secret scanner bypass；
- 恶意路径、symlink、大小写碰撞和 glob escape；
- projection 显示旧 available action 后再次 Command 的重授权；
- 恶意 migration checksum、未知 event schema、损坏 FTS；
- Web cursor、CSRF、Origin 和本地 launch ticket 重放。

---

## 23. Rust 存储层组织

推荐 workspace：

```text
crates/
├── apex-storage/
│   ├── src/
│   │   ├── connection.rs       # PRAGMA、路径、连接健康
│   │   ├── writer.rs           # StorageWriter actor、队列、事务
│   │   ├── read_pool.rs        # query_only read connections
│   │   ├── migration.rs        # version/checksum/backfill
│   │   ├── event_store.rs      # append/read/verify/upcast boundary
│   │   ├── unit_of_work.rs     # state + event + outbox atomic commit
│   │   ├── repositories/       # aggregate/current state repositories
│   │   ├── projections/        # strong/async projection handlers
│   │   ├── blob.rs              # metadata + filesystem CAS
│   │   ├── backup.rs            # online backup/manifest
│   │   ├── recovery.rs          # reconcile/maintenance
│   │   └── gc.rs               # blob/snapshot/archive GC
│   ├── migrations/*.sql
│   └── tests/
├── apex-domain/                # 不依赖 rusqlite
├── apex-application/           # Command/Query orchestration
└── apex-protocol/              # gRPC/REST/WS DTO
```

依赖方向：

```text
apex-domain ← apex-storage ← apex-application ← apex-protocol/UI
```

`apex-domain` 不导入 SQLite 类型；`apex-storage` 不调用 Tool、Provider、MCP、文件 materializer；这些由 Application/Port/Outbox worker 实现。Repository 返回 typed domain records，禁止把 `rusqlite::Row` 泄漏到上层。

### 23.1 Storage API 形态

```rust
pub trait Storage: Send + Sync {
    fn submit(&self, command: StorageCommand) -> StorageFuture<CommandCommit>;
    fn query(&self, query: StorageQuery) -> StorageFuture<QueryResult>;
    fn subscribe_projection(&self, name: ProjectionName) -> ProjectionWatch;
    fn health(&self) -> StorageHealth;
}

pub struct CommandCommit {
    pub command_id: CommandId,
    pub status: CommitStatus,
    pub committed_event_ids: Vec<EventId>,
    pub as_of_global_seq: GlobalSeq,
    pub operation_id: Option<OperationId>,
}
```

最终 crate 可使用 async facade，但内部 writer 是同步、串行、可测试的 actor。所有 repository transaction 接收 `&mut TransactionContext`，不能自行 commit，确保 Unit of Work 原子性。

---

## 24. 分阶段落地

### v0.1：最小可靠闭环

- `db_metadata`、`schema_migrations`、projects、actors、sessions、branches、messages；
- runs、turns、provider_calls；
- domain_events、sequence_counters、command_dedup；
- **tool_calls、permission_requests、permission_decisions、permission_rules**；
- **rule_checks、diagnostics、rulesets**；
- **checkpoints、snapshots、write_claims**；
- WAL、writer actor、基本 backup/quick_check；
- TUI 单端使用统一 Storage API；
- Spec requirements/design/tasks/review 的核心表和 Markdown materialization intent。

> ADR-0007 / ADR-0024（跨文档一致性审查）：需求文档 §5.1 与系统总体架构 §18 规定 v0.1 已含工具、权限、规则校验、Checkpoint 与文件快照，API 协议 §17.1 也要求 v0.1 提供对应 API。原表格把这些表推到 v0.3/v0.5，会造成"API 有接口而 DB 无表"。Write Claim 依 ADR-0024 一并提前到 v0.1（v0.1 单会话场景可为薄实现，但接口与事件从第一天正确，使 `INV-TG-007` 成立）。

### v0.3：三端共享与安全闭环

- gRPC/REST Query 对接 projection；
- session/spec/run strong projections；
- Blob upload/commit/reference；
- reconnect cursor、event_store_id、outbox broadcast；
- Tauri/Web/TUI contract tests。

### v0.5：编排与扩展

- workflows/nodes/edges/attempts/agents/write_claims；
- snapshots、shadow Git metadata、restore operation；
- Skills、MCP server/tools/calls；
- memories、revisions、FTS5、memory projection；
- scheduler projection 和 operation journal recovery。

### v0.7：可靠性与可观测性

- upcaster registry、projection generation swap；
- full reconcile、privacy purge、archive/GC；
- backup verification、restore drill、failure injection；
- access audit、diagnostic export、容量与 WAL 控制；
- Hook/Plugin capability audit。

### v1.0：完整产品门

- 全部 Domain Event registry 与 schema golden；
- migration upgrade matrix 覆盖至少两个旧版本；
- 事件/投影重建可重复并通过 digest 校验；
- 跨三端一致的 Command/Query/Event SDK；
- 破坏性恢复、磁盘满、损坏库、断电、长 reader、慢消费者均有操作手册；
- 无任何 UI/Plugin/Worker 直写 SQLite 的静态和运行时检测。

---

## 25. 关键 ADR

| ADR | 决策 |
|---|---|
| ADR-DB-001 | 单用户单库；`event_store_id` 是数据库历史身份 |
| ADR-DB-002 | `rusqlite` bundled SQLite；StorageWriter 单写者 |
| ADR-DB-003 | WAL + `synchronous=FULL` 默认耐久性 |
| ADR-DB-004 | Current State + Event Store + Projection 混合持久化 |
| ADR-DB-005 | event global_seq 提交序列由 counter 事务分配 |
| ADR-DB-006 | Domain Event append-only；修正使用补偿事件 |
| ADR-DB-007 | 大内容使用 CAS Blob；数据库保存 metadata/ref |
| ADR-DB-008 | Spec/Memory/Checkpoint 文件采用 DB intent + atomic materialization |
| ADR-DB-009 | FTS5 为可重建投影，Markdown/Artifact revision 才是权威 |
| ADR-DB-010 | Migration forward-only + checksum + expand/backfill/contract |
| ADR-DB-011 | 不自动重试不可探测的 Bash/MCP 外部副作用 |
| ADR-DB-012 | event cursor 绑定 `event_store_id`，filtered stream 使用 scanned watermark |

待在实现前最终锁定：

- bundled SQLite 的最低版本与 Windows/macOS/Linux 构建矩阵；
- `synchronous=NORMAL` 是否作为用户可选 profile；
- 大于多少 payload 强制 Blob（默认 64 KiB/256 KiB 两级）；
- 是否引入库级加密 backend；
- privacy purge 对事件 payload 的法规/产品保留边界；
- Event Store 是否做完整 hash chain/signature manifest。

---

## 26. 跨文档一致性检查清单

实现或评审任何 schema 变更时，必须核对：

- [ ] `SpecSkipped` 仍是不可逆审计事件，`specs.skipped/skip_reason` 与事件 payload 语义一致；
- [ ] `global_seq`、`aggregate_version`、`message_seq`、`turn.ordinal` 没有混用；
- [ ] API 的 `event_store_id`、`as_of_global_seq`、`projection_revision` 能从数据库得到；
- [ ] filtered stream 对应 `scanned_through_global_seq`，不是最后匹配事件 seq；
- [ ] Command duplicate 可返回原始 Accepted 或 Rejected；
- [ ] `causation_event_id` 只能引用本库已存在事件，客户端不能任意伪造；
- [ ] `operation_id` 在 intent 提交前固定，外部 I/O 不在 transaction 内执行；
- [ ] Query 没有把 transient event 当成业务事实；
- [ ] Event/Projection rebuild 不调用 Bash、MCP、Provider 或文件写入；
- [ ] Markdown watcher 能区分 Core materialization、外部编辑与 orphan；
- [ ] Blob 跨 project 不能重新绑定；
- [ ] Write claim 对文件 scope overlap 做应用级 canonical path 判断；
- [ ] UI available action 只是提示，Command 仍重新授权和检查 expected version；
- [ ] 数据库损坏、event gap、projection lag 时禁止继续执行高风险 Tool；
- [ ] 三端共享同一个 Application/Storage API，不各自维护 session 状态副本。

---

## 附录 A：启动 SQL 骨架

以下仅是首个 bootstrap migration 的骨架，实际表应按本文件表族拆分为多个 migration，避免一个超大不可回滚脚本：

```sql
PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;
PRAGMA synchronous = FULL;
PRAGMA trusted_schema = OFF;
PRAGMA application_id = 1095779672;

CREATE TABLE IF NOT EXISTS db_metadata (
    singleton_id INTEGER PRIMARY KEY CHECK (singleton_id = 1),
    event_store_id TEXT NOT NULL UNIQUE,
    database_format_version INTEGER NOT NULL,
    application_id INTEGER NOT NULL,
    created_at_us INTEGER NOT NULL,
    last_opened_at_us INTEGER NOT NULL,
    maintenance_mode TEXT NOT NULL DEFAULT 'normal'
);

CREATE TABLE IF NOT EXISTS sequence_counters (
    name TEXT PRIMARY KEY,
    value INTEGER NOT NULL CHECK (value >= 0)
) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS schema_migrations (
    migration_id TEXT PRIMARY KEY,
    version INTEGER NOT NULL UNIQUE,
    name TEXT NOT NULL,
    checksum TEXT NOT NULL,
    applied_at_us INTEGER NOT NULL,
    duration_ms INTEGER NOT NULL,
    tool_version TEXT NOT NULL,
    execution_mode TEXT NOT NULL
);

INSERT INTO sequence_counters(name, value)
VALUES ('global_event_seq', 0)
ON CONFLICT(name) DO NOTHING;
```

## 附录 B：标准 Command 提交 SQL 轮廓

```sql
BEGIN IMMEDIATE;

-- 1. 幂等检查
SELECT command_id, payload_digest, status, response_json
FROM command_dedup
WHERE command_id = :command_id;

-- 2. 读取 expected version，领域层决定是否可写
SELECT version, last_event_seq
FROM sessions
WHERE session_id = :session_id;

-- 3. 预留 event sequence
SELECT value FROM sequence_counters WHERE name = 'global_event_seq';
UPDATE sequence_counters
SET value = value + :event_count
WHERE name = 'global_event_seq' AND value = :old_value;

-- 4. CAS 更新 current state
UPDATE sessions
SET state = :new_state,
    version = version + 1,
    updated_at_us = :now_us,
    last_event_seq = :last_event_seq
WHERE session_id = :session_id AND version = :expected_version;

-- 5. append event / strong projection / dedup / outbox
INSERT INTO domain_events (...)
VALUES (...);

INSERT INTO command_dedup (...)
VALUES (...);

INSERT INTO outbox (...)
VALUES (...);

COMMIT;
```

实现必须把所有 `...` 替换为显式列清单，禁止生产 SQL 依赖列顺序。

## 附录 C：恢复检查伪代码

```rust
fn recover(storage: &Storage) -> Result<RecoveryReport> {
    storage.sqlite_recovery()?;
    storage.apply_pending_migrations()?;
    storage.quick_check()?;
    storage.verify_event_store()?;
    storage.verify_aggregate_versions()?;
    storage.rebuild_or_catch_up_projections()?;
    storage.reconcile_materialization_intents()?;
    storage.reconcile_blob_uploads()?;
    storage.reconcile_operations_without_replaying_side_effects()?;
    storage.reconcile_claims_and_snapshots()?;
    storage.publish_recovery_completed()?;
    Ok(report)
}
```

`publish_recovery_completed` 只有在数据库已恢复到可接受状态后执行；如果仍处于 readonly recovery，不发布“正常恢复完成”事件，而是返回维护诊断。

## 附录 D：最终产品存储公理

1. SQLite 是本机业务写入权威，Core 是唯一写者；
2. 一个成功 Command 的事实、当前态、幂等结果和必要 outbox 同事务提交；
3. Domain Event 只追加、不覆盖、不删除；
4. `global_seq` 只用于数据库级提交顺序；
5. 外部副作用必须先有 intent，不能持有数据库事务等待 I/O；
6. 不确定的副作用结果必须显式记录 unknown；
7. Projection、FTS5 和缓存可重建，不得成为唯一事实源；
8. Markdown 镜像通过 intent/materializer/watcher/reconcile 保持可恢复一致；
9. Blob 是内容寻址但不是无权限公开资源；
10. Migration 只前进、带 checksum、可观测、可恢复；
11. 发现 integrity/event gap 时停止高风险操作；
12. 恢复是确定性重建业务状态，不是盲目重放不可逆外部副作用。








