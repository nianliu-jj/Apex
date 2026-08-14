# Apex—— Workspace快照、Write Claim与隔离工作区详细设计

> 文档状态：详细设计基线（面向最终完整产品）  
> 适用版本：Apex v0.1 ～ v1.x（按 §分阶段交付 分档启用；档位表以需求文档 §5.3 为准）  
> 编制日期：2026-08-08
>
> 适用范围：Apex 最终完整产品；覆盖主工作区、Git Worktree、Standalone Workspace、子 Agent 隔离、路径互斥、影子 Git 快照、文件级/Patch 回滚、工作区漂移和崩溃恢复。
>
> 本文细化《系统总体架构设计》《领域模型与事件规范》《SQLite数据模型与迁移设计》《Agent Runtime与DAG调度器详细设计》《Tool Gateway与权限引擎详细设计》中的 Workspace、Worktree、Write Claim、Snapshot 和 Restore 契约。

---

## 0. 文档目的与范围

### 0.1 要解决的问题

Apex 允许主 Agent、子 Agent、Workflow Node、用户编辑器和外部进程同时接触同一个项目。仅靠“Agent 声明自己要写哪些文件”不能保证安全，因为还存在：

- `src/` 与 `src/auth.rs` 的父子路径冲突；
- glob、大小写、Unicode、符号链接和 Junction 指向同一目标；
- Shell 命令的实际写范围大于声明范围；
- 用户在 Agent 执行期间手工修改同一文件；
- Agent 崩溃后遗留 Claim、进程、Worktree 或未提交变更；
- Git index、branch、reflog、submodule 和 ignored/untracked 文件不受普通 diff 完整覆盖；
- Snapshot 已创建但数据库 intent 未提交，或反之；
- 部分回滚破坏后续 DAG 节点的依赖前提；
- 文件恢复与上下文恢复被错误混为一谈；
- 隔离 Worktree 中的 Bash/Git 通过 `-C`、`--git-dir`、环境变量或绝对路径逃逸到主工作区。

本文定义一套面向最终产品的统一协议，使 Apex 能够回答：

1. 当前 Agent 对哪些规范化路径拥有写互斥权；
2. Worktree 的真实身份、基线和允许的操作边界是什么；
3. 每次写操作前后工作区发生了哪些可验证变化；
4. 哪些变化可以安全恢复，哪些需要用户解决冲突；
5. 崩溃后如何 Fence 旧执行者、保存遗留变更并释放资源；
6. 隔离工作区结果如何通过可审查 Patch 合并回目标工作区；
7. 如何保证整个过程不污染用户 `.git`、branch、index 和 reflog。

### 0.2 设计目标

- **权限与互斥分离**：Permission 回答“允许做什么”，Write Claim 回答“当前是否可以并发做”。
- **路径身份稳定**：冲突判定基于平台规范化身份，而不是未经解析的字符串。
- **先建立基线，再产生副作用**：任何含写操作的 Run/Node/Tool 必须能追溯到 pre-write Snapshot 或等价基线。
- **Snapshot 不污染用户 Git**：影子仓库独立管理对象，不修改用户 index、branch、HEAD 或 reflog。
- **Restore 是新 Operation**：回滚不能篡改历史 Snapshot，必须重新经过权限、Claim、Snapshot 和 Rules Gate。
- **默认保留用户变更**：检测到基线漂移或冲突时停止，不以 last-writer-wins 强制覆盖。
- **隔离覆盖全部副作用**：`isolation: worktree` 同时约束文件工具、Shell、Git、Hook、Skill 和 MCP 产生的本地文件写入。
- **确定性恢复**：Claim、Lease、Fence、Snapshot intent、Worktree provisioning 和 Restore 状态均可从持久事实重建。

### 0.3 非目标

本文不定义：

- 用户 Git hosting、PR 或代码评审平台的完整集成；
- 通用分布式文件锁服务；
- 网络远端系统的完整事务回滚；
- 所有文件系统的底层驱动实现；
- Rules/Verification 的具体 lint/test 规则。

---

## 1. 核心架构结论

1. **Project 与 Worktree 分开建模**：Project 表示逻辑项目及信任边界，Worktree 表示某个具体可访问文件根和 Git/Standalone 身份。
2. **Canonical Path Scope 是统一语言**：Agent Profile、Workflow、Permission、Write Claim、Tool Gateway、Snapshot 和 Restore 共享同一规范化路径模型。
3. **Claim 是租约，不是永久锁**：所有 active Claim 都有 lease token、lease deadline、owner Attempt 和 Fence 约束。
4. **多路径 Claim 原子获取**：一个 Node 需要的全部写范围要么同时获取，要么全部失败，避免死锁和半占有。
5. **不能证明不相交就按冲突处理**：glob、symlink、case folding 和动态 Shell 写范围采用保守原则。
6. **只读共享，写入隔离**：纯只读 Agent 可以共享 Worktree；可写 Agent 使用 path claim 或 isolated worktree。
7. **Snapshot 是不可变内容对象**：Snapshot 创建后不修改；Restore 结果单独记录。
8. **内容寻址支持幂等恢复**：同一 Worktree 状态可复用 Snapshot object，但引用、业务原因和 retention 独立记录。
9. **部分回滚先规划后应用**：Plan 固定 source/target/baseline、路径、Patch digest、冲突和 DAG 影响，用户审批后仍需复验。
10. **恢复优先保存证据**：发现孤儿变更时先 Capture quarantine Snapshot，再终止进程或释放 Claim。

---

## 2. 术语与领域边界

### 2.1 核心术语

| 术语 | 定义 |
|---|---|
| Project Root | 用户登记并授予信任策略的逻辑项目根 |
| Worktree | 某个具体文件系统根；可为主 checkout、Git worktree、Apex 隔离目录或 standalone 目录 |
| Workspace Identity | Worktree 的规范化路径、文件系统身份、仓库身份和基线信息 |
| Canonical Path | 经过平台适配器规范化后、相对于 Worktree Root 的稳定路径 |
| Path Scope | file、directory 或 glob 范围及递归语义 |
| Write Claim | 对一个 Worktree 内一组 Path Scope 的互斥租约 |
| Lease | Core 授予 owner Attempt 的限时执行权 |
| Fence Token | 阻止旧 Worker/进程提交结果的随机令牌 |
| Snapshot | Worktree 某一时刻的不可变文件状态引用 |
| Snapshot Manifest | Snapshot 中每个文件的路径、类型、digest、mode 和 object ref |
| ChangeSet | 两个 Snapshot 或 Snapshot 与当前工作区之间的规范化差异 |
| Restore Plan | 在执行前固定的恢复目标、Patch、冲突和安全摘要 |
| Isolation Workspace | 为单个 Agent/Node 创建、与主工作区隔离的 Worktree |
| Workspace Drift | 当前文件状态与某操作固定的 baseline 不一致 |
| Quarantine Snapshot | 为孤儿/未知变更保存证据的受限 Snapshot |

### 2.2 领域关系

```text
Project 1 ── * Worktree
Worktree 1 ── * Snapshot
Worktree 1 ── * WriteClaim
Agent/NodeAttempt 1 ── 0..1 Worktree Binding
Agent/NodeAttempt 1 ── * WriteClaim
ToolCall 1 ── 0..1 PreSnapshot
ToolCall 1 ── 0..1 PostSnapshot
Snapshot 1 ── * SnapshotRestore
RestorePlan 1 ── 1 RestoreOperation
```

### 2.3 权威层级

| 内容 | 权威位置 |
|---|---|
| Project/Worktree identity | SQLite Current State + Domain Event |
| Claim 生命周期 | SQLite `write_claims` +事件 |
| Snapshot 文件内容 | 影子 Git object store /不可变 Blob |
| Snapshot 生命周期、引用和授权 | SQLite `snapshots`、refs、Operation Journal |
| 用户源代码当前状态 | 实际 Worktree 文件系统 |
| Restore 决策与结果 | SQLite Restore Operation + Event |
| Diff/Patch 大正文 | 内容寻址 Blob |

文件存在不等于业务事实已提交；DB 记录 ready 但对象缺失也属于完整性故障。

---

## 3. Worktree 身份模型

### 3.1 Worktree 来源

沿用 SQLite 设计：

```text
main           Project 主工作区
 git            用户已存在的 Git worktree
apex_isolated  Apex 创建的隔离工作区
external       用户登记的外部/standalone 工作区
```

一个 Project 只能有一个 active primary Worktree，但可以有多个非主 Worktree。不同 Project 不得通过相同 canonical root 产生两个可写身份；若用户从嵌套目录重复登记，应提示现有 Project/Worktree 关系。

### 3.2 WorkspaceIdentity

```rust
struct WorkspaceIdentity {
    project_id: ProjectId,
    worktree_id: WorktreeId,
    display_path: PathBuf,
    canonical_root: CanonicalAbsolutePath,
    filesystem_id: FileSystemIdentity,
    repository: RepositoryIdentity,
    source: WorktreeSource,
    case_mode: CaseMode,
    unicode_mode: UnicodeMode,
    symlink_policy: SymlinkPolicy,
    mount_boundary_policy: MountBoundaryPolicy,
    created_revision: u64,
}
```

`FileSystemIdentity` 应尽可能记录卷/设备标识、文件 ID 能力、大小写敏感性、路径长度限制和原子 rename 能力。`RepositoryIdentity` 可为 Git common-dir identity、bare repository identity 或 standalone digest identity。

### 3.3 RepositoryIdentity

Git 仓库身份不能只靠 `.git` 文本路径判断，应解析：

- `.git` 目录或 gitfile；
- `git rev-parse --git-common-dir`；
- `git rev-parse --show-toplevel`；
- 当前 HEAD、branch、index path；
- submodule/superproject 边界；
- bare 仓库状态；
- object format（SHA-1/SHA-256）；
- safe.directory 和 ownership 诊断。

禁止为识别身份而执行未经信任的项目 hook 或配置命令。

### 3.4 Worktree 状态

建议状态：

```text
registering
  → active
  → draining
  → archived

registering → failed
active → unavailable
unavailable → active
active/draining → deleting → deleted
```

现有 SQLite `status TEXT` 可支持扩展，但实现前应以 migration 加 CHECK，不能依赖自由字符串。

### 3.5 Worktree Binding

Agent Execution Envelope 固定：

```text
project_id
worktree_id
canonical_root_digest
repository_identity_digest
baseline_snapshot_id?
baseline_workspace_digest
isolation_mode
allowed_write_scopes
lease/fence
```

Tool Adapter 不接受模型传入新的 cwd 作为真实执行根；cwd 只能是绑定 Worktree 下经过规范化的相对目录。

---

## 4. 路径规范化

### 4.1 CanonicalPathScope

```rust
struct CanonicalPathScope {
    worktree_id: WorktreeId,
    kind: ScopeKind,       // File | Directory | Glob
    relative_path: String,
    path_key: String,
    recursive: bool,
    case_mode: CaseMode,
    symlink_resolution: SymlinkResolution,
    source_expression: String,
    digest: Digest,
}
```

领域层保存相对路径和稳定 `path_key`，绝对物理路径只在受控 Adapter/诊断中使用，避免跨机器或日志泄漏。

### 4.2 规范化步骤

```text
1. 验证输入编码、NUL、保留设备名和长度
2. 解析相对/绝对路径语义
3. 绑定 worktree root
4. 归一化 separator 和 dot segments
5. 拒绝 root escape
6. 按平台处理 drive/UNC/verbatim prefix
7. 解析已存在父链的 symlink/junction/reparse point
8. 检测 mount/device 边界
9. 按文件系统规则 case fold / Unicode normalize
10. 生成 relative canonical path、path_key、resolution report
```

### 4.3 Windows 特殊规则

Windows 适配器必须处理：

