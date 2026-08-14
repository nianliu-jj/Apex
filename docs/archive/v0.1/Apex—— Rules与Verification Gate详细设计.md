# Apex—— Rules与Verification Gate详细设计

> 文档状态：详细设计基线（面向最终完整产品）  
> 适用版本：Apex v0.1 ～ v1.x（按 §分阶段交付 分档启用；档位表以需求文档 §5.3 为准）  
> 编制日期：2026-08-08
>
> 适用范围：Apex 最终完整产品；覆盖 Ruleset 发现与编译、RuleCheck、Diagnostic、Pre/Post Tool Gate、Spec Gate、Node/Workflow Completion Gate、Restore/ApplyPatch Gate、Repair Run、Verification Artifact、例外与审计。
>
> 上游依据：`Apex—— 需求分析文档.md`、`Apex—— 系统总体架构设计.md`、`Apex—— 领域模型与事件规范.md`、`Apex—— API与实时事件协议设计.md`、`Apex—— SQLite数据模型与迁移设计.md`、`Apex—— Agent Runtime与DAG调度器详细设计.md`、`Apex—— Tool Gateway与权限引擎详细设计.md`、`Apex—— Workspace快照、Write Claim与隔离工作区详细设计.md`。

---

## 0. 文档目的与范围

### 0.1 要解决的问题

Apex 的 Spec 流水线要求“需求—设计—任务—实现—验证—交付”形成可审计闭环，编码规范引擎要求每次 Write/Edit 后自动执行增量检查，Agent Runtime 又要求 Node/Workflow 只能在满足结构化证据和完成门禁后进入终态。仅把 lint/test 命令塞进 Prompt 或在 UI 中显示绿色文字，无法形成可靠的产品语义。

本文定义一套独立于 Provider、客户端和具体检查工具的 **Rules 与 Verification Gate 内核**，回答：

1. 规则来源如何发现、合并、编译和版本化；
2. 一个 RuleCheck 的输入、输出、证据和失败类型如何固定；
3. PreTool、PostTool、Spec、Node、Workflow、Restore、ApplyPatch 等 Gate 如何串联；
4. `pass / fail / inconclusive / stale / skipped / waived` 如何区分；
5. 诊断如何稳定指纹、去重、定位、修复和关闭；
6. 自动修复如何通过新的 Repair Run 受控执行，而不是由 Checker 静默改文件；
7. 规则失败、检查器故障、工作区漂移和用户例外如何形成不同状态；
8. 如何把本地文件验证、外部副作用验证、DAG 完成和最终验收报告统一起来。

### 0.2 设计目标

- **确定性**：相同 Ruleset、输入 digest、工具版本和环境声明，得到可比较的相同结论。
- **证据优先**：模型声称“已经完成”不构成通过，Gate 必须绑定可验证的 Artifact、Snapshot、ChangeSet、命令收据或外部 receipt。
- **与权限分离**：Rules/Verification 可以阻断或要求验证，但不能授予 Permission、Capability、Project Trust 或 Write Claim。
- **输入冻结**：每次检查固定 ruleset revision、文件/ChangeSet digest、命令/config digest 和 runner profile。
- **增量优先**：PostToolUse 默认只检查受影响 Scope，但必须有依赖闭包和定期全量校验机制。
- **失败语义准确**：违反规则、检查器崩溃、超时、输入漂移、用户跳过和未执行不能混成一个 failed。
- **修复可追溯**：修复 Agent 是新的 Agent/Run/Operation，拥有自己的 Claim、Snapshot、Gate 和审计。
- **高危默认阻断**：安全硬规则、Scope 越界、未知副作用、关键 Gate 缺失时 fail closed。
- **跨端一致**：TUI、Desktop、Web 看到同一 Ruleset、Diagnostic、Gate 和证据投影。
- **可恢复**：Gate、RuleCheck、Repair Run 和 Verification Artifact 都能在崩溃、断线和重启后对账。

### 0.3 非目标

本文不定义：

- 某种具体语言的 lint 规则内容；
- 某个测试框架、编译器或 SAST 工具的实现细节；
- Provider Prompt 中的质量评价话术；
- 用户 Git hosting、CI 平台的全部功能；
- 远端支付、发布、消息发送等外部系统的通用事务实现。

---

## 1. 核心架构结论

1. **Ruleset 是不可变 revision**：加载、解析、合并、编译和发布后生成稳定 digest；运行中的 RuleCheck 不追随文件热变更。
2. **RuleCheck 是事实对象，Gate 是决策对象**：RuleCheck 记录某个检查器对固定输入的结果，Gate 聚合多个 Check/Artifact/Policy 证据决定是否允许状态转换。
3. **Gate 不等于 Rule**：Gate 可以要求测试、Snapshot、Claim、Spec Review、外部 receipt 或 DAG 条件；Rule 只是其中一种证据来源。
4. **输入 digest 是通过资格**：验证结果必须绑定 Workspace baseline、ChangeSet、Artifact revision、Ruleset revision、Runner profile 和依赖环境摘要。
5. **`stale` 不是 `fail`**：输入变化后旧结果失效，必须重新检查；不能把过期结果显示为失败，也不能继续作为完成证据。
6. **`inconclusive` 不是 `pass`**：检查器故障、环境不足、网络断开、输出不完整或外部状态未知时，Gate 默认阻断或进入人工对账。
7. **warning 与 error 分离**：默认 warning 不阻断，但 Project 可以收紧；Project 不能放宽 Builtin hard safety deny 或产品级必需 Gate。
8. **PostTool violation 与 Tool 成功分离**：工具可以已经成功产生文件变化，同时 RuleCheck 发现违反规则；ToolCall 为 `succeeded_with_violations`，Run/Node 仍可能 blocked。
9. **Checker 不直接写用户工作区**：自动修复必须由新的 Repair Agent/Run 通过 Tool Gateway 执行；Checker 只能产生 Diagnostic、Patch suggestion 或受限 Artifact。
10. **验证不改变历史**：重新验证、修复和例外都产生新的 RuleCheck/GateAttempt/Workflow Revision，不覆盖原始结果。
11. **验证结果需要最小充分范围**：增量检查可按 changed paths 执行，但跨文件依赖、配置、生成代码和安全边界变化必须扩大范围。
12. **本地通过不代表外部成功**：本地测试与文件 Snapshot 不能证明远端发布、网络写入或第三方服务副作用已完成；外部结果需要 Adapter receipt 和独立 reconciliation。

---

## 2. 术语与领域边界

| 术语 | 定义 |
|---|---|
| Rule | 一个可版本化、可执行或可解释的约束单元 |
| Ruleset | 按来源、优先级和目标范围合并后发布的 Rule revision 集合 |
| Rule Source | 项目 `apex/rules/`、兼容规范文件、全局规则或内置规则的来源记录 |
| Rule Compiler | 把 Markdown/YAML/TOML/代码配置编译为规范化 Ruleset 的组件 |
| RuleCheck | 使用固定 Ruleset 与输入对一个或多个 Rule 执行检查的事实记录 |
| Diagnostic | RuleCheck 或其他验证器产生的结构化问题、建议或信息 |
| Fingerprint | 根据规则、位置、消息模板和证据生成的稳定诊断指纹 |
| Verification | 对代码、Artifact、Workspace、Workflow 或外部 receipt 的证据核验 |
| Gate | 决定某个业务状态转换是否可发生的组合决策点 |
| Gate Attempt | 一次 Gate 评估及其输入、证据、结论和解释 |
| Evidence | 支持 Gate 决策的 Snapshot、ChangeSet、Command receipt、Test report、Artifact 或外部 receipt |
| Runner Profile | 验证器执行所需的工具版本、沙箱、资源预算和环境声明 |
| Repair Run | 针对 Diagnostic 创建的独立修复 Agent/Run/Operation |
| Exception/Waiver | 有权限的用户或组织对特定规则/诊断在受限范围内的例外决定 |
| Stale | 输入或依赖事实已改变，原结果不再适用于当前目标 |
| Inconclusive | 检查没有产生足够证据，无法判断通过或失败 |
| Completion Gate | 阻止 Node、Workflow、Spec 或交付进入终态的 Gate |

### 2.1 领域职责边界

```text
Rule Compiler       负责把来源编译为可执行 Ruleset
Rule Runner         负责在固定输入上运行 RuleCheck
Diagnostic Store    负责诊断归一化、指纹、生命周期和证据引用
Gate Evaluator      负责聚合证据并决定状态转换
Repair Planner      负责将诊断编译为新的修复计划
Tool Gateway        负责修复/测试命令的 Permission、Claim、Snapshot 与执行
Runtime/Scheduler   负责把 Gate 结果接入 Node/Run/Workflow 状态机
Application/API    负责 Command、Query、Event 和审批
```

### 2.2 Rules 不能取代的能力

Rules Engine 不拥有以下权力：

- 不能通过规则结果授予工具 Capability；
- 不能绕过 Permission、Project Trust、Sandbox 或 Write Claim；
- 不能把未经用户确认的 Spec 当成 approved；
- 不能直接修改工作区或用户 Git 状态；
- 不能把外部操作 unknown 变成 succeeded；
- 不能以 warning 形式覆盖 Builtin hard safety deny；
- 不能把自然语言中的“已经修复”直接当成证据。

---

## 3. 规则来源与优先级

### 3.1 来源优先级

默认发现顺序与需求文档一致：

```text
1. Project: <project_root>/apex/rules/
2. Project compatibility: AGENTS.md / CLAUDE.md / project rule files
3. Global: ~/apex/rules/
4. Builtin: Apex safety baseline and product invariants
```

“优先级”不表示低层规则可以覆盖高层安全拒绝。它表示：同名规则的配置继承、说明合并和默认值解析顺序。最终 effect 仍遵循 hard deny > deny > ask > allow，且项目只能收紧质量规则，不能放宽产品硬规则。

### 3.2 规则来源记录

```rust
struct RuleSourceRevision {
    source_revision_id: SourceRevisionId,
    project_id: Option<ProjectId>,
    scope: RuleSourceScope,
    path: CanonicalPath,
    source_type: SourceType,
    source_digest: Digest,
    parser_version: String,
    trust_class: TrustClass,
    discovered_at: Timestamp,
    loaded_at: Timestamp,
    status: SourceStatus,
}
```

规则源文件本身是不可信输入。加载器必须限制：

- 可读取的根目录；
- 文件大小、嵌套深度和 include 数量；
- include/import 是否允许跨项目；
- 规则脚本是否需要编译或执行；
- source 中包含的命令、路径和网络端点；
- 对外部内容的 taint 标记。

### 3.3 兼容文件解析

`AGENTS.md`、`CLAUDE.md` 和自然语言规则不能直接作为安全硬规则执行。Apex 将其分成：

1. 可识别的结构化约束；
2. 供 Agent/Checker 参考的说明性内容；
3. 无法确定语义的待确认条目。

只有经过 Rule Compiler 明确解析并生成 normalized Rule 的条目才可以参与自动 Gate。无法结构化的内容可进入 Context，但不得伪装成已验证的阻断规则。

### 3.4 规则文件变更

发现规则来源变化时：

- 当前正在执行的 RuleCheck 继续使用原 Ruleset revision；
- 新 Tool/Node/Spec Gate 使用新 revision；
- 旧 Gate 结果不能自动适用于新 revision；
- 规则更新产生 `ruleset.updated` 事件；
- 受影响的未完成 Node 标记 `verification_stale`，由 Scheduler 重新评估；
- 规则解析失败不静默使用旧配置，除非明确存在仍有效的 pinned revision。

### 3.5 Builtin 规则

Builtin 规则覆盖产品安全与一致性不变量，例如：

```text
project_root_escape = hard_deny
secret_exfiltration = hard_deny_or_block
user_git_pollution = hard_deny
scope_violation = hard_deny
stale_baseline_apply = hard_deny
unknown_external_side_effect = block
missing_required_pre_snapshot = block
invalid_fence = hard_deny
checker_result_without_input_binding = invalid
```

Builtin 规则带有 Apex 版本、migration policy 和兼容测试；不能通过项目规则、Permission 模式或 `/skip-spec` 关闭。

---

## 4. Ruleset 模型与编译

### 4.1 Rule 结构

```rust
struct Rule {
    rule_id: RuleId,
    revision: u64,
    source_revision_id: SourceRevisionId,
    name: String,
    description: String,
    category: RuleCategory,
    severity: Severity,
    enforcement: Enforcement,
    trigger: TriggerSet,
    selector: Selector,
    input_requirements: Vec<InputRequirement>,
    evaluator: EvaluatorSpec,
    fix_strategy: Option<FixStrategy>,
    resource_budget: ResourceBudget,
    evidence_requirements: Vec<EvidenceRequirement>,
    enabled: bool,
}
```

### 4.2 Rule 分类

```text
safety              路径、权限、secret、Git 和隔离安全
correctness         编译、类型、测试、Schema、协议一致性
architecture        模块边界、依赖方向、禁止 API
style               格式、命名、文件组织
spec_acceptance     对照需求/验收标准
artifact_integrity  输出格式、引用、digest、版本
workflow            DAG 输入、产物、依赖和完成条件
external_reconcile  外部 receipt、幂等与对账
```

分类影响默认 severity、Gate 适用范围和资源上限，但不能绕过统一权限链路。

### 4.3 Enforcement

