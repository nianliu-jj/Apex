# Apex —— 跨文档一致性审查与 ADR 清单

> 版本：v0.2（审查 + 回写完成）
> 审查日期：2026-08-08
> 回写日期：2026-08-09
> 审查范围：`..` 下 16 份 Apex 设计/执行文档（共 33,620 行）+ `../../README.md`（1,021 行）
> 当前状态：**32 项冲突全部按建议方案回写完毕，36 项 ADR 均已落到对应文档并带溯源标注**；回写中另发现并修复 2 项（C-33、C-34）。改动前的文档快照见 `docs_backup_before_adr_writeback_20260808_220841/`。
>
> **阅读提示**：§1.1 总览表的「状态」列反映当前文档现状；§1.2 起的冲突详情**保留审查当时的原文引用，未随回写修改**，作为评审对照基线使用。

---

## 0. 审查范围与方法

### 0.1 受审文档

| # | 文档 | 行数 | 定位 |
|---|------|------|------|
| 1 | 需求分析文档 | 585 | 上游基线 |
| 2 | 系统总体架构设计 | 958 | 上游基线 |
| 3 | 领域模型与事件规范 | 1829 | 上游基线（事件/状态机权威） |
| 4 | SQLite 数据模型与迁移设计 | 2609 | schema 权威 |
| 5 | API 与实时事件协议设计 | 2709 | 详细设计 |
| 6 | Agent Runtime 与 DAG 调度器 | 2707 | 详细设计 |
| 7 | Tool Gateway 与权限引擎 | 2478 | 详细设计 |
| 8 | Workspace 快照、Write Claim 与隔离工作区 | 2580 | 详细设计 |
| 9 | Context 与 Checkpoint 系统 | 1740 | 详细设计 |
| 10 | Rules 与 Verification Gate | 3101 | 详细设计 |
| 11 | MCP、Skill、Hook 与 Plugin 扩展系统 | 2827 | 详细设计 |
| 12 | Credential 与敏感数据治理 | 2541 | 详细设计 |
| 13 | Observability、审计与运维控制面 | 2093 | 详细设计 |
| 14 | Deployment、升级与灾备 | 2038 | 详细设计 |
| 15 | 项目开发计划（最小粒度 TUI 闭环） | 2081 | 总体执行计划 |
| 16 | v0.1 MVP 逐功能可运行阶段计划 | 744 | v0.1 执行级权威 |

### 0.2 方法与覆盖度声明

原始 15 份文档审查采用**双轨审查**：① 对跨文档共享的契约面（事件名、命名空间、状态机、存储布局、schema、错误码、版本档位、常量、capability）逐项检索并回读原文；② 6 个分片代理各自完整通读 2–3 份文档产出对账清单。新增的第 16 份 v0.1 执行级计划及 S00 日志实现由本报告 §0.5 进行增量对账；所有纳入结论均回读原文或运行产物验证。

- 下列每条冲突均附行号，属**已确认**，非推测。
- 复核确有必要：代理报告中已发现并剔除 1 条假阳性、修正 2 条框定不准的结论（见附录 C）。行号偶有 ±1 偏差，本报告中的行号以我回读时的实际位置为准。
- 未做穷尽覆盖：各文档内部散文论证、示例代码细节、protobuf 字段编号、36 张新增表的字段级外键。这些可能仍存在局部不一致。

### 0.3 一项关键观察：漂移方向是单向的

多数冲突不是随机分歧，而是**详细设计代表更新的一代设计，上游基线文档未同步**。典型例证：

- Snapshot 状态机：SQLite（第 1403 行）与 Workspace（第 836–843 行）互相一致，共同偏离领域模型 §5.13；
- RuleCheck 状态机：Rules 文档明确自述在"消解上游歧义"；
- Wasm 后端：架构列为待评估项，扩展系统 ADR-EXT-002 已作出混合方案决议。

因此多数 ADR 的正确姿态是**回写上游**，而非撤回下游。少数例外（如 Credential 的命令行注入、Deployment 取消项目级可提交资产）是下游放宽了上游的安全或产品约束，须驳回或补论证。

### 0.4 一致性良好的部分（已确认）

- 核心阈值常量：上下文水位 `60/75/85`、MCP 超时 `30s`、并发上限 `min(16, 2*cpu)` —— 跨文档一致。
- `/skip-spec` → wire `spec.skipped` → Rust `SpecSkipped` 三层映射 —— 跨 5 份文档一致。
- 规则来源 4 层优先级 —— Rules §126-130 与需求文档逐字一致。
- MCP 命名空间 `mcp__<server>__<tool>`、外部内容 taint 约束 —— 一致。
- Skills 三层渐进加载、metadata 层内容限定 —— 一致。
- error/warning 阻断语义、修复 Agent 独立 Run、checker 崩溃≠代码违规、模型自报不构成验收 —— 一致。
- 不变式编号 `INV-*-NNN` —— 无跨文档撞号。
### 0.5 2026-08-08 执行计划与日志实现增量审查

新增 `Apex—— v0.1 MVP逐功能可运行阶段计划.md` 后，v0.1 的功能执行粒度由总体计划中的 Epic/任务编号下沉为 `S00`～`S24`。二者权威边界已固定：总体开发计划负责产品里程碑、架构依赖和版本路线；新计划负责每个可运行阶段的输入、实现、命令、可见结果、自动验收和回滚。

首个实现 `apex-observability` 已与现有文档完成以下对账：

| 对账面 | 结论 |
|---|---|
| 需求中的“执行过程、调用详情和进度可查询” | S00 记录稳定 `message_code`、started/progress/completed/failed、进度数值和运行定位字段 |
| 总体架构的 `tracing`/Observability 边界 | 日志 crate 独立于 TUI/Core/SQLite；日志不成为业务状态和恢复事实源 |
| Observability 的单行位置式日志与本地默认策略 | 每次运行一个本地 Spring Boot 风格的单行位置式日志文件；同一任务链路使用 `traceId` UUID 贯穿；默认无远程 telemetry |
| 线程与协程语义 | 同时记录 OS 线程和显式 `TaskContext`；不依赖 Tokio 未承诺的内部 task ID |
| 源码定位与路径安全 | `source.file` 只记录编译器 callsite 的最后一级代码文件名，`source.line` 记录代码行；用户/项目路径仍按分类、相对路径或 digest 治理 |
| Secret 治理 | S00 已实现敏感字段名脱敏；完整内容扫描/safe view 仍由后续 S22 接入，禁止把 Prompt、Provider 原文或敏感正文交给日志 API |
| 失败隔离 | 初始化、flush、shutdown 返回类型化错误；日志 writer 不通过 panic 改变业务状态 |

当前实现验收：workspace 测试 5 项通过、Clippy `-D warnings` 通过，demo 可生成连续 sequence、单一 PID、多 OS 线程、两个 Tokio 任务及真实源码文件/行号。S00-001～S00-008 标记为 `verified`，S00-009 必须随每个后续阶段持续接入，不能提前关闭。


---

## 1. 冲突清单

分级标准：**P0** = 影响已冻结的产品定位或数据正确性，必须先冻结再编码；**P1** = 影响协议/schema 稳定性，编码前须统一；**P2** = 文档质量问题，不阻塞编码。

### 1.1 总览

> **状态说明（2026-08-09 更新）**：全部 32 项冲突已按 §2 建议方案回写到对应设计文档，36 项 ADR 均在受影响文档中带 `> ADR-00NN（跨文档一致性审查）` 溯源标注。下表「状态」列记录回写结果。**本报告 §1.2 起的冲突详情保留审查当时的原文引用，未随回写修改**——它是对照基线，用于评审时核对改动是否忠实于决策，不要按它判断文档现状。