- `C:\`、UNC、`\\?\`、`\\.\`；
- 大小写不敏感但保留显示大小写；
- `CON`、`NUL`、`PRN` 等设备名；
- Alternate Data Streams，如 `file.txt:stream`；
- Junction、mount point、symlink 和其他 reparse point；
- 尾随空格/点、8.3 short name alias；
- 不同 drive 之间 rename 非原子；
- path canonicalization 与最终 handle 打开之间的 TOCTOU。

### 4.4 POSIX 特殊规则

处理 symlink、hard link、bind mount、case-sensitive/insensitive volume、Unicode normalization 差异、设备边界和权限变化。仅调用 `realpath` 不足以处理尚不存在的写目标，应解析最近存在父目录并记录剩余 suffix。

### 4.5 TOCTOU 防护

执行文件写入时必须尽量采用 handle-relative/openat 风格、no-follow 选项和最终 handle identity 校验。若平台能力不足：

1. 执行前再次 canonicalize；
2. 比较父目录文件身份；
3. 写临时文件；
4. 原子替换前再次验证；
5. 不匹配则中止并返回 `PATH_IDENTITY_CHANGED`。

---

## 5. Path Scope 冲突模型

### 5.1 基本冲突

- 同一文件与同一文件冲突；
- 目录与其任意后代冲突；
- recursive directory 与子目录/文件冲突；
- 文件路径与指向同一 inode/file ID 的 hard link 冲突；
- 经过 symlink/Junction 指向同一目标的路径冲突；
- 大小写折叠后相同的路径冲突；
- 无法证明不相交的 glob 按冲突处理。

### 5.2 交集判定

```rust
fn intersects(a: &CanonicalPathScope, b: &CanonicalPathScope) -> ConflictResult {
    ensure_same_worktree(a, b)?;
    if a.path_key == b.path_key { return Conflict; }
    if ancestor_relation(a, b) && recursive_reaches(a, b) { return Conflict; }
    if identity_alias(a, b) { return Conflict; }
    if glob_intersection_provable(a, b) { return Conflict; }
    if glob_disjoint_provable(a, b) { return Disjoint; }
    UnknownConflict
}
```

`UnknownConflict` 在准入层按 Conflict 处理，但 UI 可显示“无法证明两个动态范围不相交”。

### 5.3 Glob 约束

最终产品不允许任意正则作为 Claim Scope。支持受限 glob：

```text
*     单路径段内任意字符
?     单字符
**    跨目录递归
[ab] 受限字符类
{a,b} 有界 alternatives
```

禁止未限制的否定模式、回溯型正则和超大组合展开。编译器必须设置状态数和复杂度上限，超限返回 `PATH_SCOPE_TOO_COMPLEX`。

### 5.4 静态声明与实际写入

Agent/Profile/Workflow 声明的是最大写范围；ToolCall 分析得到本次实际范围：

```text
actual_write_scope ⊆ active_claim_scope ⊆ delegated_write_scope
```

任一包含关系无法证明时，ToolCall 进入 ask/deny/block，不能以“Agent 大概只写这里”放行。

---

## 6. Write Claim 设计

### 6.1 Claim 的语义

Write Claim 是 Core 对某个 Worktree 内路径范围授予的互斥租约。它解决：

- 同一 Worktree 的 Agent/Node 并发写冲突；
- Workflow 调度准入；
- Tool 操作前的路径所有权确认；
- 崩溃后旧 Worker 的 Fence 和恢复；
- 用户界面显示当前谁占用了哪些写范围。

Write Claim **不**解决：

- 用户是否有权修改文件；
- Project Trust；
- Tool Gateway Permission；
- Spec/Rules Gate；
- 外部服务副作用；
- 文件系统本身的强制锁。

### 6.2 状态模型

持久化状态沿用 SQLite 设计：

```text
requested → active → releasing → released
active → expired
active → revoked
requested → revoked
```

Runtime 恢复还需要识别 `suspect`，但为避免破坏既有数据库枚举，建议增加独立字段：

```text
reconcile_state = healthy | inspecting | suspect | confirmed_orphan
```

`state=active + reconcile_state=suspect` 表示业务 Claim 尚未安全释放，不能被新 Claim 忽略；只有 Recovery Reconciler 确认 owner、Lease 和 Workspace 后，才能转 released/expired/revoked。

### 6.3 Claim 记录

```rust
struct WriteClaim {
    claim_id: ClaimId,
    project_id: ProjectId,
    worktree_id: WorktreeId,
    owner_agent_id: AgentId,
    owner_attempt_id: Option<NodeAttemptId>,
    scopes: Vec<CanonicalPathScope>,
    state: ClaimState,
    reconcile_state: ReconcileState,
    lease_token: SecretRef,
    fence_token: FenceToken,
    acquired_at: Timestamp,
    lease_until: Timestamp,
    last_heartbeat_at: Timestamp,
    policy_snapshot: PolicySnapshot,
    baseline_snapshot_id: Option<SnapshotId>,
    version: u64,
}
```

Lease token/Fence token 只保存 hash 或受限引用，不能写入普通 Event、Context、Prompt 或日志。

### 6.4 Claim 获取事务

```text
AcquireWriteClaim(command)
  → validate Actor/Agent/Attempt/Worktree
  → canonicalize all scopes
  → sort scopes by path_key
  → BEGIN IMMEDIATE
  → expire only provably stale claims
  → query candidate active claims
  → application-level intersection check
  → verify baseline and project trust revision
  → insert requested/active claim
  → emit claim.requested + claim.acquired
  → commit
```

SQLite 索引只用于缩小候选集合。父子目录、glob、alias 和 symlink 判定必须在同一 writer transaction 的一致视图内完成。

### 6.5 多 Claim 原子获取

Workflow Node 获取多个范围时不能逐个成功、逐个等待。采用：

```text
normalize → sort → deduplicate → validate all → acquire all in one transaction
```

失败返回完整冲突列表：

```json
{
  "code": "RESOURCE_CONFLICT",
  "worktree_id": "wt_...",
  "requested_scopes": ["src/auth.rs", "tests/auth/**"],
  "conflicts": [
    {
      "claim_id": "clm_...",
      "owner_agent_id": "agt_...",
      "overlap": "src/"
    }
  ],
  "retry_after_us": 500000
}
```

### 6.6 Lease 与心跳

- Lease 必须有最大 TTL，不接受无限期 Claim；
- Heartbeat 只能延长仍拥有有效 Fence 的 Claim；
- Worker 不得自行修改 `lease_until`；
- Core 在 lease deadline 后才可标记 expired；
- 网络延迟导致的迟到 heartbeat 不能复活已被新 owner 取代的 Claim；
- 活跃 Tool/Node 的 Lease 与 Process supervisor Lease 分开，但必须关联。

推荐：

```text
claim_lease_default = 30s
claim_heartbeat_interval = 10s
claim_renewal_grace = 2 * heartbeat_interval
```

实际配置由 Run/Node budget 和本地负载调整。

### 6.7 Fence

每次新的 Claim acquisition、ownership transfer、新 Attempt 接管或对旧 Owner 执行 fenced recovery 时，都生成新的高强度随机 Fence Token，并递增 `claim_version`。普通 heartbeat/续租只延长当前 ownership term 的 deadline，不旋转 Fence Token，避免让正在执行的合法写入瞬间失效。结果提交必须携带：

```text
claim_id
claim_version
lease_token_hash
fence_token
attempt_id
operation_id
```

Core 只接受当前 version/token 匹配的结果；旧 Worker 的迟到写入返回 `STALE_FENCE_TOKEN`，原始观察可审计但不能改变当前状态。

### 6.8 释放

正常释放顺序：

```text
stop new writes
→ finish/commit current operation
→ post snapshot/rules
→ release claim in Core transaction
→ emit claim.released
→ wake compatible waiters
```

异常释放先进入 releasing/inspecting，再由 Reconciler 验证是否仍有进程或未对账 Operation。不能仅因 daemon 内存中没有 owner 就立即释放。

---

## 7. Claim 准入与其他门禁

### 7.1 写操作准入顺序

```text
schema/path/shell normalization
→ capability/trust/hard policy
→ permission decision
→ spec/workflow write scope
→ acquire Write Claim
→ PreTool rules
→ pre snapshot
→ operation intent/dispatch
```

Permission 可以先于 Claim，避免持有 Claim 等用户；审批后重新校验参数 digest、Spec、Rules、Claim、baseline 和 cancellation。

### 7.2 必须同时满足

```text
effective capability
∩ permission allow
∩ project trust
∩ spec gate
∩ rules gate
∩ write claim
∩ workspace binding
∩ snapshot baseline
∩ runtime fence
```

任何交集为空都不能执行。

### 7.3 Permission 与 Claim 等待分离

| 等待 | 是否创建 PermissionRequest | 是否占执行 slot | 恢复依据 |
|---|---:|---:|---|
| awaiting_permission | 是 | 否 | Approval revision/expiry |
| awaiting_claim | 否 | 否 | Claim released/revoked |
| awaiting_user | 否 | 否 | 新用户 Command |
| awaiting_external_reconcile | 否 | 否 | Operation 对账 |

用户批准不等于 Claim 已获取；Claim 获取失败也不能解释成权限拒绝。

### 7.4 Shell 动态写范围

Shell AST 无法证明写入范围时：

- 如果执行可移入隔离 Worktree，优先强制隔离；
- 否则申请宽范围 Claim 和高风险 Permission；
- 若宽范围超过 Agent Ceiling、Project Policy 或安全上限，拒绝；
- 执行后由 Snapshot/Diff 发现实际变更，超出声明范围进入 violation/conflict。

`git -C`、`--git-dir`、`GIT_DIR`、`GIT_WORK_TREE`、脚本动态 cwd 等必须经过 Worktree escape 检查，不能只看命令开头。

---

## 8. 隔离模式

### 8.1 三种模式

沿用 Agent/SQLite 设计：

```text
shared_readonly
path_claim
worktree
```

### 8.2 shared_readonly

适用于只读检索、分析和轻量诊断：

- Agent 不得获得写 Claim；
- Tool Gateway 将 Write/Edit/Delete/Shell write 视为 deny；
- 可读取项目根内允许的内容；
- 读取结果以 observed digest 绑定；
- 发现工作区变化时只刷新 Query，不覆盖历史 Context。

### 8.3 path_claim

适用于可预测、小范围写入：

- Agent 必须声明 allowed_write_scopes；
- Scheduler 原子获取 Claim；
- ToolCall 实际 scope 必须是 Claim 子集；
- 每个含写 Run 有 pre/post Snapshot 或文件级 checksum；
- 超范围变更进入 `CLAIM_SCOPE_VIOLATION`，不能自动提交成功。

### 8.4 worktree

适用于高风险、跨模块、长时间或 Shell 写范围未知的任务：

- Apex 创建独立 Worktree；
- Agent 的 cwd、环境、工具路径和 Git metadata 全部绑定到该 Worktree；
- 主 Worktree 不接受该 Agent 的直接文件写入；
- 完成后生成 ChangeSet/Patch 和 Verification Report；
- 合并回主 Worktree 是新的 ApplyPatch/Restore-like 高风险 Operation；
- 合并前重新获取目标路径 Claim，重新校验目标 baseline。

### 8.5 模式升级与降级

运行中发生以下情况应升级到 worktree 或阻塞：

- Shell 写范围从静态变为动态；
- 实际变更超过 path claim；
- 需要跨模块且无法一次声明完整范围；
- 检测到用户并发编辑；
- Git 操作可能改变 index/history；
- 发生路径 alias 或 symlink 不确定性。

已在主 Worktree 执行的副作用不能通过切换模式“抹掉”；升级前先保存 Snapshot 和诊断。

---

## 9. 隔离 Worktree Provisioning

### 9.1 创建流程

```text
CreateIsolatedWorktree
  → validate project trust and repository identity
  → choose base snapshot/commit
  → reserve workspace directory
  → create provisioning operation intent
  → create git worktree or materialize standalone copy
  → verify root/repository identity
  → capture baseline snapshot
  → register Worktree active
  → bind Agent/NodeAttempt
  → emit worktree.created