```text
hard_deny       无法由用户例外覆盖，立即阻断危险动作
block           阻止目标状态转换，允许创建修复或人工决策
error           默认阻断完成；Project 可调整显示，不可放宽安全等价项
warning         默认不阻断；可被 Project 收紧为 error
info            仅提供信息
advisory        仅提供优化建议
```

`hard_deny` 主要用于在副作用发生前的安全规则；PostTool 发现已经产生的违规时，RuleCheck 记录 violation，Operation 不能被伪装成未执行。

### 4.4 Trigger

```text
ruleset_compile
spec_stage
pre_tool
post_tool
post_snapshot
repair_post_tool
node_start
node_completion
workflow_completion
restore_plan
restore_post
apply_patch_plan
apply_patch_post
release_delivery
reconcile
manual_audit
```

Rule 的 trigger 是声明性上界，实际是否执行还由 Gate Profile、Scope、风险和输入要求决定。

### 4.5 Selector 与 Scope

Selector 可以按以下维度筛选：

- Project/Worktree identity；
- 文件 canonical path、language、extension、generated 状态；
- Tool、Operation、Node type、Artifact type；
- Spec/Workflow revision；
- changed path、ChangeSet category；
- sensitive level、external side effect kind；
- Actor/Agent capability ceiling。

Selector 的路径语义必须复用 Workspace 文档的 canonical Path Scope，不能使用未经规范化的字符串 glob。

### 4.6 Evaluator 类型

```text
builtin_predicate
command_check
process_check
library_check
ast_check
schema_check
artifact_query
snapshot_diff_check
dependency_graph_check
external_receipt_check
human_review_required
```

`command_check` 和 `process_check` 必须通过 Tool Gateway/Verification Runner 调度，不能由 Rule Engine 直接启动任意 Shell。验证命令本身也要有 tool revision、sandbox、timeout、输出策略和 Operation receipt。

### 4.7 编译结果

```rust
struct CompiledRuleset {
    ruleset_id: RulesetId,
    revision: u64,
    project_id: Option<ProjectId>,
    source_revisions: Vec<SourceRevisionId>,
    rules: Vec<CompiledRule>,
    precedence_digest: Digest,
    compiler_version: String,
    schema_version: u32,
    ruleset_digest: Digest,
    warnings: Vec<CompilerDiagnostic>,
    status: RulesetStatus,
}
```

编译后的 Ruleset 采用规范排序、规范 JSON/MessagePack 编码和 digest。规则列表顺序不能依赖文件系统遍历或 SQLite 返回顺序。

### 4.8 编译失败

- 安全硬规则解析失败：Project Ruleset 不可发布，相关 Gate 使用 Builtin 安全基线并进入 `inconclusive` 或 blocked；
- 普通质量规则解析失败：Ruleset 发布失败，不能静默丢弃；
- 说明性规则无法结构化：记录 compiler warning，不能自动阻断除非用户将其转为结构化 Rule；
- include 循环、路径越界、规则 ID 冲突和 matcher 过宽：编译失败；
- 新 Ruleset 未完成编译前，继续使用当前 pinned revision 的运行不会被打断。

---

## 5. RuleCheck 输入契约

### 5.1 固定输入

每个 RuleCheck 必须固定：

```rust
struct RuleCheckInput {
    project_id: ProjectId,
    worktree_id: Option<WorktreeId>,
    operation_id: Option<OperationId>,
    tool_call_id: Option<ToolCallId>,
    node_attempt_id: Option<NodeAttemptId>,
    trigger: Trigger,
    ruleset_id: RulesetId,
    ruleset_digest: Digest,
    input_scope: Vec<PathScope>,
    workspace_baseline: Option<BaselineRef>,
    snapshot_id: Option<SnapshotId>,
    change_set_id: Option<ChangeSetId>,
    artifact_refs: Vec<ArtifactRef>,
    command_digest: Option<Digest>,
    config_digest: Digest,
    runner_profile: RunnerProfileRef,
    dependency_digest: Option<Digest>,
    environment_digest: Digest,
    input_digest: Digest,
}
```

### 5.2 Input Digest 组成

`input_digest` 至少由以下规范化字段计算：

```text
ruleset_digest
trigger
canonical input scope
workspace identity
baseline/snapshot/change_set digest
artifact revision + checksum
command/config digest
runner profile + tool versions
dependency/environment digest
```

任何字段改变都必须使结果变为 stale 或触发新 RuleCheck。不能只比较文件 mtime。

### 5.3 增量范围

PostToolUse 默认使用：

```text
direct_changed_paths
  + imported/config dependency closure
  + generated artifact relations
  + rule-declared companion scopes
  + safety boundary files
```

以下变化默认扩大到 Project 或模块级：

- build/test/lint 配置；
- lockfile、workspace manifest、compiler version；
- 权限/规则/沙箱配置；
- public API、schema、migration；
- generated code source；
- `../../../.gitignore`、secret policy、path mapping；
- 规则声明的 global dependency。

### 5.4 Input Snapshot 与 ChangeSet

- pre Tool Check 可以绑定 pre Snapshot 或稳定文件 checksum；
- post Tool Check 优先绑定 post Snapshot 与 ChangeSet；
- Node Completion 必须绑定 Node before/after Snapshot 或等价完整变更证据；
- Restore/ApplyPatch Gate 必须绑定 `pre_rollback`/pre-apply 与 post Snapshot；
- 没有 ready Snapshot 或 ChangeSet 不完整时，结果最多为 inconclusive，不得为 pass。

### 5.5 未捕获文件

检查器发现目标范围内存在未纳入 input manifest 的文件时：

- 对安全、发布、Restore 和 Workspace integrity Gate：阻断；
- 对仅展示性的 style 检查：可记录 inconclusive；
- 不得把“未扫描”显示为“无问题”。

---

## 6. RuleCheck 生命周期与结果语义

### 6.1 持久生命周期

**本节修订领域模型 §5.12 的 RuleCheck 状态机**（原为 `queued → running → passed | violations_found | checker_failed | timed_out | cancelled`），改为生命周期与业务结论分离的二维模型：

```text
queued → running → completed
                  ├→ cancelled
                  ├→ interrupted
                  └→ unknown
```

`completed` 表示检查器进程和结果收据已经稳定提交，不代表业务上通过。

原状态机的四个终态在新模型中的落点：

| 领域模型原状态 | 新模型 |
|---|---|
| `passed` | `state=completed, verdict=pass` |
| `violations_found` | `state=completed, verdict=fail, failure_kind=violations_found` |
| `checker_failed` | `state=completed, verdict=inconclusive, failure_kind=checker_failed` |
| `timed_out` | `state=completed, verdict=inconclusive, failure_kind=checker_timeout` |
| `cancelled` | `state=cancelled` |

领域模型要求「`violations_found` 是有效检查结果，`checker_failed` 是检查基础设施故障，二者必须分别统计和处理」——该要求在新模型中由 `verdict`（fail vs inconclusive）与 `failure_kind` 两个维度共同保证，区分度不降低反而提高：`verdict=fail` 表示代码确有问题，`verdict=inconclusive` 表示未能取得结论，二者在查询与告警上天然可分。

> ADR-0008（跨文档一致性审查）：本节原写作"沿用领域模型的生命周期"，但给出的取值集与领域模型 §5.12 不同，属**修订**而非沿用。已改为显式声明修订并给出完整映射表。领域模型 §5.12 与 SQLite `rule_checks` 表已同步更新。


### 6.2 业务 verdict

在 lifecycle 之外增加独立的 `verdict`：

```text
pass
fail
inconclusive
stale
skipped
waived
```

这解决既有 SQLite `rule_checks.state` 使用 `failed`，而领域模型使用 `violations_found/checker_failed/timed_out` 的歧义：

- `state` 表示 RuleCheck 是否完成、取消或异常；
- `verdict` 表示业务检查结论；
- `failure_kind` 表示 checker_failed、timed_out、violations_found 等原因。

### 6.3 Failure Kind

```text
violations_found
checker_failed
checker_timeout
runner_unavailable
input_missing
input_unstable
workspace_drift
ruleset_invalid
output_invalid
permission_denied
sandbox_denied
external_unknown
cancelled_by_user
interrupted_by_crash
```

`violations_found` 通常对应 `state=completed, verdict=fail`；`checker_failed` 通常对应 `state=completed, verdict=inconclusive`；`workspace_drift` 通常对应 `verdict=stale`；具体 Gate Profile 可以在失败语义上进一步收紧，但不能把无证据变为 pass。

### 6.4 状态转换

```text
queued
  → running
  → completed + pass/fail/inconclusive
  → completed + stale
  → cancelled
  → interrupted → unknown/reconciled
```

规则检查器退出码为 0 不直接等于 pass。Runner 必须验证输出 schema、输入 digest、工具版本、退出状态和结果完整性。

### 6.5 幂等与复用

可复用结果必须满足：

- 同一 ruleset_digest；
- 同一 input_digest；
- 同一 Runner profile 与 checker version；
- 结果未被撤销、污染或标记 stale；
- 引用的 Snapshot/Artifact/Blob 仍可读取；
- 当前 Gate 允许复用该 trigger 的结果。

复用必须创建新的 Gate evidence reference，不能伪造新的 RuleCheck 执行。

### 6.6 取消与崩溃

取消只取消尚未产生不可逆副作用的检查运行；已启动的进程需通过 Process Supervisor 停止并记录结果。规则检查本身通常可安全重试，但若 checker 生成了外部副作用，必须按 Tool Operation 对账。

---

## 7. Diagnostic 模型

### 7.1 结构

```rust
struct Diagnostic {
    diagnostic_id: DiagnosticId,
    project_id: ProjectId,
    rule_check_id: Option<RuleCheckId>,
    gate_attempt_id: Option<GateAttemptId>,
    source_type: DiagnosticSource,
    source_id: String,
    rule_id: Option<RuleId>,
    severity: Severity,
    enforcement: Enforcement,
    code: String,
    message_template: String,
    rendered_message: SafeText,
    path: Option<CanonicalPath>,
    range: Option<SourceRange>,
    fingerprint: String,
    evidence_refs: Vec<ContentRef>,
    fix_hint: Option<FixHint>,
    taint: TaintInfo,
    state: DiagnosticState,
    created_at: Timestamp,
    resolved_at: Option<Timestamp>,
}
```

### 7.2 Fingerprint

Fingerprint 使用规范化字段计算：

```text
project identity
rule_id + rule revision
canonical path
source range normalized to stable anchor
diagnostic code
message template, not volatile rendered values
relevant symbol/AST anchor
```

不把绝对路径、随机临时目录、行号单独作为唯一依据。代码移动后，若 AST/symbol anchor 仍能匹配，Diagnostic 可以迁移；无法可靠迁移则创建新 fingerprint 并标记旧项 stale。

### 7.3 Severity 与状态

```text
info        open → acknowledged/resolved
warning     open → suppressed/resolved
error       open → repaired/waived/resolved
critical    open → repaired/waived，仅限有资格主体
hard        open → repaired，不允许 waiver
```

`waived` 不是 `resolved`。它表示问题仍可能存在，但当前 Gate 在受限条件下允许继续，且必须有 waiver expiry、scope、reason、approver 和 policy revision。

### 7.4 Diagnostic 合并

重复检查不能无限生成相同问题。以 `(source_type, source_id, fingerprint)` 关联：

- 同一结果重复上报：更新 observed count 和 last_seen；
- 输入新 revision 仍存在：保留关联历史；
- 新检查未发现旧 fingerprint：旧 Diagnostic 进入 resolved/stale，不能直接删除；
- Repair Run 产生新问题：新 RuleCheck 和新 fingerprint 独立记录；
- 用户确认“误报”：记录 suppression/exception，不改写 checker 原始结果。

### 7.5 内容与安全

Diagnostic message、source line、patch suggestion、test output 可能包含 secret、Prompt Injection 或恶意文本：

- 原文存 Blob/ContentRef，按敏感级别访问；
- Event/UI 默认只返回脱敏摘要；
- 外部命令输出标记为 untrusted/tainted；
- 不把 Diagnostic 原文未经标记地注入 System Prompt；
- Repair Agent 获取的是结构化、边界明确的修复输入。

---

## 8. Verification Gate 统一模型

### 8.1 Gate 不是单个命令

Gate 是一个有版本的决策配置，可能要求多个并行或串行 Evidence：

```rust
struct GateDefinition {
    gate_id: GateId,
    revision: u64,
    kind: GateKind,
    subject_type: SubjectType,
    required_checks: Vec<CheckRequirement>,
    required_artifacts: Vec<ArtifactRequirement>,
    required_state_predicates: Vec<StatePredicate>,
    risk_policy: RiskPolicy,
    aggregation: AggregationPolicy,
    repair_policy: Option<RepairPolicy>,
    waiver_policy: Option<WaiverPolicy>,
    timeout: Duration,
    definition_digest: Digest,
}
```

### 8.2 Gate 类型

```text
spec_stage_gate
spec_approval_gate
pre_tool_gate
post_tool_gate
restore_plan_gate
restore_post_gate
apply_patch_plan_gate
apply_patch_post_gate
node_start_gate
node_completion_gate
workflow_completion_gate
verification_delivery_gate
release_gate
reconcile_gate
```

### 8.3 Gate Subject

