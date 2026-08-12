# 任务计划：Apex 全量架构文档重构

## 目标
在不实现代码的前提下，归档旧文档并生成一套内部一致、可直接指导后续实现的 Apex 完整产品架构文档。

## 当前阶段
Phase 11：原子模块系分文档与 TUI 交互原型

## 各阶段

### Phase 1：需求基线与资料盘点
- [x] 完成逐项需求澄清并获得用户确认
- [x] 归档旧 README 与 docs 文档
- [x] 盘点现有 Apex 文档的可复用结论和已知缺陷
- [x] 研究 AiAgent 目录中的参考实现
- [x] 将关键发现记录到 findings.md
- **Status:** complete

### Phase 2：文档信息架构与跨文档契约
- [x] 定义新文档目录、权威关系和引用规则
- [x] 冻结领域术语、ID、状态机、协议与错误模型
- [x] 建立需求到文档、模块和验收标准的追踪矩阵
- [x] 记录架构决策及理由
- **Status:** complete

### Phase 3：生成全新文档集
- [x] 生成 README 与文档总册
- [x] 生成需求、总体架构、Cargo workspace 和 Trait 契约
- [x] 生成客户端、存储、日志、Provider、权限、Spec、Context、Agent DAG 文档
- [x] 生成 Skills、MCP、Plugin、发布迁移、质量风险文档
- [x] 为 L4 设计补齐 Mermaid 架构图、流程图、时序图、状态机和 ER 图
- **Status:** complete

### Phase 4：一致性与可实现性验证
- [x] 校验所有已确认需求均有落点
- [x] 校验路径、状态枚举、Trait 名称和协议跨文档一致
- [x] 校验 Mermaid 代码块、Markdown 链接和文档索引
- [x] 校验仅有文档变更且未触碰用户代码改动
- **Status:** complete

### Phase 5：交付
- [x] 更新 README 文档导航
- [x] 汇总生成/归档文件与验证证据
- [x] 向用户交付完整结果
- **Status:** complete

### Phase 6：原子化功能开发执行计划
- [x] 将完整产品拆为阶段、原子任务、依赖和责任边界
- [x] 为每个任务定义输入、产出、验证步骤、通过标准和阻塞处理
- [x] 补充全局验证流程图、阶段门、回归策略和证据目录
- [x] 更新 README/文档总册导航
- [x] 验证计划编号、链接、Mermaid 围栏和与架构文档的一致性
- **Status:** complete

### Phase 7：项目术语与缩写表
- [x] 从权威文档提取编号、领域、协议、安全、扩展和发布术语
- [x] 明确 RQ/AC/EP/VAL/G/RC 的含义和追踪关系
- [x] 区分 Session/Run/Turn、Checkpoint/Snapshot 和两套 L1–L4
- [x] 新增统一术语表并接入 README、文档总册和领域文档导航
- [x] 校验链接、Markdown 围栏、必备术语和非文档变更边界
- **Status:** complete

### Phase 8：执行计划三端拆分与 TUI 优先化
- [x] 将 `S10` 拆分为 TUI、Desktop、Web 三个独立执行轨道
- [x] 将 TUI 测试 demo 作为 S10 的首个交付任务
- [x] 明确 Desktop/Web 只能在 TUI 轨道冻结后启动
- [x] 更新执行计划中的阶段门、验证流程与并行规则
- [x] 同步更新 progress.md 与 findings.md 的计划记录
- **Status:** complete

### Phase 9：执行计划一致性收尾
- [x] 解除共享前端安全规则对 Web 专属认证的隐式依赖
- [x] 将高层路线图中的客户端实施顺序与 TUI 优先化对齐
- [x] 重新检查 docs/16 与 docs/15 的一致性
- [x] 更新 progress.md 与 findings.md 记录本轮收尾
- **Status:** complete

### Phase 10：版本迭代执行计划
- [x] 分析 Reasonix/Pi 提交历史，提炼 P1–P10 迭代模式
- [x] 生成 `docs/17-version-iteration-execution-plan.md`（v0.1–v1.3 版本切片）
- [x] 登记新增 EP-1201–EP-1208、VAL-214–VAL-221
- **Status:** complete

### Phase 11：原子模块系分文档与 TUI 交互原型
- [ ] 更新 task_plan.md 与 progress.md
- [ ] 建立 `docs/design/` 目录、统一系分模板与索引 README
- [ ] 撰写 28 篇原子模块系分文档（M01–M28，按 v0.1 切片归属分层）
- [ ] 建立 `docs/prototype/` 目录与 TUI 交互原型 HTML（主界面 + 四个面板 + 审批/权限交互 + 关键弹层）
- [ ] 全局一致性校验：编号唯一、跨文档引用有效、Mermaid 围栏平衡
- **Status:** in_progress