| ID | 级别 | 冲突 | 关联 ADR | 状态 |
|----|------|------|----------|------|
| C-01 | P0 | 存储布局四套并存，项目级可提交资产被取消 | ADR-0002 | 已回写（混合方案） |
| C-02 | P0 | 36 张表游离于 schema 权威之外 | ADR-0011 | 已回写（迁移号 0011–0016） |
| C-03 | P0 | RuleCheck 状态机被改写，且违反上游强制要求 | ADR-0008 | 已回写（三维模型） |
| C-04 | P0 | **Run 状态 schema 无法存储 3 个基线状态** | ADR-0017 | 已回写（CHECK 修正） |
| C-05 | P0 | **verification 四值结论全部消失，`VerificationVerdict` 悬空** | ADR-0018 | 已回写（四值+映射） |
| C-06 | P0 | **Credential 允许命令行注入，违反硬基线且自相矛盾** | ADR-0019 | 已回写（条款删除） |
| C-07 | P1 | 版本路线图分叉，凭空出现 v0.4/v0.6/v0.8 | ADR-0001 | 已回写（归并 5 档） |
| C-08 | P1 | Snapshot 状态机：SQLite+Workspace vs 领域模型 | ADR-0020 | 已回写（下游为准） |
| C-09 | P1 | 事件名与命名空间大面积漂移（仅 1/9 逐字一致） | ADR-0003/0004/0005/0006 | 已回写（去 `.v1`、统一 `claim.*`） |
| C-10 | P1 | 4 处核心表名与基线不符 + `metric_samples` 缺失 | ADR-0011 | 已回写（含补 DDL） |
| C-11 | P1 | `Gate` 作为新聚合未进入领域模型 | ADR-0009 | 已回写（登记聚合） |
| C-12 | P1 | Permission 求值顺序 5 步 vs 11 步，硬拒绝降到第 5 位 | ADR-0012 | 已回写（硬拒绝前置） |
| C-13 | P1 | Hook 事件 6→9、返回值 4→7，且同文档内三种拼写 | ADR-0021 | 已回写（统一拼写） |
| C-14 | P1 | 数据分类 4 级 vs 5 级（Obs 自称"沿用"却不同） | ADR-0022 | 已回写（5 级） |
| C-15 | P1 | MVP 工具集缺 Bash/Task，且工具改名加 `builtin__` 前缀 | ADR-0023 | 已回写（补回+去前缀） |
| C-16 | P1 | ToolCall/Node 状态 schema 与基线不符 | ADR-0017 | 已回写（CHECK 修正） |
| C-17 | P1 | Write Claim 交付阶段矛盾（v0.1 强制 vs v0.5 交付） | ADR-0024 | 已回写（v0.1 交付） |
| C-18 | P1 | 错误码新增 11 个 + 仅 Observability 自建前缀族 | ADR-0010 | 已回写（并入基线族） |
| C-19 | P1 | ID 前缀新增 4 个 + `rev_` 语义冲突 | ADR-0025 | 已回写（登记+改名） |
| C-20 | P2 | Deployment 引入容器/服务化/自动更新/远程转正 | ADR-0026 | 已回写（版本归属表） |
| C-21 | P2 | 启动恢复 12 步在三份文档中变成 8/9/14 步 | ADR-0027 | 已回写（按 12 步重写） |
| C-22 | P2 | Context 压缩阶梯 4 档 vs Level 0–4 | ADR-0013 | 已回写（Level 0–4） |
| C-23 | P2 | Spec frontmatter 字段集三套 | ADR-0014 | 已回写（并集+必填） |
| C-24 | P2 | `repair.*` 命名空间使用未声明（单文档内部矛盾） | ADR-0007 | 已回写（补入注册表） |
| C-25 | P2 | 文档版本头/状态标记三套并存 | ADR-0015 | 已回写（统一体例） |
| C-26 | P2 | README 全部 8 条链接失效 + 残留他机绝对路径 | ADR-0016 | 已回写（24 链接验证） |
| C-27 | P0 | **Skill metadata 不在稳定前缀，Skill 无法被发现** | ADR-0031 | 已回写（移入稳定段） |
| C-28 | P1 | Turn 语义两套定义（含多次 Attempt vs 单次调用边界） | ADR-0032 | 已回写（Turn 含多 attempt） |
| C-29 | P1 | 上下文分层两套（6 层 vs 10 项），层序与归属均不同 | ADR-0033 | 已回写（7 层为准） |
| C-30 | P1 | Provider 接口裂成两套互不相交且都不完整 | ADR-0034 | 已回写（合并接口） |
| C-31 | P1 | 重试上限三处取值不同且未界定层级口径 | ADR-0035 | 已回写（三层口径） |
| C-32 | P2 | Checkpoint 触发条件 7 条 vs 10 条，RT 缺基线项 | ADR-0036 | 已回写（补基线项） |

回写过程中另发现并修复 2 项审查时未列出的缺陷：

| ID | 级别 | 问题 | 处置 |
|----|------|------|------|
| C-33 | P1 | `metric_samples` 被架构 §10.2 声明属 observability 表族，但**全库无 DDL** | 已补表结构 + 高基数标签拒绝规则 |
| C-34 | P2 | `legacy_ambiguous` / `legacy_unpinned` 两个迁移状态值只存在于散文，未进枚举 | 已加入对应 CHECK 约束 |


### 1.2 P0 冲突详情

#### C-01 存储布局四套并存

| 出处 | 布局 |
|------|------|
| 系统总体架构设计 | 用户级 `~/apex/`；项目级 `<project>/apex/`（Spec 等可提交资产随仓库走） |
| Deployment §4.1 | 平台原生目录：Linux `${XDG_STATE_HOME:-~/.local/state}/apex/`、macOS `~/Library/Application Support/Apex/`、Windows `%APPDATA%\Apex\`；DB 在 Home 根而非 `state/` |
| Deployment §容器模式（第 1361–1371 行） | `/data/apex` 持久卷 + `/backup` 卷 |
| SQLite 数据模型 | **自身不一致**：第 38 行用 `~/apex/state/apex.db`，第 105 行用另一套 |

影响不止路径字符串。Deployment 第 258–264 行把 `specs/`、`checkpoints/`、`memory/`、`snapshots/`、`worktrees/`、`project.toml` 全部移入 `<APEX_HOME>/projects/<project-id>/`，全文**未出现** `<project>/apex/`。而基线把"Spec 工件随仓库提交、可 code review、可多人共享"作为需求文档中的产品能力——改布局等于删能力。

其余缺口：Deployment 目录清单缺 `rules/`、`skills/`、`mcp.json`、`diagnostics/`；SQLite 目录清单缺 `auth.json`、`rules/`、`skills/`、`mcp.json`。Deployment 第 282 行的 `auth.json` 兼容路径仍写 `~/apex/auth.json`，但同文档第 1250 行的权限表把它算作 Apex Home 子项——而 Home 已不在 `~/apex/`，两处无法同时成立。

**须决策**：是否接受平台原生目录（更符合 OS 规范、利于打包发行），以及若接受，项目级可提交资产落在哪里。二者可共存（用户级走平台原生 + 项目级保留 `<project>/apex/`），这是建议方案。


#### C-02 36 张表游离于 schema 权威之外

SQLite 文档定义 68 张表并自称 schema 唯一源。6 份详细设计文档另外定义 36 张表，与主 schema **零重叠**：

| 文档 | 新增表 |
|------|--------|
| 扩展系统 | `extensions`、`extension_revisions`、`extension_instances`、`extension_grants`、`extension_generations`、`skill_revisions`、`skill_resource_loads`、`mcp_schema_revisions`、`hook_subscriptions` |
| Observability | `audit_records`、`metric_samples`、`alerts`、`incidents`、`health_checks`、`operator_actions`、`maintenance_run_steps`、`support_bundles` |
| Credential | `credentials`、`credential_versions`、`credential_grants`、`credential_leases`、`credential_usages`、`redaction_records`、`data_lineage_edges` |
| Rules | `gate_definitions`、`gate_attempts`、`gate_evidence`、`verification_results`、`waivers`、`ruleset_revisions` |
| Workspace | `snapshot_files`、`workspace_baselines`、`restore_plans`、`restore_conflicts` |
| Context | `context_blocks`、`context_builds` |

主 schema 的迁移序列到 `0010_projections` 为止，其中 `0009_extensions_memory` 是唯一疑似预留槽，但对应 DDL 只存在于扩展文档。`audit_records` 尤其关键——审计是 P0 安全能力，其表结构不在 schema 权威内意味着审计链的完整性无人把关。

**须决策**：36 张表分配迁移号并回写主 schema，还是主 schema 改为"按模块联邦、各文档自持 DDL"。建议前者，否则外键、索引与 `PRAGMA foreign_keys` 一致性无处校验。

#### C-03 RuleCheck 状态机被改写

领域模型（第 728–734 行）：

```text
queued → running → passed | violations_found | checker_failed | timed_out | cancelled
```

并强制要求：`violations_found`（有效检查结果）与 `checker_failed`（检查器基础设施故障）**必须分别统计和处理**。

Rules 文档第 428–441 行声称"沿用领域模型的生命周期"，实际给出：

```text
queued → running → completed | cancelled | interrupted | unknown
```

它把 `passed` / `violations_found` / `checker_failed` / `timed_out` 全部折叠进 `completed`，业务结果改由另一维度（verdict）承载。这有两个问题：一是"沿用"的说法与实际内容不符，最容易误导实现者；二是折叠后 `violations_found` 与 `checker_failed` 在生命周期维度不再可分，直接违反上游的强制要求——除非 verdict 维度显式复现该区分，而 Rules 文档未说明这一点。

**须决策**：确认二维模型（lifecycle × verdict）是有意演进并回写领域模型，同时明确 `checker_failed` 在新模型中的落点；或撤回改写。

补充：Rules 第 1893–1900 行给出旧数据迁移映射，其中"无法分类的旧 `failed` 迁为 `completed + inconclusive + legacy_ambiguous`"引入了一个**未进入任何枚举清单的新状态值** `legacy_ambiguous`。扩展系统第 2565 行有同类问题（`legacy_unpinned`）。迁移产生的状态值必须先进枚举。

#### C-04 Run 状态 schema 无法存储 3 个基线状态

SQLite 第 684 行（已回读确认）：

```sql
CHECK (state IN ('queued','running','awaiting_approval','blocked',
                 'completed','failed','cancelled','interrupted'))