Gate Subject 必须是不可变或可定位的事实：

- Spec/Artifact revision；
- ToolCall/Operation；
- NodeAttempt；
- Workflow Revision；
- RestorePlan/Restore Operation；
- ApplyPatchPlan/Apply Operation；
- Project release candidate；
- Reconciliation incident。

不能对“当前 UI 看起来的工作区”直接给出永久通过。

### 8.4 Gate 状态

```text
pending
running
passed
failed
blocked
inconclusive
stale
waived
cancelled
unknown
```

Gate `passed` 只表示在固定输入和 policy 下满足要求。输入 digest、Ruleset revision、Workspace identity、Spec revision 或 Gate Definition 改变后，旧 Gate 自动转为 stale 或需要重新评估。

### 8.5 Gate verdict 聚合

默认聚合：

```text
hard_deny / critical failure       => failed or blocked
required check inconclusive       => inconclusive
required check stale              => stale
required check missing            => inconclusive
error violation                   => failed
warning only                      => pass with warnings
valid waiver                      => waived
all required evidence pass        => passed
```

`blocked` 用于需要用户决策、Claim、Permission、外部对账或 Repair Run 的暂停；`failed` 表示已确定不满足；`inconclusive` 表示证据不足；`stale` 表示证据曾有效但已不再适用。

### 8.6 Gate Explanation Tree

每次 Gate 都生成解释树：

```text
Gate result
├─ policy snapshot
├─ definition revision
├─ subject identity
├─ input digest
├─ required evidence
│  ├─ RuleCheck verdict
│  ├─ test receipt
│  ├─ Snapshot/ChangeSet
│  ├─ Artifact checksum
│  └─ external receipt
├─ failed/blocked predicates
├─ waiver/exception references
└─ next actions
```

UI 可以展示摘要，但 API 和审计必须能获取完整树或受权限保护的 ContentRef。

---

## 9. Spec Gate 与规则内嵌

### 9.1 Spec 阶段 Gate

Spec 流水线至少包含：

```text
requirements draft -> requirements review
requirements approved -> design generation
 design approved -> task compilation
 tasks approved -> implementation admission
 implementation verified -> verification delivery
 verification approved -> spec completed
```

每个批准绑定 Artifact revision、checksum、review actor、review decision 和 Gate Definition revision。Artifact 被修改后，下游批准和验证结果按依赖关系标记 stale。

### 9.2 Rules 内嵌到 Design

生成 `design.md` 时，Spec Compiler 从当前 Ruleset 中抽取与该 feature 相关的：

- architecture constraints；
- language/build/test requirements；
- security and secret requirements；
- write scope 和隔离要求；
- required Verification Profile；
- acceptance criteria 模板；
- 禁止项与例外策略。

内嵌内容保存 `ruleset_digest` 和 Rule refs。Design 文本是人类可审阅镜像，权威约束仍是不可变 Ruleset 与 Artifact revision，防止 Markdown 手工编辑后产生隐式语义漂移。

### 9.3 Tasks 编译

`tasks.md` 编译为 Workflow 时，每个 Node 声明：

```yaml
verification:
  preconditions:
    - spec_revision_approved
  post_tool_profiles:
    - incremental-quality
  completion_profiles:
    - compile
    - unit-tests
    - changed-paths-within-claim
  acceptance_criteria:
    - AC-REQ-004
  evidence_outputs:
    - test-report
    - change-set
```

未声明验证要求不代表无需验证；Builtin 和 Project 默认 Completion Profile 仍适用。

### 9.4 `/skip-spec`

`/skip-spec` 只跳过完整 Spec Artifact 流水线，不跳过：

- Project Trust；
- Permission；
- Write Claim；
- pre/post Snapshot；
- Builtin hard rules；
- PostTool 增量检查；
- Node/Run 基础完成 Gate；
- 审计与 `spec_skipped=true` 标记。

Skip 产生不可删除的审计事实，并使最终 Verification Report 显示缺失的 Spec acceptance mapping。需要正式发布时，Release Gate 可要求补齐 Spec 或由有资格用户提交独立例外。

### 9.5 Acceptance Criteria

Acceptance Criterion 必须有稳定 ID：

```rust
struct AcceptanceCriterion {
    criterion_id: CriterionId,
    artifact_revision_id: ArtifactRevisionId,
    statement: String,
    verification_method: VerificationMethod,
    required_evidence: Vec<EvidenceRequirement>,
    severity: CriterionSeverity,
    owner: Option<ActorRef>,
}
```

自然语言 Criterion 不能由模型自评直接通过。若无法绑定自动化 Evidence，则使用 `human_review_required`，并明确审阅对象、展示材料和批准范围。

---

## 10. PreTool Gate

### 10.1 运行顺序

PreTool Gate 位于副作用前：

```text
normalize request
  -> capability/trust/hard policy
  -> permission decision
  -> spec binding
  -> pre-tool rules
  -> claim admission
  -> pre snapshot
  -> adapter dispatch
```

具体实现可为避免长时间持有 Claim，把纯计算规则放在 Claim 前；任何依赖当前 Workspace 身份的规则必须在 Claim 获取后、Adapter dispatch 前复验。

### 10.2 PreTool 输入

- canonical Tool Revision 与 arguments digest；
- Actor/Agent/Run/Node identity；
- Project Trust revision；
- Capability snapshot；
- Permission decision/ref；
- canonical read/write/network scopes；
- risk classification；
- Spec/Workflow binding；
- Workspace baseline；
- sandbox/credential plan；
- Ruleset revision。

### 10.3 PreTool 决策

```rust
pub enum PreToolVerdict {
    Allow { constraints: ExecutionConstraints },
    Ask { reason: DiagnosticRef, risk_delta: RiskDelta },
    Deny { diagnostic: DiagnosticRef },
    Block { blockers: Vec<BlockerRef> },
    ProposeRewrite { new_request: ProtectedContentRef },
}
```

`ProposeRewrite` 不能沿用旧 Permission、argument digest 或 Claim；接受后创建新的 ToolCall 并从 normalization 重新开始。

### 10.4 强制 PreTool 规则

典型规则：

- 写路径未 canonicalize 或越出 Project；
- 声明 Scope 超过 Agent capability ceiling；
- Shell 写范围未知且未隔离；
- `git -C`、`--git-dir`、`GIT_DIR` 等逃逸；
- Credential 注入超过 Tool 所需 Scope；
- 缺少必须的 Spec/Task binding；
- 高风险写操作缺少可用 Snapshot 能力；
- Restore/ApplyPatch Plan digest 或 baseline 已变化；
- checker/hook 试图获得未声明的网络或写能力。

### 10.5 PreTool Checker 失败

安全关键 PreTool Checker 超时、崩溃或输出无效时 fail closed。纯观测或低风险提示型 Rule 可以 inconclusive 并继续，但 Gate 必须显示“未检查”，不能显示通过。

### 10.6 Hooks 与 Rules

Hook 是扩展执行机制，Rule 是领域约束。Hook 可以：

- 产生 Diagnostic；
- 请求收紧参数或 sandbox；
- 提供额外 evidence；
- 请求 Ask/Block。

Hook 不能：

- 授予 Permission/Capability；
- 隐藏 Builtin Diagnostic；
- 修改 Tool 结果或状态历史；
- 直接写目标 Worktree；
- 把自身超时解释为 pass。

---

## 11. PostTool 增量 Gate

### 11.1 运行顺序

```text
Adapter execution receipt
  -> post Snapshot
  -> ChangeSet generation
  -> actual scope audit
  -> PostTool RuleChecks
  -> PostTool Hooks
  -> ToolCall result classification
  -> continue / block / repair
```

PostTool Gate 使用实际 ChangeSet，而不是仅使用 Tool 声明或 stdout。

### 11.2 增量检查

默认 lint-staged 模式：

- 只对 changed/new/renamed 文件运行文件级检查；
- 删除文件执行引用、manifest 和 ownership 检查；
- 配置/API/schema 变化扩大依赖范围；
- generated 文件按 source-of-truth 关系检查；
- 二进制、symlink、submodule 进入专用规则；
- Scope 越界始终全量阻断，不受增量优化影响。

### 11.3 Tool 状态映射

| Adapter 结果 | PostTool Gate | ToolCall 结果 |
|---|---|---|
| success | pass | `succeeded` |
| success | warning only | `succeeded` + warnings |
| success | fail/block | `succeeded_with_violations` |
| success | inconclusive required | `succeeded_with_violations` 或 blocked |
| failed, known no effect | 任意 | `failed` |
| side effect unknown | 不可判定 | `reconcile_required` |
| cancelled after effect | post evidence required | `interrupted/reconcile_required` |

已发生的副作用不能因 Gate 未通过而把 ToolCall 改成“未执行”。

### 11.4 Actual Scope Audit

PostTool Gate 必须验证：

```text
actual_write_scope ⊆ active_claim_scope ⊆ delegated_write_scope
actual_network_scope ⊆ authorized_network_scope
actual_secret_use ⊆ credential_plan
```

越界产生高危 Diagnostic、停止后续写入并触发 Reconcile/Quarantine。不能在事后自动扩大 Claim 或 Permission 来掩盖违规。

### 11.5 检查器选择

Rule Planner 根据 ChangeSet 选择检查器：

```text
.rs                  rustfmt/clippy/unit selector/architecture AST
.ts/.vue             formatter/lint/typecheck/component rules
Cargo.toml           dependency/license/build graph/full compile
migration/schema     migration integrity/compatibility
API protocol         schema compatibility/generated clients
rules/config         Ruleset compile + affected Gate stale analysis
sensitive files      secret/security scan
```

具体工具由 Project Runner Profile 配置；架构只要求选择可解释、版本固定和范围可审计。

### 11.6 增量不足时的升级

以下情况自动升级为模块/Project 全量检查：

- 依赖图缺失或不可信；
- ChangeSet 包含公共 API 或类型导出；
- 构建/测试配置变化；
- 规则版本变化；
- 多次 Repair 后仍循环出现诊断；
- 增量结果与最近全量结果矛盾；
- Release/Workflow Completion Profile 明确要求全量。

---

## 12. Node Completion Gate

### 12.1 必须满足的基础条件

NodeAttempt 进入 completed 前至少验证：

- AgentOutcome schema 正确；
- 必要 Artifact 已 materialize；
- actual changed paths 在 Claim/Task Scope 内；
- 所有 Tool Operation 已终结或已对账；
- 没有 `EXTERNAL_OPERATION_UNKNOWN`；
- required RuleCheck/Verification 已得到非 stale 结果；
- Snapshot/ChangeSet 与当前 Workspace baseline 一致；
- Acceptance Criteria 已有证据；
- Lease/Fence 当前有效；
- Workflow/Node revision 仍接受该 Attempt。

### 12.2 Completion Profile

```rust
struct CompletionProfile {
    profile_id: VerificationProfileId,
    revision: u64,
    checks: Vec<CheckRequirement>,
    artifact_requirements: Vec<ArtifactRequirement>,
    scope_policy: ScopePolicy,
    warning_policy: WarningPolicy,
    inconclusive_policy: InconclusivePolicy,
    retry_policy: VerificationRetryPolicy,
    repair_policy: RepairPolicy,
    profile_digest: Digest,
}
```

### 12.3 完成原子性

以下事实应在同一 Core 事务中提交：

- GateAttempt terminal；
- NodeAttempt terminal 或 blocked；
- AgentOutcome ref；
- ChangeSet/Snapshot refs；
- RuleCheck/Verification refs；
- Diagnostic summary；
- Claim release intent；
- `node_attempt.completed/blocked` Event 与 Outbox。

长时间检查在事务外运行，但完成决策必须在事务内重新验证 input digest、Workflow revision 和 Fence。

### 12.4 多输出节点

Node 生成多个 Artifact/模块时，Gate 可以对每个输出分组评估，但 Node 的完成条件遵循定义的聚合策略：

```text
all_required      所有 required output 通过
quorum            仅适用于非安全并行评审
any                只适用于候选探索节点
manual_selection  用户选定候选后再验证目标
```

安全、实现和发布节点默认 `all_required`。

### 12.5 被阻断节点

Node Gate 未通过时：

- `failed`：已确定违反要求，可创建 Repair Run；
- `inconclusive`：检查环境或证据不足，先重试/修复基础设施；
- `stale`：重新固定 baseline 并执行检查；
- `blocked`：等待用户、Permission、外部对账或 Repair 决策；
- `waived`：仅在 Profile 允许且 Waiver 有效时完成，并在 Outcome 标记技术债务。

---

## 13. Workflow Completion Gate

### 13.1 Workflow 级证据

Workflow Completion 不能只看所有 Node 显示 completed，还要验证：

- 当前 Workflow Revision 的 required Nodes 全部有效；
- 没有因 partial rollback 或 Spec 变更失效的结果；
- integration/merge 节点已验证组合状态；
- Project 级全量测试/规则满足要求；
- Acceptance Criteria 覆盖率达到 Profile；
- 所有外部副作用已成功或完成 reconciliation；
- 没有未处理 critical Diagnostic；
- Verification Report 已生成并绑定当前 revision。

### 13.2 DAG 结果有效性

每个 NodeOutcome 保存输入和输出 digest。Workflow Gate 根据 Artifact、Path、Context 和外部 receipt 依赖图判断是否仍有效：