```

创建过程中的目录、Git metadata 和 object 共享均不得直接由 Agent/Plugin 控制。

### 9.2 Git Worktree

优先使用 Git 原生 worktree：

```text
git worktree add --detach <isolated_path> <base_commit>
```

实际执行必须由受信任 Apex Git Adapter 以结构化 argv 调用，不接受模型拼接命令。创建后验证：

- `git-common-dir` 与登记仓库一致；
- worktree root 在预期隔离目录；
- HEAD 与 base commit 一致；
- index path 不指向主 Worktree；
- hooks/config 不绕出授权范围；
- branch/ref 更新策略符合 isolation profile。

### 9.3 Standalone/无 Git 项目

无 Git 或 Git 不可用时：

- 以 Snapshot Manifest 为 base；
- 只复制必要文件或使用内容寻址 materializer；
- 保留 mode、mtime 策略、symlink 和忽略规则诊断；
- 生成 `repository_identity=standalone:<digest>`；
- 完成后输出 file-level ChangeSet/Patch；
- 不声称具备 Git merge 语义。

### 9.4 Windows 与跨卷

隔离目录默认放在与项目同卷的位置，确保原子 rename、权限和相对路径行为可预测。跨卷时必须把 provisioning 标记为 `degraded_isolation`，禁止依赖跨卷 rename 的原子性，并加强校验与清理。

### 9.5 Provisioning 失败

失败分支必须：

1. 将 Worktree 标记 failed/unavailable；
2. 保存错误和已创建路径的受限诊断；
3. 删除仅由本次 operation 创建且确认未被用户接管的临时内容；
4. 若无法确认目录归属，转 quarantine，不递归删除；
5. 不释放仍可能被孤儿进程使用的 Claim，交给 Recovery Reconciler。

---

## 10. Worktree Escape 防护

### 10.1 文件工具

Read/Write/Edit/Delete/Glob 等内置工具的所有路径先绑定 Worktree Root，再做 canonicalize。绝对路径、`..`、symlink/Junction、mount 和 hard-link alias 都必须验证；越界写入即使用户曾批准普通项目写权限，也需要新的明确 scope 或硬拒绝。

### 10.2 Shell 与进程

Execution Envelope 固定：

```text
cwd = verified directory under worktree root
env.GIT_DIR = absent unless injected by trusted Git Adapter
env.GIT_WORK_TREE = absent unless injected by trusted Git Adapter
HOME/TMP = isolation scoped where required
PATH = policy controlled
```

必须解析/拦截：

- `cd` 到主 Worktree 或项目外；
- `git -C`；
- `--git-dir`、`--work-tree`；
- `GIT_DIR=...`、`GIT_WORK_TREE=...`；
- PowerShell `Set-Location`、Provider drive；
- cmd `cd /d`；
- 脚本、子 shell 和命令替换中的路径逃逸；
- 从隔离目录启动的后台进程后续写主 Worktree。

### 10.3 Git Adapter

对常见 Git 操作提供结构化 Adapter，避免通用 Shell：

```text
status
diff
show
add-to-shadow-index
write-tree
worktree-add/worktree-remove
apply-patch-dry-run
apply-patch
```

用户 Git 的 `index`、`HEAD`、branch、reflog 和 config 默认只读。改变这些状态必须是独立 Git Operation，不能伪装成 Snapshot 内部实现。

### 10.4 Hook、Skill、MCP 和 Plugin

隔离不是仅限制内置文件工具。扩展执行必须继承同一 cwd、path policy、sandbox、Credential 和 Worktree Binding。任何扩展无法接受这些限制时，不得在隔离 Agent 中启用。

---

## 11. 影子 Git 架构

### 11.1 存储布局

需求约定：

```text
~/apex/snapshots/<project_hash>/<worktree_hash>/.git
```

推荐实际布局：

```text
~/apex/snapshots/
└── <project_hash>/
    └── <worktree_hash>/
        ├── repo.git/             # bare shadow repository
        ├── manifests/            # optional materialized manifests
        ├── locks/                # adapter-owned short locks
        ├── quarantine/           # incomplete/corrupt objects
        └── metadata.json         # non-secret repository identity
```

用户要求的 `.git` 语义可由 `repo.git` 实现；具体目录名通过 ADR 固定。关键要求是每个 Project+Worktree 独立影子 object namespace，不污染用户 `.git`。

### 11.2 对象模型

Snapshot 的核心对象：

```text
blob objects    文件内容
 tree objects    路径层级与 mode
 snapshot root  Git tree OID 或等价 manifest digest
 manifest blob  扩展 metadata、忽略/错误/平台信息
```

Snapshot 默认不需要创建用户可见 commit；SQLite 中的 snapshot_id、git_object_id、base_snapshot_id 和事件提供业务身份。

### 11.3 objects/info/alternates

若用户仓库和影子仓库 object format、所有权和生命周期兼容，可通过 `objects/info/alternates` 只读共享用户 object store，减少重复磁盘。必须注意：

- alternates 不能依赖临时 Worktree 私有 object；
- 用户 GC/repack 可能影响可达性，关键对象需 promote/copy；
- 不允许影子仓库向用户 object store 写对象；
- SHA-1 与 SHA-256 仓库不能混用；
- 跨权限边界和网络文件系统可禁用共享。

最终产品不能把 alternates 当唯一持久性保证；Snapshot ready 前必须证明其对象在 Apex retention 期内可读取。

### 11.4 Shadow Index

Capture 可以使用影子仓库自己的临时 index：

```text
GIT_DIR=<shadow repo>
GIT_WORK_TREE=<registered worktree>
GIT_INDEX_FILE=<apex temp index>
git add --all -- <scoped paths>
git write-tree
```

这些环境变量只由受信任 Adapter 注入，并严格绑定已验证路径；通用 Agent Shell 不可直接获得它们。

### 11.5 文件覆盖范围

Snapshot Manifest 必须说明是否包含：

- tracked files；
- untracked files；
- ignored files；
- symlink；
- executable/mode bits；
- empty directories；
- submodule gitlink；
- sparse checkout 缺失项；
- special file/device/socket；
- large file/LFS pointer 与实际内容。

默认策略：

| 类型 | 默认 |
|---|---|
| tracked | 包含 |
| Agent 新建 untracked | 包含 |
| ignored | 仅 Claim/Tool 触及或策略要求时包含 |
| secrets/sensitive | 允许本地 Snapshot，但加密/受限/不进入 Context |
| special files | 不复制内容，记录诊断并阻断可逆性声明 |
| submodule | 记录 gitlink + 子模块状态，不自动递归写 |
| empty dir | Git 无法表示，manifest 可选记录 |

### 11.6 Snapshot 可逆性等级

```text
full_local_reversible
scoped_reversible
metadata_incomplete
contains_external_effects
non_reversible
```

审批 UI 必须区分“本地文件可恢复”和“整个操作可恢复”。网络、远端 Git push、数据库外部写入不能因为有本地 Snapshot 就声明可回滚。

---

## 12. Snapshot 生命周期

### 12.1 状态机

沿用 SQLite：

```text
intent → creating → ready
intent/creating → failed
creating → unknown
ready → deleting → deleted
```

Snapshot ready 后不可修改。Restore 不改变 Snapshot 状态；通过 `snapshot_restores` 记录。

### 12.2 类型

```text
turn_before
turn_after
node_before
node_after
manual
pre_rollback
import
```

额外业务原因写 metadata：tool_before、tool_after、quarantine、worktree_baseline、patch_apply_before 等，不无限扩张 kind 枚举。

### 12.3 Capture 两阶段协议

```text
CaptureSnapshotIntent
  → transaction:
      snapshots(state=intent)
      operation_journal(intent)
      snapshot.capture_requested event
      outbox
  → adapter verifies worktree identity
  → build shadow index/tree/manifest outside DB transaction
  → fsync/verify objects and digest
  → CommitSnapshot
  → transaction:
      snapshot(state=ready, git_object_id, content_digest)
      blob_refs/retention refs
      snapshot.created event
      projection/outbox
