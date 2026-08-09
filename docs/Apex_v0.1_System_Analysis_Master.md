# Apex——v0.1 系统分析文档总册

> 文档状态：待审核
> 编制日期：2026-08-09
> 重要门禁：本总册及三份分卷审核通过前，禁止进入 Apex 代码开发；本轮仅撰写、审查和修正文档。

## 1. 文档目的
本套文档将 Apex v0.1 MVP 逐功能可运行阶段计划中的每个功能转化为可审核的系统分析规格。每项必须说明功能目标、范围边界、输入输出、处理流程、状态一致性、异常恢复、日志安全、测试验收、回滚方式和后续代码注释要求。

## 2. 文档组成
| 文档 | 覆盖范围 |
|---|---|
| 本总册 | 统一模板、开发门禁、横切约束、追踪矩阵和审核清单 |
| Apex_v0.1_System_Analysis_S00-S08.md | 日志、Workspace、TUI、Fake Application、Fake Chat、Approval Mock、IPC、SQLite |
| Apex_v0.1_System_Analysis_S09-S16.md | Project、Session、Provider、Agent Loop、Permission、Read、Write、Edit、Bash、Task |
| Apex_v0.1_System_Analysis_S17-S24.md | Spec、Rules、Context、Snapshot、Observability、集成和发布 |

覆盖统计：S00 日志 9 项，S01 至 S24 开发计划 173 项，合计 182 个可审核功能条目。

## 3. 基线与冲突优先级
需求分析文档是需求基线，系统总体架构设计是架构基线，领域模型与事件规范是领域基线，v0.1 MVP 逐功能计划是执行基线。发生冲突时，优先级为：用户确认约束、需求、总体架构和领域规范、领域详细设计、本套系统分析、代码。冲突必须先形成 ADR 或暂停编码。

## 4. 状态与开发门禁
状态依次为 planned、ready、coding、runnable、verified、done，也可以进入 blocked。planned 表示未审核，ready 表示允许编码，runnable 表示有真实入口，verified 表示代码、测试、日志、安全和文档均通过。当前所有新条目为 planned，允许进入代码开发为否。

## 5. 统一分析模板
每个功能条目必须具有九部分：功能目标、范围与边界、输入输出与处理流程、状态数据与一致性、异常取消与恢复、日志与安全、测试与可运行验收、后续代码注释要求、审核结论栏。任何一部分缺失都不得进入 ready。

## 6. 全局架构约束
TUI 只处理输入、状态投影和渲染；Application 和 Core 负责编排；Domain 维护聚合和不变量；Storage 维护事件和投影；Provider、Tool、Extension 通过 Port 接入。SQLite Event Store 是业务事实来源，日志只是诊断数据。Command 使用 command_id 幂等，副作用使用 operation_id，任务链使用 run_id，轮次使用 turn_id，跨组件因果使用 correlation_id。

## 7. 日志与诊断上下文
日志必须是一行文本，不使用 JSON，不使用 key=value。字段顺序为时间、级别、PID、任务链 UUID、线程或协程、logger、最后一级文件名和行号、主消息。不记录客户端地址。诊断上下文来源于 TelemetryContext 或 TaskContext，仅在 DEBUG 级别展示并放在主消息前，INFO 及以上隐藏。日志禁止包含 Secret、Prompt 原文、Provider 原文、完整工具参数和敏感文件正文。

## 8. 后续代码详细注释门禁
公共模块、类型和函数需要说明职责、输入约束、输出承诺和兼容性；状态机说明允许及拒绝的转换和不变量；异步任务、channel、锁和单写者说明并发模型、背压、取消传播和关闭顺序；迁移说明 schema、事务、重建和回滚；安全代码说明信任边界、默认拒绝、路径规范化和敏感数据处理；复杂算法说明选择原因、复杂度和边界；测试说明 fixture 意图和需求映射。禁止只翻译代码语句的空洞注释。

## 9. 功能追踪矩阵
| 阶段 | 功能数 | 主要交付 | 分卷 |
|---|---:|---|---|
| S00 | 9 | 日志和全阶段门禁 | S00-S08 |
| S01-S06 | 35 | TUI 最小闭环 | S00-S08 |
| S07-S08 | 16 | IPC 与 SQLite 恢复 | S00-S08 |
| S09-S11 | 22 | 生命周期、Provider、Agent | S09-S16 |
| S12-S16 | 35 | 权限和工具 | S09-S16 |
| S17-S19 | 24 | Spec 和 Rules | S17-S24 |
| S20-S22 | 24 | Context、Snapshot、Observability | S17-S24 |
| S23-S24 | 17 | 集成和发布 | S17-S24 |
| 合计 | 182 | 完整 v0.1 MVP | 三个分卷 |

## 10. 审核流程和清单
先审总册，再依次审 S00-S08、S09-S16、S17-S24。逐项确认目标、接口、状态、不变量、异常、日志和验收。只有条目达到 ready，且依赖 verified 或明确获准使用稳定 Port 或 Fake，才允许编码。缺少关键注释不得进入 runnable。

- [ ] 182 个编号与执行计划一致且无重复。
- [ ] 每项都有九部分分析。
- [ ] 没有把日志作为业务状态来源。
- [ ] 没有绕过 Port、Permission、Tool Gateway、Rules Gate 或 Snapshot。
- [ ] 所有副作用有 operation_id、取消边界和恢复结论。
- [ ] 敏感数据禁区明确。
- [ ] 文档审核通过前代码开发保持禁止。