```text
valid
needs_reverify
invalidated
externally_diverged
manual_review
```

任何 invalidated required Node 都阻止 Workflow 完成，即使历史 Attempt 曾经 passed。

### 13.3 集成验证

并行隔离 Worktree Patch 汇聚后，必须在目标集成 Workspace 上执行新的 Gate：

- ApplyPatch post Gate；
- conflict/merge diagnostic；
- module integration checks；
- full/affected test suite；
- changed scope audit；
- final Snapshot/ChangeSet；
- dependency and acceptance mapping。

不能把各隔离 Worktree 单独通过的结果简单相加为集成通过。

### 13.4 Completion 与 Delivery 分离

Workflow completed 表示实现工作流在当前 Workspace/Revision 上满足内部条件；Verification Delivery Gate 还负责生成面向用户的 `verification.md` 和最终审阅。用户拒绝交付报告不会抹去检查事实，但 Spec 仍不能进入最终 completed。

---

## 14. Restore 与 ApplyPatch Gate

### 14.1 Restore Plan Gate

执行 Restore 前验证：

- source Snapshot ready 且完整；
- RestorePlan digest、scope、expected baseline 未变化；
- 路径全部 canonicalize 且位于目标 Worktree；
- 当前用户编辑不会被静默覆盖；
- Claim、Permission、Risk 和 Reversibility 满足；
- binary/symlink/submodule 冲突策略明确；
- 必须创建 `pre_rollback` Snapshot；
- partial rollback 的 DAG 影响分析已完成。

### 14.2 Restore Post Gate

Restore 后验证：

- 每个 RestoreAction receipt 完整；
- expected result digest 与 post Snapshot 匹配；
- 没有范围外变化；
- 三方合并结果通过更高强度的 Verification Profile；
- 原本需要保留的用户编辑仍存在或有冲突 Artifact；
- DAG invalidation/Workflow Revision 已提交；
- 外部副作用差异被标记为 `externally_diverged`。

### 14.3 ApplyPatch Plan Gate

隔离结果回传前验证：

- source result 已 freeze；
- source base、source post、target baseline 均有 digest；
- Patch scope 不超过 delegated scope；
- Patch 不包含禁止文件、secret 或 Worktree escape；
- target drift/conflict 已分析；
- required tests 可在 target 集成状态重跑；
- Patch approval 固定 `plan_digest + patch_digest`。

### 14.4 ApplyPatch Post Gate

Apply 后在 target Worktree 执行：

- exact scope audit；
- post Snapshot/ChangeSet；
- target baseline verification；
- merge conflict scan；
- incremental + integration Verification；
- source/target provenance audit；
- isolation Worktree retention decision。

source Worktree 上的 pass 不可替代 target post Gate。

---

## 15. Verification Runner 架构

### 15.1 Runner 位置

```text
Gate Evaluator
  -> Verification Planner
  -> Verification Operation intent
  -> Tool Gateway authorization
  -> Verification Runner/Sandbox
  -> Process/MCP/Library Adapter
  -> Evidence Parser
  -> RuleCheck/Gate result commit
```

Rule Engine 不直接调用 `std::process::Command` 绕过 Tool Gateway。

### 15.2 Runner Profile

```rust
struct RunnerProfile {
    runner_profile_id: RunnerProfileId,
    revision: u64,
    executor_kind: ExecutorKind,
    tool_revisions: Vec<ToolRevisionRef>,
    sandbox_profile: SandboxProfileRef,
    network_policy: NetworkPolicy,
    credential_policy: CredentialPolicy,
    environment_allowlist: BTreeMap<String, String>,
    cwd_policy: CwdPolicy,
    resource_budget: ResourceBudget,
    output_schema: SchemaRef,
    cache_policy: CachePolicy,
    profile_digest: Digest,
}
```

### 15.3 默认权限

验证器默认：

- 只读目标 Workspace；
- 无网络；
- 无 Credential；
- 无用户主目录访问；
- 仅写专用临时目录、构建缓存或隔离 Worktree；
- 构建产物不得进入用户工作区，除非 Rule/Profile 明确声明且通过 Claim/Snapshot；
- 不执行项目 hook，除非 Profile 显式允许并隔离。

### 15.4 有写入的检查器

Formatter、codegen、test snapshot update 等可能写文件。默认拆成：

```text
check mode      只报告差异，不写目标
fix proposal    生成 Patch Artifact
repair apply    新 Repair Run 经 Tool Gateway 应用
```

无法提供 check-only 模式的工具应在隔离 Worktree 中运行，再从 diff 产生 Diagnostic/Patch suggestion。

### 15.5 输出解析

优先使用机器可读格式：JSON、SARIF、JUnit、compiler JSON、structured MCP result。文本解析器必须版本固定、可测试，并保留原始输出 ContentRef。无法可靠解析时结果为 inconclusive，而不是根据包含“success”字符串判断通过。

### 15.6 资源限制

- per check timeout；
- Gate total timeout；
- CPU、内存、进程、文件描述符、磁盘和输出上限；
- stdout/stderr 分块、截断和 Blob 存储；
- Process tree cancellation；
- network egress policy；
- 项目/会话并发额度；
- 慢检查 backpressure。

### 15.7 环境摘要

`environment_digest` 至少包含：

- OS/arch；
- Apex/Runner version；
- compiler/interpreter/checker version；
- lockfile/dependency graph digest；
- relevant env allowlist；
- sandbox profile；
- timezone/locale，若影响结果；
- external service endpoint identity，若适用。

不把 secret value 写入 digest 可见材料；使用 Credential revision/ref 或受保护摘要。

---

## 16. Verification 结果与 Evidence

### 16.0 VerificationVerdict（封闭枚举）

`VerificationVerdict` 是 `verification.md` 中每个验收条目的最终结论，取值封闭：

```text
passed        证据齐全且满足验收标准
failed        证据齐全且确定不满足
blocked       因权限、依赖、外部对账或待修复而无法得出结论
not_run       该条目本次未执行（未触发、被跳过或前置未完成）
```

这与需求文档 §3.1.1 及系统总体架构 §5.3 规定的四值一致，是**面向用户的验收语义**，不要与 §6.2 的 `RuleCheck.verdict`（面向单次检查的执行结论）混用。二者的聚合关系：

| RuleCheck 侧 | 聚合到 VerificationVerdict |
|---|---|
| `pass` | `passed` |
| `fail` | `failed` |
| `inconclusive` / `stale` | `blocked` |
| `skipped` | `not_run` |
| `waived` | `passed`（须携带 waiver 引用与 TTL，并在报告中显式标注） |
| 无 RuleCheck 记录 | `not_run` |

规则：

- 无证据不得记为 `passed`；`waived` 聚合为 `passed` 时必须可追溯到 Waiver 事实，不得隐藏；
- `blocked` 与 `not_run` 必须区分——前者尝试过但无法定论，后者根本没跑；
- 模型自报"完成"不构成 `passed`；
- §26.3 报告表格与本枚举一致，不再使用 `unknown` / `unverified` 等同义写法（原 `unknown` → `blocked`，原 `unverified` → `not_run`）。

> ADR-0018（跨文档一致性审查）：`VerificationVerdict` 原被 §16.1 引用但全文未定义取值，且文档内并存三套候选值集（§6.2 六值、§26.3 四值 + `unverified`）。现明确区分两层语义：`RuleCheck.verdict` 是检查级执行结论（§6.2 六值），`VerificationVerdict` 是验收级用户结论（本节四值，与上游基线一致），并给出聚合映射。

### 16.1 VerificationResult

```rust
struct VerificationResult {
    verification_id: VerificationId,
    check_id: RuleCheckId,
    subject: SubjectRef,
    verdict: VerificationVerdict,
    input_digest: Digest,
    ruleset_digest: Digest,
    runner_profile_digest: Digest,
    started_at: Timestamp,
    finished_at: Timestamp,
    diagnostics: Vec<DiagnosticRef>,
    artifacts: Vec<EvidenceRef>,
    metrics: VerificationMetrics,
    failure_kind: Option<FailureKind>,
    receipt_digest: Digest,
}
```

### 16.2 Evidence 类型

```text
snapshot_ref
change_set_ref
artifact_revision_ref
command_receipt
process_receipt
test_report
coverage_report
lint_report
schema_validation
compiler_report
security_report
external_receipt
human_review
waiver_ref
reconcile_report
```

### 16.3 Evidence 完整性

Evidence 必须包含：

- producer identity/version；
- subject/input digest；
- creation timestamp；
- content digest；
- sensitivity/taint；
- retention policy；
- schema version；
- verification status；
- source Operation/Attempt。

引用 Blob 缺失、digest 不匹配或 schema 不可解析时，依赖它的 Gate 变为 inconclusive/stale。

### 16.4 Test Report

统一测试报告至少表示：

```text
suite/case stable id
status: passed/failed/skipped/error/flaky/unknown
duration
attempt count
source location
stdout/stderr/content refs
environment digest
coverage relation, optional
```

`skipped` case 不能自动计入 passed；是否允许由 Completion Profile 决定。Flaky 重试必须保留全部 Attempt，不能只展示最后一次绿色结果。

### 16.5 Verification Cache

Cache key：

```text
check definition digest
input digest
runner profile digest
tool version digest
relevant environment digest
```

安全 hard checks、external receipt、human review 和实时状态验证默认不可从普通内容缓存复用。Cache hit 产生 provenance，允许审计“为什么没有重新运行”。

---

## 17. Repair Run 与自动修复闭环

### 17.1 原则

Checker 只产生问题和建议，不直接修改目标 Workspace。修复必须建模为新的 Repair Run：

```text
Diagnostic set
  -> RepairPlan
  -> risk/scope review
  -> Repair Agent/Run
  -> Permission + Claim + pre Snapshot
  -> Tool Operations
  -> post Snapshot + ChangeSet
  -> re-run original checks
  -> Node/Workflow Gate re-evaluation
```

### 17.2 RepairPlan

```rust
struct RepairPlan {
    repair_plan_id: RepairPlanId,
    project_id: ProjectId,
    subject: SubjectRef,
    source_diagnostics: Vec<DiagnosticRef>,
    expected_input_digest: Digest,
    target_scopes: Vec<PathScope>,
    proposed_strategy: RepairStrategy,
    verification_profile: VerificationProfileId,
    max_attempts: u32,
    risk: RiskLevel,
    plan_digest: Digest,
    expires_at: Timestamp,
}
```

RepairPlan 固定源诊断、目标 baseline 和 scope。任何源文件或 Ruleset 变化都要求重规划。

### 17.3 Repair 策略

```text
agent_edit          由 Repair Agent 分析并修改
apply_suggested_patch 应用 Checker 产生的固定 Patch
formatter_fix       在隔离 Workspace 生成格式化 Patch
config_change       高风险，通常需要用户批准
manual_fix          只展示诊断和建议
infrastructure_fix  修复 Runner/依赖，不改业务代码
```

### 17.4 自动修复准入

自动创建 Repair Run 仅在以下条件下允许：

- Rule 标记 fixable；
- scope 可确定且在父 Agent delegated scope 内；
- 风险不超过 Repair Policy；
- 不涉及 Credential、远端副作用、用户 Git history 或安全 hard deny 的策略变更；
- baseline 未漂移；
- 有可用 pre Snapshot；
- 未超过 attempt/cost/time budget；
- 没有检测到 repair cycle。

### 17.5 Repair Loop

```text
attempt 1 -> verify
  pass                       => close diagnostics
  new bounded diagnostics    => attempt 2 if policy allows
  same fingerprints          => cycle detected, block
  larger scope/risk          => ask user/replan
  checker inconclusive       => infrastructure path
  workspace drift            => stale/rebase plan
```

Repair 不得无限迭代。建议默认：

```text
max_auto_repair_attempts = 2
max_scope_growth = 0 without new approval
max_new_error_count = 0 for automatic continuation
```

### 17.6 Cycle Detection

Cycle key 包含：

```text
sorted diagnostic fingerprints
workspace/change_set digest
repair strategy
ruleset digest
```

出现相同 key 或在有限状态间往返时，停止自动修复，生成 `REPAIR_CYCLE_DETECTED` Diagnostic。

### 17.7 修复归因

修复结果必须区分：

- 原 Diagnostic resolved；
- 原 Diagnostic waived；
- 原位置变化导致 stale；
- 引入新的 Diagnostic；
- Checker/Ruleset 变化导致不可比较；
- 只修复症状但 Acceptance Criterion 仍失败。

不能仅以“原错误数量减少”宣称修复成功。

---

## 18. Exception、Suppression 与 Waiver

### 18.1 三种概念

| 类型 | 适用对象 | 含义 |
|---|---|---|
| Suppression | Diagnostic 展示/重复问题 | 降低噪声，通常不改变 hard Gate |
| Exception | Rule 在受限 scope/time 内不适用 | Policy 级例外 |
| Waiver | 已知 Gate 条件未满足但授权继续 | 决策级技术债务事实 |

### 18.2 Waiver 结构

```rust
struct Waiver {
    waiver_id: WaiverId,
    subject: SubjectRef,
    rule_ids: Vec<RuleId>,
    diagnostic_fingerprints: Vec<String>,
    scope: Vec<PathScope>,
    reason: String,
    risk_acceptance: String,
    approved_by: ActorId,
    approval_context: ApprovalContextRef,
    policy_revision: PolicyRevision,
    created_at: Timestamp,
    expires_at: Timestamp,
    max_uses: Option<u32>,
    state: WaiverState,
}
```