```

### 12.4 Capture 输入

```rust
struct SnapshotCaptureRequest {
    project_id: ProjectId,
    worktree_id: WorktreeId,
    kind: SnapshotKind,
    base_snapshot_id: Option<SnapshotId>,
    path_scope: SnapshotScope,
    expected_workspace_identity: Digest,
    expected_baseline_digest: Option<Digest>,
    owner_run_id: Option<RunId>,
    owner_turn_id: Option<TurnId>,
    owner_attempt_id: Option<NodeAttemptId>,
    retention: RetentionPolicy,
    operation_id: OperationId,
}
```

### 12.5 Capture 一致性

文件系统快照不是天然原子。Capture 采用：

1. 记录 start workspace fingerprint；
2. 枚举范围；
3. 对每个文件读取 metadata + content，必要时重试；
4. 构建 tree/manifest；
5. 再次记录 end fingerprint；
6. 如果范围内文件在 capture 中变化，则重试或标记 unstable；
7. 超过重试上限返回 `WORKSPACE_UNSTABLE`。

支持平台原生 snapshot/VSS/APFS 等能力时可作为优化，但不能改变领域语义。

### 12.6 内容寻址复用

若 `content_digest` 已存在，Adapter 可以复用 object/tree，但仍创建新的 Snapshot 业务记录或增加引用，因为 Run/Turn/Actor/retention 不同。不能只返回旧 snapshot_id 而丢失新的因果链。

---

## 13. Snapshot Manifest

### 13.1 Manifest 结构

```json
{
  "format": "apex.snapshot.manifest.v1",
  "snapshot_id": "snp_01K...",
  "project_id": "prj_01K...",
  "worktree_id": "wt_01K...",
  "repository_identity_digest": "sha256:...",
  "canonical_root_digest": "sha256:...",
  "capture_started_at": "2026-08-08T10:00:00Z",
  "capture_finished_at": "2026-08-08T10:00:01Z",
  "path_scope": ["src/**"],
  "tree_oid": "...",
  "base_snapshot_id": "snp_...",
  "files": [],
  "omissions": [],
  "reversibility": "full_local_reversible",
  "manifest_digest": "sha256:..."
}
```

### 13.2 File Entry

```json
{
  "path": "src/auth.rs",
  "path_key": "src/auth.rs",
  "kind": "regular",
  "mode": "100644",
  "size_bytes": 4231,
  "content_digest": "sha256:...",
  "git_blob_oid": "...",
  "file_identity": "volume:file-id",
  "symlink_target": null,
  "sensitive": false
}
```

Manifest 数组按 path_key 排序并 canonicalize。绝对物理路径、secret 和临时 Lease token 不进入 Manifest。

### 13.3 Omission

无法捕获的内容不能静默忽略：

```json
{
  "path": "tmp/service.sock",
  "reason": "unsupported_special_file",
  "impact": "snapshot_not_fully_reversible"
}
```

只要 omission 影响目标 Claim 范围，就必须降低 reversibility，必要时阻断高风险写入。

---

## 14. Pre/Post Snapshot 策略

### 14.1 风险矩阵

| 操作 | Pre | Post |
|---|---|---|
| 普通 Read | 否 | 否 |
| 单文件 Edit | checksum/文件级基线 | 是 |
| 多文件 Write/Delete | 必须 | 必须 |
| Shell 已知小范围写 | scope Snapshot | 必须 |
| Shell 未知写范围 | isolated Worktree 或宽范围 Snapshot | 必须 |
| Git index/history | Git/index baseline | 是 |
| ApplyPatch/Restore | 必须 | 必须 |
| Workflow Node | node_before | node_after |
| Turn 含写 | turn_before | turn_after |
| 外部远端操作 | 本地 Snapshot 仅作证据 | 收据/对账 |

### 14.2 Snapshot 合并与复用

Turn、Node、Tool 三层都可能要求 Snapshot。Planner 可以复用满足以下条件的 ready Snapshot：

- 同一 Worktree；
- Snapshot scope 覆盖所需范围；
- 在目标操作前创建；
- baseline digest 未变化；
- policy/reversibility 满足要求；
- 未被标记 invalid/corrupt。

复用不能省略新的业务引用和因果关系。

### 14.3 快照失败

需要 Pre Snapshot 的写操作在 Capture 失败时默认不执行。只有明确 Policy 允许、用户看到不可逆风险并强确认后才可继续；CRITICAL 和批量删除不得降级跳过。

---

## 15. ChangeSet 与 Diff

### 15.1 规范 ChangeSet

```rust
struct ChangeSet {
    change_set_id: ChangeSetId,
    from_snapshot_id: SnapshotId,
    to_snapshot_id: Option<SnapshotId>,
    target_workspace_digest: Option<Digest>,
    entries: Vec<FileChange>,
    summary: ChangeSummary,
    patch_blob_id: Option<BlobId>,
    digest: Digest,
}
```

### 15.2 FileChange

```text
added
modified
deleted
renamed
copied
mode_changed
symlink_changed
submodule_changed
binary_changed
unknown
```

每项包含 old/new digest、mode、size、path、rename confidence 和来源 ToolCall/NodeAttempt。

### 15.3 Rename 检测

Rename 是展示和 Patch 优化，不是事实基础。权威事实仍是旧路径删除 + 新路径添加。相似度阈值、二进制和超大文件下可不推断 rename，避免错误配对。

### 15.4 Diff 安全

Diff/Patch 可能含 secret。默认 Event/UI 只返回统计和 ContentRef；完整 Patch 存受限 Blob，下载经过 Project scope、Actor 和敏感级别检查。

### 15.5 归因

同一 Turn 多个 Tool 修改同一文件时，Snapshot diff 只能证明总变化。若需要精确 Tool 归因，每个写 Tool 使用文件 checksum/轻量 post record；不能凭时间戳猜测归属。

---

## 16. Restore 与 Rollback 模型

### 16.1 恢复不是“覆盖回去”

Restore 是受审计、受 Claim 保护、可再次撤销的新写操作，不修改既有 Snapshot，也不抹除原操作历史。一次恢复至少产生：

1. RestorePlan；
2. 风险与冲突分析；
3. 必要的 Permission/Rules 审批；
4. 目标范围 Write Claim；
5. `pre_rollback` Snapshot；
6. Restore Operation；
7. post Snapshot 与 ChangeSet；
8. Event、Audit 与结果收据。

因此，“回滚”在 Apex 中表示追加一个补偿性事实，而不是让时间线倒退。

### 16.2 Restore 模式

| 模式 | 输入 | 典型场景 | 默认策略 |
|---|---|---|---|
| `restore_file` | Snapshot + 单一路径 | 恢复误改文件 | 精确 digest 校验 |
| `restore_scope` | Snapshot + 目录/Scope | 恢复一组文件 | 先列出影响清单 |
| `restore_hunks` | ChangeSet + hunk 集合 | 部分撤销补丁 | 三方应用 |
| `restore_changeset` | ChangeSet | 撤销一次操作 | 反向 Patch |
| `restore_workspace` | Snapshot | 整体工作区恢复 | 高风险、强确认 |
| `restore_metadata` | mode/symlink/index 元数据 | 权限或链接恢复 | 平台能力检查 |

`restore_workspace` 不得隐式包含 Snapshot 未捕获的路径。对 ignored、submodule、special file 等应显式显示覆盖能力与不可恢复项。

### 16.3 RestorePlan

```rust
struct RestorePlan {
    restore_plan_id: RestorePlanId,
    project_id: ProjectId,
    worktree_id: WorktreeId,
    source_snapshot_id: SnapshotId,
    source_change_set_id: Option<ChangeSetId>,
    mode: RestoreMode,
    requested_scope: Vec<PathScope>,
    effective_scope: Vec<PathScope>,
    expected_workspace_digest: Digest,
    expected_file_digests: BTreeMap<PathKey, Option<Digest>>,
    actions: Vec<RestoreAction>,
    conflicts: Vec<RestoreConflict>,
    risk: RiskLevel,
    reversibility: ReversibilityLevel,
    plan_digest: Digest,
    expires_at: DateTime<Utc>,
}
```

Plan 是不可变对象。审批和执行都绑定 `plan_digest`；任何路径、目标 digest、Snapshot 状态或 Policy 变化都使旧 Plan 失效，必须重新规划。

### 16.4 RestoreAction

```text
create_file
replace_file
remove_file
create_directory
remove_empty_directory
replace_symlink
change_mode
apply_text_hunks
apply_binary_blob
update_submodule_pointer
skip_unsupported
```

每个 Action 必须包含目标路径、期望当前 digest、期望结果 digest、敏感级别、估算字节数和回滚能力。目录删除只允许删除由 Plan 明确列出且执行时仍为空的目录。

### 16.5 冲突类型

```text
TARGET_MODIFIED
TARGET_MISSING
TARGET_CREATED
TYPE_CHANGED
PATH_ESCAPED
CASE_COLLISION
SYMLINK_CHANGED
CLAIM_CONFLICT
WORKSPACE_BASELINE_CHANGED
PATCH_CONTEXT_MISMATCH
BINARY_CONFLICT
SUBMODULE_DIRTY
UNSUPPORTED_FILE_TYPE
SNAPSHOT_INCOMPLETE
POLICY_CHANGED
```

默认策略是 fail closed。仅文本 hunk 可在 Policy 允许时进入三方合并；二进制、symlink、submodule 和路径类型变化不做猜测式合并。

### 16.6 三方恢复

三方恢复使用：

- Base：被撤销操作开始前的内容；
- Ours：执行 Restore 时目标工作区当前内容；
- Theirs：希望恢复到的内容。

自动合并成功也不等于语义正确，必须运行对应 Verification Gate。发生冲突时，Apex 写入隔离的 conflict artifact，而不是直接把冲突标记污染用户文件；用户确认后再作为新 ApplyPatch 操作应用。

### 16.7 执行顺序

```text
Load RestorePlan
  -> verify plan_digest and expiration
  -> recanonicalize target paths
  -> acquire/verify Write Claim + fence
  -> verify workspace/file baselines
  -> capture pre_rollback Snapshot
  -> apply actions in deterministic order
  -> verify expected result digests
  -> capture post Snapshot
  -> run Rules + Verification Gates
  -> complete snapshot_restores record
```

创建按父目录到子路径排序，删除按子路径到父目录排序；路径重命名使用临时名称打破环，临时路径必须位于同一受控目录并被 Claim 覆盖。

### 16.8 失败与补偿

Restore 中途失败时：

- 不把状态标为 completed；
- 保留失败现场与逐 Action 收据；
- 尝试使用 `pre_rollback` 生成自动补偿计划；
- 若无法证明补偿安全，则标记 failed 并进入人工恢复；
- Claim 在现场已封存、Owner 已停止且恢复证据已落盘后才可释放。

禁止在未知状态下静默重试非幂等文件动作。

---

## 17. 部分回滚与 DAG 语义

### 17.1 基本原则

部分回滚可能使后续节点的输入失效。Apex 不能只反向应用文件 Patch 而保持 DAG 全部绿色；必须计算受影响的依赖闭包并显式更新运行语义。

### 17.2 影响分析

Rollback Planner 至少考虑：

- 被回滚 ChangeSet 的生产 NodeAttempt；
- 读取过受影响路径的后继节点；
- 使用其 Artifact、Context、Checkpoint 或 Verification 结果的节点；
- 对相同外部资源产生副作用的补偿关系；
- 动态发现、未在静态 DAG 声明的依赖证据。

```text
rollback roots
  -> reverse artifact/path dependency index
  -> transitive downstream closure
  -> classify each node
```

节点分类：

| 分类 | 含义 | 动作 |
|---|---|---|
| `unaffected` | 输入与结果不依赖回滚内容 | 保持有效 |
| `needs_reverify` | 输出可能仍有效但证明过期 | 重新验证 |
| `invalidated` | 输入已改变 | 失效并计划重跑 |
| `needs_compensation` | 有外部副作用 | 执行补偿流程 |
| `manual_review` | 依赖证据不足 | 暂停等待决策 |

### 17.3 Workflow Revision

部分回滚产生新的 Workflow Revision：

- 原 Revision 与 NodeAttempt 保持不可变；
- 新 Revision 记录 rollback operation 与父 Revision；
- 复用仍可证明有效的节点结果；
- invalidated 节点生成新 Attempt；
- Context 以新 Snapshot/ChangeSet 重新装配；
- UI 同时展示“历史成功”和“当前已失效”，避免误导。

### 17.4 Patch 层级回滚

若只选择某些 hunk：

1. 根据原 ChangeSet 生成反向 hunk；
2. 固定 base/target digest；
3. 重新计算实际 Scope；
4. 生成 RestorePlan；
5. 进行三方适用性检查；
6. 通过 Permission、Rules 与 Verification；
7. 作为新 Operation 执行。

Hunk 位置只用于展示，执行必须使用上下文和 digest，不能只按旧行号定位。

### 17.5 外部副作用

文件 Snapshot 无法回滚网络请求、发布、付款、消息发送或远端 Git push。此类节点必须声明：

- `side_effect_kind`；
- idempotency key；
- 外部 receipt；
- 可用的 compensate action；
- compensate 的风险与权限。

没有补偿能力时，部分回滚只能恢复本地状态并将 Workflow 标记为 `externally_diverged`，不得声称已完整回滚。

### 17.6 与暂停/恢复的关系

Rollback 计划生成后 Workflow 进入 `paused_for_rollback`。恢复调度前必须满足：

- Restore 已完成或明确取消；
- 新 workspace baseline 已固化；
- 旧 Attempt 的 Lease/Fence 已失效；
- DAG invalidation 已提交；
- 必须重跑或重验的节点已排入新 Revision。

---

## 18. 隔离 Worktree 结果回传

### 18.1 回传不是复制目录

隔离 Agent 的结果必须转化为可审计 ChangeSet/Patch，再应用到目标 Worktree。禁止直接递归复制隔离目录，因为复制会绕过 Claim、Policy、路径边界、删除检测和冲突分析。

### 18.2 ApplyPatchPlan

```rust
struct ApplyPatchPlan {
    apply_plan_id: ApplyPlanId,
    source_worktree_id: WorktreeId,
    target_worktree_id: WorktreeId,
    base_snapshot_id: SnapshotId,
    source_snapshot_id: SnapshotId,
    patch_blob_id: BlobId,
    patch_digest: Digest,
    target_baseline_digest: Digest,
    scopes: Vec<PathScope>,
    actions: Vec<PatchAction>,
    conflicts: Vec<PatchConflict>,
    verification_profile: VerificationProfileId,
    plan_digest: Digest,
}
```

### 18.3 Apply 流程

```text
freeze source result
  -> capture source post Snapshot
  -> diff against source base
  -> validate result scope
  -> scan sensitive content and prohibited files
  -> calculate target scopes
  -> dry-run against target baseline
  -> request permission if required
  -> acquire target Write Claim
  -> capture target pre Snapshot
  -> apply patch atomically where possible
  -> capture target post Snapshot
  -> verify + emit result
