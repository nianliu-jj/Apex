# Apex 架构文档总册

## 1. 文档目标

本套文档定义 Apex 完整产品的目标架构，可直接用于后续 Rust、Tauri/Vue/TypeScript 与跨平台工程实施。它取代 2026-08-11 以前的全部设计文档；旧文档仅用于历史追溯。

## 2. 权威层级

首次阅读请先查看[项目术语与缩写表](00-glossary.md)。术语表用于统一释义，不改变以下规范权威顺序。

| 层级 | 权威文档 | 只定义什么 | 禁止事项 |
|---|---|---|---|
| L1 需求 | [01-requirements](01-requirements.md) | 产品 What、边界、NFR、验收追踪 | 写具体 Rust 实现 |
| L2 架构/领域 | [02-system-architecture](02-system-architecture.md)、[04-domain-model](04-domain-model.md) | 边界、依赖方向、状态语义、事件语义 | 重复完整接口 |
| L3 契约 | [05-trait-contracts](05-trait-contracts.md)、[06-protocol-and-clients](06-protocol-and-clients.md) | Trait、DTO、Wire、错误模型与兼容规则 | 引入未登记领域状态 |
| L4 主题 | `07`–`15` | 流程、算法、不变量、运维与实施指南 | 复制并改写 L1–L3 定义 |
| 决策依据 | [ADR 注册表](adr/README.md) | 选项、选择、代价与触发重审条件 | 作为第二份规范事实源 |

跨文档引用使用稳定编号：需求 `RQ-xxx`、验收 `AC-xxx`、决策 `ADR-xxx`、风险 `RISK-xxx`。核心状态、ID、事件和错误码只允许在领域/契约层定义一次。

## 3. 阅读顺序

1. [项目术语与缩写表](00-glossary.md)：RQ、AC、RC、生命周期、协议和工程术语。
2. [需求基线](01-requirements.md)：115 项已确认要求和验收边界。
3. [系统总体架构](02-system-architecture.md)：组件、部署、数据流、信任边界和关键取舍。
4. [Cargo Workspace](03-workspace-and-crates.md)：Rust 工程布局、依赖方向和 crate 职责。
5. [领域模型](04-domain-model.md)：ID、聚合、状态机、领域事件和错误分类。
6. [Trait 契约](05-trait-contracts.md)：应用层 Port 与适配器边界。
7. [协议与客户端](06-protocol-and-clients.md)：本地 gRPC、REST/WebSocket 和三端一致性。
8. [存储、文件与日志](07-storage-files-logging.md)。
9. [Spec、Rules 与验证](08-spec-rules-verification.md)。
10. [Tool、权限与终端](09-tool-permission-terminal.md)。
11. [Context、Checkpoint 与 Memory](10-context-checkpoint-memory.md)。
12. [Agent、DAG、Snapshot 与重放](11-agent-dag-snapshot-replay.md)。
13. [Provider 与多模态](12-provider-multimodal.md)。
14. [Skills、MCP 与 Plugin](13-skills-mcp-plugins.md)。
15. [安装、升级与运维](14-install-upgrade-operations.md)。
16. [质量、风险与实施计划](15-quality-risks-roadmap.md)。
17. [原子化功能开发执行计划](16-implementation-execution-plan.md)：按 EP/VAL/G 阶段执行，当前只作为后续开发计划。
18. [版本迭代执行计划](17-version-iteration-execution-plan.md)：基于 Reasonix/Pi 提交史分析的发布序列视图，将文档 16 的分层任务重排为 v0.1–v1.3 版本切片；v1.0 及之前仅 TUI 端，Desktop/Web 顺延至 v1.1+。

## 4. 总体主题映射

| 主题 | 事实源 | 查询/投影 | 主要服务 |
|---|---|---|---|
| Spec/验证 | `specs/<feature>/*.md` 或多根 Workspace 中央目录 | SQLite 索引与审批投影 | Spec、Rule、Verification |
| Checkpoint | `checkpoint.md` + 内容寻址片段/附件 | SQLite 清单索引 | Context、Checkpoint |
| Memory | 项目 `.apex/memory/` 与全局 `~/.apex/memory/` | SQLite FTS5 | Memory |
| 会话运行态 | SQLite 事件 + 投影 | SQLite | Session、Agent、DAG、Tool |
| 诊断日志 | 会话 JSONL、系统文本日志 | 日志目录索引 | Observability |
| Snapshot | 内容寻址文件块与 Manifest | SQLite Snapshot 索引 | Snapshot、Replay |

## 5. 变更规则

- 需求变化先更新 `01-requirements.md`，再更新受影响的架构、契约和主题文档。
- 领域枚举、事件信封、Trait 或 Wire 变更必须记录兼容影响，并更新 ADR 注册表中的重审状态。
- 文档合并门至少执行：链接检查、Mermaid 围栏检查、编号唯一性、需求追踪完整性、路径/枚举/Trait 一致性检查。
- 执行计划中的 `EP`/`VAL` 编号只追加不重用；计划变更不得在主题文档中静默覆盖架构契约。
- 同一 Major 内不得删除、改名或改变已发布 Schema、事件、字段和错误码的既有语义。
- 归档文档不得被新文档反向引用为规范来源。

## 6. 非目标

- 不建设云端控制面、组织管理、Marketplace 或遥测平台。
- 不承诺跨 Provider 输出逐字确定性，不支持实时视频。
- 不以 Shadow Git 保存 Snapshot，也不要求用户 Git 工作区保持干净。
- 当前文档交付不包含代码、构建产物、数据库迁移或发布包。