### 18.3 不可 Waive 的条件

默认不可例外：

- Project Root/path escape；
- 无效 Fence/Lease；
- 未授权 Credential 或 secret exfiltration；
- Permission/Capability/Trust 缺失；
- Snapshot/Restore baseline 欺骗；
- Scope violation；
- 审计篡改；
- 明确的用户数据破坏风险；
- Builtin hard safety deny。

### 18.4 Waiver 生效

Waiver 必须固定 subject/input/ruleset/scope。以下变化使其失效：

- 文件或 Artifact digest 改变；
- Rule 升级为更高安全级别；
- Project identity/Trust 改变；
- risk 增大；
- scope 扩大；
- expiry/max uses 到达；
- approver 资格被撤销。

### 18.5 审批体验

用户必须看到：

- 当前失败的具体 Rule/Gate；
- 将继续执行的状态转换；
- 受影响 Scope 与数据；
- 已有恢复能力；
- 外部不可逆影响；
- 例外有效期和传播范围；
- 推荐替代方案。

“忽略全部 warning/error”不是默认按钮。

---

## 19. Staleness 与重新验证

### 19.1 Stale 触发条件

```text
workspace baseline changed
snapshot/change_set changed
ruleset revision changed
gate definition changed
spec/artifact revision changed
runner/checker version changed
dependency/lockfile changed
relevant environment changed
external receipt expired/revoked
waiver expired
partial rollback invalidated input
```

### 19.2 失效传播

```text
changed fact
  -> Evidence dependency index
  -> affected RuleChecks
  -> affected GateAttempts
  -> affected NodeOutcomes
  -> affected Workflow/Spec completion
```

传播是追加状态事实，不修改原结果。历史 UI 显示“当时 passed，当前 stale”。

### 19.3 重新验证策略

- 仅说明文本变化且不影响规则语义：可复用；
- 单文件实现变化：重跑受影响增量 Profile；
- config/public API/schema 变化：扩大到模块/Project；
- Ruleset 变化：按 Rule selector/index 计算影响；
- Runner version 变化：重跑依赖该 Runner 的 required check；
- Release Gate：默认使用当前工具版本和全量 Profile。

### 19.4 Race 防护

Checker 执行结束后、提交结果前必须重新比较 input digest。若 Workspace 在检查期间变化：

- 原始执行收据仍保存；
- RuleCheck verdict 标记 stale；
- 不提交为当前 Gate pass；
- Scheduler 根据 policy 重新排队；
- 连续漂移触发 backoff 或隔离验证 Workspace。

### 19.5 验证隔离快照

长时间测试优先在冻结 Snapshot/隔离 Worktree 上运行，减少用户编辑导致的 stale。最终 Apply/Delivery Gate 仍需确认冻结状态与目标当前状态的关系。

---

## 20. 外部副作用验证与 Reconcile Gate

### 20.1 本地 Evidence 的边界

以下事实不能由本地文件检查证明：

- API 请求是否被远端接收；
- deployment 是否真正生效；
- payment/message 是否发送；
- remote Git push 是否完成；
- MCP Tool 是否执行但响应丢失；
- 第三方系统状态是否被后续操作改变。

### 20.2 外部 Receipt

```rust
struct ExternalReceipt {
    receipt_id: ExternalReceiptId,
    operation_id: OperationId,
    provider: ProviderIdentity,
    idempotency_key: String,
    request_digest: Digest,
    remote_resource_id: Option<String>,
    remote_status: ExternalStatus,
    observed_at: Timestamp,
    expires_at: Option<Timestamp>,
    raw_receipt_ref: ProtectedContentRef,
    reconciler_version: String,
    receipt_digest: Digest,
}
```

### 20.3 Reconcile Gate

结果：

```text
confirmed_succeeded
confirmed_failed
confirmed_not_executed
unknown
compensated
externally_diverged
```

只有 confirmed 状态可以满足对应 Completion Requirement。`unknown` 必须阻断需要确定性的 Workflow 完成，不能因重试超时自动转 failed。

### 20.4 Compensation

补偿也是新的高风险 Operation，需要 Permission、Rules、Receipt 和 Gate。补偿成功不等价于历史副作用从未发生；Audit 和 Verification Report 必须保留原操作与补偿链。

---

## 21. Gate 调度、并发与资源管理

### 21.1 计划阶段

Verification Planner 将 GateDefinition 编译为执行图：

```text
input materialization
  -> parallel cheap checks
  -> dependency/build preparation
  -> tests/security checks
  -> external/human evidence
  -> aggregation
```

只有不存在数据依赖且资源互不冲突的 Check 才能并行。

### 21.2 优先级

建议优先执行：

1. zero-cost state predicates；
2. hard safety checks；
3. format/schema/static fast checks；
4. compile/typecheck；
5. unit/integration tests；
6. expensive security/e2e/external checks；
7. human review。

快速失败可以取消尚未启动的非必要昂贵 Check，但已需要生成完整报告的 Profile 可继续收集其余诊断。

### 21.3 Slot 与配额

Verification 使用独立资源池，但仍受 Project/Session/Device 全局限制：

```text
verification_process_slots
verification_cpu_budget
verification_memory_budget
verification_disk_budget
verification_network_budget
external_checker_rate_limit
```

Repair Agent 消耗 Agent/Tool 额度，不能伪装成免费 Checker。

### 21.4 去重

相同 input/cache key 的并发 Check 采用 single-flight：

- 首个 Check 作为 producer；
- 其他 Gate 订阅结果；
- producer 取消不必取消仍有订阅者的执行；
- 每个 Gate 保留独立 evidence binding；
- 订阅者的 timeout 可先结束，不影响 producer。

### 21.5 公平性

防止大型 Workflow 长期占用全部测试槽：按 Project、Session、Workflow 权重和等待老化调度。交互式 PostTool 快速检查可有低延迟 lane，但不能饿死 Release/全量检查。

---

## 22. Command、Query 与 API

### 22.1 Commands

```text
ReloadRuleSources
CompileRuleset
ActivateRulesetRevision
StartRuleCheck
CancelRuleCheck
RetryRuleCheck
EvaluateGate
CancelGateAttempt
CreateRepairPlan
ApproveRepairPlan
StartRepairRun
CancelRepairRun
CreateWaiver
RevokeWaiver
AcknowledgeDiagnostic
SuppressDiagnostic
ResolveDiagnostic
GenerateVerificationReport
ApproveVerificationDelivery
ReconcileExternalEvidence
```

所有 Command 携带 `command_id/idempotency_key`、Actor、Project、causation/correlation、expected version 和必要 digest。

### 22.2 Queries

```text
GET /api/projects/{project_id}/rulesets
GET /api/rulesets/{ruleset_id}
GET /api/rulesets/{ruleset_id}/rules
GET /api/projects/{project_id}/rule-checks
GET /api/rule-checks/{rule_check_id}
GET /api/projects/{project_id}/diagnostics
GET /api/diagnostics/{diagnostic_id}
GET /api/gates/{gate_attempt_id}
GET /api/gates/{gate_attempt_id}/explanation
GET /api/repairs/{repair_run_id}
GET /api/projects/{project_id}/waivers
GET /api/workflows/{workflow_id}/verification
GET /api/specs/{spec_id}/verification-report
```

### 22.3 Command 响应

长时间操作返回：

```json
{
  "operation_id": "op_...",
  "rule_check_id": "rchk_...",
  "gate_attempt_id": "gatea_...",
  "accepted_at": "2026-08-08T10:00:00Z",
  "state": "queued"
}
```

不因 HTTP/gRPC 断开取消已接受的 Check/Gate；客户端通过 Query/Event 恢复。

### 22.4 Diagnostic Query

支持：

- project/worktree/spec/workflow/node/tool 过滤；
- severity/enforcement/state/rule/category；
- current/stale/historical；
- path/symbol/fingerprint；
- repairable/waivable；
- source Check/Gate；
- 分页和稳定 sort key。

完整源码片段、Patch 和测试输出使用授权 ContentRef，不直接嵌入列表。

### 22.5 人工审阅 Command

Human review 决定绑定：

- subject revision/checksum；
- Gate Definition revision；
- 展示给用户的 evidence digest；
- decision：approve/reject/request_changes；
- Actor 与认证上下文；
- comment ContentRef；
- expiry 或 scope，若适用。

---

## 23. 事件协议

### 23.1 Ruleset 事件

```text
rule_source.discovered
rule_source.changed
rule_source.invalid
ruleset.compile_requested
ruleset.compiled
ruleset.compile_failed
ruleset.activated
ruleset.superseded
```

### 23.2 RuleCheck 事件

```text
rule_check.queued
rule_check.started
rule_check.progress
rule_check.completed
rule_check.cancelled
rule_check.interrupted
rule_check.stale
rule_check.cache_hit
```

### 23.3 Diagnostic 事件

```text
diagnostic.opened
diagnostic.reobserved
diagnostic.acknowledged
diagnostic.suppressed
diagnostic.waived
diagnostic.resolved
diagnostic.stale
```

### 23.4 Gate 事件

```text
gate.evaluation_requested
gate.started
gate.evidence_attached
gate.passed
gate.failed
gate.blocked
gate.inconclusive
gate.stale
gate.waived
gate.cancelled
```

### 23.5 Repair/Verification 事件

```text
repair.plan_created
repair.plan_approved
repair.run_created
repair.run_started
repair.run_completed
repair.run_failed
repair.cycle_detected
verification.report_generated
verification.delivery_requested
verification.delivery_approved
verification.delivery_rejected
external_evidence.reconciled
```

### 23.6 Event 内容约束

事件包含 ID、revision、digest、状态、计数和安全摘要。以下内容通过 ContentRef：

- 完整规则源；
- 编译后的 Ruleset Blob；
- Diagnostic source excerpt；
- stdout/stderr；
- Test report；
- Patch suggestion；
- Gate explanation tree；
- Verification report。

---

## 24. SQLite 持久化设计

### 24.1 与既有表的兼容

沿用现有：

- `rulesets`；
- `rule_checks`；
- `diagnostics`；
- `tool_calls`；
- `operations`/`operation_journal`；
- `snapshots`、`snapshot_restores`；
- `runs`、`node_attempts`、`workflows`；
- `blobs`、`events`、`audit_logs`。

### 24.2 RuleCheck 状态消歧

现有表中 `state` 的 `passed/failed` 混合了生命周期和业务结论。建议迁移为：

```text
state = queued | running | completed | cancelled | interrupted | unknown
verdict = pass | fail | inconclusive | stale | skipped | waived
failure_kind = nullable enum/string
```

迁移旧数据：

```text
old passed  -> completed + pass
old failed + violation evidence -> completed + fail + violations_found
old failed + runner error       -> completed + inconclusive + checker_failed
old cancelled                   -> cancelled + null
old interrupted                 -> interrupted + null
```

无法分类的旧 failed 迁为 `completed + inconclusive + legacy_ambiguous`，不得假定代码未通过或已通过。

> ADR-0029（跨文档一致性审查）：`legacy_ambiguous` 已加入领域模型 §5.12 的 `failure_kind` 枚举与 SQLite `rule_checks` 的 CHECK 约束。该值仅供迁移使用，新产生的 RuleCheck 不得写入。

### 24.3 建议新增表

```sql
CREATE TABLE ruleset_revisions (
    ruleset_revision_id TEXT PRIMARY KEY,
    ruleset_id TEXT NOT NULL,
    project_id TEXT,
    revision INTEGER NOT NULL,
    ruleset_digest TEXT NOT NULL,
    compiler_version TEXT NOT NULL,
    compiled_blob_id TEXT,
    state TEXT NOT NULL,
    created_at_us INTEGER NOT NULL,
    activated_at_us INTEGER,
    superseded_at_us INTEGER,
    UNIQUE(ruleset_id, revision),
    UNIQUE(project_id, ruleset_digest)
);

CREATE TABLE gate_definitions (
    gate_definition_id TEXT PRIMARY KEY,
    project_id TEXT,
    gate_kind TEXT NOT NULL,
    revision INTEGER NOT NULL,
    definition_digest TEXT NOT NULL,
    definition_blob_id TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at_us INTEGER NOT NULL,
    UNIQUE(project_id, gate_kind, revision)
);

CREATE TABLE gate_attempts (
    gate_attempt_id TEXT PRIMARY KEY,
    gate_definition_id TEXT NOT NULL,
    subject_type TEXT NOT NULL,
    subject_id TEXT NOT NULL,
    subject_revision TEXT,
    input_digest TEXT NOT NULL,
    state TEXT NOT NULL,
    verdict TEXT,
    explanation_blob_id TEXT,
    started_at_us INTEGER,
    finished_at_us INTEGER,
    created_at_us INTEGER NOT NULL,
    version INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE gate_evidence (
    gate_attempt_id TEXT NOT NULL,
    evidence_kind TEXT NOT NULL,
    evidence_id TEXT NOT NULL,
    evidence_digest TEXT NOT NULL,
    required INTEGER NOT NULL DEFAULT 1,
    status TEXT NOT NULL,
    PRIMARY KEY(gate_attempt_id, evidence_kind, evidence_id)
);

CREATE TABLE verification_results (
    verification_id TEXT PRIMARY KEY,
    rule_check_id TEXT NOT NULL,
    subject_type TEXT NOT NULL,
    subject_id TEXT NOT NULL,
    input_digest TEXT NOT NULL,
    verdict TEXT NOT NULL,
    runner_profile_digest TEXT NOT NULL,
    receipt_digest TEXT NOT NULL,
    result_blob_id TEXT,
    created_at_us INTEGER NOT NULL
);

CREATE TABLE waivers (
    waiver_id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    subject_type TEXT NOT NULL,
    subject_id TEXT NOT NULL,
    scope_json TEXT NOT NULL CHECK(json_valid(scope_json)),
    policy_revision TEXT NOT NULL,
    state TEXT NOT NULL,
    approved_by_actor_id TEXT NOT NULL,
    reason_blob_id TEXT,
    expires_at_us INTEGER NOT NULL,
    max_uses INTEGER,
    used_count INTEGER NOT NULL DEFAULT 0,
    created_at_us INTEGER NOT NULL
);
```