```

若 source Agent 写出了声明范围之外的内容，结果进入 `scope_violation`，默认不生成可直接批准的 ApplyPlan。

### 18.4 基线关系

最理想情况是 source base 与 target current 相同，可直接应用。否则：

- 仅不相交路径变化：可重新基线并继续；
- 文本同路径变化：尝试三方合并；
- 二进制、rename、类型变化冲突：人工处理；
- target 当前状态无法稳定读取：返回 `WORKSPACE_BASELINE_CHANGED`。

### 18.5 Source 保留

Apply 完成后不能立即删除隔离 Worktree。至少保留到：

- post Snapshot ready；
- Verification 结束；
- Patch/Manifest 已持久化；
- Audit 收据可读取；
- Retention Policy 允许回收。

失败或冲突的隔离 Worktree进入 quarantine，避免自动清理销毁唯一证据。

---

## 19. 用户 Git 状态保护

### 19.1 不变量

Apex 的 Snapshot、Diff、Restore 和隔离执行不得隐式改变用户仓库的：

- 当前 branch/HEAD；
- `.git/index` 与 staged 状态；
- stash；
- reflog；
- hooks 配置；
- remotes；
- sparse-checkout 配置；
- submodule 工作状态。

需要改变这些状态的显式 Git Tool 必须按普通高风险写操作单独授权和审计。

### 19.2 Dirty Workspace

用户工作区可以一开始就是 dirty。Snapshot baseline 必须同时记录：

- HEAD/OID（若存在）；
- index digest；
- tracked worktree diff digest；
- untracked/ignored policy digest；
- filesystem identity；
- capture 时刻检测到的并发变化。

Apex 不得把“相对 HEAD 的变化”误当成“本次 Agent 的变化”；Agent 归因使用 pre/post Snapshot 差异。

### 19.3 并发 Git 操作

检测到用户或其他进程执行 checkout、reset、rebase、commit、index 更新时：

- 当前写 Tool 在安全点暂停或失败；
- baseline 标记 drifted；
- 不自动 reset 用户状态；
- 创建新的 Snapshot/ApplyPlan；
- 涉及 Worktree identity 改变时撤销旧 Claim 并重新获取。

### 19.4 Shadow Git 与真实 Git 解耦

Shadow Git 只负责内容树、差异和恢复材料。即使通过 `objects/info/alternates` 读取真实 Git objects，也不能向真实 object store、refs、index 写入。若共享对象随后被 GC，Apex 必须能通过已 materialize 的必要 Blob/Tree 继续恢复；因此 alternates 是空间优化，不是持久性承诺。

---

## 20. 外部修改与 Workspace Drift

### 20.1 Watcher 不是事实源

文件系统 Watcher 用于降低延迟，不作为唯一正确性来源。事件可能丢失、合并、乱序，也可能因编辑器原子替换而只看到 rename。执行前后的 digest 与重新 canonicalize 才是权威判断。

### 20.2 Drift 分类

```text
benign_metadata      时间戳等无内容变化
in_scope_expected    当前 Owner 操作造成且有收据
in_scope_unattributed Claim 范围内但无法归因
out_of_scope         声明范围外变化
identity_changed     inode/file-id/reparse/repo identity 变化
git_state_changed    HEAD/index/worktree 管理状态变化
unknown              无法稳定观测
```

### 20.3 处理策略

| Drift | 处理 |
|---|---|
| benign_metadata | 记录后继续 |
| in_scope_expected | 纳入 post Snapshot |
| in_scope_unattributed | 暂停并检查 Owner/外部进程 |
| out_of_scope | Scope violation；停止写入 |
| identity_changed | fence 当前 Lease，重新解析 |
| git_state_changed | `WORKSPACE_BASELINE_CHANGED` |
| unknown | fail closed 或转隔离 Worktree |

### 20.4 稳定读取

对关键文件采用 `stat -> read/hash -> stat`。若 size、mtime、file-id 或 link identity 在读取前后变化，则本次读取不稳定，有限重试后返回 drift。超大文件可使用 chunk hash，但 Manifest 必须标记算法和完整性级别。

### 20.5 外部编辑保留优先

无法区分用户编辑与 Agent 编辑时，优先保留外部内容并停止自动覆盖。任何强制覆盖都需要新的显式确认，且确认界面必须展示将丢失的 current digest/preview 和可恢复 Snapshot。

---

## 21. 原子写入、删除与文件系统适配

### 21.1 单文件替换

默认单文件写入采用：

1. 在目标父目录创建随机临时文件；
2. 写入完整内容和校验摘要；
3. flush，并按平台能力执行 fsync；
4. 检查目标路径仍符合 canonical path 与 Claim；
5. 原子 rename/replace；
6. flush 父目录（平台支持时）；
7. 读取回校验 post digest。

临时文件名称不得进入用户可见的 ChangeSet，若失败残留必须登记并由 Cleanup 任务处理。Windows 下 replace、打开句柄、杀毒软件锁定等异常必须映射为可诊断错误，不得无限重试。

### 21.2 删除

删除前记录目标的 file-id、类型、digest 和父目录 digest。文件删除在 Claim 覆盖且 baseline 未变化时执行；目录删除仅允许空目录或由 Plan 明确列出全部子项。对跨设备、网络文件系统和特殊挂载点，不做“看似递归”的危险删除。

### 21.3 重命名

rename 必须在同一 Worktree 内完成，并将旧路径和新路径均纳入 Scope。跨目录 rename 要验证两端父目录权限与 Claim；跨设备时转为 copy+verify+delete，但该过程按非原子操作处理并提高风险级别。

### 21.4 Symlink、Junction 与 Reparse Point

默认不跟随链接遍历。读取链接自身的 target 与 metadata；写入链接指向内容需要额外 Policy 和最终解析检查。Windows Junction、mount point、reparse point 与 POSIX symlink 都可能造成 Scope 越界，必须在每个目录边界重新检查。

### 21.5 特殊文件

Socket、device、named pipe、sparse file、ADS、FIFO 等不作为普通 Blob 处理。Apex 应：

- 在 Manifest 标记类型；
- 默认只记录 metadata 或 skip；
- 禁止把特殊文件内容复制到普通目标；
- 将“不支持恢复”作为显式 reversibility 降级，而不是成功。

### 21.6 大文件与流式处理

Snapshot、hash、Patch 和 Restore 均支持流式处理，避免一次性载入内存。Blob 使用分片和 content-addressed digest；分片失败可重试，Manifest 只有在全部分片校验后才进入 ready。

---

## 22. Snapshot 与 Restore 状态机

### 22.1 Snapshot 状态

沿用 SQLite `snapshots.state`：

```text
intent -> creating -> ready
                    ├-> failed
                    └-> unknown
ready -> deleting -> deleted
```

`unknown` 表示进程或适配器在未能提交最终结果时退出，不能直接视为 failed。Reconciler 必须重新检查 Manifest、Blob 和文件系统，再决定恢复为 ready、failed 或 quarantine。

### 22.2 Restore 状态

沿用 SQLite `snapshot_restores.state`：

```text
requested -> approved -> running -> completed
                         ├-> failed
                         ├-> cancelled
                         └-> unknown
```

任何 `unknown` 都要求保留 target Worktree、Operation、Lease、pre/post evidence，并执行恢复审计。服务重启后只能基于幂等 Action receipt 继续，不得盲目从第一步重复写。

### 22.3 Snapshot 事务边界

数据库事务不能包住整个文件扫描。采用 Outbox/Intent 模式：

```text
BEGIN IMMEDIATE
  insert snapshot(intent)
  insert capture job/outbox
COMMIT

adapter captures files and blobs

BEGIN IMMEDIATE
  verify manifest and blob refs
  update snapshot(ready or failed)
  insert snapshot.created/failed event
COMMIT
```

Snapshot 与 Operation、Attempt、Node 的引用只能指向已经创建的 Snapshot ID；ready 前不得作为恢复源使用。

### 22.4 Restore Action Receipt

每个 Action 记录：

```text
action_id
sequence
path_key
before_digest
expected_digest
actual_digest
status
started_at/completed_at
error_code
fence_token
```

执行恢复、服务重启或人工诊断都可据此判断某一步是否已成功，避免重复删除或覆盖。

---

## 23. API、Command 与实时事件

### 23.1 Command

统一通过 CommandBus 写入：

```text
CreateWriteClaim
RenewWriteClaim
ReleaseWriteClaim
RevokeWriteClaim
CaptureSnapshot
PlanRestore
ApproveRestore
ApplyRestore
CancelRestore
CreateApplyPatchPlan
ApproveApplyPatch
ApplyPatch
QuarantineWorktree
ReleaseWorktree
ReconcileWorktree
```

Command 必须携带 `project_id`、`actor_id`、`idempotency_key`、`causation_id` 和可选 `correlation_id`。命令返回 Operation/Plan/Claim 的稳定 ID，不同步返回长时间文件扫描结果。

### 23.2 查询

```text
GET /api/projects/{project_id}/worktrees
GET /api/worktrees/{worktree_id}/claims
GET /api/worktrees/{worktree_id}/snapshots
GET /api/snapshots/{snapshot_id}
GET /api/snapshots/{snapshot_id}/changes
GET /api/restores/{restore_id}
GET /api/restores/{restore_id}/conflicts
GET /api/worktrees/{worktree_id}/drift
```

完整 Patch/Blob 使用短期授权的 ContentRef，不在普通列表接口中直接返回。

### 23.3 事件

```text
worktree.created
worktree.provisioned
worktree.drift_detected
worktree.quarantined
worktree.reconciled
claim.requested
claim.acquired
claim.waiting
claim.renewed
claim.released
claim.revoked
claim.expired
snapshot.intent_created
snapshot.creating
snapshot.ready
snapshot.failed
snapshot.quarantined
restore.requested
restore.approved
restore.running
restore.action_completed
restore.conflict
restore.completed
restore.failed
patch.plan_created
patch.applied
patch.rejected
```

事件只承载摘要、ID、digest、scope 和状态；敏感内容使用受限 Blob 引用。UI 若需要实时进度，通过 Event + Query 拼装，不能把事件作为唯一状态存储。

### 23.4 错误码

建议统一错误码：

```text
CLAIM_CONFLICT
CLAIM_EXPIRED
CLAIM_FENCE_REJECTED
CLAIM_SCOPE_INVALID
WORKTREE_NOT_READY
WORKTREE_IDENTITY_CHANGED
WORKSPACE_BASELINE_CHANGED
SNAPSHOT_NOT_READY
SNAPSHOT_INCOMPLETE
RESTORE_PLAN_EXPIRED
RESTORE_CONFLICT
PATCH_CONTEXT_MISMATCH
PATH_CANONICALIZATION_FAILED
SYMLINK_ESCAPE
SCOPE_VIOLATION
SPECIAL_FILE_UNSUPPORTED
USER_GIT_STATE_CHANGED
```

错误必须包含 safe diagnostic、retryability、conflict IDs 和下一步建议；不得在错误文本中回显 secret 或完整敏感路径，除非 Actor 获得对应权限。

---

## 24. SQLite 映射与新增持久化对象

### 24.1 既有表的职责

本设计不替换已有 SQLite 模型：

- `projects`：项目边界与根路径；
- `worktrees`：工作区身份、路径、来源、状态；
- `agents`：Agent isolation mode 与 capability；
- `write_claims` / `write_claim_scopes`：写租约与路径集合；
- `snapshots`：不可变快照的生命周期和引用；
- `snapshot_restores`：恢复执行状态；
- `operations` / `tool_calls` / `node_attempts`：因果与执行收据；
- `events` / `audit_logs`：事件流与审计。

### 24.2 对 `write_claims` 的补充

现有 Claim state 保持：

```text
requested / active / releasing / released / expired / revoked
```

若需要表达“进程退出后疑似遗留”，不复用上述 state，而增设：

```text
reconcile_state = healthy | inspecting | suspect | confirmed_orphan
```

这样不会破坏终态查询，也修复“suspect 是状态还是诊断标签”的歧义。

### 24.3 建议新增表

```sql
CREATE TABLE workspace_baselines (
    baseline_id TEXT PRIMARY KEY,
    worktree_id TEXT NOT NULL,
    operation_id TEXT,
    snapshot_id TEXT NOT NULL,
    workspace_digest TEXT NOT NULL,
    git_head TEXT,
    git_index_digest TEXT,
    policy_digest TEXT NOT NULL,
    captured_at TEXT NOT NULL,
    state TEXT NOT NULL,
    FOREIGN KEY(worktree_id) REFERENCES worktrees(worktree_id),
    FOREIGN KEY(snapshot_id) REFERENCES snapshots(snapshot_id)
);