## Phase 11 模块切分（原子粒度）

| 编号 | 模块 | 系分文档 | 版本归属 |
|---|---|---|---|
| M01 | 工程基座（workspace/CI/lint） | design/m01-engineering-foundation.md | v0.1 |
| M02 | 领域模型与契约（apex-domain/apex-ports） | design/m02-domain-contracts.md | v0.1 |
| M03 | Provider 层（Anthropic/OpenAI 首发） | design/m03-provider-layer.md | v0.1 |
| M04 | 会话存储（SQLite 事件源 + JSONL） | design/m04-session-storage.md | v0.1 |
| M05 | Agent Loop 与会话运行时 | design/m05-agent-loop-runtime.md | v0.1 |
| M06 | Spec 流水线引擎 | design/m06-spec-pipeline.md | v0.1 |
| M07 | 工具系统（Read/Write/Edit/Bash/Glob/Grep/Task） | design/m07-tool-system.md | v0.1 |
| M08 | 权限引擎（简化清单→AST） | design/m08-permission-engine.md | v0.1/v0.3 |
| M09 | TUI 应用框架（布局/组件/渲染） | design/m09-tui-framework.md | v0.1 |
| M10 | TUI 可观测面板 | design/m10-tui-observability-panel.md | v0.1→v0.4 |
| M11 | 上下文管理（Epoch/截断/Checkpoint） | design/m11-context-management.md | v0.1/v0.2 |
| M12 | 快照与文件回滚 | design/m12-snapshot-rollback.md | v0.2 |
| M13 | 持久终端（PTY/ConPTY） | design/m13-persistent-terminal.md | v0.2 |
| M14 | Prefix cache 与 token 预算 | design/m14-prefix-cache-token-budget.md | v0.2 |
| M15 | AST 命令语义分析 | design/m15-ast-command-analysis.md | v0.3 |
| M16 | 规范校验引擎（三层） | design/m16-rule-verification.md | v0.3 |
| M17 | Project Trust 与授权存储 | design/m17-trust-grant.md | v0.3 |
| M18 | Subagent 调度与写路径互斥 | design/m18-subagent-write-claim.md | v0.4 |
| M19 | 活动投影（ActivityView） | design/m19-activity-projection.md | v0.4 |
| M20 | Skills 系统 | design/m20-skills.md | v0.5 |
| M21 | MCP 集成 | design/m21-mcp-integration.md | v0.5 |
| M22 | Plugin 系统 | design/m22-plugin-system.md | v0.5/v0.9 |
| M23 | 记忆系统（FTS5 召回） | design/m23-memory-system.md | v0.6 |
| M24 | DAG 工作流引擎 | design/m24-dag-workflow.md | v0.7 |
| M25 | 确定性重放与补偿回滚 | design/m25-replay-compensation.md | v0.7 |
| M26 | Provider 扩展与多模态 | design/m26-provider-multimodal.md | v0.8 |
| M27 | 发布运维（安装/更新/日志/诊断） | design/m27-release-operations.md | v0.9 |
| M28 | 质量硬化（chaos/性能/安全审计） | design/m28-quality-hardening.md | v0.9 |

## 关键问题
1. 无阻塞问题；需求基线已由用户明确审核通过。

## 已做决策
| 决策 | 理由 |
|------|------|
| 采用 L4 完整设计流程 | 涉及跨端、存储、安全、多 Agent、扩展与恢复等系统级边界 |
| 旧文档完整归档后重建 | 用户选择保留历史，同时避免新旧权威文档混杂 |
| 当前代码不作为兼容约束 | 用户要求以全新目标架构为唯一基线 |
| 本轮只修改文档 | 用户明确禁止代码实现 |
| 不恢复当前已删除代码文件 | 这些删除是现有工作区状态，属于用户改动 |
| 旧文档归档目录为 `docs/archive/legacy-2026-08-11/` | 保留原文件名与完整历史，便于追溯 |

## 遇到的错误
| 错误 | 尝试次数 | 解决方案 |
|------|---------|---------|
| 一次 Ruby/Perl 批量校验在当前环境被信号 9 终止 | 2 | 改用 zsh、`awk` 与 `rg` 的轻量逐文件检查，链接、围栏和术语覆盖均通过 |

## 备注
- 重大设计决策前重新读取本计划和 findings.md。
- 外部参考实现只作为不可信资料来源，不覆盖用户确认的需求基线。