```

基线 Run 状态为 11 值。差异有两类：

- **命名**：基线 `waiting_approval`，schema 写成 `awaiting_approval`；
- **缺失**：`waiting_user`、`paused`、`cancel_requested` 三个状态在 CHECK 约束中不存在。

这不是文档措辞问题——CHECK 约束会在运行时**拒绝写入**。而 API 协议第 734–735 行定义了 `PauseRun` / `ResumeRun` 命令，REST 第 787–788 行也暴露了对应端点。按当前 schema 实现，暂停一个 Run 会触发 CHECK 约束失败。

同类问题还有两处（见 C-16）：`tool_calls`（第 1219 行）用 `evaluating/running/approved/unknown`，基线为 `validating/executing`，且缺 `awaiting_claim`、`preflight`、`postflight`、`succeeded_with_violations`、`reconcile_required` 五个状态；`workflow_nodes.state`（第 1078 行）**完全没有 CHECK 约束**，而基线 Node 有 12 个状态。

**须决策**：以哪一方为准。若以基线为准，需改 CHECK 并补迁移；若以 schema 为准，需说明 `PauseRun` 命令如何落库。

#### C-05 verification 四值结论全部消失

基线 verification 结论为 `passed / failed / blocked / not_run`。在 Rules 与扩展系统两份文档中：

- `not_run` —— **全文零命中**（已用 grep 确认）；
- `passed` —— 改写为 `pass`；
- `blocked` —— 只作为 Gate 状态存在（第 660 行），不再是 verification 结论；
- `failed` —— 改写为 `fail`。

即基线四值**无一原样保留**。更严重的是第 1249 行 `verdict: VerificationVerdict` 引用了一个**全文未定义取值的枚举**，且同一文档给出三套互不映射的候选值集：

| 出处 | 取值 |
|------|------|
| 第 448–454 行 | `pass / fail / inconclusive / stale / skipped / waived`（六值） |
| 第 2156–2162 行（§26.3 表格） | `pass / fail / inconclusive / unknown` + 无 Evidence 时显示 `unverified` |
| 第 1249 行 | `VerificationVerdict`（未定义） |

验证结论是 Gate 判定的输入，直接决定 Node/Run 能否完成。取值集未冻结意味着这条链路无法实现。

**须决策**：冻结 `VerificationVerdict` 的封闭取值集，给出与基线四值的迁移映射，并统一 §26.3 表格。

#### C-06 Credential 允许命令行注入

Credential 第 560 行（已回读确认）在 Secret 注入通道列表中写：

> 5. 命令参数仅在无替代方案且有额外审批时使用。

这与两处硬约束冲突：

1. **基线**明确规定密钥不进入命令行参数——这是硬禁止，不是默认值。命令行参数对同机任意进程可见（`/proc/<pid>/cmdline`、`ps`、Windows WMI），"额外审批"无法改变这个事实。
2. **同一文档第 662 行**自列的 hard deny 条件包含：「注入通道会把 Secret 暴露在命令行或公共日志」。§7.3 允许的事，正是本文档 hard deny 列表禁止的事。

这是本轮唯一一处**下游放宽了上游安全硬约束**的冲突，且文档内部自相矛盾。建议直接删除第 560 行第 5 项，保留前 4 种通道（keychain handle / stdin+pipe / 严格 ACL 临时文件 / 子进程私有环境变量）已覆盖实际需求。

#### C-27 Skill metadata 不在稳定前缀

基线（架构 §8.1）规定 Stable prefix 必含 **skill metadata**——这是 Skills 三层渐进加载（metadata 常驻 → body 触发时加载 → resources 按需读取）的第一层，模型靠它知道有哪些 Skill 可用。

两份上下文相关文档的稳定前缀构成中均无此项：

| 出处 | 稳定前缀构成 |
|------|------------|
| Agent Runtime 第 633–640 行 | 系统策略、产品规则、项目指令、Agent Profile、Spec Revision（无 skill，也无工具目录） |
| Context 第 777–790 行 | Core Safety Policy → Actor/Agent Boundary → Project Trusted Instructions → Spec Artifact Heads → Ruleset → Tool Catalog → Workflow/Node → Checkpoint → Tail → Retrieval（无 skill） |

更彻底的是，`skill` 一词在 **Agent Runtime 全文零命中**（grep 确认）。

后果是功能性的而非文档性的：metadata 层不常驻 prompt，模型就不知道任何 Skill 存在，也就永远不会触发 body 加载——整个 Skills 系统在运行时不可达。扩展系统文档设计的三层加载机制因此落空。

**须决策**：在两份文档的稳定前缀中补入 skill metadata 层，并明确其排序位置（建议紧邻 Tool Catalog，同样按名称稳定排序以保 prefix cache）。

### 1.3 P1 冲突详情

#### C-07 版本路线图分叉

基线为 5 档：`v0.1 / v0.3 / v0.5 / v0.7 / v1.0`。当前实际存在两派：

| 立场 | 文档 | 档位 |
|------|------|------|
| 与基线一致 | 需求分析、系统总体架构、API 协议（§17）、项目开发计划（§2.1） | v0.1 / v0.3 / v0.5 / v0.7 / v1.0 |
| 偏离 | 扩展系统（§34） | v0.4 / v0.5 / **v0.6** / v0.7 / **v0.8** / v1.0 |
| 偏离 | Credential（§27） | v0.3 / v0.5 / **v0.6** / v0.7 / **v0.8** / v1.0 |
| 偏离 | Deployment（§31） | v0.5 / v0.7 / v1.0 / **v1.x**，且内容整体右移 |
| 不映射 | Observability（§1814-1840） | 用 P0/P1/P2 分期，不给版本号映射 |
| 偏离 | SQLite（§24） | v0.1 内容大幅缩水（见下） |

三类问题：

1. **Deployment 内容右移两档**：把"单机可用基线（TUI + apexd + SQLite + Provider）"标为 v0.5（基线 v0.1），"三端共享"标为 v0.7（基线 v0.3）。同一交付物差两个大版本，排期无法对齐。
2. **SQLite v0.1 缺能力**：§24 的 v0.1 不含 `tool_calls`、`permission_requests`、`rule_checks`、`checkpoints`、`snapshots`，推到 v0.3 与 v0.5。但基线 v0.1 明确含工具、权限、规则、Checkpoint、快照，API 协议 §17.1 也要求 v0.1 提供这些 API。**API 有接口而 DB 无表**。
3. **凭空新增档位**：v0.4 / v0.6 / v0.8 / v1.x。另有 4 份文档头标注"适用版本 v0.5～v1.x"，却覆盖 v0.1/v0.3 才需要的能力。

项目开发计划新增的 `v0.0.x`（TUI 垂直切片）与 `v0.2.x`（稳定化）**不算冲突**——它是在 v0.1 前后插入更细执行粒度，内容映射与基线一一对应。

#### C-08 Snapshot 状态机三方分歧

| 出处 | 取值 |
|------|------|
| 领域模型 §5.13 | `creating → created → restoring → restored`；`failed / conflict / restore_failed / expired` |
| SQLite 第 1403 行 | `intent / creating / ready / failed / deleting / deleted / unknown` |
| Workspace 第 836–843 行 | 同 SQLite（自述"沿用 SQLite"，属实） |

SQLite 与 Workspace 互相一致，共同偏离领域模型。差异是结构性的：新模型把 restore 相关状态移出 Snapshot、放入独立的 `snapshot_restores` 实体（Workspace 第 1500–1506 行），这实际上落实了领域模型 §5.13 自己的建议（"建议 Snapshot 与 SnapshotRestore 分表"）。但基线的 `conflict` 与 `expired` 在新模型中无对应，需补。

#### C-09 事件名与命名空间大面积漂移

领域模型 §7.3 定义的 9 个规则/扩展类事件中，**仅 `repair.run_created` 一个逐字保留**：

| 基线事件 | 详细设计实际 |
|----------|--------------|
| `rule.check_started` | `rule_check.started` |
| `rule.passed` | `gate.passed` |
| `rule.violation_found` | **无对应** |
| `rule.checker_failed` | **无对应** |
| `skill.loaded` | 拆为 `SkillMetadataLoaded` / `SkillBodyLoaded` / `SkillResourceLoaded` |
| `mcp.call_finished` | `McpCallFinished` |
| `hook.denied` | `HookBlocked` |
| `plugin.crashed` | `ExtensionCrashed` |
| `repair.run_created` | 一致 |

同时存在四类漂移，各自需独立决策：

- **命名风格三套**：领域模型用 `snake_case.点分`；Credential 与扩展系统用 `PascalCase`；架构 §15.3 也用 PascalCase（领域模型第 1024 行称 PascalCase 是 Rust 枚举名，但架构把它列为 wire 事件，语义不清）。
- **同一事实两个名**：`tool.call_finished`（领域模型/Tool Gateway/API）vs `tool.call.completed.v1`（Observability/Deployment）。
- **同一批事件两个命名空间**：`claim.*`（领域模型/Workspace）vs `write_claim.*`（Agent Runtime 第 1770–1774 行）；Agent Runtime 第 1858 行又混用 `claim.released`。
- **`.v1` 后缀约定**：领域模型把版本放在 envelope 字段 `schema_version`，详细设计普遍编进事件名（约 90 处）。二者重复，须二选一。

命名空间总数从声明的 20 个膨胀到实际约 48 个，新增未登记的包括 `credential data network process context workspace support_bundle maintenance incident alert git events path upgrade traces observability audit backup restore health metrics projection blob secret extension message database privacy`。其中 `alert.*`/`alerts.*`、`incident.*`/`incidents.*`、`event.*`/`events.*` 存在单复数混用——部分是事件与 capability 的合理分工（capability 格式为 `<domain>.<action>.v<major>`），但缺统一规则说明，读者无法判断前缀归属哪类。

#### C-10 核心表名与基线不符

| 基线表名 | SQLite 实际 | 行号 |
|----------|-------------|------|
| `events` | `domain_events` | 780 |
| `reviews` | `artifact_reviews` | 980 |
| `nodes` / `edges` | `workflow_nodes` / `workflow_edges` | 1072 / 1094 |
| `approvals` | 拆为 `permission_requests` + `permission_decisions` | 1231 / 1255 |
| `metric_samples` | **全文无该表** | — |

前四项属命名优化（更明确），可回写基线；`metric_samples` 缺失需确认是否由 Observability 的同名表承担（该表定义在 Observability 文档，属 C-02 的游离表之一）。

另有两处一致性缺口：`sessions.active_run_id`（第 588 行）无唯一/排他约束，`runs` 表也无 partial unique index，因此"一个 Session 同时最多一个非终局主线 Run"这条领域公理在 schema 层无强制；`artifacts.head_revision_id`（第 1007 行）的外键靠应用层校验。

#### C-11 `Gate` 作为新聚合未进入领域模型

Rules 文档引入 `Gate`，含独立 10 值状态机（第 653–666 行）：`pending running passed failed blocked inconclusive stale waived cancelled unknown`，6 张支撑表，以及确定性聚合规则（第 670–685 行）。

领域模型 §4.2 聚合目录中无 `Gate`。它已是 Node/Run/Workflow 完成判定的实际决策点（Rules 第 101 行："Runtime/Scheduler 负责把 Gate 结果接入 Node/Run/Workflow 状态机"），却不在领域权威文档中——一个承担核心职责的隐形聚合。

#### C-12 Permission 求值顺序不一致

| 出处 | 顺序 |
|------|------|
| 系统总体架构 §7.2 | 硬拒绝 → 项目安全策略 → Agent profile 上限 → 用户批准规则 → 当前模式（5 步） |
| Tool Gateway §10.1（第 484–502 行） | 11 步：identity/lease → schema → capability → trust gate → **builtin hard deny（第 5 位）** → 敏感数据 → scoped deny → spec 阶段 → 权限模式 → claim 冲突 → preflight hook |

架构把硬拒绝放第 1 位，Tool Gateway 放第 5 位。Tool Gateway 自身第 35 行与 §10.3 又都声明"hard deny 优先级最高"，与其 §10.1 的排序自相矛盾。

安全结论上二者通常等价（前 4 步只会更严，不会放行硬拒绝项），但"硬拒绝优先"是架构写明的安全不变式，须显式对齐或补等价性论证。Tool Gateway 的 11 步版本更可实施，建议以它为准并回写架构。

#### C-13 Hook 事件与返回值增删，且同文档内三种拼写

- **事件 6 → 9**：新增 `RuleCheckRequested`、`CheckpointCreated`、`ExtensionChanged`（扩展系统第 1260–1262 行）。
- **返回值 4 → 7**：新增 `BlockCompletion`、`ProposeRewrite`、`AsyncCheckScheduled`，且 `add_diagnostics` 改名 `DiagnosticOnly`。

更严重的是同一文档内三处写法不一致：

| 出处 | 取值 |
|------|------|
| 第 1363–1371 行 | 七值 PascalCase，含 `DiagnosticOnly` |
| 第 1406 行（合并优先级） | 六值 snake_case，写作 `diagnostics` |
| 第 2763 行（附录 C） | 四值，写作 `add_diagnostics` |

同一概念三种拼写、两种规模。附录 C 与正文差 3 个取值。

#### C-14 数据分类 4 级 vs 5 级

Observability 第 183–185 行称"沿用 Credential 文档的数据分级"，随后给出 5 级：`public / internal / confidential / sensitive / secret_prohibited`。

Credential 第 171–176 行实际为 4 级：`public / internal / sensitive / secret`。

即新增 `confidential` 层、`secret` 改名 `secret_prohibited`。"沿用"的表述与实际不符——这是本轮第三次出现"声称沿用实则改写"（另两处：Rules 的 RuleCheck 状态机、Workspace 的 Snapshot 状态机）。数据分级决定脱敏与存储策略，两套分级会导致同一字段在两个子系统被判定为不同敏感度。

#### C-15 MVP 工具集缺 Bash/Task

基线 MVP 工具集为 `Read Write Edit Bash Glob Grep Task`。Tool Gateway：

- 第 2159 行 Phase 1 仅含 `Read/Write/Edit/Glob/Grep`，`Bash` 推到 Phase 2（第 2169 行），**`Task` 未出现在任何 Phase**；
- 第 182–188 行工具改名加前缀：`builtin__read` / `builtin__write` / `builtin__shell` / `builtin__task`；
- 第 126 行只读内置工具新增基线未声明的 `List`（另有 `ApplyPatch`、`PowerShell`、`DirectExec` 出现在类型表但未进命名空间示例）。

`Task` 是子 Agent 派生的唯一入口，缺失它意味着 MVP 无法派生修复子任务——而增量规范检查的修复闭环依赖它。

#### C-16 ToolCall / Node 状态 schema 不符

见 C-04 的补充说明。`tool_calls`（第 1219 行）与 `workflow_nodes`（第 1078 行，无 CHECK）均与基线状态集不符。

#### C-17 Write Claim 交付阶段矛盾

- Tool Gateway 第 56 行（INV-TG-007）与第 950–959 行：**所有写操作必须先取得 Write Claim**，属强制不变式；
- Tool Gateway 五个 Phase（第 2155–2203 行）**均未安排 Write Claim 交付**；
- Workspace 第 2236 行：Phase 1 即交付 Claim + Lease + fence，第 2284 行列为"不得延期"；
- 基线：write claim 属 v0.5。

三方各执一词。若按基线（v0.5）实施，则 v0.1 的写工具违反 INV-TG-007；若按 Workspace（Phase 1）实施，则与基线路线图冲突。

Claim 相关还有三处跨文档不一致：错误码（TG `WRITE_CLAIM_CONFLICT` vs WS `CLAIM_CONFLICT`）、fence 归属（TG 绑定 execution attempt vs WS 绑定 claim ownership term）、释放时机（TG 在结果提交事务内释放 vs WS 要求 post snapshot + rules 之后）。释放时机的分歧有实际后果：按 TG 的做法，PostTool checker 后置运行时已无写互斥保护。

#### C-18 错误码新增与前缀族

新增 11 个未进基线的错误码：`ACTOR_MISMATCH`、`SCOPE_MISMATCH`、`PAGE_CURSOR_INVALID`、`PAGE_CURSOR_EXPIRED`、`RATE_LIMITED`、`TIMEOUT`、`PROTOCOL_VERSION_UNSUPPORTED`（API）、`STORAGE_BACKPRESSURED`、`DATABASE_MIGRATING`（SQLite）、`REGISTRY_GENERATION_CONFLICT`（扩展系统）。

Observability 另定义 28 个 `OBS_*` 前缀码，是唯一自建前缀族的文档，其中 `OBS_PROJECTION_LAGGING`、`OBS_CURSOR_EXPIRED` 与基线同名码语义重复。

游标过期一事有三套表示：`PAGE_CURSOR_EXPIRED`（分页）、`CURSOR_EXPIRED`（事件游标）、字符串 `"cursor_expired"`（API 第 1774 行 gap 字段）。

#### C-19 ID 前缀新增与 `rev_` 语义冲突

新增 4 个未进基线的前缀：`ins_`、`estore_`、`con_`、`req_`（API）。扩展系统另有 `ext_`、`exr_`、`exi_`、`hki_`、`rchk_`、`gatea_`。

**语义冲突**：SQLite 第 253 行的前缀示例中 `rev_` 紧随 `art_`，表示 artifact revision；而基线中 artifact revision 是 `arv_`，`rev_` 是 Review。两者混用会导致 ID 类型解析错误。

SQLite 前缀表仅列 8 个，基线 23 个中的 `wt_ msg_ turn_ agt_ wfl_ wfn_ clm_ tol_ per_ rck_ ckp_ snp_ arv_` 未出现（文档说明约束主要由 Rust newtype 执行，非硬冲突，但清单不完整）。

#### C-28～C-32 Agent Runtime 与 Context 的分歧

两份文档共同负责 Agent 执行与上下文装配，却对若干共享概念给出不同定义：

| 项 | Agent Runtime | Context |
|----|--------------|---------|
| **Turn 语义** | 第 146/174 行：Turn 内含多次 ProviderAttempt | 第 112 行：Turn = 一次 Provider 调用边界 |
| **上下文分层** | 第 635–640 行：6 层 | 第 255–262 行 L0–L6 七层 + 第 777–790 行装配序列 10 项。层数、层序、Checkpoint 归属均不同 |
| **Provider 接口** | 第 1974–1982 行 `ProviderPort { stream, reconcile }` | 第 768–772 行 `ProviderEncoder { capabilities, encode, estimate }` |
| **Checkpoint 触发** | 第 663–671 行 7 条 | 第 366–377 行 10 条 |
| **Checkpoint 内容** | 第 646–657 行扁平清单 | 第 446–452 行七分层 |
| **启动恢复** | 第 1580–1588 行 Phase 1–9 | 第 901–911 行步骤 1–10；未声明谁权威 |
| **Pause 与 Claim** | 第 1434 行长期 Pause 默认释放 Claim | 第 973 行只释放"可释放"Claim |

其中三项影响较大：

- **Turn 语义**（C-28）：基线 §5.5 明确"一个 Turn 可在 `provider_streaming` ↔ `tool_pending` 间多轮循环，每次新请求记录 ProviderCall attempt"——Runtime 的定义正确，Context 收窄了。收窄会导致 Provider 重试被误建为新 Turn，破坏 `turn.ordinal` 连续性。
- **Provider 接口**（C-30）：两套互不相交，且都不含基线要求的 `id()` / `capabilities()` / `stream()` / `count_tokens()` 全集。Context 用 `estimate(&[ContentBlock]) -> TokenEstimate` 替代 `count_tokens`——估算与精确计数在预算判定上不等价，会影响 60/75/85 阈值触发的准确性。
- **重试上限口径**（C-31）：第 1528 行 `max_attempts: 3`、第 2529 行 `max_transparent_retries = 2`、第 2667 行节点示例 `max_attempts: 2`，三处取值不同且未界定层级（Provider 透明重试 / Node attempt / Run attempt）。基线只说"Provider 有界重试最多 3 次"，未分层。须先定义层级再分别赋值。

Checkpoint 触发（C-32）：Context 的 10 条是基线的超集（另加 Turn start/end、schema 变更），属合理细化；Agent Runtime 的 7 条**缺** Spec 阶段完成、用户命令、token 60/75/85、Run 结束/失败——这些是基线明确要求的触发点。

状态与阻塞码的增删：Agent Runtime 第 1432 行让 Agent 进入 `paused`（基线 Agent 无此态，且该文档第 291 行自称 paused 由 Run 承载，自相矛盾）；第 1711/1738 行发布 `workflow.failed` / `workflow.interrupted`（基线 Workflow 无此二态，该文档第 347 行又自称以领域枚举为准）；新增阻塞码 `WORKSPACE_BASELINE_CHANGED`、`PROJECT_UNAVAILABLE`、`context_capacity_blocked`、`ExternalReconcileRequired`。

两处需澄清而非改判的措辞：

- Context 第 1233 行给"同一 `agent_id + run_id` 最多一个 active Provider Turn"加例外"除非 Workflow 明确允许并行 Node"。并行 Node 本属不同 Run/Agent，该例外在 `agent_id + run_id` 范围内不成立——是措辞歧义，不是放宽公理。
- Context 第 558–568 行把 `P3 approved_spec` 排在 `P4 current_workflow_and_checkpoint` 前，与"Checkpoint-first"倾向相反。但两者概念不同：该阶梯是**压力下的保留优先级**，基线 §8.2 是**溢出后的重建加载顺序**。不是同一件事，但倾向相反易误导，建议显式说明关系。

另外 Agent Runtime 第 1120 行把全局并发上限 `min(16, 2*cpu)` 降为"建议，最终值由设备探测、用户设置和 Provider 限额共同决定"，而第 2539 行又称安全硬上限不可提高——两处未界定谁是硬上限。基线为固定值。

### 1.4 P2 冲突详情

| ID | 详情 |
|----|------|
| C-20 | Deployment 引入多项基线未声明的形态：容器部署 + `/data/apex` 持久卷 + liveness 探针（第 1361–1371 行）；Windows Service / launchd / systemd unit（第 1337–1359 行）；Development/Nightly/Stable/Enterprise 四发布通道与受控自动更新（第 558–563 行）；把 Remote Single-User 列为正式部署模式（第 83–87 行，基线为"显式启用的后续模式"）；放宽多实例（第 353 行，基线为每 OS 用户一个）。另第 178–179 行与第 1916 行的 database format 值（5 vs 4）自相矛盾。 |
| C-21 | 启动恢复 12 步（领域模型 §10.2）在下游变成三套：Deployment 第 320–330/1120–1129/1511–1520 行三套流程（9 态/10 项/8 步）均未映射 12 步，附录 E 无任何 `recovery.*` 事件；Observability 第 1608–1625 行为 14 步，缺 daemon instance lock、outbox 恢复、Markdown write intent reconcile、Snapshot reconcile、FTS 重建。Observability 第 1620 行"释放过期 Write Claim"与基线"stale claim 先置 `suspect`"冲突——直接释放可能与仍在运行的 owner 并发写。 |
| C-22 | 基线降级阶梯 4 档（软提示 → 工具结果裁短 → 历史占位化 → 结构化摘要，架构 §8.2）；Context 第 734–740 行为 Level 0–4 五档，首档是"去重复引用"而非"软提示"，"历史占位化"无对应档。属细化但未标明取代关系。 |
| C-23 | Spec frontmatter 字段三套：需求 §3.1.3 为 `status/created_at/updated_at/version`；架构 §5.2 为 `id/feature/kind/status/version/created_at/updated_at/content_sha256`；领域模型 §12.4 另加 `format_version`。 |
| C-24 | 领域模型 §7.2 命名空间表未含 `repair.*`，§7.3 第 1123 行却定义 `repair.run_created`。单文档内部矛盾。 |
| C-25 | 文档头三套体例并存：`版本 v0.1 + 日期 + 状态待评审`（需求/架构/领域模型/API）、`文档状态：架构基线 + 版本 v1.0-draft`（Runtime/Tool Gateway）、`文档状态：详细设计基线 + 适用版本 v0.5～v1.x`（Credential/Deployment/扩展/Observability）。Context/Rules/Workspace 用英文 `Draft for final product architecture` 且无版本号。 |
| C-26 | `../docs/README.md` 是 8 个参考项目的横向对比索引，**未收录任何 Apex 文档**。其 8 条链接全部失效（链接用连字符 `opencode-实现原理分析.md`，实际文件名用空格 `opencode 实现原理分析.md`）。第 3 行残留他机绝对路径 `/Users/nianjiu/Worksapce/AiAgent/docs/`（含拼写错误 `Worksapce`），当前项目在 `D:\AiAgent`。需求文档 §9.1 的 8 条引用有同样的连字符问题。 |

### 1.5 其他跨文档不一致（低风险，建议一并修正）

| 项 | 分歧 |
|----|------|
| 分页上限 | API 第 2243 行"默认 50，最大 200" vs SQLite 第 1801 行"默认 50、最大 500" |
| 事件批量上限 | API 第 2240 行"≤100 events 或 ≤512 KiB" vs SQLite 第 1800 行"100～1000 行或 1 MiB" |
| protocol_version 类型 | API 用 `"1.2"` 字符串 vs SQLite 第 437/548 行 INTEGER |
| config_revision 类型 | API 第 705 行 string vs SQLite 第 473 行 INTEGER（同文档第 513 行又是 TEXT） |
| Artifact 导入命令名 | API `ImportArtifactFromFile` vs SQLite 第 1044 行 `ImportExternalArtifactEdit` |
| Command 同步响应 | 基线三种（Accepted/Rejected/Duplicate）vs SQLite 第 382 行 `Accepted/Completed`；`command_dedup.status`（第 843 行）另有 6 值且无 `duplicate` |
| 缺表 | API 定义 `AcceptRuleException`、`CreateRepairRun` 及对应 REST 端点，SQLite 无 `rule_exceptions` / repair run 表 |
| 敏感文件默认 | 基线"默认只读或询问" vs Tool Gateway 第 906 行"普通 Agent 不读取"（默认拒绝）；glob 集扩展为含 `p12/pfx/.ssh/.gnupg/secrets*` |
| force push 定级 | 基线列为硬风险不可覆盖 vs Tool Gateway 第 710 行归 CRITICAL 一次性强确认 |
| arity 归一 | 基线示例 `git checkout *` vs Tool Gateway 第 809–817 行明确拒绝把 `git checkout <ref>` 与 `git checkout -- <path>` 归一（后者更安全，建议采纳并更新基线示例） |
| stdio 进程回收 | 基线 `pgrep -P` 策略 vs 扩展系统第 658–665 行 Job Object（Windows）/ process group、cgroup（Unix）；`pgrep` 全文零命中。后者更健壮，建议采纳 |
| Canonical path 基准 | Tool Gateway 用 project-relative vs Workspace 第 85/226 行用 Worktree-relative；Workspace 第 2362 行审查清单又要求 project-root-relative（自相矛盾） |
| 面板投影 | 基线 4 个 vs Observability 第 581–616 行仅 skill/mcp/subagent 三个，缺 Memory（`memories`+`recalls`） |
| 一致性等级 | 领域模型 §11.2 逐投影给出强/近实时/异步/可重建 vs Observability 第 447–456 行只给 `eventual/at_least_seq/strong_current` 且无逐投影映射 |
| hash chain | 基线列为未冻结安全 ADR vs Observability 第 1615/1009/1224 行已作为启动步骤与默认告警 |
| 配置格式 | 基线 TOML vs Credential 附录 A 用 YAML `apiVersion: apex.dev/v1` |
| 协议版本号 | 扩展系统内 `apex.dev/v1`、`"1.0"`、`"1.2"`、`">=1.0 <2.0"`、`Plugin protocol v1` 五套并存，关系未定义 |
| 顺序种类 | 基线仅允许 4 种逻辑顺序 vs API 新增 `stream_seq`、`transient_seq`、`scanned_through_global_seq` |


---

## 2. 重复定义清单

重复本身不都是问题——详细设计复述上游契约有助于自包含阅读。风险在于**复述与原文产生漂移**，或**读者不知道哪份是权威**。

| 内容 | 权威源 | 复述位置 | 状态 |
|------|--------|----------|------|
| `EventEnvelope` | 领域模型 §7.1 | 架构 §3、API 协议、项目开发计划 | 需核对字段集一致性 |
| `CommandEnvelope` | 领域模型 §6.1 | API 协议、项目开发计划 | 需核对字段集一致性 |
| RuleCheck 状态机 | 领域模型 §5.12 | Rules §6.1 | **已漂移**（见 C-03） |
| Permission 求值顺序 | 架构 §7.2 | Tool Gateway §10.1 | **已漂移**（见 C-10） |
| 事件命名空间表 | 领域模型 §7.2 | 各详细设计散布 | **已漂移**（见 C-08） |
| Checkpoint 触发条件 | 架构 §8.2 | Context §8.2 | 一致 |
| 上下文水位阈值 | 需求 §3.5.1 | Context §8.2、架构 §8.2 | 一致 |
| Skills 发现路径 | 需求 §3.8.1 | 架构 §9.1、扩展系统 §373-377 | 一致 |
| 规则来源优先级 | 需求 §3.2 | 架构 §7.3、Rules §126-128 | 一致 |
| Hook 事件集 | 架构 §9.4 | 扩展系统 §1254-1264 | 一致（`Stop` 显式细化为 `AgentStop`/`SessionStop` 并说明兼容映射） |
| 数据库 DDL | SQLite 文档 | 6 份详细设计另建 36 表 | **已分裂**（见 C-02） |

建议在每份详细设计开头统一加一句权威声明，例如："本文复述的上游契约仅供阅读连贯，冲突时以《领域模型与事件规范》为准。"

---

## 3. 统一 ADR 清单

### 3.1 现状

各详细设计文档已自建 11 组、共 100 项带前缀 ADR：

| 前缀 | 数量 | 出处 |
|------|------|------|
| `ADR-DB-*` | 12 | SQLite 数据模型 §25 |
| `ADR-AR-*` | 10 | Agent Runtime §32 |
| `ADR-TG-*` | 10 | Tool Gateway §32 |
| `ADR-WS-*` | 10 | Workspace §33 |
| `ADR-RV-*` | 12 | Rules §33 |
| `ADR-CTX-*` | 9 | Context §28 |
| `ADR-EXT-*` | 9 | 扩展系统 §35 |
| `ADR-CRED-*` | 8 | Credential §28 |
| `ADR-OBS-*` | 12 | Observability §29 |
| `ADR-DEP-*` | 12 | Deployment §32 |
| （无前缀，表格形式） | 12 | API 协议 §18 |

另有两处**未编号**的待固化清单：架构 §20（8 项）、领域模型 §15（10 项）。

问题：这 100 项全部是**文档内部的局部决策记录**，没有全局编号、没有状态字段（proposed/accepted/superseded）、没有跨文档去重。架构 §20 与领域模型 §15 的 18 项待固化事项，与各详细设计的 100 项之间存在覆盖关系但无交叉引用——无法回答"架构 §20 第 4 项（Shadow Git 兼容策略）是否已被 ADR-WS-006 冻结"这类问题。

### 3.2 已冻结 ADR（本次审查产出，2026-08-09 全部回写）

下列 36 项对应第 1 节的 32 个冲突（部分冲突拆为多项决策）。**全部已按「建议方案」列执行并回写到对应设计文档**，每处改动在目标文档中带 `> ADR-00NN（跨文档一致性审查）` 溯源标注，写明原状与改动理由。

其中 4 项在审查报告中原本写作「二选一」而未给推荐，已由用户拍板：

| ID | 主题 | 用户决策 | 影响面 |
|----|------|----------|--------|
| ADR-0003 | 事件版本载体 | **版本回归 `schema_version` 字段，事件名去掉 `.v1`** | 97 个事件名跨 4+ 份文档；36 个 capability 标识符（`credential.use.v1` 等）与 schema 标识符**保留**后缀，未受影响 |
| ADR-0002 | 存储布局 | **混合方案**：用户级平台原生目录 + 保留 `<project>/apex/` | Spec/Rules/Skills/Memory 仍随仓库提交、可 code review |
| ADR-0024 | Write Claim 版本归属 | **提前到 v0.1**（v0.1 可为薄实现，但接口与事件从第一天正确） | `INV-TG-007` 自 v0.1 成立；需求 §5.2、架构 §18、Tool Gateway Phase 1、Workspace 阶段映射同步 |
| ADR-0022 | 数据分类级数 | **5 级**（新增 `confidential`，`secret` → `secret_prohibited`） | Credential §3 为权威，Observability「沿用」改为「采用」 |

| ID | 主题 | 建议方案 | 依据 | 阻塞 |
|----|------|----------|------|------|
| ADR-0001 | 版本档位表 | 5 档基线（v0.1/v0.3/v0.5/v0.7/v1.0）为唯一；v0.4/v0.6/v0.8 降为子阶段；Deployment §31 重标；v0.0.x/v0.2.x 保留为执行粒度 | 4 份文档已一致，仅 3 份偏离 | 排期 |
| ADR-0002 | 存储布局与项目级资产 | 用户级采用平台原生目录；**保留** `<project>/apex/` 承载 Spec/Rules/Memory 等可提交资产；修正 SQLite 文档自相矛盾 | 兼顾 OS 规范与需求文档的团队协作能力 | P0 |
| ADR-0003 | 事件版本载体 | **已定**：版本进 envelope `schema_version` 字段，事件名去掉 `.v1`；capability 标识符保留 `.v<major>` 后缀 | 事件名在事件流中长期持久化，升版本不应产生新名字迫使消费者同时订阅多名 | P0 |
| ADR-0004 | 事件命名空间注册表 | 建立唯一注册表，明确事件/capability 两类前缀规则与单复数约定 | 20 → 48 已失控 | P1 |
| ADR-0005 | 工具完成事件名 | 统一为 `tool.call_finished`（3 份文档在用，且与 `tool.call_started` 对仗） | 多数派 + 命名对称 | P1 |
| ADR-0006 | 写租约事件命名空间 | 统一为 `claim.*`（领域模型已声明，Workspace 已采用） | 权威源 + 多数派 | P1 |
| ADR-0007 | `repair.*` 命名空间 | 补登记进 §7.2，或将 `repair.run_created` 并入 `rule.*` | 单文档内部矛盾 | P2 |
| ADR-0008 | RuleCheck 状态模型 | 采纳二维（lifecycle × verdict × failure_kind）并**回写领域模型 §5.12**，显式说明 `checker_failed` 落点以满足上游可分性要求 | 二维模型确实更清晰 | P0 |
| ADR-0009 | `Gate` 聚合登记 | 将 Gate 提升为领域模型正式聚合，补入 §4.2 聚合目录与状态机章节 | 已承担核心判定职责 | P1 |
| ADR-0010 | 错误码命名规范 | 确定是否全局启用模块前缀；若不启用，Observability 的 28 个 `OBS_*` 去前缀并消除 2 处重复 | 一致性 | P1 |
| ADR-0011 | Schema 权威边界 | 36 张表分配迁移号并回写 SQLite 主 schema；或明确改为联邦式并定义外键校验机制 | 审计表尤其不能游离 | P0 |
| ADR-0012 | Permission 求值顺序 | 以 Tool Gateway 11 步为准，回写架构 §7.2，并补硬拒绝优先性的等价性说明 | 11 步更可实施 | P1 |
| ADR-0013 | 上下文降级阶梯 | 以 Context 的 Level 0–4 为准，标明取代架构 §8.2 的 4 档表述 | 细化版更完整 | P2 |
| ADR-0014 | Spec frontmatter 字段集 | 以架构 §5.2 的 8 字段为准，加领域模型的 `format_version`，回写需求文档 | 最完整 | P2 |
| ADR-0015 | 文档头体例 | 统一为：文档状态 + 版本 + 日期 + 上游依据 + 适用版本 | 文档质量 | P2 |

| ADR-0016 | README 定位与链接 | 修复 8 条链接（连字符→空格）、移除他机绝对路径、增加 Apex 16 份受审文档索引；同步修复需求 §9.1 | 链接全失效 | P2 |
| ADR-0017 | Run / ToolCall / Node 状态集 | 以基线状态集为准，修正 CHECK 约束并补 `workflow_nodes` 缺失的 CHECK；统一 `waiting_approval` 命名 | schema 会在运行时拒绝合法状态 | P0 |
| ADR-0018 | `VerificationVerdict` 取值 | 冻结封闭枚举，给出与基线 `passed/failed/blocked/not_run` 的迁移映射，统一 Rules §26.3 表格 | 枚举悬空则 Gate 链路无法实现 | P0 |
| ADR-0019 | Secret 注入通道 | **删除**"命令参数"选项，保留 keychain handle / stdin+pipe / 严格 ACL 临时文件 / 子进程私有环境变量四种 | 违反上游硬禁止 + 与本文档 hard deny 自相矛盾 | P0 |
| ADR-0020 | Snapshot 状态与 Restore 分离 | 采纳 SQLite+Workspace 的分表模型并回写领域模型；补 `conflict` 与 `expired` 的落点 | 下游两文档已一致，且落实了领域模型自身建议 | P1 |
| ADR-0021 | Hook 事件集与返回值 | 确认 9 事件 / 7 返回值为最终集合，回写架构；修正扩展系统内三处拼写不一致（统一 `DiagnosticOnly`），同步附录 C | 同文档内三种写法 | P1 |
| ADR-0022 | 数据分类级数 | **已定**：采用 5 级（`public`/`internal`/`confidential`/`sensitive`/`secret_prohibited`），Credential §3 为权威，Observability 同步并把"沿用"改为"采用" | 4 级会把"元信息"（路径、参数摘要、用量）与"内容本身"（Prompt 正文、源码）压进同一级，面板脱敏粒度过粗 | P1 |
| ADR-0023 | MVP 工具集与命名 | 确认 `Task` 与 `Bash` 是否属 v0.1（基线为是）；确认 `builtin__` 前缀与 `List`/`ApplyPatch` 是否进 MVP | `Task` 缺失则修复闭环断裂 | P1 |
| ADR-0024 | Write Claim 交付阶段 | 三选一并统一：若 claim 属 v0.5，则 INV-TG-007 须加版本限定；若属 v0.1，则回写基线路线图 | 不变式与路线图互斥 | P1 |
| ADR-0025 | ID 前缀注册表 | 消除 `rev_` 歧义（artifact revision 用 `arv_`），登记新增前缀，补全 SQLite 前缀清单 | ID 类型解析错误 | P1 |
| ADR-0026 | 部署形态边界 | 明确容器/服务化/自动更新/远程访问各自属哪个版本，以及是否进 v1.0 范围 | 基线为本机优先 | P2 |
| ADR-0027 | 启动恢复步骤 | 以领域模型 §10.2 的 12 步为准，三份下游文档逐步映射；修正"释放过期 Claim"为"先置 suspect" | 直接释放有并发写风险 | P1 |
| ADR-0028 | Claim fence 归属与释放时机 | fence 绑定 claim ownership term（WS 方案）；释放须在 post snapshot + rules 之后 | 否则 PostTool 校验期无写互斥 | P1 |
| ADR-0029 | 迁移引入的状态值 | `legacy_ambiguous`、`legacy_unpinned` 先进枚举清单再用于迁移 | 清单外状态值 | P2 |
| ADR-0030 | 事件命名风格 | 冻结 wire 格式为 snake_case 点分；PascalCase 仅用于 Rust 枚举名，并在架构 §15.3 明确标注 | 三套风格并存 | P1 |
| ADR-0031 | Skill metadata 入稳定前缀 | 在 Runtime 与 Context 的稳定前缀中补入 skill metadata 层，位置建议紧邻 Tool Catalog 并按名称稳定排序 | 缺失则 Skills 系统运行时不可达 | P0 |
| ADR-0032 | Turn 边界定义 | 以 Runtime/基线为准：一个 Turn 含多次 ProviderAttempt；修正 Context 第 112 行 | 收窄会破坏 `turn.ordinal` 连续性 | P1 |
| ADR-0033 | 上下文分层模型 | **已定**：以 Context 文档的 7 层模型（L0–L6）与 §11.2 装配序列为权威，Agent Runtime 的 6 层表述改为引用；Skill Metadata 补入稳定段 | Context 侧更完整且含 Tool Catalog / Skill Metadata | P1 |
| ADR-0034 | Provider 接口全集 | 合并为含 `id/capabilities/stream/count_tokens` 的单一 trait；明确 `estimate` 与 `count_tokens` 各自用途 | 估算不能替代精确计数 | P1 |
| ADR-0035 | 重试层级与上限 | 先定义三层（Provider 透明重试 / Node attempt / Run attempt），再分别赋值并统一三处配置 | 口径未界定 | P1 |
| ADR-0036 | Checkpoint 触发条件 | 以 Context 的 10 条为准，回补 Runtime 缺失的基线触发点 | Runtime 缺基线必需项 | P2 |

### 3.3 存量 ADR 待办

除上述 16 项外，还需处理：

1. **架构 §20 的 8 项**与**领域模型 §15 的 10 项**：逐项标注是否已被某个 `ADR-XX-NNN` 冻结，未冻结的纳入全局序列。已可确认对应关系的例如：架构 §20 第 4 项（Shadow Git 无 Git/裸仓库/Windows 兼容）↔ `ADR-WS-006`；第 7 项（插件 Wasm vs 子进程）↔ `ADR-EXT-002`（已决为混合方案）。
2. **100 项局部 ADR 补齐元数据**：状态（proposed/accepted/superseded）、日期、决策人、影响文档。
3. **SQLite §25 尾部 6 项**"待在实现前最终锁定"（bundled SQLite 最低版本与构建矩阵、`synchronous=NORMAL` 是否可选、Blob 阈值 64 KiB/256 KiB、是否库级加密、privacy purge 保留边界、是否完整 hash chain）——这些是有默认倾向但未冻结的，应转为正式 ADR。

4. **两条已落笔但与基线冲突的 ADR 需上游裁决**：
   - `ADR-RV-002`（生命周期与 Verdict 分离）与基线 RuleCheck 状态集直接冲突（见 C-03）；
   - `ADR-EXT-002`（混合 Wasm + 受监督子进程）把架构列为"待评估"的 ADR 改为已决议（见 §3.5）。

### 3.5 已被下游 ADR 实际冻结的上游待定项

以下架构 §20 / 领域模型 §15 的待固化项，已在某份详细设计中作出决议，建议直接采纳并标注上游为 superseded：

| 上游待定项 | 下游决议 | 结论 |
|-----------|---------|------|
| 架构 §20-4 Shadow Git 无 Git/裸仓库/Windows 兼容 | `ADR-WS-006` | 独立 repo/index/ref namespace；alternates 仅作读取优化 |
| 架构 §20-6 Worktree 隔离触发与 patch 合并 | `ADR-WS-005` | 不递归复制，一律 patch 回传并经 Claim/Rules/Verification |
| 架构 §20-7 插件 API Wasm vs 子进程 | `ADR-EXT-002` | 混合：纯计算走 Wasm/WASI，生态二进制与 OS API 走受监督子进程，第三方 native 不进 Core |
| 领域模型 §15 Aggregate persistence | `ADR-DB-004` | Current State + Event Store + Projection 混合 |
| 领域模型 §15 Unknown operation | `ADR-DB-011`、`ADR-AR-010`、`ADR-EXT-008`、`ADR-TG-008` | 一致：不自动重试，显式阻断并 reconcile |
| 领域模型 §15 Blob storage | `ADR-DB-007` | CAS Blob + DB 保存 metadata/ref |

### 3.6 其余悬空决策（有默认倾向，未冻结）

代理审查在各文档中另摘出约 25 项悬空点，其中数值型缺口较集中，建议一并冻结：

| 主题 | 现状 | 出处 |
|------|------|------|
| Repair 自动迭代预算 | `max_auto_repair_attempts=2` 等，措辞为"建议默认" | Rules 1407-1413 |
| MCP/进程重连退避参数 | 只写"指数退避和熔断"，无初始间隔/倍率/上限/阈值 | 扩展 665、1019 |
| Hook 熔断阈值 | 四个维度全部无具体数值 | 扩展 1460-1467 |
| Hook 结果缓存 TTL | 字段存在，无默认值与失效语义 | 扩展 1379 |
| Skill body token 上限 | "建议 <5k" 与硬上限 1 MiB、contextTokens 5000 三者关系未定 | 扩展 851 |
| 单 server 并发 / 单 session extension context | "由 policy/token budget 决定"，无产品默认值 | 扩展 2313-2314 |
| Claim lease/heartbeat 数值 | 倾向 30s/10s/2×interval，与 Run/Node budget 绑定规则未定 | Workspace 456-460 |
| 验证资源池 6 项配额 | 均无数值 | Rules 1660-1666 |
| page cursor TTL / Blob chunk 大小 / command 限流阈值 | 均有倾向无数值 | API 505-508、1938、2244 |
| 影子仓库目录名 | 倾向 `repo.git`，与基线 `.git` 二选一未定 | Workspace 750 |
| 中文 FTS 分词 | 倾向 unicode61，是否加应用分词列未定 | SQLite 1623-1628 |
| Organization/device policy 层 | 标注"未来"，是否进 v1 未定 | Tool Gateway 527 |
| `ExecuteRawCommand` | 倾向保留但默认关闭，是否发布未定 | API 649 |

有意不冻结的两项（不必强行决策，但应在文档中显式标注为非验收条件）：Rules 2342-2351 的验证延迟 SLO、扩展 2276-2289 的 8 项性能目标——两处原文都已声明是工程目标而非业务正确性条件。


### 3.4 建议的 ADR 治理方式

```text
docs/adr/
├── README.md                      # 索引 + 状态总览
├── 0001-version-milestones.md
├── 0002-storage-layout.md
└── ...
```

每篇统一格式：`状态 / 日期 / 上下文 / 决策 / 后果 / 替代方案 / 影响文档`。各详细设计文档内的局部 ADR 保留原位，但在全局索引中登记为 `ADR-XX-NNN`，避免二次编号。

---

## 4. 回写执行记录

原计划的处理顺序已于 2026-08-09 全部执行完毕：

1. **7 项 P0 已冻结并回写**：
   - ADR-0019（Secret 命令行注入）—— 条款删除，补充 `/proc/<pid>/cmdline`、`ps`、Windows 进程枚举的可见性论证；
   - ADR-0031（Skill metadata 入稳定前缀）—— Agent Runtime §9.1 与 Context §11.2 均已补入；
   - ADR-0017（Run/ToolCall/Node 状态集）—— 三处 CHECK 修正，`PauseRun` 现可落库；
   - ADR-0018（`VerificationVerdict`）—— 冻结四值并给出与 RuleCheck 六值的聚合映射；
   - ADR-0002（存储布局）、ADR-0011（schema 权威，迁移号 0011–0016）、ADR-0008（RuleCheck 三维模型）；
   - ADR-0001（版本档位）—— 三份偏离文档归并回 5 档。
2. **P1 命名与契约类已集中回写**：97 个事件名去 `.v1`、`claim.*` 与 `tool.call_finished` 统一、命名空间注册表补至 48 个、错误码并入基线族、Provider 接口三投影关系厘清、重试三层口径界定。
3. **P2 与文档质量项已完成**：文档头体例统一、README 24 条链接验证通过、frontmatter 字段集统一、压缩阶梯以 Level 0–4 为准。
4. **§3.6 的数值型缺口仍未补齐**——这些不阻塞架构决策，但阻塞可测试的实现，是后续工作的首要项（见 §5）。

回写中另发现并修复 2 项审查时未列出的缺陷：`metric_samples` 全库无 DDL（C-33）、两个迁移状态值未进枚举（C-34）。

一个通用建议：本轮出现三次"声称沿用上游、实则改写"（Rules 的 RuleCheck 状态机、Workspace 的 Snapshot 状态机、Observability 的数据分级）。这类表述比直接改写更危险，因为读者会跳过核对。建议在文档规范中加一条：复述上游契约时若有任何改动，必须写"**修订**上游 X"并列出差异，不得写"沿用"。

---

## 5. 后续建议

按优先级：

1. **补齐 §3.6 的约 25 项数值缺口**——Repair 迭代预算、MCP 重连退避参数、Hook 熔断阈值与缓存 TTL、Claim lease/heartbeat 时长、验证资源池 6 项配额、page cursor TTL、Blob chunk 大小、command 限流阈值等。这些当前都写作"建议默认"或留空，编码时无据可依，且多数属安全或稳定性参数，不宜由实现者临场决定。
2. **处理附录 A 的未覆盖项**——protobuf 字段编号与 REST 路由表的完整比对、36 张新增表的字段级外键与索引一致性、mermaid 图与正文一致性。其中外键一致性建议优先，因为 `0011`–`0016` 六个迁移引入的跨模块引用尚未系统校验。
3. **落实 ADR 治理结构**（§3.4）——建立 `docs/adr/` 目录，把本轮 36 项与各文档自建的 100 项局部 ADR 统一编号、补状态字段（proposed/accepted/superseded），并标注架构 §20 与领域模型 §15 的 18 项待固化事项各自被哪条 ADR 冻结。
4. **建立文档一致性回归检查**——本轮多数冲突（事件名、状态枚举、路径、版本档位）可用脚本静态校验。建议纳入 CI，避免同类漂移再次累积。

---

## 附录 A：审查未覆盖项

以下内容本轮未做穷尽核验，如需完整保证建议追加专项审查：

- protobuf 字段编号、REST 路由表的完整比对；
- 36 张新增表的字段级外键、索引与 `PRAGMA foreign_keys` 一致性；
- 16 份受审文档中所有 mermaid 图与相邻正文的一致性；
- 各文档内部散文论证与示例代码细节；
- 参考项目分析文档（8 份）与 Apex 设计的引用准确性。

## 附录 B：证据等级说明

| 等级 | 含义 | 本报告中的条目 |
|------|------|---------------|
| 已回读确认 | 我直接读取双方原文并比对 | 全部 P0（含 C-27）；C-07～C-12、C-24～C-26、C-28～C-31 的关键项；§1.5 中标注行号的多数项 |
| 代理报告 + 抽样复核 | 代理通读产出，我回读了其中关键条目 | C-13～C-23、C-32、§1.5 其余项、§3.6 |
| 代理报告未复核 | 仅代理单方结论 | §1.5 中未标注具体行号的少数项、§3.6 部分数值缺口 |

引用行号均来自审查日（2026-08-08）的文档状态，文档修订后需重新定位。

---

## 附录 C：复核中剔除与修正的条目

记录在此以便追溯，也说明为何代理结论需要复核。

**已剔除（假阳性）**

| 原主张 | 实际情况 |
|--------|---------|
| Agent Runtime 第 885–896 行"运行中原子切换 Workflow active revision"违反基线"Workflow 永久绑定 `tasks_revision_id`" | 混淆了两个不同的版本概念。该段描述的是**节点增删产生新 `workflow_revision`**，而领域模型 §4.8 第 407 行明确要求"动态增删节点必须产生新 `workflow_revision` 并重新校验 DAG"。与基线**完全一致**，非冲突。`tasks_revision_id`（Spec 绑定）与 `workflow_revision`（DAG 结构版本）是两个独立字段。 |

**已修正框定**

| 原主张 | 修正后 |
|--------|--------|
| Context 第 558–568 行"溢出恢复优先级顺序反了" | 该阶梯定义的是**压力下的内容保留优先级**，基线 §8.2 定义的是**溢出后的重建加载顺序**，二者不是同一概念。保留为"倾向相反、易误导，建议显式说明关系"，不作为顺序错误。 |
| Context 第 1233 行"给单 active Turn 公理加例外" | 例外条件"除非 Workflow 明确允许并行 Node"在 `agent_id + run_id` 范围内不成立（并行 Node 属不同 Run/Agent）。属措辞歧义，需澄清而非放宽公理。 |