CREATE TABLE snapshot_files (
    snapshot_id TEXT NOT NULL,
    path_key TEXT NOT NULL,
    entry_type TEXT NOT NULL,
    digest TEXT,
    mode TEXT,
    size INTEGER,
    sensitive_level TEXT NOT NULL,
    capture_state TEXT NOT NULL,
    PRIMARY KEY(snapshot_id, path_key),
    FOREIGN KEY(snapshot_id) REFERENCES snapshots(snapshot_id)
);

CREATE TABLE restore_plans (
    restore_plan_id TEXT PRIMARY KEY,
    source_snapshot_id TEXT NOT NULL,
    target_worktree_id TEXT NOT NULL,
    plan_digest TEXT NOT NULL,
    expected_workspace_digest TEXT NOT NULL,
    mode TEXT NOT NULL,
    state TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    created_by TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE restore_conflicts (
    conflict_id TEXT PRIMARY KEY,
    restore_plan_id TEXT NOT NULL,
    path_key TEXT,
    conflict_type TEXT NOT NULL,
    base_digest TEXT,
    ours_digest TEXT,
    theirs_digest TEXT,
    resolution_state TEXT NOT NULL,
    artifact_blob_id TEXT
);
```

生产实现可按规模把 `snapshot_files` 放入 Manifest 索引或独立 Blob；但必须能按 `snapshot_id + path_key` 查询和校验，不能只保存不可检索的大 JSON。

### 24.4 索引与约束

建议索引：

```sql
CREATE INDEX idx_claim_scope_active
ON write_claim_scopes(path_key, claim_id);

CREATE INDEX idx_snapshot_worktree_time
ON snapshots(worktree_id, created_at DESC);

CREATE INDEX idx_snapshot_file_digest
ON snapshot_files(path_key, digest);

CREATE INDEX idx_restore_target_state
ON snapshot_restores(worktree_id, state, created_at DESC);
```

冲突检查、Lease 续期、Reconciler 竞态必须在 `BEGIN IMMEDIATE` 下执行；文件系统动作在事务外，完成后以 receipt 回写。

### 24.5 迁移规则

迁移必须：

- 为旧 Worktree 补齐 canonical_path 与 identity 状态；
- 为旧 Snapshot 标记 capture policy 和 reversibility；
- 对没有完整 Manifest 的旧快照标记 `legacy_incomplete`；
- 不删除历史审计记录；
- 迁移失败可重入；
- 在读路径兼容旧 schema，直到所有租户完成迁移。

---

## 25. 崩溃恢复、Lease Reconcile 与孤儿工作区

### 25.1 启动扫描

服务启动或 Worker 恢复时按项目扫描：

1. 未终结的 Claims；
2. active Attempt 与 owner process；
3. creating/unknown Snapshot；
4. running/unknown Restore；
5. provisioned/quarantined Worktree；
6. 未完成的 Apply Operation；
7. 临时文件、lock、outbox 和未引用 Blob。

### 25.2 Owner/Fence 检查

Claim 只有同时满足以下条件才可自动恢复：

- Lease 未过期；
- owner Attempt 仍属于 active Operation；
- process/worker heartbeat 可证明存活；
- Worktree identity 未改变；
- Fence token 未被新 Owner 取代；
- workspace baseline 未产生不可解释 drift。

任一条件不满足，旧 Claim 不得续租；先设置 `reconcile_state=inspecting`，再决定安全释放或 quarantine。

### 25.3 孤儿变化保护

疑似进程退出但 Worktree 有未归因变化时：

1. fence 旧 Claim；
2. capture quarantine Snapshot；
3. 写入 orphan ChangeSet 与诊断；
4. Worktree 标记 `quarantined` 或 `needs_reconcile`；
5. 只读展示证据；
6. 用户或受权恢复流程选择保留、导出、应用或丢弃。

不可在没有 quarantine Snapshot 的情况下直接清空目录或释放唯一隔离 Worktree。

### 25.4 Snapshot Unknown 恢复

对于 `snapshot.state=unknown`：

- 若 Manifest 完整且所有 Blob 校验通过：转 ready；
- 若部分 Blob 存在但不可补齐：转 failed/incomplete；
- 若目标目录 identity 不一致：转 quarantine；
- 若无法判断是否捕获了全部文件：禁止作为 Restore source。

### 25.5 Restore Unknown 恢复

读取 Action receipt：

- 已有 expected digest 的 Action 可标记 completed；
- 当前 digest 既非 before 也非 expected：标记 conflict；
- 没有 receipt 且目标可能已写：重新 capture 当前现场，不重放；
- 仅在生成新 Plan 后才可继续。

---

## 26. 清理、保留与垃圾回收

### 26.1 保留层级

| 对象 | 默认保留触发点 |
|---|---|
| `turn_before/after` | Turn retention |
| `node_before/after` | Workflow/审计 retention |
| `pre_rollback` | 至少覆盖后续验证与补偿窗口 |
| quarantine Snapshot | 用户确认或长期审计终点 |
| Patch/Conflict Blob | 对应 Operation/Restore 完结后按 policy |
| isolated Worktree | Source Result 与验证完成后 |
| shadow Git object | 仍有 Snapshot/Blob 引用时 |

实际时长由 Project Policy 决定，并受合规、敏感级别、项目保留锁影响。

### 26.2 引用追踪

GC 只删除无强引用对象。强引用包括：

- ready Snapshot Manifest；
- Snapshot restore source；
- ChangeSet/ApplyPlan；
- Audit/Event 保留的 ContentRef；
- quarantine 证据；
- 用户 pin 或 legal hold。

Shadow Git 的 object alternates 不算独立保留引用；Apex 必须依据自己的 Snapshot/Blob 引用图决定是否 materialize 或继续保留对象。

### 26.3 删除流程

```text
mark candidate
  -> recheck references in transaction
  -> state=deleting
  -> delete external blobs/worktree
  -> verify absence or quarantine failure
  -> state=deleted
```

删除 Worktree 前必须确认：没有 active Claim、没有 running Operation、没有未导出的冲突、没有未完成 Restore；失败进入 quarantine，不反复强删。

### 26.4 Sensitive 数据

敏感 Blob 使用独立加密/密钥策略和更短访问令牌。GC 日志不记录内容；删除完成后记录 hash、范围、操作者和结果，但不保留可恢复副本，除非 retention/legal hold 要求。

---

## 27. 可观测性与审计

### 27.1 Metrics

核心指标：

```text
apex_claim_wait_seconds{project,scope_kind}
apex_claim_active_total{isolation_mode}
apex_claim_conflict_total{reason}
apex_claim_orphan_total
apex_snapshot_capture_seconds{kind,size_bucket}
apex_snapshot_bytes_total{entry_type}
apex_snapshot_failure_total{reason}
apex_snapshot_dedup_ratio
apex_restore_seconds{mode,result}
apex_restore_conflict_total{type}
apex_workspace_drift_total{class}
apex_worktree_quarantine_total{reason}
apex_patch_apply_total{result}
apex_gc_reclaimed_bytes_total{object_kind}
```

Path、文件名、用户内容不得作为高基数 Metric label。

### 27.2 Trace

一条写链路至少包含：

```text
Command
  -> Admission
  -> Permission/Rules
  -> ClaimAcquire
  -> BaselineVerify
  -> SnapshotCapture(pre)
  -> Tool/Restore/Apply
  -> SnapshotCapture(post)
  -> Diff
  -> Verification
  -> ClaimRelease
```

Trace attribute 使用 ID、scope count、size bucket、digest prefix 和结果，不记录 secret、源码内容或完整命令参数。

### 27.3 Audit

必须审计：

- 谁请求/批准/拒绝写入或恢复；
- 声明范围与实际范围；
- Claim token/fence 的摘要；
- pre/post Snapshot 与 ChangeSet；
- Worktree identity；
- Git 基线与 drift；
- Policy/Rules 版本；
- 自动合并与人工冲突解决；
- 清理、quarantine、导出和销毁。

审计日志追加写，不依赖可被工作区内 Agent 修改的文件。

### 27.4 告警

建议告警条件：

- active Claim 长期无 heartbeat；
- Claim wait P95/P99 激增；
- Snapshot failed/unknown 比例异常；
- `actual_write_scope` 越界；
- quarantine 数量持续增长；
- Restore 失败后没有可用 `pre_rollback`；
- alternates 指向对象缺失；
- user Git state 被非显式 Git Operation 修改；
- Blob 引用计数与 Manifest 扫描不一致。

---

## 28. 性能与容量设计

### 28.1 优先正确，再做复用

首版优化顺序：

1. Snapshot/Claim 正确性；
2. content-addressed Blob 去重；
3. 增量目录扫描；
4. Git object/Tree 复用；
5. 并行 hash 与分片上传；
6. 热 Manifest 索引；
7. 后台压缩与 GC。

不能通过跳过 baseline、缩小未知 Scope 或直接读写真实 `.git/index` 换取速度。

### 28.2 增量扫描

增量索引可使用 path_key、file-id、size、mtime、mode、parent digest 作为候选缓存，但内容 digest 才是最终证明。Watcher 事件可缩小扫描集合，定期全量校验用于发现漏报。

### 28.3 Snapshot 分层

```text
L0 operation checksum: 单文件/小范围即时校验
L1 manifest snapshot: 目录元数据 + 内容引用
L2 materialized blobs: 可独立恢复的内容
L3 archival pack: 冷存储、压缩、长期保留
```

风险较低的短时操作可先生成 L1，并异步 materialize L2，但在 L2 完成前 reversibility 必须显示为 provisional；需要强恢复保障的操作在写前等待 materialization。

### 28.4 超大仓库

超大 monorepo 采用：

- Scope-pruned scan；
- ignore/profile-aware traversal；
- chunked hashing；
- Git tracked tree 复用；
- 并发度和 I/O 限流；
- per-project Snapshot quota；
- backpressure，而非内存积压。

### 28.5 隔离 Worktree 池

可维护按仓库 identity 分组的预热池，但领取前必须：

- 重新验证 base commit；
- 确保无残留 Claim/Process；
- 清理后进行完整 digest 校验；
- 生成新的 worktree_id/fence；
- 不复用上一 Actor 的敏感 Blob 或环境。

无法证明干净时销毁或 quarantine，不能直接回池。

---

## 29. 安全威胁模型

### 29.1 主要威胁

| 威胁 | 例子 | 控制 |
|---|---|---|
| 路径逃逸 | `../../..`、symlink、junction | canonicalize + final identity check |
| Claim 绕过 | shell 间接写声明外路径 | 进程隔离 + post scope audit |
| Git 逃逸 | `git -C`、`--git-dir`、`GIT_DIR` | 参数/环境拦截 + sandbox |
| TOCTOU | 校验后替换链接 | parent handle/file-id/fence |
| 证据污染 | Agent 修改 Snapshot | Snapshot store 与 workspace 隔离 |
| Secret 泄漏 | Patch/Event 含凭证 | scanning + ContentRef + ACL |
| 恢复覆盖用户编辑 | baseline 过期 | digest check + new approval |
| 对象丢失 | alternates 源被 GC | materialize required objects |
| Lease 复活 | 旧进程恢复写入 | monotonic claim_version + cryptographic Fence Token |
| 清理误删 | computed path 越界 | verified root + literal deletion |

### 29.2 不可信进程假设

工具进程、脚本、语言服务器和项目 hooks 默认不可信。仅在参数层包装命令不足以阻止所有写入；生产级强隔离应结合 OS sandbox、受控 mount/ACL、容器或平台等价能力。无法提供强隔离的平台必须在 Capability 中明确降级，不得宣传为 hard enforcement。

### 29.3 Snapshot Store 边界

Snapshot/Blob/Manifest 根目录不能位于 Agent 可写项目目录中。写入只由 Snapshot Adapter 使用受限句柄完成；内容按 project/tenant 隔离，Blob 去重若跨租户必须使用加密域或禁用跨域可推断去重。

---

## 30. 测试与故障注入

### 30.1 单元测试

覆盖：

- canonical path 和 path_key；
- parent/child/file/glob Scope 相交；
- Claim 状态与 fence；
- Snapshot Manifest digest；
- RestorePlan digest；
- Patch reverse/three-way；
- Windows/POSIX 文件类型映射；
- Git 参数与环境逃逸识别。

### 30.2 Property-based 测试

关键性质：

```text
canonicalize(canonicalize(p)) == canonicalize(p)
conflicts(a,b) == conflicts(b,a)
unknown_intersection(a,b) => conflicts(a,b)
actual_write_scope ⊆ active_claim_scope
restore(snapshot(x), mutate(x)) == x  // 在支持的文件类型和无冲突前提下
manifest_digest 不受遍历顺序影响
旧 fence 永远不能覆盖新 fence 的写入
```

随机生成大小写、Unicode 规范化、长路径、保留名、链接环、深目录和 glob 组合。

### 30.3 集成测试

场景至少包括：

1. 两个 Agent 写同一文件；
2. 一个 Claim 目录、另一个 Claim 子文件；
3. 多 Scope 原子获取中途冲突；
4. Lease 到期后旧进程继续写；
5. 用户在 ApplyPatch 前编辑同一文件；
6. Snapshot 创建中进程崩溃；
7. Restore 第 N 个 Action 后崩溃；
8. source base 与 target diverge；
9. dirty index + staged/unstaged/untracked 共存；
10. symlink/junction 指向项目外；
11. `git -C`、`--git-dir`、环境变量逃逸；
12. alternates 源对象被 GC；
13. ignored secret 被 Snapshot policy 拒绝；
14. partial rollback 使下游 DAG 失效；
15. isolated Worktree 清理失败进入 quarantine。

### 30.4 故障注入点

```text
after_claim_commit
before_pre_snapshot_ready
after_blob_write_before_manifest
before_atomic_rename
after_file_replace_before_receipt
after_restore_action_receipt
before_post_snapshot
before_claim_release
during_worktree_remove
during_gc_reference_recheck
```

每个点模拟进程终止、磁盘满、权限变化、文件锁、网络 Blob 超时和 SQLite busy。

### 30.5 验收标准

- 不出现两个冲突 active Claim；
- 任一已允许写操作都能找到 pre baseline 或明确不可逆审批；
- Snapshot ready 时全部引用可校验；
- Restore 不覆盖 baseline 变化而不报告冲突；
- 真实 `.git/index`、branch、reflog 在非 Git Operation 中保持字节/语义不变；
- orphan 写入不会在 reconcile 中被静默删除；
- partial rollback 后 DAG 不保留错误的有效性标记；
- 所有越界尝试产生 `SCOPE_VIOLATION` 或更强阻断。

---

## 31. Rust 模块与端口设计

### 31.1 模块建议

```text
crates/apex-workspace-domain/
  worktree.rs
  path.rs
  scope.rs
  claim.rs
  snapshot.rs
  restore.rs
  patch.rs
  drift.rs

crates/apex-workspace-service/
  claim_service.rs
  snapshot_service.rs
  restore_service.rs
  patch_service.rs
  reconcile_service.rs
  retention_service.rs

crates/apex-workspace-adapters/
  fs/
  git_shadow/
  sqlite/
  blob_store/
  sandbox/
  watcher/
```

Domain crate 不依赖 SQLite、Git CLI 或具体 OS API；Adapter 把平台差异映射成稳定错误。

### 31.2 端口

```rust
#[async_trait]
pub trait WorkspacePort {
    async fn inspect_identity(&self, root: &CanonicalPath) -> Result<WorkspaceIdentity>;
    async fn stable_stat(&self, path: &CanonicalPath) -> Result<StableEntry>;
    async fn apply_actions(
        &self,
        actions: &[WorkspaceAction],
        fence: FenceToken,
    ) -> Result<Vec<ActionReceipt>>;
}

#[async_trait]
pub trait ClaimPort {
    async fn acquire(&self, request: ClaimRequest) -> Result<ClaimLease>;
    async fn renew(&self, lease: &ClaimLease) -> Result<ClaimLease>;
    async fn release(&self, lease: &ClaimLease) -> Result<()>;
    async fn verify_fence(&self, claim_id: ClaimId, fence: FenceToken) -> Result<()>;
}

#[async_trait]
pub trait SnapshotPort {
    async fn capture(&self, request: CaptureRequest) -> Result<SnapshotRef>;
    async fn diff(&self, request: DiffRequest) -> Result<ChangeSet>;
    async fn plan_restore(&self, request: RestoreRequest) -> Result<RestorePlan>;
    async fn restore(&self, plan: &RestorePlan, lease: &ClaimLease) -> Result<RestoreReceipt>;
    async fn retain(&self, snapshot_id: SnapshotId, reason: RetainReason) -> Result<()>;
    async fn release(&self, snapshot_id: SnapshotId, reason: RetainReason) -> Result<()>;
}

#[async_trait]
pub trait IsolatedWorktreePort {
    async fn provision(&self, request: ProvisionRequest) -> Result<ProvisionedWorktree>;
    async fn freeze_result(&self, worktree_id: WorktreeId) -> Result<SnapshotRef>;
    async fn quarantine(&self, worktree_id: WorktreeId, reason: QuarantineReason) -> Result<()>;
    async fn destroy(&self, worktree_id: WorktreeId) -> Result<()>;
}
```

### 31.3 Fence 强制点

Fence 不只在 Service 入口验证，还应传到 Adapter 写入调用。每批 Action 前、长操作安全点和最终 commit 前重新验证；否则旧 Worker 在入口验证后暂停、随后恢复，仍可能覆盖新 Owner。

### 31.4 Error 类型

```rust
pub enum WorkspaceError {
    CanonicalizationFailed,
    SymlinkEscape,
    IdentityChanged,
    ClaimConflict(ConflictSet),
    LeaseExpired,
    FenceRejected,
    BaselineChanged(DriftReport),
    SnapshotIncomplete,
    RestoreConflict(Vec<RestoreConflict>),
    PatchContextMismatch,
    UnsupportedFileType,
    Quarantined,
    AdapterFailure(SafeDiagnostic),
}
```

错误类型与 API code 一一映射，内部 source chain 写受保护日志，前端仅获取 safe diagnostic。

---

## 32. 分阶段交付

> ADR-0001 / ADR-0024（跨文档一致性审查）：本节 Phase 与产品版本档位的映射为 **Phase 1 → v0.1、Phase 2 → v0.1（影子 Git 部分）+ v0.5（隔离 Worktree 部分）、Phase 3 → v0.5～v0.7**。需注意需求文档 §5.1 把"文件快照（影子 Git + 基础回滚）"列入 v0.1，因此 Phase 2 中的 shadow repository、object alternates 与 materialization 属 v0.1 范围；隔离 Worktree provisioning、ApplyPatchPlan 与 quarantine/reconcile 可留到 v0.5 随 DAG 并行调度一同交付。Phase 1 的 Claim + Lease + fence 属 v0.1，与 Tool Gateway `INV-TG-007` 一致。

### Phase 1：边界与最小安全闭环

- Project/Worktree identity；
- path canonicalization；
- file/directory Scope；
- SQLite Claim + Lease + fence；
- 单文件/目录 Snapshot；
- pre/post Snapshot；
- file/scope restore；
- Audit/Event；
- 真实 Git 状态保护测试。

完成标准：共享工作区的已知范围写入可串行化、可审计、可恢复。

### Phase 2：Shadow Git 与隔离 Worktree

- shadow repository；
- object alternates + materialization；
- isolated Worktree provisioning；
- source result freeze；
- ApplyPatchPlan；
- shell/Git escape control；
- quarantine/reconcile。

完成标准：高风险 Agent 可在隔离空间工作，结果通过 Patch 安全回传，不污染用户 `.git`。

### Phase 3：高级恢复与 DAG 联动

- hunk/ChangeSet partial rollback；
- three-way restore；
- dependency closure；
- Workflow Revision；
- downstream invalidation/reverify；
- external compensation metadata。

完成标准：局部回滚不会让 DAG 保留错误的成功语义。

### Phase 4：规模化与合规

- chunked Blob、冷热分层；
- retention/legal hold；
- 多租户加密边界；
- worktree pool；
- 全量故障注入；
- 容量/性能 SLO；
- 管理员诊断与导出。

### 32.1 不得延期的基础安全项

即使采用 MVP，也不能延期：

- canonical path 与 symlink escape 检查；
- Claim 原子获取；
- fence token；
- 写前 baseline；
- Snapshot Store 与项目写域隔离；
- user Git index/branch/reflog 保护；
- Restore baseline 检查；
- orphan evidence quarantine。

---

## 33. 架构决策记录（ADR 摘要）

### ADR-WS-001：Project 与 Worktree 分离

**决定**：所有写、Snapshot 和 Claim 绑定 Worktree；Project 负责逻辑归属。

**理由**：同一项目可有主工作区、外部 Worktree、Apex 隔离 Worktree，路径和基线不同。

### ADR-WS-002：Claim 是租约，不是权限

**决定**：Permission/Rules 与 Write Claim 分开评估。

**理由**：前者回答“是否允许”，后者回答“现在是否安全独占”；两者失败语义和等待策略不同。

### ADR-WS-003：未知相交即冲突

**决定**：无法证明两个 Scope 不相交时视为冲突。

**理由**：宁可降低并发，也不能允许不可预测的并发写破坏可恢复性。

### ADR-WS-004：Snapshot 不可变，Restore 是新操作

**决定**：恢复不修改历史 Snapshot，不删除原 Operation。

**理由**：保持审计、因果和重复恢复能力。

### ADR-WS-005：隔离结果通过 Patch 回传

**决定**：不递归复制隔离目录到用户工作区。

**理由**：Patch 可预览、检查范围、检测删除与冲突，并经过 Claim/Rules/Verification。

### ADR-WS-006：Shadow Git 不写用户 `.git`

**决定**：独立 repo/index/ref namespace；alternates 仅作读取优化。

**理由**：避免污染 branch、index、stash、reflog 和用户工具状态。

### ADR-WS-007：Reconcile 先保全证据

**决定**：孤儿 Claim 释放前捕获 quarantine Snapshot。

**理由**：进程死亡不代表其文件变化无价值，也不能把用户并发编辑当残留清理。

### ADR-WS-008：部分回滚产生 Workflow Revision

**决定**：原 DAG/Attempt 不可变，新 Revision 表达 invalidation 和重跑。

**理由**：避免历史重写，并明确区分“曾成功”和“当前仍有效”。

### ADR-WS-009：Watcher 只作提示

**决定**：正确性依赖 stable stat、digest、identity 与 fence。

**理由**：跨平台 Watcher 事件不完整且可能乱序。

### ADR-WS-010：持久状态与诊断状态分离

**决定**：Claim 生命周期保持既有 state；以 `reconcile_state` 表达 `suspect` 等诊断阶段。

**理由**：避免终态查询、唯一约束与恢复逻辑产生歧义。

---

## 34. 设计审查清单

### 34.1 Worktree 与路径

- [ ] Project 与 Worktree ID 是否始终分开？
- [ ] 所有外部路径是否先转 project-root-relative canonical path？
- [ ] Windows 大小写、Unicode、UNC、ADS、设备名是否覆盖？
- [ ] symlink/junction/reparse 的最终身份是否复验？
- [ ] 路径比较是否使用 path_key 而非 UI 字符串？

### 34.2 Claim

- [ ] 多 Scope 是否在一个事务中全取或全不取？
- [ ] parent/child、file/directory/glob 是否正确冲突？
- [ ] 未知相交是否 fail closed？
- [ ] Lease 是否有 heartbeat、expiry、单调 claim_version 和高强度随机 Fence Token？
- [ ] Adapter 写入前是否再次检查 fence？
- [ ] `actual_write_scope` 是否验证为 active Claim 子集？
- [ ] Claim wait 是否与 permission wait 分开？

### 34.3 Snapshot

- [ ] 所有写 Run 是否有可证明的 pre baseline？
- [ ] Snapshot ready 是否代表 Manifest/Blob 全部可校验？
- [ ] ignored/untracked/sensitive/special file 策略是否显式？
- [ ] alternates 对象丢失时是否仍可恢复？
- [ ] Snapshot Store 是否位于 Agent 不可写边界？
- [ ] pre/post Snapshot 能否关联 Operation、ToolCall、NodeAttempt？

### 34.4 Restore 与 Patch

- [ ] Plan 是否绑定 current baseline 和 plan_digest？
- [ ] Apply 前是否重新 canonicalize 和获取 Claim？
- [ ] 是否必有 `pre_rollback`？
- [ ] 二进制/类型变化冲突是否禁止猜测合并？
- [ ] 失败 Action 是否有 receipt 和可诊断现场？
- [ ] partial rollback 是否更新 DAG 有效性？
- [ ] 外部副作用是否明确为不可由文件 Snapshot 撤销？

### 34.5 Git 与隔离

- [ ] 非 Git Operation 是否保证不变更用户 index/branch/reflog？
- [ ] `git -C`、`--git-dir`、`GIT_DIR`、`GIT_WORK_TREE` 是否被控制？
- [ ] source result 是否以 Snapshot/ChangeSet 冻结？
- [ ] Apply 是否经过 target Claim、pre/post Snapshot 和 Verification？
- [ ] 失败 Worktree 是否进入 quarantine 而非强制清理？

### 34.6 恢复与运维

- [ ] creating/unknown Snapshot 是否可 reconcile？
- [ ] running/unknown Restore 是否按 receipt 恢复？
- [ ] orphan Claim 是否先保存变化？
- [ ] GC 是否进行事务内引用复查？
- [ ] 日志、Metric、Event 是否避免泄露源码和 secret？
- [ ] 是否有崩溃、磁盘满、文件锁、SQLite busy 故障注入？

---

## 35. 与既有 Apex 文档的一致性

| 既有文档 | 本文落实内容 |
|---|---|
| `Apex—— 需求分析文档.md` | Shadow Git、turn 前后快照、文件/补丁回滚、write_paths 互斥、隔离 Agent |
| `Apex—— 系统总体架构设计.md` | Project/Worktree 边界、SnapshotPort、路径规范化、用户 Git 零污染 |
| `Apex—— 领域模型与事件规范.md` | Claim/Snapshot/Restore 不变量、Operation 因果、不可变事件 |
| `Apex—— API与实时事件协议设计.md` | 异步 Command、Operation ID、Event + Query、ContentRef |
| `Apex—— SQLite数据模型与迁移设计.md` | worktrees、write_claims/scopes、snapshots、snapshot_restores 状态和事务边界 |
| `Apex—— Agent Runtime与DAG调度器详细设计.md` | Lease/Fence、workspace drift、partial rollback、Revision/invalidation |
| `Apex—— Tool Gateway与权限引擎详细设计.md` | Permission 与 Claim 分离、Scope Gate、Shell/Git 逃逸、写前 Snapshot |
| `Apex—— Context与Checkpoint系统详细设计.md` | Snapshot/Checkpoint 引用、恢复后的 Context 重装配、内容引用与保留 |

### 35.1 解决的跨文档歧义

1. **Claim 的 `suspect`**：不加入已有持久 lifecycle state，改为独立 `reconcile_state`；
2. **Snapshot vs Checkpoint**：Snapshot 是 Workspace 内容证据，Checkpoint 是 Runtime 恢复锚点，可引用 Snapshot 但不等同；
3. **Project vs Worktree**：所有路径锁和文件恢复以 Worktree 为物理边界；
4. **Permission vs Claim**：批准不代表获得写租约，租约等待也不触发重复授权；
5. **Rollback vs History Rewrite**：回滚是新 Operation 和新 Workflow Revision；
6. **Shadow Git alternates**：共享对象仅优化，必要对象必须 materialize 才能承诺长期恢复。

### 35.2 下一份设计的输入

本文将以下事实交给 `Apex—— Rules与Verification Gate详细设计.md`：

- Rules 在 Claim 之前可评估计划，在实际写入后必须评估 ChangeSet；
- Restore、ApplyPatch 和 scope violation 都必须有专用 Gate；
- Verification 绑定 Snapshot/ChangeSet digest，Workspace drift 后旧结果失效；
- 自动三方合并、partial rollback 和 ignored/sensitive 文件需要更高验证配置；
- Gate 结果必须区分 `pass / fail / inconclusive / stale`；
- 远端副作用只能验证/补偿，不能由本地 Snapshot 宣称已撤销。

---

## 附录 A：Claim 冲突伪代码

```rust
fn scopes_conflict(a: &PathScope, b: &PathScope, fs: &ScopeIndex) -> bool {
    if a.worktree_id != b.worktree_id {
        return false;
    }

    match prove_disjoint(a, b, fs) {
        Proof::Disjoint => false,
        Proof::Intersecting => true,
        Proof::Unknown => true,
    }
}

fn acquire_claim(tx: &mut ImmediateTx, req: ClaimRequest) -> Result<ClaimLease> {
    let scopes = normalize_sort_dedup(req.scopes)?;
    let active = tx.load_overlapping_claim_candidates(req.worktree_id, &scopes)?;

    let conflicts = active
        .iter()
        .filter(|c| c.state == ClaimState::Active)
        .filter(|c| c.scopes.iter().any(|x| scopes.iter().any(|y| scopes_conflict(x, y, tx.scope_index()))))
        .collect::<Vec<_>>();

    if !conflicts.is_empty() {
        return Err(WorkspaceError::ClaimConflict(conflicts.into()));
    }

    tx.insert_claim_with_scopes(req, scopes)
}
```

候选索引可提高性能，但最终冲突判定不能只依赖字符串前缀；glob、大小写和链接身份必须进入判定。

---

## 附录 B：Snapshot Capture 伪代码

```rust
async fn capture(req: CaptureRequest) -> Result<SnapshotRef> {
    let intent = repo.create_snapshot_intent(&req).await?;
    let root_identity = workspace.inspect_identity(&req.root).await?;
    let mut manifest = ManifestBuilder::new(intent.snapshot_id, root_identity);

    for path in walker.walk_scopes(&req.scopes).await? {
        let canonical = canonicalizer.resolve_beneath(&req.root, &path).await?;
        policy.assert_capturable(&canonical, &req.capture_policy)?;

        let entry = workspace.stable_read(&canonical).await?;
        let blob = match entry.kind {
            EntryKind::RegularFile => Some(blob_store.put_stream(entry.stream).await?),
            EntryKind::Symlink => Some(blob_store.put_bytes(entry.link_target).await?),
            _ => None,
        };
        manifest.push(entry, blob)?;
    }

    let sealed = manifest.seal()?;
    blob_store.verify_references(&sealed).await?;
    repo.mark_snapshot_ready(intent.snapshot_id, sealed).await
}
```

---

## 附录 C：Restore 执行伪代码

```rust
async fn apply_restore(plan: RestorePlan, lease: ClaimLease) -> Result<RestoreReceipt> {
    plan.verify_digest_and_expiry(clock.now())?;
    claim.verify_active_and_fence(&lease).await?;
    workspace.verify_baseline(plan.expected_workspace_digest).await?;

    let pre = snapshots.capture(CaptureRequest::pre_rollback(&plan)).await?;
    let restore = repo.start_restore(&plan, &pre).await?;

    for action in plan.actions.iter() {
        claim.verify_active_and_fence(&lease).await?;
        workspace.verify_action_baseline(action).await?;
        let receipt = workspace.apply_action(action, lease.fence_token).await?;
        repo.record_action_receipt(restore.id, receipt).await?;
    }

    let post = snapshots.capture(CaptureRequest::post_restore(&plan)).await?;
    let changes = snapshots.diff(DiffRequest::between(pre.id, post.id)).await?;
    verification.run_for_restore(&plan, &changes).await?;
    repo.complete_restore(restore.id, post, changes).await
}
```

---

## 附录 D：典型冲突示例

| Claim A | Claim B | 结果 | 原因 |
|---|---|---|---|
| `file: src/a.rs` | `file: src/b.rs` | 可并行 | 可证明不相交 |
| `directory: src/**` | `file: src/a.rs` | 冲突 | 父子包含 |
| `glob: src/**/*.rs` | `file: src/a.rs` | 冲突 | 明确匹配 |
| `glob: generated/**` | `glob: src/**` | 可并行 | 静态根不相交且无链接逃逸 |
| `glob: **/*.rs` | `file: tests/a.rs` | 冲突 | 可能匹配 |
| `file: Foo.txt` | `file: foo.txt` | 平台相关，默认冲突 | 大小写语义未知或不敏感 |
| `file: link/x` | `file: outside/x` | 冲突/拒绝 | link 越出 Worktree |
| 不同 Worktree 同相对路径 | 不同 Worktree | 可并行 | 物理边界不同；共享外部资源另算 |

---

## 附录 E：核心不变量汇总

```text
I1  每个写动作属于且仅属于一个 Project/Worktree/Operation。
I2  每个路径在使用前都已 canonicalize，并在写前复验身份。
I3  冲突 Scope 不会同时拥有 active Write Claim。
I4  旧 fence token 永远不能代表新 Lease 写入。
I5  actual_write_scope ⊆ active_claim_scope ⊆ delegated_write_scope。
I6  需要可恢复的写操作在执行前有 ready baseline Snapshot。
I7  ready Snapshot 是不可变、可校验的 Manifest + Blob 引用集合。
I8  Restore/ApplyPatch 是新写操作，具有自己的 Claim、pre/post Snapshot 和审计。
I9  非显式 Git Operation 不改变用户 branch、index、stash、reflog。
I10 隔离 Worktree 结果只能通过受控 ChangeSet/Patch 回传。
I11 unknown/suspect/orphan 状态下先保全证据，再释放或清理。
I12 partial rollback 后所有依赖结果都被重新证明、失效或补偿。
I13 Snapshot 只能撤销被捕获的本地副作用，不夸大远端可逆性。
I14 GC 不删除任何仍被 Snapshot、Restore、Audit、quarantine 或 legal hold 引用的对象。
```

---

**文档结论**：Apex 的 Workspace 安全不能由单一 Git 快照、单一路径锁或一次权限确认实现。完整产品必须把规范路径、Write Claim Lease/Fence、不可变 Snapshot、Shadow Git、隔离 Worktree、Patch 回传、Restore Plan、DAG invalidation、故障 Reconcile 与保留审计组合成同一条可验证链路。只有当“允许写、能够独占、写前可恢复、写后可归因、冲突可停止、崩溃不丢证据、回滚不伪造历史”同时成立，Apex 才能向用户提供可信的自主 Agent 文件操作能力。