### 24.4 Diagnostic 扩展

建议对既有 `diagnostics` 增加：

```text
rule_id
rule_revision
message_template
symbol_anchor
last_seen_at_us
observation_count
state
waiver_id
sensitive_level
evidence_blob_id
```

`UNIQUE(source_type, source_id, fingerprint)` 保留单次 source 去重；跨 Check 历史关联使用独立 `diagnostic_occurrences` 表更清晰。

### 24.5 索引

```sql
CREATE INDEX idx_rule_check_cache
ON rule_checks(ruleset_digest, input_digest, state, verdict);

CREATE INDEX idx_gate_subject
ON gate_attempts(subject_type, subject_id, created_at_us DESC);

CREATE INDEX idx_gate_current
ON gate_attempts(state, created_at_us)
WHERE state IN ('pending','running','blocked','unknown');

CREATE INDEX idx_diagnostic_current
ON diagnostics(project_id, state, severity, created_at_us DESC);

CREATE INDEX idx_waiver_expiry
ON waivers(project_id, state, expires_at_us);
```

### 24.6 事务边界

数据库事务只负责：

- Check/Gate intent；
- 状态和 version；
- Evidence refs；
- final verdict；
- Node/Workflow 状态转换；
- Event/Outbox。

长时间 process、MCP、文件扫描和外部验证在事务外执行，结果以 Operation receipt 和 input digest 回写。

---

## 25. 崩溃恢复与 Reconcile

### 25.1 启动扫描

服务启动时扫描：

- compiling Ruleset；
- queued/running/unknown RuleCheck；
- pending/running/unknown GateAttempt；
- active Repair Run；
- 未完成 Verification Operation；
- 引用缺失 Blob 的 Evidence；
- 已过期 Waiver；
- stale 未传播的 Gate/Node/Workflow。

### 25.2 RuleCheck 恢复

- process 已结束且 receipt 完整：解析并提交 completed；
- process 存活且 Lease/Fence 有效：重新接管监督；
- process 身份未知：终止或隔离，标记 interrupted/unknown；
- 输出部分存在：保留 Blob，不能作为 pass；
- input 已变化：提交 stale；
- 可证明纯检查且无副作用：允许创建新 Attempt 重跑。

### 25.3 Gate 恢复

Gate Evaluator 从持久化 Definition、Subject、Evidence refs 重建：

- 所有 Evidence 已 terminal：重新聚合；
- 缺失 required Check：重新排队；
- Subject revision 改变：stale；
- external receipt unknown：blocked/inconclusive；
- 人工 Review 已决定：验证 checksum 后恢复；
- 无法证明是否已发布 terminal Event：根据 aggregate version/outbox 对账。

### 25.4 Repair 恢复

Repair Agent 恢复遵循 Runtime/Workspace 规则：

- Fence 旧 Attempt；
- 捕获 orphan/quarantine Snapshot；
- 检查 RepairPlan baseline；
- 已有文件变化不盲目重放；
- 生成 post ChangeSet 后重新执行源 RuleCheck；
- 无法归因时 block，等待人工处理。

### 25.5 Evidence 完整性扫描

后台任务定期验证：

- Blob digest；
- Evidence ref 可达性；
- Ruleset compiled blob；
- Test report schema；
- Gate explanation tree；
- Retention/GC 引用。

发现损坏时，依赖 Gate 标记 inconclusive/stale，并产生高优先级运维 Diagnostic。

---

## 26. Verification Report 与交付

### 26.1 `verification.md` 的定位

`verification.md` 是面向人类和 Git 审计的导出 Artifact，不是唯一事实源。权威数据来自 GateAttempt、RuleCheck、Diagnostic、Evidence、Snapshot、ChangeSet 和 Review revision。导出文件必须包含 source revision/digest，手工编辑后生成新 Artifact revision，不能反向伪造 Gate 通过。

### 26.2 报告结构

建议结构：

```markdown
# Verification Report

## Subject
- Spec / Workflow revision
- Workspace / Snapshot / ChangeSet
- Ruleset and Gate profile

## Acceptance Criteria
- Criterion ID
- Status
- Evidence
- Notes

## Rule and Test Summary
- Passed / Failed / Warning / Inconclusive / Stale

## Changes
- Added / Modified / Deleted / Renamed

## Diagnostics
- Open / Repaired / Waived / Suppressed

## External Operations
- Confirmed / Compensated / Unknown

## Exceptions and Technical Debt
- Waivers and expiry

## Reproduction
- Runner profiles and commands

## Final Gate
- Verdict / reviewers / timestamps / digests
```

### 26.3 Acceptance 覆盖矩阵

每个 Criterion 映射到至少一个 Evidence：

每个 Criterion 映射到至少一个 Evidence。Status 列使用 §16.0 的 `VerificationVerdict` 四值：

| Criterion | Evidence | Status（可能取值） |
|---|---|---|
| 功能行为 | test cases / human review | passed / failed / not_run |
| API 兼容 | schema diff / contract test | passed / failed / not_run |
| 性能约束 | benchmark report | passed / failed / blocked / not_run |
| 安全约束 | security checks / review | passed / failed / blocked / not_run |
| 运维约束 | config/deployment receipt | passed / blocked / not_run |

> ADR-0018：本表原混用 `pass/fail/inconclusive/unknown` 四种非枚举写法，已统一为 §16.0 的封闭四值（原 `inconclusive`/`unknown` → `blocked`）。

没有 Evidence 的 Criterion 显示 `not_run`，不能被模型生成的说明文字自动标记 `passed`。

### 26.4 报告稳定性

- 按 Criterion ID、Rule ID、path_key 稳定排序；
- 时间和随机 Operation ID 放在元数据区，避免无意义 diff；
- 长输出使用 ContentRef；
- secret/敏感路径脱敏；
- 明确区分当前结果与历史结果；
- 标注 cached/reused Evidence；
- 标注 flaky、skipped、waived、stale 和 inconclusive。

### 26.5 用户确认门

用户批准最终交付时固定：

```text
verification artifact revision + checksum
workflow/spec revision
final gate attempt id + input digest
change set digest
open diagnostics summary
waiver summary
external operation summary
```

任一关键 digest 变化后旧批准失效。

---

## 27. 安全威胁模型

### 27.1 主要威胁

| 威胁 | 示例 | 控制 |
|---|---|---|
| 恶意规则源 | 项目 Rule 要求上传源码 | source trust + no implicit network/credential |
| Checker 越权 | lint 脚本修改主工作区 | sandbox + read-only + isolated worktree |
| 假阳性通过 | 输出包含“success” | structured schema + receipt validation |
| 伪造 Evidence | Agent 写测试报告文件 | producer identity + digest + protected store |
| 缓存投毒 | 不同环境共用 pass | complete cache key + trust boundary |
| 路径逃逸 | checker follow symlink | canonical scope + sandbox |
| Secret 泄漏 | stdout/diagnostic 含 token | scanning + protected ContentRef |
| Prompt Injection | 测试输出诱导 Repair Agent | taint + structured prompt boundary |
| Waiver 滥用 | 永久忽略全部错误 | bounded scope/TTL/actor/policy |
| Repair 无限循环 | Agent 反复引入错误 | attempt budget + cycle detection |
| Stale pass | 文件变化仍显示绿色 | input digest + invalidation graph |
| 外部状态误报 | 超时当作失败后重试 | reconcile gate + idempotency receipt |

### 27.2 Rule Source 信任等级

```text
builtin_trusted
organization_signed
user_global_trusted
project_trusted_config
project_untrusted_text
external_untrusted
```

信任等级决定规则是否可以请求 command checker、网络、Credential 或自定义代码。项目文本默认不能引入新的高权限 Checker。

### 27.3 自定义 Checker

允许自定义 Checker 时必须具备：

- signed/versioned manifest；
- hash-pinned executable/container/module；
- input/output schema；
- declared capabilities；
- sandbox profile；
- resource limits；
- provenance and publisher；
- enablement approval；
- revocation path。

不执行从 Diagnostic、LLM 输出或远端页面动态拼接的任意命令。

### 27.4 结果存储边界

Ruleset compiled Blob、Evidence、Diagnostic 原文和 Gate Explanation 存放在 Agent 不可写的受保护 Store。Agent 可以提交 observation，但 Core/Runner 才能签发被 Gate 接受的 Evidence receipt。

### 27.5 Fail Open 边界

只有明确标记 `advisory`、不影响安全/正确性完成条件的 Check 才可 fail open。Fail open 本身生成 Diagnostic 和 Audit；不能把 inconclusive 改写为 pass。

---

## 28. 可观测性与审计

### 28.1 Metrics

```text
apex_ruleset_compile_seconds{source_count_bucket,result}
apex_ruleset_compile_failure_total{reason}
apex_rule_check_seconds{category,trigger,result}
apex_rule_check_cache_hit_ratio{category}
apex_rule_check_failure_total{failure_kind}
apex_diagnostic_open_total{severity,category}
apex_gate_attempt_seconds{kind,result}
apex_gate_blocked_total{reason}
apex_verification_stale_total{cause}
apex_repair_attempt_total{result}
apex_repair_cycle_total
apex_waiver_active_total{category}
apex_external_reconcile_total{result}
apex_verification_queue_seconds{profile}
```

Rule ID、文件路径、Diagnostic message 不能直接作为高基数 label。

### 28.2 Trace

典型链路：

```text
Tool/Node/Workflow Command
  -> Gate Evaluation
  -> Verification Plan
  -> RuleCheck Operation
  -> Tool Gateway Authorization
  -> Runner/Process
  -> Output Parse
  -> Diagnostic Persist
  -> Gate Aggregate
  -> Runtime State Transition
```

Trace attribute 使用 ID、digest prefix、category、scope count、result 和 duration bucket，不记录源码、secret 或完整命令。

### 28.3 Audit

必须审计：

- 规则源发现、变更、解析失败和激活；
- Gate Definition 创建/修改；
- Check 输入、Runner profile、复用来源；
- Diagnostic 原始结论与生命周期；
- RepairPlan、Repair Run 和实际 ChangeSet；
- Waiver/Suppression 的操作者、范围、原因和过期；
- 人工 Review 展示的 Evidence digest；
- 外部对账和补偿；
- Stale 传播；
- Verification Report 导出和批准。

### 28.4 告警

建议告警：

- required Checker 持续 unavailable；
- Ruleset compile failure 阻断多个项目；
- Gate inconclusive/stale 比例异常；
- hard Diagnostic 被错误标记 waived；
- Repair cycle 激增；
- Verification cache 命中但环境 digest 不一致；
- Evidence Blob 损坏或缺失；
- active Waiver 即将过期且仍阻塞 Release；
- external unknown 长期未对账；
- Checker 出现 Workspace 越界写入。

---

## 29. 性能、缓存与容量

### 29.1 性能原则

优化顺序：

1. 输入和结果正确绑定；
2. 增量 Scope；
3. Check result cache；
4. dependency graph/index；
5. single-flight；
6. 并行 Checker；
7. remote/distributed runner；
8. 冷热 Evidence 存储。

不能通过省略 ruleset/environment digest 或跳过 Workspace drift 检查提高命中率。

### 29.2 延迟目标分类

| Gate | 目标体验 | 策略 |
|---|---|---|
| PreTool | 毫秒到低百毫秒 | builtin/static/缓存 |
| PostTool incremental | 秒级 | changed scope + fast lane |
| Node Completion | 秒到分钟 | 分层检查、并行 |
| Workflow/Release | 分钟级可接受 | 全量证据、异步进度 |
| External Reconcile | 依赖远端 | durable Operation + event |

具体 SLO 按设备和 Project Profile 配置，不把静态目标写成业务正确性条件。

### 29.3 Dependency Index

维护：

- file import/include graph；
- build target graph；
- test-to-source relation；
- schema/generated relation；
- Acceptance Criterion-to-Node/Evidence relation；
- Rule selector-to-path relation；
- Gate-to-Evidence dependency。

索引是可重建 projection，失效时扩大检查范围，不能缩小到不安全范围。

### 29.4 Cache 隔离

按 tenant/project/trust domain 隔离。共享开源依赖构建缓存不代表共享 Gate pass；最终结果仍绑定 Project input。敏感项目禁止通过 cache existence 推断文件内容。

### 29.5 Evidence 保留

- 当前 Spec/Workflow/Release 的 required Evidence 强引用；
- 历史 Gate 按审计 retention；
- 大型 stdout/test artifact 可分层归档；
- Diagnostic 摘要长期保留，原文按敏感策略；
- Waiver 和 hard failure evidence 至少保留到审计期限结束；
- GC 前事务内复查 Gate/Report/Review 引用。

---

## 30. 测试与故障注入

### 30.1 单元测试

覆盖：

- Rule precedence 与 hard deny；
- Ruleset canonical serialization/digest；
- selector/scope；
- input digest；
- RuleCheck lifecycle/verdict 映射；
- Diagnostic fingerprint；
- Gate aggregation；
- Waiver scope/expiry；
- stale propagation；
- Repair cycle detection；
- report stable ordering。

### 30.2 Property-based 测试

```text
ruleset_digest 不受文件遍历顺序影响
相同 normalized input -> 相同 input_digest
gate aggregation 不受 Evidence 返回顺序影响
hard_deny 永远不能被 allow/waiver 覆盖
input change -> prior pass cannot remain current
missing required evidence -> verdict != pass
checker crash -> verdict != fail unless violation evidence exists
waiver scope cannot expand during evaluation
repair attempts cannot exceed configured maximum
```

### 30.3 集成场景

1. Write 后增量 lint 通过；
2. Tool 成功但 PostTool Rule 失败；
3. warning 不阻断，Project 收紧后阻断；
4. Checker 崩溃与代码 violation 分离；
5. Checker 运行期间用户修改文件，结果 stale；
6. Ruleset 热更新使未完成 Node 重新验证；
7. 缓存命中保留来源 Check ID；
8. Repair Agent 修复后重跑原 Check；
9. Repair 循环被检测并阻断；
10. Waiver 过期使 Release Gate 重新失败；
11. partial rollback 使下游 Gate stale；
12. ApplyPatch 在 source pass、target integration fail；
13. skipped test 不计入 required pass；
14. flaky test 保留全部 Attempt；
15. external timeout 进入 unknown 而非 failed；
16. `/skip-spec` 仍执行 PostTool 和基础 Completion Gate；
17. 恶意项目规则试图上传源码被拒绝；
18. Checker 尝试写主 Workspace 被 sandbox 阻断；
19. Evidence Blob 丢失使 Gate inconclusive；
20. 人工审批后 subject digest 改变，审批失效。

### 30.4 故障注入点

```text
after_rulecheck_intent
before_runner_dispatch
after_process_exit_before_receipt
after_output_blob_before_parse
after_diagnostic_persist_before_gate_aggregate
after_gate_terminal_before_node_transition
after_repair_write_before_post_check
after_waiver_commit_before_event
during_stale_propagation
during_report_materialization
during_external_reconcile
```

模拟进程崩溃、SQLite busy、磁盘满、输出截断、Blob 超时、进程树无法终止、Runner 版本变化、工作区漂移和客户端断线。

### 30.5 验收标准

- required Evidence 缺失时 Gate 永不 pass；
- Checker 故障不会被统计为代码 violation；
- violation 不会被 Checker 静默修复；
- Repair 的每次写入都有 Claim、pre/post Snapshot 和新 RuleCheck；
- Stale pass 不会完成 Node/Workflow；
- hard deny 不可被 waiver、skip-spec 或 permission bypass 覆盖；
- Tool 成功与 PostTool violation 同时可准确表达；
- 外部 unknown 不被自动重放或伪造成失败；
- Verification Report 可追溯到全部当前 digest 和 Evidence；
- 三端展示的 Gate/Diagnostic 状态来自同一 Core 投影。

---

## 31. Rust 模块与端口设计

### 31.1 模块建议

```text
crates/apex-rules-domain/
  source.rs
  rule.rs
  ruleset.rs
  check.rs
  diagnostic.rs
  gate.rs
  evidence.rs
  waiver.rs
  repair.rs

crates/apex-rules-service/
  source_service.rs
  compiler_service.rs
  check_service.rs
  gate_service.rs
  diagnostic_service.rs
  repair_service.rs
  report_service.rs
  reconcile_service.rs

crates/apex-rules-adapters/
  builtin/
  command_runner/
  sarif/
  junit/
  compiler_json/
  sqlite/
  blob_store/
  external_receipt/
```

### 31.2 Ports

```rust
#[async_trait]
pub trait RuleSourcePort {
    async fn discover(&self, project: ProjectRef) -> Result<Vec<RuleSourceRevision>>;
    async fn load(&self, source: &RuleSourceRevision) -> Result<ProtectedContentRef>;
}

#[async_trait]
pub trait RuleCompilerPort {
    async fn compile(&self, input: CompileRulesetInput) -> Result<CompiledRuleset>;
}

#[async_trait]
pub trait VerificationRunnerPort {
    async fn execute(&self, request: VerificationExecution) -> Result<RunnerReceipt>;
    async fn cancel(&self, operation_id: OperationId) -> Result<()>;
    async fn reconcile(&self, operation_id: OperationId) -> Result<RunnerReconcileResult>;
}

#[async_trait]
pub trait EvidencePort {
    async fn put(&self, evidence: EvidenceInput) -> Result<EvidenceRef>;
    async fn verify(&self, evidence: &EvidenceRef) -> Result<EvidenceIntegrity>;
    async fn retain(&self, evidence: &EvidenceRef, reason: RetainReason) -> Result<()>;
}

#[async_trait]
pub trait GateEvaluatorPort {
    async fn plan(&self, request: GateRequest) -> Result<GatePlan>;
    async fn evaluate(&self, plan: &GatePlan) -> Result<GateVerdict>;
}

#[async_trait]
pub trait RepairPlannerPort {
    async fn create_plan(&self, request: RepairRequest) -> Result<RepairPlan>;
}
```

### 31.3 Domain 纯度

Domain crate 不依赖：

- SQLite/rusqlite；
- tokio process；
- Git CLI；
- HTTP/MCP SDK；
- 某个 lint/test 格式；
- UI 类型。

Adapter 将外部结果转换为稳定 Domain receipt 和 Diagnostic。

### 31.4 Error 类型

```rust
pub enum RulesError {
    SourceOutsideBoundary,
    SourceUntrusted,
    RuleParseFailed,
    RulesetCompileFailed,
    RulesetNotActive,
    InputIncomplete,
    InputDigestMismatch,
    VerificationStale,
    RunnerUnavailable,
    RunnerTimedOut,
    RunnerOutputInvalid,
    EvidenceCorrupt,
    GateFailed,
    GateInconclusive,
    GateStale,
    WaiverNotAllowed,
    WaiverExpired,
    RepairCycleDetected,
    ExternalEvidenceUnknown,
}
```

内部 source chain 写受保护日志；API 返回稳定 code、safe diagnostic、retryability、blocking IDs 和下一动作。

---

## 32. 分阶段交付

### Phase 1：最小规则闭环

- Builtin + Project `apex/rules/` 来源；
- Ruleset compile/revision/digest；
- PostTool changed-file RuleCheck；
- Diagnostic fingerprint；
- `pass/fail/inconclusive/stale`；
- Tool `succeeded_with_violations`；
- 基础 Node Completion Gate；
- 手动 Repair Run；
- Event/Query/Audit。

完成标准：每次 Write/Edit 后的规则结果可追溯，Checker 故障与代码违规分离，Node 不依赖模型自报完成。

### Phase 2：Spec 与自动 Repair

- Spec 内嵌规则；
- Acceptance Criterion model；
- incremental dependency expansion；
- 自动 RepairPlan/Repair Agent；
- cycle/attempt budget；
- Waiver/Exception；
- `verification.md` 生成与批准。

完成标准：需求、设计、任务、实现和验证形成闭环，自动修复不绕过 Tool Gateway。

### Phase 3：DAG、Workspace 与集成 Gate

- Node/Workflow Completion Profile；
- isolated Patch target verification；
- Restore/partial rollback Gate；
- stale dependency propagation；
- integration/full checks；
- Evidence cache/single-flight；
- quarantine/reconcile。

完成标准：并行 Agent 与部分回滚不会保留错误的绿色状态。

### Phase 4：发布、扩展与合规

- Release Gate；
- external receipt/compensation；
- signed custom checker；
- organization policy；
- distributed runner；
- retention/legal hold；
- SARIF/JUnit/CI integration；
- 完整 chaos/security 测试。

### 32.1 MVP 不得延期项

即使 v0.1 只实现基础 Checker，也必须保留：

- RuleCheck input/ruleset digest；
- violation 与 checker failure 分离；
- PostTool actual ChangeSet 输入；
- required Gate 缺失不 pass；
- Checker 不直接静默改文件；
- Repair 通过 Tool Gateway；
- stale result 不用于完成；
- Builtin hard rules 不可覆盖；
- Event/Audit/ContentRef 边界。

---

## 33. 架构决策记录（ADR 摘要）

### ADR-RV-001：RuleCheck 与 Gate 分离

**决定**：RuleCheck 保存检查事实，Gate 聚合检查与其他 Evidence 作状态转换决策。

**理由**：测试、Snapshot、Spec Review、外部 receipt 并不都是 Rule；一个 Check 也可能被多个 Gate 复用。

### ADR-RV-002：生命周期与 Verdict 分离

**决定**：`state` 描述执行生命周期，`verdict` 描述业务结论，`failure_kind` 描述原因。

**理由**：避免把 violations、checker crash、timeout 和 cancellation 都压成 `failed`。

### ADR-RV-003：Stale 是独立结论

**决定**：输入变化使旧结果 stale，而不是 fail 或继续 pass。

**理由**：保留历史准确性，同时阻止过期证据完成当前 Workflow。

### ADR-RV-004：Checker 默认只读

**决定**：会写文件的 Checker 在隔离 Worktree 运行并产生 Patch suggestion；目标修改由 Repair Run 完成。

**理由**：维护 Permission、Claim、Snapshot、归因和恢复链路。

### ADR-RV-005：增量检查按依赖闭包扩大

**决定**：changed files 是起点，不是固定上限；配置/API/schema 等变化自动扩大范围。

**理由**：纯文件级 lint 无法证明跨文件正确性。

### ADR-RV-006：Gate pass 绑定完整 Input Digest

**决定**：Ruleset、Snapshot/ChangeSet、Artifact、Runner、依赖和环境共同构成 input digest。

**理由**：防止 stale cache、环境漂移和错误复用。

### ADR-RV-007：Repair 是新 Run

**决定**：Checker 不静默修复；每次修复都有 Plan、Agent/Run、Claim、Snapshot 和复验。

**理由**：使修改可审批、可取消、可回滚、可审计。

### ADR-RV-008：Waiver 是技术债务事实

**决定**：Waiver 不关闭原 Diagnostic，只允许受限 Gate 转换，并带 scope/TTL/Actor。

**理由**：避免“忽略”掩盖真实质量风险。

### ADR-RV-009：外部副作用使用 Reconcile Gate

**决定**：本地验证不能推断远端执行结果；使用 idempotency receipt 与 Adapter reconciliation。

**理由**：超时和断线时 unknown 不等于 failed。

### ADR-RV-010：`verification.md` 是镜像，不是事实源

**决定**：报告由结构化事实生成，手工修改产生新 Artifact revision。

**理由**：兼顾人类/Git 审计与数据库一致性。

### ADR-RV-011：规则源默认不可信

**决定**：项目规则不能自动获得网络、Credential 或任意命令执行能力。

**理由**：clone 一个仓库不应等于授权其配置执行高权限代码。

### ADR-RV-012：Gate 聚合顺序确定

**决定**：聚合按 effect precedence、required 状态和稳定 ID，与结果到达顺序无关。

**理由**：并行 Checker 的完成顺序不应改变业务结论。

---

## 34. 设计审查清单

### 34.1 Ruleset

- [ ] 所有规则源是否保存 revision、digest、trust class？
- [ ] Builtin hard rule 是否不可被项目/用户 allow 覆盖？
- [ ] 编译顺序是否确定且不依赖目录遍历？
- [ ] 自然语言兼容文件是否避免被直接当成可执行安全规则？
- [ ] Ruleset 热更新是否使受影响结果 stale？
- [ ] include/import 是否有路径和数量边界？

### 34.2 RuleCheck

- [ ] 是否固定 ruleset/input/runner/environment digest？
- [ ] lifecycle、verdict、failure_kind 是否分离？
- [ ] violations 与 checker failure 是否分别统计？
- [ ] 退出码和文本是否经过结构化解析？
- [ ] 复用是否保留来源 Check ID？
- [ ] Checker 运行期间输入变化是否产生 stale？

### 34.3 Diagnostic

- [ ] fingerprint 是否避免依赖绝对路径和随机行号？
- [ ] 重复问题是否去重但保留 occurrence 历史？
- [ ] waiver、suppression、resolved 是否明确区分？
- [ ] source excerpt/output 是否按 ContentRef 和敏感级别保护？
- [ ] Diagnostic 是否携带 Rule、Evidence 和 Repair 关系？

### 34.4 Gate

- [ ] Gate Subject 是否绑定不可变 revision/checksum？
- [ ] required Evidence 缺失时是否不可能 pass？
- [ ] pass/fail/inconclusive/stale/blocked/waived 是否正确区分？
- [ ] Explanation Tree 是否可查询和审计？
- [ ] Node/Workflow 状态转换是否在事务中复验 digest/revision？
- [ ] Gate Definition 更新是否有 revision？

### 34.5 Runner

- [ ] 所有 command/process Checker 是否经过 Tool Gateway？
- [ ] 默认是否只读、无网络、无 secret？
- [ ] 有写 Checker 是否在隔离 Workspace 或生成 Patch？
- [ ] 是否限制 timeout、CPU、内存、进程、磁盘和输出？
- [ ] 是否保留完整 receipt 和原始输出引用？
- [ ] 自定义 Checker 是否 hash-pinned、签名并声明 capability？

### 34.6 Repair

- [ ] Repair 是否是新 Agent/Run/Operation？
- [ ] 是否固定源 Diagnostic、baseline、scope 和 plan digest？
- [ ] 是否经过 Permission、Claim、pre/post Snapshot？
- [ ] 是否重跑原 RuleCheck 和 Completion Gate？
- [ ] 是否有最大 Attempt、预算和 cycle detection？
- [ ] scope/risk 扩大是否重新审批？

### 34.7 Spec 与交付

- [ ] Design 是否引用 Ruleset digest？
- [ ] Acceptance Criterion 是否有稳定 ID 和 Evidence method？
- [ ] `/skip-spec` 是否仍执行硬规则和 PostTool Gate？
- [ ] `verification.md` 是否由结构化事实生成？
- [ ] 用户批准是否固定 Report/Workflow/ChangeSet digest？
- [ ] 未验证、skipped、flaky、waived、stale 是否醒目标识？

### 34.8 恢复与外部状态

- [ ] partial rollback 是否传播 Gate stale/invalidation？
- [ ] Restore/ApplyPatch 是否分别有 plan/post Gate？
- [ ] external unknown 是否进入 reconcile 而非自动重试？
- [ ] Repair/Check 崩溃是否可从 Operation receipt 恢复？
- [ ] Evidence 损坏是否使依赖 Gate 失效？
- [ ] GC 是否保护 Gate/Report/Review 强引用？

---

## 35. 与既有 Apex 文档的一致性

| 既有文档 | 本文落实内容 |
|---|---|
| `Apex—— 需求分析文档.md` | Spec 内嵌、PostToolUse 增量检查、修复子任务、`verification.md` 与 warning 默认不阻断 |
| `Apex—— 系统总体架构设计.md` | Rules 独立 Core 模块、副作用统一闸门、Spec Gate 状态机、单核多前端 |
| `Apex—— 领域模型与事件规范.md` | RuleCheck 不变量、Diagnostic fingerprint、Repair Run、Artifact/Review revision |
| `Apex—— API与实时事件协议设计.md` | Command/Query/Event、Operation ID、ContentRef、断线恢复和审批 checksum |
| `Apex—— SQLite数据模型与迁移设计.md` | rulesets、rule_checks、diagnostics 表族及 lifecycle/verdict 迁移 |
| `Apex—— Agent Runtime与DAG调度器详细设计.md` | Node/Workflow Completion Gate、Attempt/Lease/Fence、状态终结原子性 |
| `Apex—— Tool Gateway与权限引擎详细设计.md` | Pre/PostTool 顺序、Hook 约束、Permission 与 Rules 分离、Tool `succeeded_with_violations` |
| `Apex—— Workspace快照、Write Claim与隔离工作区详细设计.md` | Snapshot/ChangeSet input、Scope audit、Restore/ApplyPatch Gate、stale baseline |
| `Apex—— Context与Checkpoint系统详细设计.md` | Diagnostic/Evidence ContentRef、taint、Repair Agent 输入和 Checkpoint 恢复引用 |

### 35.1 解决的跨文档歧义

1. **RuleCheck `failed` 的含义**：拆分为 lifecycle `state`、业务 `verdict` 和 `failure_kind`；
2. **Tool 成功但规则失败**：ToolCall 使用 `succeeded_with_violations`，所属 Node/Run 由 Gate 决定；
3. **warning 是否阻断**：默认不阻断，Project 可收紧；hard safety 不能放宽；
4. **RuleCheck vs Verification Gate**：Check 是证据，Gate 是聚合决策；
5. **Repair 行为**：Checker 不写文件，Repair 是新的 Agent/Run；
6. **缓存通过**：复用旧 Check 但新建 Evidence binding，不伪造新执行；
7. **Stale 状态**：历史 pass 保留，但不能用于当前完成；
8. **`verification.md`**：是可审计导出 Artifact，不是最终事实源；
9. **`/skip-spec`**：只跳过 Spec 流水线，不跳过硬规则与基础验证；
10. **远端副作用**：通过 ExternalReceipt/Reconcile Gate 验证，不由文件 Snapshot 或本地 test 代替。

### 35.2 下一份设计的输入

本文将以下事实交给 `Apex—— MCP、Skill、Hook与Plugin扩展系统详细设计.md`：

- 扩展 Manifest 必须声明 capability、Rule/Hook trigger、schema、timeout、sandbox 和 publisher；
- Hook 只能收紧或产生 Diagnostic，不能授予权限、隐藏事实或直接写工作区；
- 自定义 Checker 默认不可信，必须 hash-pinned、隔离执行并输出结构化 receipt；
- Skill/Plugin 提出的 Repair 必须创建新 ToolCall/Repair Run；
- MCP 外部副作用需要 idempotency receipt 和 Reconcile Gate；
- 扩展热更新会使依赖的 RuleCheck/Gate stale；
- 扩展输出带 source/taint，不得未经边界处理进入 Prompt 或 Gate pass。

---

## 附录 A：Gate 聚合伪代码

```rust
fn aggregate_gate(
    definition: &GateDefinition,
    subject: &SubjectSnapshot,
    evidence: &[GateEvidence],
    waivers: &[Waiver],
) -> GateVerdict {
    if subject.digest != definition.expected_subject_digest {
        return GateVerdict::Stale;
    }

    for requirement in definition.requirements_in_stable_order() {
        let item = evidence.iter().find(|e| requirement.matches(e));
        let Some(item) = item else {
            return GateVerdict::Inconclusive(MissingEvidence::from(requirement));
        };

        if item.is_stale() {
            return GateVerdict::Stale;
        }
        if item.is_hard_denied() {
            return GateVerdict::Failed(item.diagnostic_refs());
        }
        if item.is_inconclusive() && requirement.required {
            return GateVerdict::Inconclusive(item.failure_refs());
        }
        if item.is_failed() && !valid_waiver(requirement, item, waivers) {
            return GateVerdict::Failed(item.diagnostic_refs());
        }
    }

    if has_applied_waiver(definition, evidence, waivers) {
        GateVerdict::Waived
    } else {
        GateVerdict::Passed
    }
}
```

聚合实现必须收集完整 explanation；伪代码为突出优先级而使用早返回，生产实现仍要记录未运行/被取消的其他 Evidence。

---

## 附录 B：PostTool RuleCheck 流程

```rust
async fn post_tool_check(receipt: ToolReceipt) -> Result<PostToolOutcome> {
    let post = snapshot.capture_post(&receipt).await?;
    let changes = snapshot.diff(receipt.pre_snapshot, post.id).await?;

    scope_auditor.verify(
        &changes.actual_write_scope,
        &receipt.active_claim_scope,
        &receipt.delegated_write_scope,
    )?;

    let ruleset = rules.current_pinned_revision(receipt.project_id).await?;
    let plan = planner.plan_incremental(&receipt, &changes, &ruleset).await?;
    let results = runner.execute_plan(plan).await?;
    let gate = gate_evaluator.evaluate_post_tool(&receipt, &changes, &results).await?;

    match gate.verdict {
        GateVerdict::Passed => Ok(PostToolOutcome::Succeeded),
        GateVerdict::Failed(_) | GateVerdict::Blocked(_) => {
            Ok(PostToolOutcome::SucceededWithViolations)
        }
        GateVerdict::Inconclusive(_) | GateVerdict::Stale => {
            Ok(PostToolOutcome::VerificationRequired)
        }
        GateVerdict::Waived => Ok(PostToolOutcome::SucceededWithWaiver),
    }
}
```

---

## 附录 C：RuleCheck 结果映射

| Runner/Parser 事实 | State | Verdict | Failure Kind |
|---|---|---|---|
| 完整执行，无违规 | completed | pass | — |
| 完整执行，有违规 | completed | fail | violations_found |
| 进程非零且结构化报告为违规 | completed | fail | violations_found |
| 进程崩溃，无完整报告 | completed | inconclusive | checker_failed |
| 超时 | completed | inconclusive | checker_timeout |
| 输出 schema 错误 | completed | inconclusive | output_invalid |
| 检查期间输入变化 | completed | stale | workspace_drift |
| 用户在启动前取消 | cancelled | — | cancelled_by_user |
| daemon 崩溃、状态未确认 | interrupted/unknown | — | interrupted_by_crash |
| 合法例外应用 | completed | waived | — |
| Profile 明确允许跳过 | completed | skipped | — |

---

## 附录 D：示例 Ruleset

```yaml
schema_version: 1
ruleset: apex-project-default

rules:
  - id: rust-no-panic-api
    category: correctness
    severity: error
    enforcement: block
    triggers: [post_tool, node_completion]
    selector:
      paths: ["crates/**/*.rs"]
    evaluator:
      type: ast_check
      checker: rust-public-api-policy@sha256:...
    fix:
      strategy: agent_edit
      max_attempts: 2

  - id: required-unit-tests
    category: spec_acceptance
    severity: error
    enforcement: block
    triggers: [node_completion, workflow_completion]
    selector:
      node_types: [implementation]
    evaluator:
      type: command_check
      runner_profile: cargo-test-workspace-v1
    evidence:
      require: test_report

  - id: no-workspace-scope-escape
    category: safety
    severity: critical
    enforcement: hard_deny
    triggers: [pre_tool, post_tool, restore_plan, apply_patch_plan]
    evaluator:
      type: builtin_predicate
      predicate: actual_and_declared_scope_within_project
```

---

## 附录 E：错误码

```text
RULE_SOURCE_OUTSIDE_BOUNDARY
RULE_SOURCE_UNTRUSTED
RULE_SOURCE_PARSE_FAILED
RULE_INCLUDE_CYCLE
RULESET_COMPILE_FAILED
RULESET_NOT_ACTIVE
RULESET_REVISION_CHANGED
RULE_CHECK_INPUT_INCOMPLETE
RULE_CHECK_INPUT_DIGEST_MISMATCH
RULE_CHECK_RUNNER_UNAVAILABLE
RULE_CHECK_TIMEOUT
RULE_CHECK_OUTPUT_INVALID
RULE_CHECK_STALE
RULE_CHECK_CANCELLED
DIAGNOSTIC_EVIDENCE_MISSING
GATE_REQUIRED_EVIDENCE_MISSING
GATE_FAILED
GATE_BLOCKED
GATE_INCONCLUSIVE
GATE_STALE
GATE_DEFINITION_CHANGED
WAIVER_NOT_ALLOWED
WAIVER_EXPIRED
WAIVER_SCOPE_MISMATCH
REPAIR_PLAN_STALE
REPAIR_SCOPE_EXPANDED
REPAIR_CYCLE_DETECTED
VERIFICATION_ARTIFACT_CORRUPT
VERIFICATION_REPORT_STALE
EXTERNAL_EVIDENCE_UNKNOWN
```

---

## 附录 F：核心不变量

```text
I1  Ruleset revision 发布后不可变，修改产生新 revision。
I2  RuleCheck 固定 ruleset/input/runner/environment digest。
I3  RuleCheck lifecycle 与业务 verdict 是不同字段。
I4  violations_found 与 checker_failed 永远不会被当成同一事实。
I5  required Evidence 缺失、损坏、unknown 或 stale 时 Gate 不能 pass。
I6  Gate pass 只适用于其固定 Subject、Input 和 Policy revision。
I7  Builtin hard safety rule 不能被 Permission、Waiver、Plugin 或 skip-spec 覆盖。
I8  Checker 不直接静默修改用户 Workspace。
I9  Repair 是新的 Agent/Run/Operation，拥有 Claim、Snapshot 和复验。
I10 Tool 成功与 PostTool violation 可以同时成立。
I11 actual_write_scope 必须是 active Claim 与 delegated scope 的子集。
I12 历史 pass 在输入变化后保留为历史，但当前状态为 stale。
I13 增量验证无法证明依赖边界时必须扩大范围或 inconclusive。
I14 本地 Snapshot/Test 不能证明远端副作用成功或已撤销。
I15 Verification Report 的每个 pass 都可追溯到当前 Evidence digest。
I16 Waiver 不删除原 Diagnostic，且必须有 scope、TTL、Actor 和 reason。
```

---

**文档结论**：Apex 的质量保障不能依赖模型记住规范、工具退出码或最终一句“测试已通过”。完整产品必须将不可变 Ruleset、固定输入 RuleCheck、结构化 Diagnostic、受限 Verification Runner、统一 Gate、Repair Run、Stale 传播、Waiver、外部 Reconcile 和 Verification Report 连接成一条证据链。只有当每次状态转换都能回答“检查了什么、基于哪个版本、由谁执行、证据在哪里、为什么通过、变化后是否仍有效、失败后如何修复”，Apex 才能真正实现 Spec 驱动且可审计的 Agent 工程交付。
