# Apex 功能开发原子化执行计划

> 生成日期：2026-08-14  
> 执行粒度：EP（Execution Plan Task）为最小可领取、可提交、可验证、可回滚单元  
> 路线范围：v0.1–v1.3  
> 状态依据：当前 Git 工作树、Cargo 清单、目录结构与提交历史；README 声明不作为实现完成证据

## 1. 文档定位与裁决规则

本计划将《Apex 原子模块系分文档》和《Apex 设计文档》转成可执行 EP 基线，并吸收 v0.2 两份历史计划中仍有效的编号与实施经验。源文档不在本次修改范围内；冲突通过本计划登记，后续由治理 EP 回写。

执行优先级：本计划的 EP 生命周期/迁移/依赖/版本裁决；原子模块系分文档的模块边界与失败语义；设计文档的全局架构与 RQ→AC→EP→VAL→Gate 链路；项目编码和 Git 规范；历史计划仅作背景。

### 1.1 EP 与 WI

- **EP**：计划与交付单元。一个 EP 只有一个主要交付物、一个主要所有者边界、一个可独立运行的 VAL，可独立提交和回滚。
- **WI**：EP 内部实现步骤。WI 不作为任务领取或版本关闭的最小单位。
- 若 WI 可独立失败、跨 crate、跨版本或可独立回滚，必须拆成不同 EP。

### 1.2 状态与工作量

- 实现状态：`已完成`、`部分完成`、`未开始`、`需重构`。
- 生命周期：`Active`、`Superseded`。Superseded 编号永久保留，不得复用。
- 工作量：`XS/S/M/L/XL`，仅表示相对复杂度，不对应人日。
- 代码存在不等于完成；必须同时满足定向测试、失败路径、追踪证据和独立复核。

## 2. 当前仓库审计

### 2.1 结论

- 当前仓库没有 `apps/` 和 `crates/` 目录，实际处于“文档与规则保留、早期代码原型已删除”的基线。
- 根 `Cargo.toml` 仍声明 `apps/apex-log-demo`、`crates/apex-observability` 两个已不存在成员，`cargo test --workspace` 在加载清单阶段失败。
- Git 历史显示提交 `d1dac67 refactor(observability): 移除早期日志原型实现` 删除早期实现，但根清单和 README 未同步。
- `rust-toolchain.toml` 固定 Rust `1.96.1`，`Cargo.toml` 的 `rust-version` 仍是 `1.85`，与编码规范不一致。
- README 仍将 workspace、observability 和 demo 标为已实现；这些陈述与工作树不符。
- 当前没有可认定为“已完成”的功能 EP。EP-0101 为需重构；治理类文档 EP 与工具链 EP 仅部分完成；其余未开始。

### 2.2 状态统计

| 实现状态 | EP 数 | 判定 |
|---|---:|---|
| 已完成 | 0 | 代码、测试、证据与门禁均存在 |
| 部分完成 | 9 | 只有设计/规则或部分配置 |
| 需重构 | 1 | 现有基线直接阻断构建或与规范冲突 |
| 未开始 | 246 | 当前工作树没有实现证据 |

### 2.3 首个执行阻塞

先完成 EP-0101、EP-0102、EP-0103：恢复可加载 workspace、统一 Rust 版本、建立最小 lint/test 基线。任何业务 crate 开发不得建立在当前损坏的 workspace 上。

## 3. 设计缺陷与重构裁决

| 缺陷 | 现象 | 风险 | 裁决 |
|---|---|---|---|
| D-01 | Workspace 与 README 残留引用已删除原型 | Cargo 全量验证阻断 | EP-0101 重建最小 workspace；EP-0102 统一 1.96.1；同步 README |
| D-02 | 模块编号混用 M-26/27/28 与 M-28/29/30 | 客户端依赖指向悬空模块 | 规范为 M-26 Desktop、M-27 Web、M-28 Trinity；EP-0011 迁移 |
| D-03 | 主 EP 注册表漏记 EP-1201–1208 | 追踪和统计得到不同全集 | 正式补录；EP-0002/0010 建立单一注册表 |
| D-04 | WI-v0.1-04b、WI-v0.3-11/12/13 重复 | WI→EP 追踪不唯一 | EP-0002 迁移重复 WI，旧 ID 只作别名 |
| D-05 | 部分 EP 跨独立交付物或版本 | 不能独立领取、验证、回滚 | 按第 4 节拆分 |
| D-06 | EP-0307 同时承担 Actor 与 Agent Loop | 并发模型与编排耦合 | EP-0307 收窄；新增 EP-0315 |
| D-07 | EP-0514 同时承担 Registry 与多类工具 | 权限/文件/Shell 风险耦合 | 新增 EP-0524/0525/0526 |
| D-08 | EP-0809 混合 Secret、脱敏与 AST 环境策略 | 跨领域跨版本 | 由 EP-0817/0818/0819 替代 |
| D-09 | 客户端 EP 跨 TUI/Desktop/Web | 无法按版本关闭 | 新增 EP-1028–1037；EP-1022 退役 |
| D-10 | 发布 EP 依赖未来版本 EP | v0.9/v1.0 Gate 不可达 | 修正依赖，拆分 Web/Trinity Gate |
| D-11 | EP-1116/1117 为多审计域大包 | 所有者与失败定位不清 | 由 EP-1123–1130 替代 |
| D-12 | 详细版本表只覆盖 v0.1–v1.0 | v1.1–v1.3 散落 | 本计划建立 v0.1–v1.3 单一视图 |

## 4. EP 编号迁移与拆分

### 4.1 Superseded 映射

| 旧 EP | 替代 EP | 原因 |
|---|---|---|
| EP-0809 | EP-0817, EP-0818, EP-0819 | 原 EP 跨独立安全域、平台或验证技术，不能作为原子执行单元 |
| EP-1022 | EP-1032, EP-1033 | 原 EP 跨独立安全域、平台或验证技术，不能作为原子执行单元 |
| EP-1116 | EP-1123, EP-1124, EP-1125, EP-1126 | 原 EP 跨独立安全域、平台或验证技术，不能作为原子执行单元 |
| EP-1117 | EP-1127, EP-1128, EP-1129, EP-1130 | 原 EP 跨独立安全域、平台或验证技术，不能作为原子执行单元 |

### 4.2 保留编号、收窄范围

| 保留 EP | 保留范围 | 移出 EP |
|---|---|---|
| EP-0307 | 仅保留 Session Actor、邮箱、状态转换与安全点；Agent Loop 移至 EP-0315。 | EP-0315 |
| EP-0514 | 仅保留 ToolRegistry、descriptor/schema 注册与查询；内置工具适配器移出。 | EP-0524, EP-0525, EP-0526 |
| EP-0603 | 仅保留 ContextEpoch 构建与预算计算；临时溢出降级移至 EP-0618。 | EP-0618 |
| EP-0707 | 仅保留通用全局/Agent/Provider/写入限流原语；DAG 接入移至 EP-0723。 | EP-0723 |
| EP-1001 | 仅保留 TUI 客户端壳、Daemon 连接、快照+增量归并与重连；窗口后端和拉起流程移出。 | EP-1028, EP-1029 |
| EP-1006 | 仅保留 Activity Projection/Panel；扩展管理移至 EP-1030。 | EP-1030 |
| EP-1008 | 仅保留 Checkpoint UI；Memory UI 移至 EP-1031。 | EP-1031 |
| EP-1025 | 仅保留共享/Desktop UI 安全边界；Web 专属回归移至 EP-1037。 | EP-1037 |
| EP-1026 | 仅保留共享 Vue 与 Desktop 单元/组件测试；Web 测试移至 EP-1036。 | EP-1036 |
| EP-1027 | 仅保留跨客户端领域状态哈希与同输入回放工具；能力矩阵与 Gate 关闭移出。 | EP-1034, EP-1035 |
| EP-1114 | 仅保留 Core/TUI 并发、限流、背压负载；WebSocket/Web 负载移至 EP-1132。 | EP-1131, EP-1132 |

### 4.3 新编号规则

- 在原领域号段尾部追加，不填历史空洞，不复用旧编号。
- 新增 33 个 EP：EP-0010、EP-0011、EP-0315、EP-0524–0526、EP-0618、EP-0723、EP-0817–0819、EP-1028–1037、EP-1121–1132。
- 新增 VAL 从 VAL-222 连续到 VAL-254。
- EP-1201–1208 是历史正式编号，本计划只补录。

## 5. 依赖图关键裁决

| EP | 原问题 | 修正 |
|---|---|---|
| EP-0305 | 被视为 v1.2 WebSocket 能力，却被 v0.1 TUI 依赖 | 提前为 v0.1 传输无关 Snapshot+since_seq 归并器 |
| EP-0314 | v0.1 优雅关闭依赖 v1.2 Control Lease | 依赖 EP-0307/0315 安全点 |
| EP-1005 | v0.1 权限 UI 依赖 v0.3 完整策略 | v0.1 依赖 EP-1201，v0.3 做兼容回归 |
| EP-1101–1103 | v0.9 TUI 构建依赖 v1.1 Tauri | 改依赖 TUI/Launcher/CLI；Desktop 由 EP-1121 |
| EP-1114 | v0.9 负载依赖 v1.2 WebSocket | 保留 Core/TUI；Web 移至 EP-1132 |
| EP-1117 | v0.9 Gate 依赖 v1.3 Trinity | 退役；EP-1129/1130 分别关闭 |
| EP-1026 | v1.1 测试隐含 v1.2 Web | 收窄 Shared/Desktop；Web 由 EP-1036 |

## 6. 版本路线与并行策略

| 版本 | Active EP 数 | 关闭目标 | 可并行主泳道 |
|---|---:|---|---|
| v0.1 | 84 | TUI、持久会话、Provider 流式对话、简化权限与核心工具闭环 | 治理/基础；存储；Daemon；Spec；工具权限；Provider；TUI |
| v0.2 | 22 | Checkpoint、内容快照、持久终端、上下文恢复 | 快照；Checkpoint；终端；Context；TUI 恢复 |
| v0.3 | 20 | 完整 AST 权限、Spec 验证与 Project Trust | AST/Policy；验证；Trust |
| v0.4 | 10 | Subagent、写路径治理与 Activity | Subagent；写入队列；Activity |
| v0.5 | 15 | Skills/MCP/Plugin 扩展闭环 | Skill；MCP；Plugin；扩展 UI |
| v0.6 | 8 | Memory 检索、冲突与管理 UI | Memory；Retrieval；Context；UI |
| v0.7 | 17 | DAG 调度、队列、取消与 Replay | DAG；Scheduler；Replay；TUI |
| v0.8 | 7 | 多 Provider、多模态与能力降级 | Provider；Multimodal；降级 |
| v0.9 | 31 | TUI RC 前发布、安全与质量 Gate | 构建；供应链；负载；审计；Gate |
| v1.0 | 3 | TUI 正式发布、回滚与 post-GA | RC；安装；回滚；post-GA |
| v1.1 | 13 | Desktop 与签名分发 | Shared UI；Desktop；Realtime；签名 |
| v1.2 | 18 | Web、Origin/Lease 与 Web 分发 | Web Server；Web UI；安全；负载；分发 |
| v1.3 | 4 | Trinity 等价性与跨客户端 Gate | 状态哈希；能力矩阵；Trinity E2E |

同一 EP 不拆给多人。不同泳道可并行，但只有前置 EP 的 VAL 证据进入主分支后才可开始。每个版本先合并契约/模型，再合并适配器，最后合并客户端与 Gate。

## 7. 模块执行索引

| 模块 | 规范名称 | Active EP 数 |
|---|---|---:|
| M-01 | 工程治理与契约 | 23 |
| M-02 | 存储、事件溯源与平台事实 | 13 |
| M-03 | Daemon、Session 与 Agent Loop | 6 |
| M-04 | Provider 核心 | 9 |
| M-05 | Spec Pipeline | 10 |
| M-06 | 工具注册与网关 | 7 |
| M-07 | 简化权限 | 2 |
| M-08 | Context Engine | 5 |
| M-09 | TUI 核心 | 11 |
| M-10 | TUI Spec/Permission | 2 |
| M-11 | Checkpoint | 10 |
| M-12 | 内容快照 | 3 |
| M-13 | 持久终端 | 6 |
| M-14 | AST 权限 | 13 |
| M-15 | 验证规则 | 5 |
| M-16 | Subagent 与写路径 | 7 |
| M-17 | 项目信任 | 2 |
| M-18 | Activity Panel | 3 |
| M-19a | Skills | 7 |
| M-19b | MCP | 6 |
| M-20 | Plugin | 4 |
| M-21 | Memory | 8 |
| M-22 | DAG | 12 |
| M-23 | Replay | 5 |
| M-24 | 多 Provider/多模态 | 7 |
| M-25a | 发布工程 | 22 |
| M-25b | 质量加固 | 10 |
| M-26 | Desktop | 13 |
| M-27 | Web | 17 |
| M-28 | Trinity | 4 |

## 8. EP 总表

| EP | 生命周期 | 版本 | 阶段 | 模块 | 实现状态 | 工作量 | 标题 |
|---|---|---|---|---|---|---|---|
| EP-0001 | Active | v0.1 | S0 | M-01 | 部分完成 | S | 固化 Feature Spec 模板 frontmatter |
| EP-0002 | Active | v0.1 | S0 | M-01 | 已完成 | S | 固化 `RQ`/`AC`/`EP`/`VAL` 编号规则 |
| EP-0003 | Active | v0.1 | S0 | M-01 | 部分完成 | M | 建立需求→AC→EP→验证追踪表 |
| EP-0004 | Active | v0.1 | S0 | M-01 | 部分完成 | M | 定义任务状态与阻塞原因 |
| EP-0005 | Active | v0.1 | S0 | M-01 | 部分完成 | M | 定义统一验证证据目录与命名 |
| EP-0006 | Active | v0.1 | S0 | M-01 | 未开始 | M | 建立代码、依赖、Schema、协议四类漂移检查 |
| EP-0007 | Active | v0.1 | S0 | M-01 | 部分完成 | M | 建立跨平台/Provider/客户端能力矩阵 fixture |
| EP-0008 | Active | v0.1 | S0 | M-01 | 部分完成 | M | 建立内存 Port、假时钟、故障注入 harness 设计 |
| EP-0009 | Active | v0.1 | S0 | M-01 | 未开始 | M | 建立封装访问器 derive 宏 crate（`Getters`/`Setters`/`Builder`/`Data`/`GettersExt`）与 CI pub 字段拦截 |
| EP-0010 | Active | v0.1 | S0 | M-01 | 未开始 | S | 实现执行计划完整性 Lint |
| EP-0011 | Active | v0.1 | S0 | M-01 | 未开始 | L | 统一模块编号、内部引用与文档锚点 |
| EP-0101 | Active | v0.1 | S1 | M-01 | 需重构 | M | 创建 workspace 根清单与成员列表 |
| EP-0102 | Active | v0.1 | S1 | M-01 | 部分完成 | M | 锁定 Rust toolchain 与 target 列表 |
| EP-0103 | Active | v0.1 | S1 | M-01 | 部分完成 | M | 配置 rustfmt/clippy/deny/audit 基线 |
| EP-0104 | Active | v0.1 | S1 | M-01 | 未开始 | M | 实现 UUIDv7/ContentHash/TraceId newtype |
| EP-0105 | Active | v0.1 | S1 | M-01 | 未开始 | M | 实现时间、generation、幂等 key 值对象 |
| EP-0106 | Active | v0.1 | S1 | M-01 | 未开始 | S | 实现唯一状态枚举及稳定字符串编码 |
| EP-0107 | Active | v0.1 | S1 | M-01 | 未开始 | M | 实现 `ApexError` 与稳定错误码 |
| EP-0108 | Active | v0.1 | S1 | M-01 | 未开始 | M | 实现 `EventEnvelope` 与 NewEvent |
| EP-0109 | Active | v0.1 | S1 | M-01 | 未开始 | M | 实现 `CommandContext`/Actor/Client identity |
| EP-0110 | Active | v0.1 | S1 | M-01 | 未开始 | S | 实现 `apex-ports` Trait 空实现编译边界 |
| EP-0111 | Active | v0.1 | S1 | M-01 | 未开始 | M | 生成 Protobuf Rust/TypeScript 类型 |
| EP-0112 | Active | v0.1 | S1 | M-01 | 未开始 | S | 建立 Rust 单元/属性测试公共 fixture |
| EP-0201 | Active | v0.1 | S2 | M-02 | 未开始 | M | 实现 Apex Home 路径解析 |
| EP-0202 | Active | v0.3 | S2 | M-14 | 未开始 | M | 实现 Home/config/key/runtime 权限诊断 |
| EP-0203 | Active | v0.1 | S2 | M-02 | 未开始 | M | 实现单实例 lock/mutex 与 stale 检查 |
| EP-0204 | Active | v0.1 | S2 | M-02 | 未开始 | M | 实现 Unix Domain Socket listener |
| EP-0205 | Active | v0.1 | S2 | M-02 | 未开始 | M | 实现 Windows Named Pipe listener |
| EP-0206 | Active | v0.2 | S2 | M-13 | 未开始 | M | 实现进程树 supervisor Port |
| EP-0207 | Active | v0.1 | S2 | M-02 | 未开始 | M | 实现普通配置加载与未知字段保留 |
| EP-0208 | Active | v0.1 | S2 | M-02 | 未开始 | M | 实现 SQLite 打开、WAL 与 busy 策略 |
| EP-0209 | Active | v0.1 | S2 | M-02 | 未开始 | S | 实现 schema_meta/feature/migration 表 |
| EP-0210 | Active | v0.1 | S2 | M-02 | 未开始 | M | 实现 EventStore append 事务 |
| EP-0211 | Active | v0.1 | S2 | M-02 | 未开始 | M | 实现 session sequence 与 aggregate version |
| EP-0212 | Active | v0.1 | S2 | M-02 | 未开始 | M | 实现 projector cursor 与投影批处理 |
| EP-0213 | Active | v0.1 | S2 | M-02 | 未开始 | M | 实现 Query Snapshot/keyset pagination |
| EP-0214 | Active | v0.1 | S2 | M-02 | 未开始 | M | 实现 Markdown 原子写/文件 generation |
| EP-0215 | Active | v0.6 | S2 | M-21 | 未开始 | M | 实现 watcher 防抖与自写去重 |
| EP-0216 | Active | v0.6 | S2 | M-21 | 未开始 | M | 实现 Markdown AST 三方合并 |
| EP-0217 | Active | v0.2 | S2 | M-12 | 未开始 | M | 实现 CAS put/open/verify |
| EP-0218 | Active | v0.2 | S2 | M-12 | 未开始 | M | 实现文件事实索引与 reconcile marker |
| EP-0219 | Active | v0.1 | S2 | M-02 | 未开始 | M | 实现 Session JSONL sink 与 10 MiB 轮转 |
| EP-0220 | Active | v0.9 | S2 | M-25a | 未开始 | M | 实现每日系统文本日志与 60 天清理 |
| EP-0221 | Active | v0.9 | S2 | M-25a | 未开始 | M | 实现日志 Ed25519 seal/verify/key rotation |
| EP-0222 | Active | v0.2 | S2 | M-25a | 未开始 | M | 实现 120/365 天 Session 归档与只读挂载 |
| EP-0223 | Active | v0.9 | S2 | M-25a | 未开始 | M | 实现升级/恢复前 SQLite+文件备份 |
| EP-0301 | Active | v0.1 | S3 | M-03 | 未开始 | M | 实现 ClientHello/ServerHello 版本协商 |
| EP-0302 | Active | v0.1 | S3 | M-03 | 未开始 | M | 实现 gRPC interceptor 身份/trace/idempotency |
| EP-0303 | Active | v1.2 | S3 | M-27 | 未开始 | M | 实现 REST DTO 到 Application Command 映射 |
| EP-0304 | Active | v1.2 | S3 | M-27 | 未开始 | M | 实现 WebSocket Subscribe/Close/错误帧 |
| EP-0305 | Active | v0.1 | S3 | M-09 | 未开始 | M | 实现 Snapshot + since_seq 合并器 |
| EP-0306 | Active | v0.1 | S3 | M-03 | 未开始 | M | 实现 durable prompt inbox |
| EP-0307 | Active | v0.1 | S3 | M-03 | 未开始 | M | 实现 Session Actor 串行提升 Turn |
| EP-0308 | Active | v1.2 | S3 | M-27 | 未开始 | M | 实现控制租约 acquire/renew/release |
| EP-0309 | Active | v1.2 | S3 | M-27 | 未开始 | M | 实现 force takeover 与旧 token fencing |
| EP-0310 | Active | v1.2 | S3 | M-27 | 未开始 | M | 实现 TUI 自动 Web enable lease |
| EP-0311 | Active | v1.2 | S3 | M-27 | 未开始 | M | 实现一次性 token exchange 与短 Cookie |
| EP-0312 | Active | v1.2 | S3 | M-27 | 未开始 | M | 实现 Origin/CSRF/CSP 校验 |
| EP-0313 | Active | v0.4 | S3 | M-18 | 未开始 | M | 实现 AgentActivityView durable/transient 投影 |
| EP-0314 | Active | v0.1 | S3 | M-03 | 未开始 | M | 实现 graceful shutdown/drain |
| EP-0315 | Active | v0.1 | S3 | M-03 | 未开始 | L | 实现 Agent Loop 编排器 |
| EP-0401 | Active | v0.1 | S4 | M-05 | 未开始 | S | 实现 requirements.md schema/parser |
| EP-0402 | Active | v0.1 | S4 | M-05 | 未开始 | S | 实现 design.md schema/parser |
| EP-0403 | Active | v0.1 | S4 | M-05 | 未开始 | S | 实现 tasks.md schema/parser |
| EP-0404 | Active | v0.1 | S4 | M-05 | 未开始 | S | 实现 verification.md renderer/schema |
| EP-0405 | Active | v0.1 | S4 | M-05 | 未开始 | M | 实现 SpecStage/StageStatus 状态机 |
| EP-0406 | Active | v0.1 | S4 | M-05 | 未开始 | M | 实现 ApprovalRecord 内容 hash 绑定 |
| EP-0407 | Active | v0.1 | S4 | M-05 | 未开始 | M | 实现上游变化失效传播图 |
| EP-0408 | Active | v0.1 | S4 | M-05 | 未开始 | M | 实现 `/skip-spec` parser 与 scope 校验 |
| EP-0409 | Active | v0.1 | S4 | M-05 | 未开始 | M | 实现 SkipGrant 审计事件与限制 |
| EP-0410 | Active | v0.1 | S4 | M-05 | 未开始 | S | 实现规则 profile registry/version hash |
| EP-0411 | Active | v0.3 | S4 | M-15 | 未开始 | M | 实现 PostToolUse 轻量安全/格式/语法门 |
| EP-0412 | Active | v0.3 | S4 | M-15 | 未开始 | M | 实现增量批次重型检查编排 |
| EP-0413 | Active | v0.3 | S4 | M-15 | 未开始 | M | 实现受限自动修复子任务 |
| EP-0414 | Active | v0.3 | S4 | M-15 | 未开始 | M | 实现最终 Verification evidence 聚合 |
| EP-0415 | Active | v0.3 | S4 | M-15 | 未开始 | M | 实现用户确认/自动完成策略 |
| EP-0501 | Active | v0.3 | S5 | M-14 | 未开始 | M | 定义 CommandAst→CommandSemantics IR |
| EP-0502 | Active | v0.3 | S5 | M-14 | 未开始 | M | 实现 sh/bash/zsh tree-sitter parser |
| EP-0503 | Active | v0.3 | S5 | M-14 | 未开始 | M | 实现 PowerShell 7 parser adapter |
| EP-0504 | Active | v0.3 | S5 | M-14 | 未开始 | M | 实现 cmd.exe parser adapter |
| EP-0505 | Active | v0.3 | S5 | M-14 | 未开始 | M | 实现 arity rule registry |
| EP-0506 | Active | v0.3 | S5 | M-14 | 未开始 | M | 实现路径 canonicalization 与 Scope overlap |
| EP-0507 | Active | v0.3 | S5 | M-14 | 未开始 | M | 实现网络目标规范化与重定向复核 |
| EP-0508 | Active | v0.3 | S5 | M-14 | 未开始 | M | 实现环境/凭据访问分类与清洗 |
| EP-0509 | Active | v0.3 | S5 | M-14 | 未开始 | M | 实现 Trust→Mode→HardDeny 单调决策顺序 |
| EP-0510 | Active | v0.3 | S5 | M-14 | 未开始 | M | 实现 plan/ask/allow 模式矩阵 |
| EP-0511 | Active | v0.3 | S5 | M-17 | 未开始 | M | 实现 Once/Run/Session/Project grant 存储 |
| EP-0512 | Active | v0.3 | S5 | M-17 | 未开始 | M | 实现 Project Trust Gate |
| EP-0513 | Active | v0.1 | S5 | M-07 | 未开始 | M | 实现 PermissionVerdict evidence/audit |
| EP-0514 | Active | v0.1 | S5 | M-06 | 未开始 | S | 实现 Tool descriptor/schema/副作用声明 |
| EP-0515 | Active | v0.1 | S5 | M-06 | 未开始 | M | 实现 Tool Gateway prepare→gate→execute pipeline |
| EP-0516 | Active | v0.1 | S5 | M-06 | 未开始 | M | 实现 Tool result bounded output/receipt |
| EP-0517 | Active | v0.2 | S5 | M-13 | 未开始 | M | 实现 Unix PTY 持久终端 |
| EP-0518 | Active | v0.2 | S5 | M-13 | 未开始 | M | 实现 Windows ConPTY 持久终端 |
| EP-0519 | Active | v0.1 | S5 | M-06 | 未开始 | M | 实现一次性非交互命令模式 |
| EP-0520 | Active | v0.2 | S5 | M-13 | 未开始 | M | 实现共享逻辑终端与 Agent channel attribution |
| EP-0521 | Active | v0.2 | S5 | M-13 | 未开始 | M | 实现终端输出 ring buffer/backpressure |
| EP-0522 | Active | v0.2 | S5 | M-13 | 未开始 | M | 实现中断 Tool recovery 分类 |
| EP-0523 | Active | v0.3 | S5 | M-14 | 未开始 | M | 接入可选 OS sandbox capability |
| EP-0524 | Active | v0.1 | S5 | M-06 | 未开始 | M | 实现 Read 与 Search 内置工具适配器 |
| EP-0525 | Active | v0.1 | S5 | M-06 | 未开始 | M | 实现 Edit 与 Patch 内置工具适配器 |
| EP-0526 | Active | v0.1 | S5 | M-06 | 未开始 | M | 实现 Shell 工具准备适配器 |
| EP-0601 | Active | v0.1 | S6 | M-08 | 未开始 | M | 实现 Provider-aware token estimator |
| EP-0602 | Active | v0.1 | S6 | M-08 | 未开始 | M | 实现 Stable/Turn/Retrieved/Recovery Source |
| EP-0603 | Active | v0.1 | S6 | M-08 | 未开始 | M | 实现 ContextEpoch 构建与替换 |
| EP-0604 | Active | v0.2 | S6 | M-11 | 未开始 | M | 实现 60/70/80/90 watermark 状态 |
| EP-0605 | Active | v0.2 | S6 | M-11 | 未开始 | M | 实现 Tool-specific SnipHinter |
| EP-0606 | Active | v0.2 | S6 | M-11 | 未开始 | M | 实现 prune 引用占位与再取回 |
| EP-0607 | Active | v0.2 | S6 | M-11 | 未开始 | M | 实现独立摘要 Provider 与当前模型 fallback |
| EP-0608 | Active | v0.2 | S6 | M-11 | 未开始 | M | 实现 Checkpoint Manifest schema |
| EP-0609 | Active | v0.2 | S6 | M-11 | 未开始 | M | 实现 Checkpoint chunk/attachment CAS writer |
| EP-0610 | Active | v0.2 | S6 | M-11 | 未开始 | M | 接入 Turn/损处理/暂停/高风险写触发点 |
| EP-0611 | Active | v0.2 | S6 | M-11 | 未开始 | M | 实现 Checkpoint reconstruction |
| EP-0612 | Active | v0.2 | S6 | M-11 | 未开始 | M | 实现 Checkpoint pin/120/365 retention |
| EP-0613 | Active | v0.6 | S6 | M-21 | 未开始 | M | 实现 Memory Markdown parser/writer |
| EP-0614 | Active | v0.6 | S6 | M-21 | 未开始 | M | 实现 Memory sensitive proposal gate |
| EP-0615 | Active | v0.6 | S6 | M-21 | 未开始 | M | 实现 FTS5 unicode61/jieba tokenizer adapter |
| EP-0616 | Active | v0.6 | S6 | M-21 | 未开始 | M | 实现召回排序、引用时机与 trace 记录 |
| EP-0617 | Active | v0.6 | S6 | M-21 | 未开始 | M | 实现 Memory delete/export/tombstone |
| EP-0618 | Active | v0.1 | S6 | M-08 | 未开始 | M | 实现临时上下文溢出降级策略 |
| EP-0701 | Active | v0.4 | S7 | M-16 | 未开始 | M | 实现 AgentProfile 与 capability ceiling |
| EP-0702 | Active | v0.4 | S7 | M-16 | 未开始 | M | 实现父 Agent→Subagent Provider/model 继承 |
| EP-0703 | Active | v0.4 | S7 | M-16 | 未开始 | M | 实现 exact_task_description/write_paths 校验 |
| EP-0704 | Active | v0.7 | S7 | M-22 | 未开始 | S | 实现 workflow YAML schema |
| EP-0705 | Active | v0.7 | S7 | M-22 | 未开始 | M | 实现 tasks.md→VersionedDagIr 编译 |
| EP-0706 | Active | v0.7 | S7 | M-22 | 未开始 | M | 实现 Ready Queue 稳定排序 |
| EP-0707 | Active | v0.4 | S7 | M-16 | 未开始 | M | 实现全局/写 Agent/Provider 限流 |
| EP-0708 | Active | v0.4 | S7 | M-16 | 未开始 | M | 将 CanonicalPathScope 接入 Scheduler |
| EP-0709 | Active | v0.4 | S7 | M-16 | 未开始 | M | 实现 Claim lease TTL/fencing |
| EP-0710 | Active | v0.4 | S7 | M-16 | 未开始 | M | 实现父 Agent write_paths 预留 |
| EP-0711 | Active | v0.7 | S7 | M-22 | 未开始 | M | 实现路径扩展暂停/重新审批 |
| EP-0712 | Active | v0.7 | S7 | M-22 | 未开始 | M | 实现 DAG 显式 mailbox edge |
| EP-0713 | Active | v0.7 | S7 | M-22 | 未开始 | M | 实现父 Agent 结构化汇聚 |
| EP-0714 | Active | v0.7 | S7 | M-22 | 未开始 | M | 实现受限 Merge Subagent 三方合并 |
| EP-0715 | Active | v0.7 | S7 | M-22 | 未开始 | M | 实现 Node 状态 reducer |
| EP-0716 | Active | v0.7 | S7 | M-22 | 未开始 | M | 实现 DAG pause/resume 安全点 |
| EP-0717 | Active | v0.7 | S7 | M-22 | 未开始 | M | 实现崩溃恢复幂等分类 |
| EP-0718 | Active | v0.7 | S7 | M-23 | 未开始 | M | 将 Snapshot 接入 Tool/Node pre-write |
| EP-0719 | Active | v0.7 | S7 | M-23 | 未开始 | M | 实现状态确定性重放 executor |
| EP-0720 | Active | v0.7 | S7 | M-23 | 未开始 | M | 实现再执行重放副作用清单与整体确认 |
| EP-0721 | Active | v0.7 | S7 | M-23 | 未开始 | M | 实现补偿式部分回滚 |
| EP-0722 | Active | v0.7 | S7 | M-23 | 未开始 | M | 记录调度决定/limit snapshot/ready hash |
| EP-0723 | Active | v0.7 | S7 | M-22 | 未开始 | L | 将限流器接入 DAG 调度器 |
| EP-0801 | Active | v0.1 | S8 | M-04 | 未开始 | M | 实现 Provider Core ModelRequest/Frame |
| EP-0802 | Active | v0.1 | S8 | M-04 | 未开始 | S | 实现 capability schema/negotiation |
| EP-0803 | Active | v0.1 | S8 | M-04 | 未开始 | M | 实现 Anthropic adapter |
| EP-0804 | Active | v0.1 | S8 | M-04 | 未开始 | M | 实现 OpenAI adapter |
| EP-0805 | Active | v0.8 | S8 | M-24 | 未开始 | M | 实现 DeepSeek adapter |
| EP-0806 | Active | v0.8 | S8 | M-24 | 未开始 | M | 实现 Kimi adapter |
| EP-0807 | Active | v0.8 | S8 | M-24 | 未开始 | M | 实现 OpenAI-Compatible adapter |
| EP-0808 | Active | v0.1 | S8 | M-04 | 未开始 | M | 实现 providers.toml profile parser |
| EP-0809 | Superseded | v0.1 | S8 | M-04 | 未开始 | M | 实现 SecretResolver 与 Provider Secret Firewall |
| EP-0810 | Active | v0.4 | S8 | M-04 | 未开始 | M | 接入 Session/Profile/DAG Provider 继承 |
| EP-0811 | Active | v0.8 | S8 | M-24 | 未开始 | M | 实现默认关闭的 failover chain |
| EP-0812 | Active | v0.1 | S8 | M-04 | 未开始 | M | 实现 retry/backoff/deadline/cancel |
| EP-0813 | Active | v0.8 | S8 | M-24 | 未开始 | M | 实现 Artifact MIME/大小/转码 Port |
| EP-0814 | Active | v1.1 | S8 | M-26 | 未开始 | M | 实现 Desktop/Web audio 与双向语音 Port |
| EP-0815 | Active | v0.8 | S8 | M-24 | 未开始 | M | 实现视频文件抽取与实时视频硬禁 |
| EP-0816 | Active | v0.8 | S8 | M-24 | 未开始 | M | 建立各 Adapter contract fixture/脱敏回放 |
| EP-0817 | Active | v0.1 | S8 | M-04 | 未开始 | M | 实现 SecretResolver 凭据解析接口 |
| EP-0818 | Active | v0.1 | S8 | M-04 | 未开始 | M | 实现 Provider Secret Firewall 与日志脱敏 |
| EP-0819 | Active | v0.3 | S5 | M-14 | 未开始 | M | 接入 AST 环境变量与秘密策略 |
| EP-0901 | Active | v0.5 | S9 | M-19a | 未开始 | S | 实现 SkillSource/Scanner Trait |
| EP-0902 | Active | v0.5 | S9 | M-19a | 未开始 | M | 实现 Claude user/project 扫描器 |
| EP-0903 | Active | v0.5 | S9 | M-19a | 未开始 | M | 实现 Codex user/project 扫描器 |
| EP-0904 | Active | v0.5 | S9 | M-19a | 未开始 | M | 实现 Apex user/project 扫描器 |
| EP-0905 | Active | v0.5 | S9 | M-19a | 未开始 | S | 实现 `apex:` frontmatter 阶段绑定 |
| EP-0906 | Active | v0.5 | S9 | M-19a | 未开始 | M | 实现 Skill content hash/signature trust |
| EP-0907 | Active | v0.5 | S9 | M-19a | 未开始 | M | 将 Skill script/Tool 绑定 Tool Gateway |
| EP-0908 | Active | v0.5 | S9 | M-19b | 未开始 | M | 实现 McpSource/Config adapter Trait |
| EP-0909 | Active | v0.5 | S9 | M-19b | 未开始 | M | 实现 Claude/Cursor/VS Code/Codex/Apex scanner |
| EP-0910 | Active | v0.5 | S9 | M-19b | 未开始 | M | 实现 MCP fingerprint/provenance 合并 |
| EP-0911 | Active | v0.5 | S9 | M-19b | 未开始 | M | 实现 Apex enable override 与显式来源同步 |
| EP-0912 | Active | v0.5 | S9 | M-19b | 未开始 | M | 实现 MCP start/stop/stdio 进程树生命周期 |
| EP-0913 | Active | v0.5 | S9 | M-19b | 未开始 | M | 实现 MCP OAuth state/PKCE/loopback |
| EP-0914 | Active | v0.5 | S9 | M-20 | 未开始 | M | 实现 Plugin C ABI manifest/capability |
| EP-0915 | Active | v0.9 | S9 | M-20 | 未开始 | M | 实现第三方 Plugin Host RPC/supervisor |
| EP-0916 | Active | v0.9 | S9 | M-20 | 未开始 | M | 实现官方签名进程内 allowlist |
| EP-0917 | Active | v0.9 | S9 | M-20 | 未开始 | M | 实现本地/Git/文件包安装与安全解包 |
| EP-1001 | Active | v0.1 | S10 | M-09 | 未开始 | M | 建立 TUI 测试 demo 与连接/重连骨架 |
| EP-1002 | Active | v0.1 | S10 | M-09 | 未开始 | M | 实现 TUI Workspace/Session 列表 |
| EP-1003 | Active | v0.1 | S10 | M-09 | 未开始 | M | 实现 TUI Prompt/Admission/Turn 视图 |
| EP-1004 | Active | v0.1 | S10 | M-10 | 未开始 | M | 实现 TUI Spec/Approval/Skip 面板 |
| EP-1005 | Active | v0.1 | S10 | M-10 | 未开始 | M | 实现 TUI Permission Ask/Allow/Deny UI |
| EP-1006 | Active | v0.4 | S10 | M-18 | 未开始 | M | 实现 TUI Agent/Skill/MCP/Subagent 活动面板 |
| EP-1007 | Active | v0.7 | S10 | M-22 | 未开始 | M | 实现 TUI DAG/Claim/Pause/Resume UI |
| EP-1008 | Active | v0.2 | S10 | M-11 | 未开始 | M | 实现 TUI Checkpoint/Memory UI |
| EP-1009 | Active | v0.2 | S10 | M-09 | 未开始 | M | 实现 TUI 共享逻辑终端 UI |
| EP-1010 | Active | v1.2 | S10 | M-27 | 未开始 | M | 实现 TUI 自动 Web lease lifecycle |
| EP-1011 | Active | v1.1 | S10 | M-26 | 未开始 | M | 建立 Vue domain stores/reducers |
| EP-1012 | Active | v1.1 | S10 | M-26 | 未开始 | M | 实现共享 Platform Adapter interface |
| EP-1013 | Active | v1.1 | S10 | M-26 | 未开始 | M | 实现 Tauri gRPC bridge |
| EP-1014 | Active | v1.2 | S10 | M-27 | 未开始 | M | 实现 Web auth bootstrap/token cleanup |
| EP-1015 | Active | v1.1 | S10 | M-26 | 未开始 | M | 实现共享 Session/Turn/Spec 页面 |
| EP-1016 | Active | v1.2 | S10 | M-27 | 未开始 | M | 实现 Web Permission/Control takeover 页面 |
| EP-1017 | Active | v1.2 | S10 | M-27 | 未开始 | M | 实现 Web Agent/DAG/Activity 页面 |
| EP-1018 | Active | v1.1 | S10 | M-26 | 未开始 | M | 实现 Desktop/Web Checkpoint/Memory 页面 |
| EP-1019 | Active | v1.1 | S10 | M-26 | 未开始 | M | 实现三端 Session/System Log 页面 |
| EP-1020 | Active | v1.1 | S10 | M-26 | 未开始 | M | 实现 Desktop audio/file picker |
| EP-1021 | Active | v1.2 | S10 | M-27 | 未开始 | M | 实现 Web audio/file upload |
| EP-1022 | Superseded | v1.1 | S10 | M-26 | 未开始 | M | 实现 Desktop/Web 视频文件引用 |
| EP-1023 | Active | v0.9 | S10 | M-09 | 未开始 | M | 完成中文/英文 message key 覆盖 |
| EP-1024 | Active | v1.1 | S10 | M-26 | 未开始 | M | 完成键盘/屏幕阅读器/颜色无关状态 |
| EP-1025 | Active | v1.1 | S10 | M-26 | 未开始 | S | 完成 Vue XSS/CSRF/URL/Secret 安全规则 |
| EP-1026 | Active | v1.1 | S10 | M-26 | 未开始 | M | 添加 TUI/Vue/Platform 单元组件测试 |
| EP-1027 | Active | v1.3 | S10 | M-28 | 未开始 | M | 添加三端等价性 E2E harness |
| EP-1028 | Active | v0.1 | S10 | M-09 | 未开始 | M | 实现原生 TUI 窗口与像素后端 |
| EP-1029 | Active | v0.1 | S10 | M-09 | 未开始 | M | 实现项目选择、Daemon 拉起与 endpoint-ready 协议 |
| EP-1030 | Active | v0.5 | S10 | M-18 | 未开始 | M | 实现扩展管理面板 |
| EP-1031 | Active | v0.6 | S10 | M-21 | 未开始 | M | 实现 Memory 管理 UI |
| EP-1032 | Active | v1.1 | S10 | M-26 | 未开始 | M | 实现 Desktop 视频文件引用界面 |
| EP-1033 | Active | v1.2 | S10 | M-27 | 未开始 | M | 实现 Web 视频文件引用界面 |
| EP-1034 | Active | v1.3 | S10 | M-28 | 未开始 | L | 实现 Trinity 能力矩阵验证器 |
| EP-1035 | Active | v1.3 | S10 | M-28 | 未开始 | L | 关闭 Trinity 等价性 Gate 与证据包 |
| EP-1036 | Active | v1.2 | S10 | M-27 | 未开始 | M | 建立 Web 客户端单元与组件测试套件 |
| EP-1037 | Active | v1.2 | S10 | M-27 | 未开始 | M | 建立 Web UI 安全回归套件 |
| EP-1101 | Active | v0.9 | S11 | M-25a | 未开始 | M | 建立 macOS x86/arm 构建流水线 |
| EP-1102 | Active | v0.9 | S11 | M-25a | 未开始 | M | 建立 Windows x86/arm 构建流水线 |
| EP-1103 | Active | v0.9 | S11 | M-25a | 未开始 | M | 建立 Linux x86/arm 构建流水线 |
| EP-1104 | Active | v0.9 | S11 | M-25a | 未开始 | M | 实现安装/卸载/用户数据保留 |
| EP-1105 | Active | v0.9 | S11 | M-25a | 未开始 | M | 实现 signed update manifest 与 SBOM |
| EP-1106 | Active | v0.9 | S11 | M-25a | 未开始 | M | 实现 Stable/Nightly/Development/Enterprise policy |
| EP-1107 | Active | v0.9 | S11 | M-25a | 未开始 | M | 实现 apex-updater 安全点替换/回滚 |
| EP-1108 | Active | v0.9 | S11 | M-25a | 未开始 | S | 完成同 Major old/new schema fixture |
| EP-1109 | Active | v0.9 | S11 | M-25a | 未开始 | M | 完成迁移中断/恢复/备份恢复演练 |
| EP-1110 | Active | v0.9 | S11 | M-25a | 未开始 | M | 完成 60/120/365 retention scheduler |
| EP-1111 | Active | v0.9 | S11 | M-25a | 未开始 | M | 完成 `apexd doctor --read-only` |
| EP-1112 | Active | v0.9 | S11 | M-25a | 未开始 | M | 完成无遥测网络基线与诊断包 |
| EP-1113 | Active | v0.9 | S11 | M-25a | 未开始 | M | 建立启动/Admission/Event/Page/FTS/RSS baseline |
| EP-1114 | Active | v0.9 | S11 | M-25a | 未开始 | S | 建立并发/限流/背压压力场景 |
| EP-1115 | Active | v0.9 | S11 | M-25a | 未开始 | M | 建立 DB/文件/Tool/DAG/Provider chaos 场景 |
| EP-1116 | Superseded | v0.9 | S11 | M-25b | 未开始 | L | 完成 AST/path/network/Secret/Plugin/Web 安全审计 |
| EP-1117 | Superseded | v0.9 | S11 | M-25b | 未开始 | M | 完成覆盖率、mutation、fuzz、E2E 门 |
| EP-1118 | Active | v1.0 | S11 | M-25b | 未开始 | M | 生成各 Feature 最终 verification.md |
| EP-1119 | Active | v1.0 | S11 | M-25b | 未开始 | M | 生成 Release Candidate 与完整回滚包 |
| EP-1120 | Active | v1.0 | S11 | M-25b | 未开始 | M | 执行独立发布评审并封存证据 |
| EP-1121 | Active | v1.1 | S11 | M-26 | 未开始 | M | 建立 Desktop 签名与安装包流水线 |
| EP-1122 | Active | v1.2 | S11 | M-27 | 未开始 | M | 建立 Web 静态资源与嵌入式分发流水线 |
| EP-1123 | Active | v0.9 | S11 | M-25b | 未开始 | L | 执行 AST、路径与终端安全审计 |
| EP-1124 | Active | v0.9 | S11 | M-25b | 未开始 | L | 执行 Secret 与持久化数据安全审计 |
| EP-1125 | Active | v0.9 | S11 | M-25b | 未开始 | L | 执行扩展与供应链安全审计 |
| EP-1126 | Active | v1.2 | S11 | M-25b | 未开始 | L | 执行 Web 与客户端边界安全审计 |
| EP-1127 | Active | v0.9 | S11 | M-25b | 未开始 | M | 建立覆盖率与变异测试 Gate |
| EP-1128 | Active | v0.9 | S11 | M-25b | 未开始 | M | 建立 Fuzz 与属性测试 Gate |
| EP-1129 | Active | v0.9 | S11 | M-25b | 未开始 | L | 建立 TUI v1.0 端到端发布 Gate |
| EP-1130 | Active | v1.3 | S11 | M-28 | 未开始 | L | 建立 Trinity 跨客户端端到端 Gate |
| EP-1131 | Active | v0.9 | S11 | M-25a | 未开始 | M | 建立 Core 与 TUI 负载及背压夹具 |
| EP-1132 | Active | v1.2 | S11 | M-27 | 未开始 | M | 建立 WebSocket 与 Web 负载夹具 |
| EP-1201 | Active | v0.1 | S5 | M-07 | 未开始 | M | 实现 v0.1 简化权限策略并预留 v0.3 迁移接口 |
| EP-1202 | Active | v0.2 | S2 | M-12 | 未开始 | M | 实现 Turn 边界内容快照与恢复校验 |
| EP-1203 | Active | v0.1 | S10 | M-09 | 未开始 | M | 实现 TUI Markdown 与代码高亮渲染 |
| EP-1204 | Active | v0.1 | S10 | M-09 | 未开始 | M | 实现 TUI 流式输出、打断与状态反馈 |
| EP-1205 | Active | v0.1 | S10 | M-09 | 未开始 | M | 实现 CLI 启动、首次运行与会话恢复入口 |
| EP-1206 | Active | v0.2 | S6 | M-08 | 未开始 | M | 补齐前缀缓存固定与失效测试 |
| EP-1207 | Active | v0.9 | S11 | M-25a | 未开始 | M | 建立 Changelog 完整性 CI |
| EP-1208 | Active | v0.9 | S11 | M-25a | 未开始 | M | 建立 design-before-code 门禁 |

## 9. Active EP 完整执行卡

以下卡片是领取任务时的最小契约。若必须扩大范围，应先新增/修订 EP，不得把额外工作静默塞入当前 EP。

### 9.1 v0.1

#### M-01 工程治理与契约

##### EP-0001 固化 Feature Spec 模板 frontmatter

- **生命周期 / 状态 / 工作量**：Active / 部分完成 / S
- **阶段 / 版本 / 模块**：S0 / v0.1 / M-01 工程治理与契约
- **需求追踪**：RQ-036–041；WI：WI-v0.1-04
- **目标**：交付 四文档模板与 schema，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“固化 Feature Spec 模板 frontmatter”所需的最小闭环；主交付物为：四文档模板与 schema。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：无。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`Cargo.toml`、`rust-toolchain.toml`、`specs/`、`schemas/`、`xtask/`、`.github/workflows/`、`docs/`、`rules/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：四文档模板与 schema；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-01`：schema 正/负 fixture；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`docs(governance): 固化 Feature Spec 模板 frontmatter`

##### EP-0002 固化 `RQ`/`AC`/`EP`/`VAL` 编号规则

- **生命周期 / 状态 / 工作量**：Active / 部分完成 / S
- **阶段 / 版本 / 模块**：S0 / v0.1 / M-01 工程治理与契约
- **需求追踪**：全部；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 编号注册表，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“固化 `RQ`/`AC`/`EP`/`VAL` 编号规则”所需的最小闭环；主交付物为：编号注册表。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0001。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`Cargo.toml`、`rust-toolchain.toml`、`specs/`、`schemas/`、`xtask/`、`.github/workflows/`、`docs/`、`rules/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：编号注册表；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-02`：重复/断号扫描；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`docs(governance): 固化 RQ/AC/EP/VAL 编号规则`

##### EP-0003 建立需求→AC→EP→验证追踪表

- **生命周期 / 状态 / 工作量**：Active / 部分完成 / M
- **阶段 / 版本 / 模块**：S0 / v0.1 / M-01 工程治理与契约
- **需求追踪**：全部；WI：WI-v0.1-05
- **目标**：交付 追踪矩阵，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“建立需求→AC→EP→验证追踪表”所需的最小闭环；主交付物为：追踪矩阵。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0002。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`Cargo.toml`、`rust-toolchain.toml`、`specs/`、`schemas/`、`xtask/`、`.github/workflows/`、`docs/`、`rules/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：追踪矩阵；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-02B`：每个 RQ 有 AC/任务/证据；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`test(governance): 建立需求→AC→EP→验证追踪表`

##### EP-0004 定义任务状态与阻塞原因

- **生命周期 / 状态 / 工作量**：Active / 部分完成 / M
- **阶段 / 版本 / 模块**：S0 / v0.1 / M-01 工程治理与契约
- **需求追踪**：RQ-038/068/069；WI：WI-v0.1-15
- **目标**：交付 `TaskStatus`/`BlockReason` 映射，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“定义任务状态与阻塞原因”所需的最小闭环；主交付物为：`TaskStatus`/`BlockReason` 映射。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0002。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`Cargo.toml`、`rust-toolchain.toml`、`specs/`、`schemas/`、`xtask/`、`.github/workflows/`、`docs/`、`rules/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：`TaskStatus`/`BlockReason` 映射；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-03`：状态机非法迁移测试；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(governance): 定义任务状态与阻塞原因`

##### EP-0005 定义统一验证证据目录与命名

- **生命周期 / 状态 / 工作量**：Active / 部分完成 / M
- **阶段 / 版本 / 模块**：S0 / v0.1 / M-01 工程治理与契约
- **需求追踪**：RQ-040/107–110；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 日志/artifact 目录约定，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“定义统一验证证据目录与命名”所需的最小闭环；主交付物为：日志/artifact 目录约定。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0002。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`Cargo.toml`、`rust-toolchain.toml`、`specs/`、`schemas/`、`xtask/`、`.github/workflows/`、`docs/`、`rules/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：日志/artifact 目录约定；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-04`：路径和 trace 完整性检查；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`test(governance): 定义统一验证证据目录与命名`

##### EP-0006 建立代码、依赖、Schema、协议四类漂移检查

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S0 / v0.1 / M-01 工程治理与契约
- **需求追踪**：RQ-111；WI：WI-v0.1-03
- **目标**：交付 CI 检查清单/脚本规范，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“建立代码、依赖、Schema、协议四类漂移检查”所需的最小闭环；主交付物为：CI 检查清单/脚本规范。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0003。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`Cargo.toml`、`rust-toolchain.toml`、`specs/`、`schemas/`、`xtask/`、`.github/workflows/`、`docs/`、`rules/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：CI 检查清单/脚本规范；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-05`：注入一处漂移应失败；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(governance): 建立代码、依赖、Schema、协议四类漂移检查`

##### EP-0007 建立跨平台/Provider/客户端能力矩阵 fixture

- **生命周期 / 状态 / 工作量**：Active / 部分完成 / M
- **阶段 / 版本 / 模块**：S0 / v0.1 / M-01 工程治理与契约
- **需求追踪**：RQ-004/005/084–090；WI：WI-v0.1-16
- **目标**：交付 矩阵数据文件，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“建立跨平台/Provider/客户端能力矩阵 fixture”所需的最小闭环；主交付物为：矩阵数据文件。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0003。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`Cargo.toml`、`rust-toolchain.toml`、`specs/`、`schemas/`、`xtask/`、`.github/workflows/`、`docs/`、`rules/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：矩阵数据文件；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-06`：缺能力与冲突配置被拒绝；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(governance): 建立跨平台/Provider/客户端能力矩阵 fixture`

##### EP-0008 建立内存 Port、假时钟、故障注入 harness 设计

- **生命周期 / 状态 / 工作量**：Active / 部分完成 / M
- **阶段 / 版本 / 模块**：S0 / v0.1 / M-01 工程治理与契约
- **需求追踪**：RQ-046/068/071；WI：WI-v0.1-14
- **目标**：交付 `apex-test-support` 规格，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“建立内存 Port、假时钟、故障注入 harness 设计”所需的最小闭环；主交付物为：`apex-test-support` 规格。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0001。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`Cargo.toml`、`rust-toolchain.toml`、`specs/`、`schemas/`、`xtask/`、`.github/workflows/`、`docs/`、`rules/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：`apex-test-support` 规格；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-07`：故障注入点清单审查；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(governance): 建立内存 Port、假时钟、故障注入 harness 设计`

##### EP-0009 建立封装访问器 derive 宏 crate（`Getters`/`Setters`/`Builder`/`Data`/`GettersExt`）与 CI pub 字段拦截

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S0 / v0.1 / M-01 工程治理与契约
- **需求追踪**：编码规范 §1.6b；WI：WI-v0.1-04b
- **目标**：交付 `apex-macros` crate + CI 检查脚本，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“建立封装访问器 derive 宏 crate（`Getters`/`Setters`/`Builder`/`Data`/`GettersExt`）与 CI pub 字段拦截”所需的最小闭环；主交付物为：`apex-macros` crate + CI 检查脚本。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0002。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`Cargo.toml`、`rust-toolchain.toml`、`specs/`、`schemas/`、`xtask/`、`.github/workflows/`、`docs/`、`rules/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：`apex-macros` crate + CI 检查脚本；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-08`：宏展开正/负 fixture 与 pub 字段拦截用例；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(governance): 建立封装访问器 derive 宏 crate（Getters/Setters/Builder/Data/GettersExt）与 CI pub 字段拦截`

##### EP-0010 实现执行计划完整性 Lint

- **生命周期 / 状态 / 工作量**：Active / 未开始 / S
- **阶段 / 版本 / 模块**：S0 / v0.1 / M-01 工程治理与契约
- **需求追踪**：全部；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 重复 ID、孤儿 EP/VAL、缺失版本/模块、跨版本逆向依赖检查器，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现执行计划完整性 Lint”所需的最小闭环；主交付物为：重复 ID、孤儿 EP/VAL、缺失版本/模块、跨版本逆向依赖检查器。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0002、EP-0003。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`Cargo.toml`、`rust-toolchain.toml`、`specs/`、`schemas/`、`xtask/`、`.github/workflows/`、`docs/`、`rules/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：重复 ID、孤儿 EP/VAL、缺失版本/模块、跨版本逆向依赖检查器；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-222：四类坏 fixture 均阻断，合法注册表通过；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`docs(governance): 实现执行计划完整性 Lint`

##### EP-0011 统一模块编号、内部引用与文档锚点

- **生命周期 / 状态 / 工作量**：Active / 未开始 / L
- **阶段 / 版本 / 模块**：S0 / v0.1 / M-01 工程治理与契约
- **需求追踪**：全部；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 M-26 Desktop、M-27 Web、M-28 Trinity 的规范映射及旧引用迁移表，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“统一模块编号、内部引用与文档锚点”所需的最小闭环；主交付物为：M-26 Desktop、M-27 Web、M-28 Trinity 的规范映射及旧引用迁移表。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0002。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`Cargo.toml`、`rust-toolchain.toml`、`specs/`、`schemas/`、`xtask/`、`.github/workflows/`、`docs/`、`rules/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：M-26 Desktop、M-27 Web、M-28 Trinity 的规范映射及旧引用迁移表；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-223：模块引用扫描无 M-29/M-30 悬空目标；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`docs(governance): 统一模块编号、内部引用与文档锚点`

##### EP-0101 创建 workspace 根清单与成员列表

- **生命周期 / 状态 / 工作量**：Active / 需重构 / M
- **阶段 / 版本 / 模块**：S1 / v0.1 / M-01 工程治理与契约
- **需求追踪**：RQ-002；WI：WI-v0.1-01
- **目标**：交付 `Cargo.toml` workspace，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“创建 workspace 根清单与成员列表”所需的最小闭环；主交付物为：`Cargo.toml` workspace。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0006。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`Cargo.toml`、`rust-toolchain.toml`、`specs/`、`schemas/`、`xtask/`、`.github/workflows/`、`docs/`、`rules/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：`Cargo.toml` workspace；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-08`：成员/路径检查；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(governance): 创建 workspace 根清单与成员列表`

##### EP-0102 锁定 Rust toolchain 与 target 列表

- **生命周期 / 状态 / 工作量**：Active / 部分完成 / M
- **阶段 / 版本 / 模块**：S1 / v0.1 / M-01 工程治理与契约
- **需求追踪**：RQ-004/005；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 toolchain/target matrix，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“锁定 Rust toolchain 与 target 列表”所需的最小闭环；主交付物为：toolchain/target matrix。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0101。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`Cargo.toml`、`rust-toolchain.toml`、`specs/`、`schemas/`、`xtask/`、`.github/workflows/`、`docs/`、`rules/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：toolchain/target matrix；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-09`：六 target dry-run；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(governance): 锁定 Rust toolchain 与 target 列表`

##### EP-0103 配置 rustfmt/clippy/deny/audit 基线

- **生命周期 / 状态 / 工作量**：Active / 部分完成 / M
- **阶段 / 版本 / 模块**：S1 / v0.1 / M-01 工程治理与契约
- **需求追踪**：RQ-045/046；WI：WI-v0.1-02
- **目标**：交付 lint/依赖配置，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“配置 rustfmt/clippy/deny/audit 基线”所需的最小闭环；主交付物为：lint/依赖配置。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0101。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`Cargo.toml`、`rust-toolchain.toml`、`specs/`、`schemas/`、`xtask/`、`.github/workflows/`、`docs/`、`rules/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：lint/依赖配置；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-10`：故意引入 warning 应失败；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(governance): 配置 rustfmt/clippy/deny/audit 基线`

##### EP-0104 实现 UUIDv7/ContentHash/TraceId newtype

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S1 / v0.1 / M-01 工程治理与契约
- **需求追踪**：04 领域契约；WI：WI-v0.1-06
- **目标**：交付 Domain IDs，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 UUIDv7/ContentHash/TraceId newtype”所需的最小闭环；主交付物为：Domain IDs。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0101。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`Cargo.toml`、`rust-toolchain.toml`、`specs/`、`schemas/`、`xtask/`、`.github/workflows/`、`docs/`、`rules/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Domain IDs；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-11`：格式、排序、不可混用测试；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(governance): 实现 UUIDv7/ContentHash/TraceId newtype`

##### EP-0105 实现时间、generation、幂等 key 值对象

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S1 / v0.1 / M-01 工程治理与契约
- **需求追踪**：RQ-027/103；WI：WI-v0.1-07
- **目标**：交付 Domain values，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现时间、generation、幂等 key 值对象”所需的最小闭环；主交付物为：Domain values。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0104。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`Cargo.toml`、`rust-toolchain.toml`、`specs/`、`schemas/`、`xtask/`、`.github/workflows/`、`docs/`、`rules/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Domain values；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-12`：边界/序列化测试；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(governance): 实现时间、generation、幂等 key 值对象`

##### EP-0106 实现唯一状态枚举及稳定字符串编码

- **生命周期 / 状态 / 工作量**：Active / 未开始 / S
- **阶段 / 版本 / 模块**：S1 / v0.1 / M-01 工程治理与契约
- **需求追踪**：04 状态机；WI：WI-v0.1-08
- **目标**：交付 Domain states，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现唯一状态枚举及稳定字符串编码”所需的最小闭环；主交付物为：Domain states。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0104。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`Cargo.toml`、`rust-toolchain.toml`、`specs/`、`schemas/`、`xtask/`、`.github/workflows/`、`docs/`、`rules/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Domain states；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-13`：新增值/未知值兼容测试；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(governance): 实现唯一状态枚举及稳定字符串编码`

##### EP-0107 实现 `ApexError` 与稳定错误码

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S1 / v0.1 / M-01 工程治理与契约
- **需求追踪**：04 错误模型；WI：WI-v0.1-09
- **目标**：交付 Error taxonomy，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 `ApexError` 与稳定错误码”所需的最小闭环；主交付物为：Error taxonomy。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0105。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`Cargo.toml`、`rust-toolchain.toml`、`specs/`、`schemas/`、`xtask/`、`.github/workflows/`、`docs/`、`rules/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Error taxonomy；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-14`：错误映射/trace 完整性；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(governance): 实现 ApexError 与稳定错误码`

##### EP-0108 实现 `EventEnvelope` 与 NewEvent

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S1 / v0.1 / M-01 工程治理与契约
- **需求追踪**：RQ-027/111；WI：WI-v0.1-10
- **目标**：交付 Event types，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 `EventEnvelope` 与 NewEvent”所需的最小闭环；主交付物为：Event types。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0104/0107。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`Cargo.toml`、`rust-toolchain.toml`、`specs/`、`schemas/`、`xtask/`、`.github/workflows/`、`docs/`、`rules/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Event types；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-15`：版本/序列/未知字段测试；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(governance): 实现 EventEnvelope 与 NewEvent`

##### EP-0109 实现 `CommandContext`/Actor/Client identity

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S1 / v0.1 / M-01 工程治理与契约
- **需求追踪**：RQ-021/023/050；WI：WI-v0.1-11
- **目标**：交付 Command context，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 `CommandContext`/Actor/Client identity”所需的最小闭环；主交付物为：Command context。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0104/0107。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`Cargo.toml`、`rust-toolchain.toml`、`specs/`、`schemas/`、`xtask/`、`.github/workflows/`、`docs/`、`rules/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Command context；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-16`：trace/idempotency 测试；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：竞态、饥饿或无界队列。使用确定性时钟/调度器并验证取消后的资源回收。
- **建议提交**：`feat(governance): 实现 CommandContext/Actor/Client identity`

##### EP-0110 实现 `apex-ports` Trait 空实现编译边界

- **生命周期 / 状态 / 工作量**：Active / 未开始 / S
- **阶段 / 版本 / 模块**：S1 / v0.1 / M-01 工程治理与契约
- **需求追踪**：05 Trait 契约；WI：WI-v0.1-12
- **目标**：交付 Port crate，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 `apex-ports` Trait 空实现编译边界”所需的最小闭环；主交付物为：Port crate。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0104–0109。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`Cargo.toml`、`rust-toolchain.toml`、`specs/`、`schemas/`、`xtask/`、`.github/workflows/`、`docs/`、`rules/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Port crate；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-17`：依赖反向引用扫描；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(governance): 实现 apex-ports Trait 空实现编译边界`

##### EP-0111 生成 Protobuf Rust/TypeScript 类型

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S1 / v0.1 / M-01 工程治理与契约
- **需求追踪**：RQ-009/012/017；WI：WI-v0.1-13
- **目标**：交付 Generated types，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“生成 Protobuf Rust/TypeScript 类型”所需的最小闭环；主交付物为：Generated types。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0108/0110。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`Cargo.toml`、`rust-toolchain.toml`、`specs/`、`schemas/`、`xtask/`、`.github/workflows/`、`docs/`、`rules/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Generated types；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-18`：codegen 可重复性；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(governance): 生成 Protobuf Rust/TypeScript 类型`

##### EP-0112 建立 Rust 单元/属性测试公共 fixture

- **生命周期 / 状态 / 工作量**：Active / 未开始 / S
- **阶段 / 版本 / 模块**：S1 / v0.1 / M-01 工程治理与契约
- **需求追踪**：RQ-046；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 test-support fixtures，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“建立 Rust 单元/属性测试公共 fixture”所需的最小闭环；主交付物为：test-support fixtures。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0101–0111。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`Cargo.toml`、`rust-toolchain.toml`、`specs/`、`schemas/`、`xtask/`、`.github/workflows/`、`docs/`、`rules/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：test-support fixtures；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-19`：假时钟/随机 ID/故障注入自测；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`test(governance): 建立 Rust 单元/属性测试公共 fixture`

#### M-02 存储、事件溯源与平台事实

##### EP-0201 实现 Apex Home 路径解析

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S2 / v0.1 / M-02 存储、事件溯源与平台事实
- **需求追踪**：RQ-008；WI：WI-v0.1-17
- **目标**：交付 HomePath API，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Apex Home 路径解析”所需的最小闭环；主交付物为：HomePath API。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0102。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-platform/`、`crates/apex-storage/`、`crates/apex-observability/`、`crates/apex-test-support/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：HomePath API；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-20`：三 OS 路径 fixture；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(storage): 实现 Apex Home 路径解析`

##### EP-0203 实现单实例 lock/mutex 与 stale 检查

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S2 / v0.1 / M-02 存储、事件溯源与平台事实
- **需求追踪**：RQ-006；WI：WI-v0.1-18
- **目标**：交付 Singleton guard，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现单实例 lock/mutex 与 stale 检查”所需的最小闭环；主交付物为：Singleton guard。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0201。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-platform/`、`crates/apex-storage/`、`crates/apex-observability/`、`crates/apex-test-support/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Singleton guard；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-22`：双启动/假 PID 测试；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(storage): 实现单实例 lock/mutex 与 stale 检查`

##### EP-0204 实现 Unix Domain Socket listener

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S2 / v0.1 / M-02 存储、事件溯源与平台事实
- **需求追踪**：RQ-009/010；WI：WI-v0.1-19/20, WI-v0.1-19
- **目标**：交付 Unix endpoint，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Unix Domain Socket listener”所需的最小闭环；主交付物为：Unix endpoint。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0203/0111。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-platform/`、`crates/apex-storage/`、`crates/apex-observability/`、`crates/apex-test-support/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Unix endpoint；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-23`：ACL/重连/路径长度；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(storage): 实现 Unix Domain Socket listener`

##### EP-0205 实现 Windows Named Pipe listener

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S2 / v0.1 / M-02 存储、事件溯源与平台事实
- **需求追踪**：RQ-009/011；WI：WI-v0.1-20
- **目标**：交付 Windows endpoint，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Windows Named Pipe listener”所需的最小闭环；主交付物为：Windows endpoint。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0203/0111。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-platform/`、`crates/apex-storage/`、`crates/apex-observability/`、`crates/apex-test-support/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Windows endpoint；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-24`：SID ACL/并发连接；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(storage): 实现 Windows Named Pipe listener`

##### EP-0207 实现普通配置加载与未知字段保留

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S2 / v0.1 / M-02 存储、事件溯源与平台事实
- **需求追踪**：RQ-111；WI：WI-v0.1-21
- **目标**：交付 Config model，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现普通配置加载与未知字段保留”所需的最小闭环；主交付物为：Config model。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0107/0201。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-platform/`、`crates/apex-storage/`、`crates/apex-observability/`、`crates/apex-test-support/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Config model；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-26`：未知字段 round-trip；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(storage): 实现普通配置加载与未知字段保留`

##### EP-0208 实现 SQLite 打开、WAL 与 busy 策略

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S2 / v0.1 / M-02 存储、事件溯源与平台事实
- **需求追踪**：RQ-007/103/104；WI：WI-v0.1-22
- **目标**：交付 DB bootstrap，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 SQLite 打开、WAL 与 busy 策略”所需的最小闭环；主交付物为：DB bootstrap。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0108/0201。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-platform/`、`crates/apex-storage/`、`crates/apex-observability/`、`crates/apex-test-support/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：DB bootstrap；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-27`：pragma/并发 writer；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(storage): 实现 SQLite 打开、WAL 与 busy 策略`

##### EP-0209 实现 schema_meta/feature/migration 表

- **生命周期 / 状态 / 工作量**：Active / 未开始 / S
- **阶段 / 版本 / 模块**：S2 / v0.1 / M-02 存储、事件溯源与平台事实
- **需求追踪**：RQ-111；WI：WI-v0.1-23
- **目标**：交付 Migration catalog，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 schema_meta/feature/migration 表”所需的最小闭环；主交付物为：Migration catalog。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0208。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-platform/`、`crates/apex-storage/`、`crates/apex-observability/`、`crates/apex-test-support/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Migration catalog；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-28`：重复迁移/中断恢复；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(storage): 实现 schema_meta/feature/migration 表`

##### EP-0210 实现 EventStore append 事务

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S2 / v0.1 / M-02 存储、事件溯源与平台事实
- **需求追踪**：RQ-026/027；WI：WI-v0.1-24
- **目标**：交付 Event append，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 EventStore append 事务”所需的最小闭环；主交付物为：Event append。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0108/0208。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-platform/`、`crates/apex-storage/`、`crates/apex-observability/`、`crates/apex-test-support/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Event append；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-29`：optimistic conflict/幂等；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(storage): 实现 EventStore append 事务`

##### EP-0211 实现 session sequence 与 aggregate version

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S2 / v0.1 / M-02 存储、事件溯源与平台事实
- **需求追踪**：RQ-027；WI：WI-v0.1-25
- **目标**：交付 Sequence allocator，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 session sequence 与 aggregate version”所需的最小闭环；主交付物为：Sequence allocator。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0210。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-platform/`、`crates/apex-storage/`、`crates/apex-observability/`、`crates/apex-test-support/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Sequence allocator；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-30`：无 gap/并发竞争；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(storage): 实现 session sequence 与 aggregate version`

##### EP-0212 实现 projector cursor 与投影批处理

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S2 / v0.1 / M-02 存储、事件溯源与平台事实
- **需求追踪**：RQ-026；WI：WI-v0.1-26
- **目标**：交付 Projector runtime，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 projector cursor 与投影批处理”所需的最小闭环；主交付物为：Projector runtime。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0210/0211。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-platform/`、`crates/apex-storage/`、`crates/apex-observability/`、`crates/apex-test-support/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Projector runtime；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-31`：重放投影 hash；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(storage): 实现 projector cursor 与投影批处理`

##### EP-0213 实现 Query Snapshot/keyset pagination

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S2 / v0.1 / M-02 存储、事件溯源与平台事实
- **需求追踪**：RQ-001/114；WI：WI-v0.1-27
- **目标**：交付 Query store，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Query Snapshot/keyset pagination”所需的最小闭环；主交付物为：Query store。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0212。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-platform/`、`crates/apex-storage/`、`crates/apex-observability/`、`crates/apex-test-support/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Query store；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-32`：10k 分页基准；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(storage): 实现 Query Snapshot/keyset pagination`

##### EP-0214 实现 Markdown 原子写/文件 generation

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S2 / v0.1 / M-02 存储、事件溯源与平台事实
- **需求追踪**：RQ-025/028；WI：WI-v0.1-28
- **目标**：交付 FileFactStore，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Markdown 原子写/文件 generation”所需的最小闭环；主交付物为：FileFactStore。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0201/0105。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-platform/`、`crates/apex-storage/`、`crates/apex-observability/`、`crates/apex-test-support/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：FileFactStore；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-33`：崩溃注入/权限/rename；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(storage): 实现 Markdown 原子写/文件 generation`

##### EP-0219 实现 Session JSONL sink 与 10 MiB 轮转

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S2 / v0.1 / M-02 存储、事件溯源与平台事实
- **需求追踪**：RQ-107–109；WI：WI-v0.1-29
- **目标**：交付 SessionLogSink，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Session JSONL sink 与 10 MiB 轮转”所需的最小闭环；主交付物为：SessionLogSink。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0108/0201。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-platform/`、`crates/apex-storage/`、`crates/apex-observability/`、`crates/apex-test-support/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：SessionLogSink；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-38`：JSONL/hash-chain/轮转；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(storage): 实现 Session JSONL sink 与 10 MiB 轮转`

#### M-03 Daemon、Session 与 Agent Loop

##### EP-0301 实现 ClientHello/ServerHello 版本协商

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S3 / v0.1 / M-03 Daemon、Session 与 Agent Loop
- **需求追踪**：RQ-009/012/111；WI：WI-v0.1-30
- **目标**：交付 HandshakeService，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 ClientHello/ServerHello 版本协商”所需的最小闭环；主交付物为：HandshakeService。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0111/0204/0205。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-daemon/`、`crates/apex-application/`、`crates/apex-client-sdk/`、`proto/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：HandshakeService；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-43`：major/minor/feature；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(daemon): 实现 ClientHello/ServerHello 版本协商`

##### EP-0302 实现 gRPC interceptor 身份/trace/idempotency

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S3 / v0.1 / M-03 Daemon、Session 与 Agent Loop
- **需求追踪**：RQ-009/021；WI：WI-v0.1-31
- **目标**：交付 gRPC middleware，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 gRPC interceptor 身份/trace/idempotency”所需的最小闭环；主交付物为：gRPC middleware。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0301。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-daemon/`、`crates/apex-application/`、`crates/apex-client-sdk/`、`proto/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：gRPC middleware；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-44`：未认证/重复请求；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(daemon): 实现 gRPC interceptor 身份/trace/idempotency`

##### EP-0306 实现 durable prompt inbox

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S3 / v0.1 / M-03 Daemon、Session 与 Agent Loop
- **需求追踪**：RQ-026；WI：WI-v0.1-32
- **目标**：交付 Inbox admission，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 durable prompt inbox”所需的最小闭环；主交付物为：Inbox admission。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0210/0302。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-daemon/`、`crates/apex-application/`、`crates/apex-client-sdk/`、`proto/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Inbox admission；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-48`：重复提交/崩溃；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(daemon): 实现 durable prompt inbox`

##### EP-0307 实现 Session Actor 串行提升 Turn

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S3 / v0.1 / M-03 Daemon、Session 与 Agent Loop
- **需求追踪**：RQ-001/024；WI：WI-v0.1-33, WI-v0.1-34
- **目标**：交付 SessionRuntime，形成可独立验证、可单独回滚的闭环。
- **范围**：仅保留 Session Actor、邮箱、状态转换与安全点；Agent Loop 移至 EP-0315。
- **非范围**：不包含：EP-0315；不顺带修改其他版本能力。
- **前置依赖**：EP-0306/0212。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-daemon/`、`crates/apex-application/`、`crates/apex-client-sdk/`、`proto/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：SessionRuntime；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-49`：并发输入/安全点；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：竞态、饥饿或无界队列。使用确定性时钟/调度器并验证取消后的资源回收。
- **建议提交**：`feat(daemon): 实现 Session Actor 串行提升 Turn`

##### EP-0314 实现 graceful shutdown/drain

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S3 / v0.1 / M-03 Daemon、Session 与 Agent Loop
- **需求追踪**：RQ-024/068；WI：WI-v0.1-35
- **目标**：交付 Daemon shutdown，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 graceful shutdown/drain”所需的最小闭环；主交付物为：Daemon shutdown。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0307、EP-0315。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-daemon/`、`crates/apex-application/`、`crates/apex-client-sdk/`、`proto/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Daemon shutdown；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-56`：Tool/DAG 安全点；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(daemon): 实现 graceful shutdown/drain`

##### EP-0315 实现 Agent Loop 编排器

- **生命周期 / 状态 / 工作量**：Active / 未开始 / L
- **阶段 / 版本 / 模块**：S3 / v0.1 / M-03 Daemon、Session 与 Agent Loop
- **需求追踪**：RQ-009、RQ-054、RQ-059、RQ-074；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 模型调用、工具回合、事件追加与安全点协同的 Agent Loop，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Agent Loop 编排器”所需的最小闭环；主交付物为：模型调用、工具回合、事件追加与安全点协同的 Agent Loop。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0307、EP-0406、EP-0515、EP-0603、EP-0802。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-daemon/`、`crates/apex-application/`、`crates/apex-client-sdk/`、`proto/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：模型调用、工具回合、事件追加与安全点协同的 Agent Loop；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-224：纯对话、工具回合、取消、失败恢复状态机；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：竞态、饥饿或无界队列。使用确定性时钟/调度器并验证取消后的资源回收。
- **建议提交**：`feat(daemon): 实现 Agent Loop 编排器`

#### M-04 Provider 核心

##### EP-0801 实现 Provider Core ModelRequest/Frame

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S8 / v0.1 / M-04 Provider 核心
- **需求追踪**：RQ-084–086；WI：WI-v0.1-36
- **目标**：交付 Provider core types，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Provider Core ModelRequest/Frame”所需的最小闭环；主交付物为：Provider core types。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0108/0110。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-provider/`、`crates/apex-domain/`、`crates/apex-test-support/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Provider core types；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-134`：消息/流 round-trip；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(provider): 实现 Provider Core ModelRequest/Frame`

##### EP-0802 实现 capability schema/negotiation

- **生命周期 / 状态 / 工作量**：Active / 未开始 / S
- **阶段 / 版本 / 模块**：S8 / v0.1 / M-04 Provider 核心
- **需求追踪**：RQ-085–088；WI：WI-v0.1-37
- **目标**：交付 ModelCapabilities，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 capability schema/negotiation”所需的最小闭环；主交付物为：ModelCapabilities。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0801。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-provider/`、`crates/apex-domain/`、`crates/apex-test-support/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：ModelCapabilities；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-135`：缺能力拒绝；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(provider): 实现 capability schema/negotiation`

##### EP-0803 实现 Anthropic adapter

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S8 / v0.1 / M-04 Provider 核心
- **需求追踪**：RQ-084；WI：WI-v0.1-38
- **目标**：交付 `apex-provider-anthropic`，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Anthropic adapter”所需的最小闭环；主交付物为：`apex-provider-anthropic`。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0801/0802。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-provider/`、`crates/apex-domain/`、`crates/apex-test-support/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：`apex-provider-anthropic`；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-136`：Tool/reasoning/stream；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(provider): 实现 Anthropic adapter`

##### EP-0804 实现 OpenAI adapter

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S8 / v0.1 / M-04 Provider 核心
- **需求追踪**：RQ-084；WI：WI-v0.1-39
- **目标**：交付 `apex-provider-openai`，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 OpenAI adapter”所需的最小闭环；主交付物为：`apex-provider-openai`。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0801/0802。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-provider/`、`crates/apex-domain/`、`crates/apex-test-support/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：`apex-provider-openai`；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-137`：Responses/Realtime；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(provider): 实现 OpenAI adapter`

##### EP-0808 实现 providers.toml profile parser

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S8 / v0.1 / M-04 Provider 核心
- **需求追踪**：RQ-091；WI：WI-v0.1-40
- **目标**：交付 Provider profiles，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 providers.toml profile parser”所需的最小闭环；主交付物为：Provider profiles。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0207/0801。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-provider/`、`crates/apex-domain/`、`crates/apex-test-support/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Provider profiles；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-141`：明文配置/权限/未知字段；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(provider): 实现 providers.toml profile parser`

##### EP-0812 实现 retry/backoff/deadline/cancel

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S8 / v0.1 / M-04 Provider 核心
- **需求追踪**：RQ-089；WI：WI-v0.1-42
- **目标**：交付 Retry policy，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 retry/backoff/deadline/cancel”所需的最小闭环；主交付物为：Retry policy。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0803–0807。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-provider/`、`crates/apex-domain/`、`crates/apex-test-support/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Retry policy；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-145`：429/5xx/半流；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(provider): 实现 retry/backoff/deadline/cancel`

##### EP-0817 实现 SecretResolver 凭据解析接口

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S8 / v0.1 / M-04 Provider 核心
- **需求追踪**：RQ-062、RQ-065；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 凭据引用、分层查找、缺失/过期错误与测试替身，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 SecretResolver 凭据解析接口”所需的最小闭环；主交付物为：凭据引用、分层查找、缺失/过期错误与测试替身。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0104、EP-0201。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-provider/`、`crates/apex-domain/`、`crates/apex-test-support/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：凭据引用、分层查找、缺失/过期错误与测试替身；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-230：不返回明文到领域事件，查找优先级确定；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：误放行或敏感信息泄漏。默认拒绝；正例、负例和绕过语料必须同时覆盖。
- **建议提交**：`feat(provider): 实现 SecretResolver 凭据解析接口`

##### EP-0818 实现 Provider Secret Firewall 与日志脱敏

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S8 / v0.1 / M-04 Provider 核心
- **需求追踪**：RQ-062、RQ-065、RQ-110；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 Provider 请求注入边界、结构化脱敏与泄漏检测测试，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Provider Secret Firewall 与日志脱敏”所需的最小闭环；主交付物为：Provider 请求注入边界、结构化脱敏与泄漏检测测试。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0817、EP-0801。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-provider/`、`crates/apex-domain/`、`crates/apex-test-support/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Provider 请求注入边界、结构化脱敏与泄漏检测测试；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-231：事件、日志、错误、快照均无凭据明文；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：误放行或敏感信息泄漏。默认拒绝；正例、负例和绕过语料必须同时覆盖。
- **建议提交**：`feat(provider): 实现 Provider Secret Firewall 与日志脱敏`

#### M-05 Spec Pipeline

##### EP-0401 实现 requirements.md schema/parser

- **生命周期 / 状态 / 工作量**：Active / 未开始 / S
- **阶段 / 版本 / 模块**：S4 / v0.1 / M-05 Spec Pipeline
- **需求追踪**：RQ-030/036；WI：WI-v0.1-43, WI-v0.1-53
- **目标**：交付 Requirements document model，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 requirements.md schema/parser”所需的最小闭环；主交付物为：Requirements document model。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0001/0214。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-spec/`、`specs/`、`schemas/`、`xtask/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Requirements document model；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-57`：正/负 frontmatter；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(spec): 实现 requirements.md schema/parser`

##### EP-0402 实现 design.md schema/parser

- **生命周期 / 状态 / 工作量**：Active / 未开始 / S
- **阶段 / 版本 / 模块**：S4 / v0.1 / M-05 Spec Pipeline
- **需求追踪**：RQ-037；WI：WI-v0.1-44
- **目标**：交付 Design model，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 design.md schema/parser”所需的最小闭环；主交付物为：Design model。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0401。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-spec/`、`specs/`、`schemas/`、`xtask/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Design model；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-58`：上游 hash 校验；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(spec): 实现 design.md schema/parser`

##### EP-0403 实现 tasks.md schema/parser

- **生命周期 / 状态 / 工作量**：Active / 未开始 / S
- **阶段 / 版本 / 模块**：S4 / v0.1 / M-05 Spec Pipeline
- **需求追踪**：RQ-030/062/064；WI：WI-v0.1-45
- **目标**：交付 Task model，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 tasks.md schema/parser”所需的最小闭环；主交付物为：Task model。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0402。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-spec/`、`specs/`、`schemas/`、`xtask/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Task model；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-59`：依赖/路径/循环拒绝；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(spec): 实现 tasks.md schema/parser`

##### EP-0404 实现 verification.md renderer/schema

- **生命周期 / 状态 / 工作量**：Active / 未开始 / S
- **阶段 / 版本 / 模块**：S4 / v0.1 / M-05 Spec Pipeline
- **需求追踪**：RQ-040/041；WI：WI-v0.1-46
- **目标**：交付 Verification writer，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 verification.md renderer/schema”所需的最小闭环；主交付物为：Verification writer。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0401–0403。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-spec/`、`specs/`、`schemas/`、`xtask/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Verification writer；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-60`：输入 hash/缺证据失败；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(spec): 实现 verification.md renderer/schema`

##### EP-0405 实现 SpecStage/StageStatus 状态机

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S4 / v0.1 / M-05 Spec Pipeline
- **需求追踪**：RQ-036/037；WI：WI-v0.1-47
- **目标**：交付 Stage reducer，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 SpecStage/StageStatus 状态机”所需的最小闭环；主交付物为：Stage reducer。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0106/0401。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-spec/`、`specs/`、`schemas/`、`xtask/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Stage reducer；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-61`：非法跳阶段；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(spec): 实现 SpecStage/StageStatus 状态机`

##### EP-0406 实现 ApprovalRecord 内容 hash 绑定

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S4 / v0.1 / M-05 Spec Pipeline
- **需求追踪**：RQ-037/038；WI：WI-v0.1-48
- **目标**：交付 Approval service，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 ApprovalRecord 内容 hash 绑定”所需的最小闭环；主交付物为：Approval service。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0210/0405。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-spec/`、`specs/`、`schemas/`、`xtask/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Approval service；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-62`：内容变化自动失效；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(spec): 实现 ApprovalRecord 内容 hash 绑定`

##### EP-0407 实现上游变化失效传播图

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S4 / v0.1 / M-05 Spec Pipeline
- **需求追踪**：RQ-038；WI：WI-v0.1-49
- **目标**：交付 Invalidation plan，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现上游变化失效传播图”所需的最小闭环；主交付物为：Invalidation plan。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0405/0406。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-spec/`、`specs/`、`schemas/`、`xtask/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Invalidation plan；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-63`：requirements→下游传播；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(spec): 实现上游变化失效传播图`

##### EP-0408 实现 `/skip-spec` parser 与 scope 校验

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S4 / v0.1 / M-05 Spec Pipeline
- **需求追踪**：RQ-039；WI：WI-v0.1-50
- **目标**：交付 Skip command，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 `/skip-spec` parser 与 scope 校验”所需的最小闭环；主交付物为：Skip command。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0405/0306。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-spec/`、`specs/`、`schemas/`、`xtask/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Skip command；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-64`：run/session/all/过期；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(spec): 实现 /skip-spec parser 与 scope 校验`

##### EP-0409 实现 SkipGrant 审计事件与限制

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S4 / v0.1 / M-05 Spec Pipeline
- **需求追踪**：RQ-039；WI：WI-v0.1-51
- **目标**：交付 Skip audit，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 SkipGrant 审计事件与限制”所需的最小闭环；主交付物为：Skip audit。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0408/0210。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-spec/`、`specs/`、`schemas/`、`xtask/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Skip audit；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-65`：绕过 Spec 但不能绕安全门；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：原子性或时序错误导致数据损坏。故障注入覆盖断电、重复、乱序和部分写入。
- **建议提交**：`test(spec): 实现 SkipGrant 审计事件与限制`

##### EP-0410 实现规则 profile registry/version hash

- **生命周期 / 状态 / 工作量**：Active / 未开始 / S
- **阶段 / 版本 / 模块**：S4 / v0.1 / M-05 Spec Pipeline
- **需求追踪**：RQ-045；WI：WI-v0.1-52
- **目标**：交付 Rule catalog，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现规则 profile registry/version hash”所需的最小闭环；主交付物为：Rule catalog。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0108/0401。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-spec/`、`specs/`、`schemas/`、`xtask/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Rule catalog；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-66`：未知/变更 profile；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(spec): 实现规则 profile registry/version hash`

#### M-06 工具注册与网关

##### EP-0514 实现 Tool descriptor/schema/副作用声明

- **生命周期 / 状态 / 工作量**：Active / 未开始 / S
- **阶段 / 版本 / 模块**：S5 / v0.1 / M-06 工具注册与网关
- **需求追踪**：RQ-052/057；WI：WI-v0.1-54, WI-v0.1-55, WI-v0.1-56
- **目标**：交付 Tool registry，形成可独立验证、可单独回滚的闭环。
- **范围**：仅保留 ToolRegistry、descriptor/schema 注册与查询；内置工具适配器移出。
- **非范围**：不包含：EP-0524、EP-0525、EP-0526；不顺带修改其他版本能力。
- **前置依赖**：EP-0107、EP-0108。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-tool-gateway/`、`crates/apex-tools/`、`crates/apex-command-ast/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Tool registry；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-85`：未知 schema/超限；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(tools): 实现 Tool descriptor/schema/副作用声明`

##### EP-0515 实现 Tool Gateway prepare→gate→execute pipeline

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S5 / v0.1 / M-06 工具注册与网关
- **需求追踪**：AC-006/008；WI：WI-v0.1-58
- **目标**：交付 Gateway orchestration，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Tool Gateway prepare→gate→execute pipeline”所需的最小闭环；主交付物为：Gateway orchestration。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0409/0513/0514。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-tool-gateway/`、`crates/apex-tools/`、`crates/apex-command-ast/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Gateway orchestration；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-86`：顺序/幂等/拒绝；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(tools): 实现 Tool Gateway prepare→gate→execute pipeline`

##### EP-0516 实现 Tool result bounded output/receipt

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S5 / v0.1 / M-06 工具注册与网关
- **需求追踪**：RQ-107/108；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 Tool outcome，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Tool result bounded output/receipt”所需的最小闭环；主交付物为：Tool outcome。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0515/0217。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-tool-gateway/`、`crates/apex-tools/`、`crates/apex-command-ast/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Tool outcome；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-87`：大输出/副作用不一致；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(tools): 实现 Tool result bounded output/receipt`

##### EP-0519 实现一次性非交互命令模式

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S5 / v0.1 / M-06 工具注册与网关
- **需求追踪**：RQ-057；WI：WI-v0.1-57
- **目标**：交付 RunOnce adapter，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现一次性非交互命令模式”所需的最小闭环；主交付物为：RunOnce adapter。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0515/0517。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-tool-gateway/`、`crates/apex-tools/`、`crates/apex-command-ast/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：RunOnce adapter；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-90`：无 stdin/超时；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(tools): 实现一次性非交互命令模式`

##### EP-0524 实现 Read 与 Search 内置工具适配器

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S5 / v0.1 / M-06 工具注册与网关
- **需求追踪**：RQ-051、RQ-052、RQ-053；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 Read/Search descriptor、参数校验、边界化结果与审计事件，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Read 与 Search 内置工具适配器”所需的最小闭环；主交付物为：Read/Search descriptor、参数校验、边界化结果与审计事件。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0514、EP-1201、EP-0201。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-tool-gateway/`、`crates/apex-tools/`、`crates/apex-command-ast/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Read/Search descriptor、参数校验、边界化结果与审计事件；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-225：路径边界、空结果、超限结果与拒绝测试；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(tools): 实现 Read 与 Search 内置工具适配器`

##### EP-0525 实现 Edit 与 Patch 内置工具适配器

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S5 / v0.1 / M-06 工具注册与网关
- **需求追踪**：RQ-051、RQ-052、RQ-056、RQ-069；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 Edit/Patch descriptor、预检、PatchSet 输出与冲突报告，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Edit 与 Patch 内置工具适配器”所需的最小闭环；主交付物为：Edit/Patch descriptor、预检、PatchSet 输出与冲突报告。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0514、EP-1201、EP-0214。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-tool-gateway/`、`crates/apex-tools/`、`crates/apex-command-ast/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Edit/Patch descriptor、预检、PatchSet 输出与冲突报告；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-226：越界、冲突、原子写入与回滚测试；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(tools): 实现 Edit 与 Patch 内置工具适配器`

##### EP-0526 实现 Shell 工具准备适配器

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S5 / v0.1 / M-06 工具注册与网关
- **需求追踪**：RQ-047、RQ-048、RQ-049、RQ-050、RQ-052；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 Shell descriptor、受限 argv 请求、权限请求与受限执行请求，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Shell 工具准备适配器”所需的最小闭环；主交付物为：Shell descriptor、受限 argv 请求、权限请求与受限执行请求。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0514、EP-1201、EP-0107。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-tool-gateway/`、`crates/apex-tools/`、`crates/apex-command-ast/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Shell descriptor、受限 argv 请求、权限请求与受限执行请求；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-227：危险命令、环境变量、工作目录与审批分支测试；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(tools): 实现 Shell 工具准备适配器`

#### M-07 简化权限

##### EP-0513 实现 PermissionVerdict evidence/audit

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S5 / v0.1 / M-07 简化权限
- **需求追踪**：RQ-050/052；WI：WI-v0.1-60
- **目标**：交付 Decision evidence，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 PermissionVerdict evidence/audit”所需的最小闭环；主交付物为：Decision evidence。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0104、EP-1201。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-permission/`、`crates/apex-domain/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Decision evidence；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-84`：无 LLM/trace 完整；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(permission): 实现 PermissionVerdict evidence/audit`

##### EP-1201 实现 v0.1 简化权限策略并预留 v0.3 迁移接口

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S5 / v0.1 / M-07 简化权限
- **需求追踪**：RQ-047、RQ-048、RQ-049、RQ-050、RQ-052、RQ-056；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 SimplifiedPermissionPolicy、确认/拒绝事件与迁移适配口，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 v0.1 简化权限策略并预留 v0.3 迁移接口”所需的最小闭环；主交付物为：SimplifiedPermissionPolicy、确认/拒绝事件与迁移适配口。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0104、EP-0513。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-permission/`、`crates/apex-domain/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：SimplifiedPermissionPolicy、确认/拒绝事件与迁移适配口；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-214：ask/allow/deny、记忆范围与 v0.3 替换契约；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：误放行或敏感信息泄漏。默认拒绝；正例、负例和绕过语料必须同时覆盖。
- **建议提交**：`feat(permission): 实现 v0.1 简化权限策略并预留 v0.3 迁移接口`

#### M-08 Context Engine

##### EP-0601 实现 Provider-aware token estimator

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S6 / v0.1 / M-08 Context Engine
- **需求追踪**：RQ-074；WI：WI-v0.1-61
- **目标**：交付 Token budget Port，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Provider-aware token estimator”所需的最小闭环；主交付物为：Token budget Port。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0801（可先用 fake）。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-context/`、`crates/apex-provider/`、`crates/apex-test-support/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Token budget Port；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-95`：边界/多模态 token；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(context): 实现 Provider-aware token estimator`

##### EP-0602 实现 Stable/Turn/Retrieved/Recovery Source

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S6 / v0.1 / M-08 Context Engine
- **需求追踪**：RQ-074/077；WI：WI-v0.1-62
- **目标**：交付 ContextSource，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Stable/Turn/Retrieved/Recovery Source”所需的最小闭环；主交付物为：ContextSource。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0105/0406。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-context/`、`crates/apex-provider/`、`crates/apex-test-support/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：ContextSource；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-96`：hash/优先级；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(context): 实现 Stable/Turn/Retrieved/Recovery Source`

##### EP-0603 实现 ContextEpoch 构建与替换

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S6 / v0.1 / M-08 Context Engine
- **需求追踪**：RQ-075；WI：WI-v0.1-63, WI-v0.1-64
- **目标**：交付 Epoch builder，形成可独立验证、可单独回滚的闭环。
- **范围**：仅保留 ContextEpoch 构建与预算计算；临时溢出降级移至 EP-0618。
- **非范围**：不包含：EP-0618；不顺带修改其他版本能力。
- **前置依赖**：EP-0601、EP-0602。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-context/`、`crates/apex-provider/`、`crates/apex-test-support/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Epoch builder；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-97`：失败不消费 inbox；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(context): 实现 ContextEpoch 构建与替换`

##### EP-0618 实现临时上下文溢出降级策略

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S6 / v0.1 / M-08 Context Engine
- **需求追踪**：RQ-074；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 仅在硬预算触发的临时截断、告警事件与可诊断原因，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现临时上下文溢出降级策略”所需的最小闭环；主交付物为：仅在硬预算触发的临时截断、告警事件与可诊断原因。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0603。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-context/`、`crates/apex-provider/`、`crates/apex-test-support/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：仅在硬预算触发的临时截断、告警事件与可诊断原因；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-228：不静默截断、预算内零损失、预算外确定性降级；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(context): 实现临时上下文溢出降级策略`

#### M-09 TUI 核心

##### EP-0305 实现 Snapshot + since_seq 合并器

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S3 / v0.1 / M-09 TUI 核心
- **需求追踪**：AC-001；WI：WI-v1.2-03, WI-v0.2-21
- **目标**：交付 Client SDK reducer，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Snapshot + since_seq 合并器”所需的最小闭环；主交付物为：Client SDK reducer。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0213、EP-0301。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-tui/`、`crates/apex-tui/`、`crates/apex-client-sdk/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Client SDK reducer；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-47`：乱序/gap/resync；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(tui): 实现 Snapshot + since_seq 合并器`

##### EP-1001 建立 TUI 测试 demo 与连接/重连骨架

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v0.1 / M-09 TUI 核心
- **需求追踪**：RQ-009；WI：WI-v0.1-65a, WI-v0.1-65b, WI-v0.1-65
- **目标**：交付 TUI demo shell，形成可独立验证、可单独回滚的闭环。
- **范围**：仅保留 TUI 客户端壳、Daemon 连接、快照+增量归并与重连；窗口后端和拉起流程移出。
- **非范围**：不包含：EP-1028、EP-1029；不顺带修改其他版本能力。
- **前置依赖**：EP-0301、EP-0305、EP-1003。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-tui/`、`crates/apex-tui/`、`crates/apex-client-sdk/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：TUI demo shell；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-167`：fake daemon smoke、UDS/pipe 重连；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`test(tui): 建立 TUI 测试 demo 与连接/重连骨架`

##### EP-1002 实现 TUI Workspace/Session 列表

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v0.1 / M-09 TUI 核心
- **需求追踪**：AC-001；WI：WI-v0.1-66
- **目标**：交付 TUI navigation，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 TUI Workspace/Session 列表”所需的最小闭环；主交付物为：TUI navigation。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-1001/0213。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-tui/`、`crates/apex-tui/`、`crates/apex-client-sdk/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：TUI navigation；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-168`：分页/权限；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(tui): 实现 TUI Workspace/Session 列表`

##### EP-1003 实现 TUI Prompt/Admission/Turn 视图

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v0.1 / M-09 TUI 核心
- **需求追踪**：AC-001/003；WI：WI-v0.1-67
- **目标**：交付 TUI session panel，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 TUI Prompt/Admission/Turn 视图”所需的最小闭环；主交付物为：TUI session panel。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-1002/0306。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-tui/`、`crates/apex-tui/`、`crates/apex-client-sdk/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：TUI session panel；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-169`：幂等/阻塞；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(tui): 实现 TUI Prompt/Admission/Turn 视图`

##### EP-1028 实现原生 TUI 窗口与像素后端

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v0.1 / M-09 TUI 核心
- **需求追踪**：RQ-009、RQ-114、RQ-115；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 窗口生命周期、输入事件、像素/文本栅格适配与缩放处理，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现原生 TUI 窗口与像素后端”所需的最小闭环；主交付物为：窗口生命周期、输入事件、像素/文本栅格适配与缩放处理。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-1001。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-tui/`、`crates/apex-tui/`、`crates/apex-client-sdk/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：窗口生命周期、输入事件、像素/文本栅格适配与缩放处理；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-233：启动、缩放、宽字符、焦点与关闭测试；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(tui): 实现原生 TUI 窗口与像素后端`

##### EP-1029 实现项目选择、Daemon 拉起与 endpoint-ready 协议

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v0.1 / M-09 TUI 核心
- **需求追踪**：RQ-009；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 项目选择器、单实例 Daemon 拉起、端点就绪与失败恢复，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现项目选择、Daemon 拉起与 endpoint-ready 协议”所需的最小闭环；主交付物为：项目选择器、单实例 Daemon 拉起、端点就绪与失败恢复。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0201、EP-0301、EP-1001。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-tui/`、`crates/apex-tui/`、`crates/apex-client-sdk/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：项目选择器、单实例 Daemon 拉起、端点就绪与失败恢复；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-234：首次启动、并发启动、端口冲突、超时与重试 E2E；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(tui): 实现项目选择、Daemon 拉起与 endpoint-ready 协议`

##### EP-1203 实现 TUI Markdown 与代码高亮渲染

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v0.1 / M-09 TUI 核心
- **需求追踪**：RQ-115；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 Markdown/代码块渲染器与主题适配，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 TUI Markdown 与代码高亮渲染”所需的最小闭环；主交付物为：Markdown/代码块渲染器与主题适配。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-1003。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-tui/`、`crates/apex-tui/`、`crates/apex-client-sdk/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Markdown/代码块渲染器与主题适配；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-215：Markdown、宽字符、代码块快照测试；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(tui): 实现 TUI Markdown 与代码高亮渲染`

##### EP-1204 实现 TUI 流式输出、打断与状态反馈

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v0.1 / M-09 TUI 核心
- **需求追踪**：RQ-009、RQ-114；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 流式增量渲染、取消、超时和恢复状态反馈，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 TUI 流式输出、打断与状态反馈”所需的最小闭环；主交付物为：流式增量渲染、取消、超时和恢复状态反馈。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0307、EP-1003。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-tui/`、`crates/apex-tui/`、`crates/apex-client-sdk/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：流式增量渲染、取消、超时和恢复状态反馈；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-216：流式、取消、重连与重复帧测试；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：竞态、饥饿或无界队列。使用确定性时钟/调度器并验证取消后的资源回收。
- **建议提交**：`feat(tui): 实现 TUI 流式输出、打断与状态反馈`

##### EP-1205 实现 CLI 启动、首次运行与会话恢复入口

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v0.1 / M-09 TUI 核心
- **需求追踪**：RQ-009；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 CLI、双击启动、首次配置、最近项目与会话恢复流程，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 CLI 启动、首次运行与会话恢复入口”所需的最小闭环；主交付物为：CLI、双击启动、首次配置、最近项目与会话恢复流程。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0201、EP-0808、EP-1001。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-tui/`、`crates/apex-tui/`、`crates/apex-client-sdk/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：CLI、双击启动、首次配置、最近项目与会话恢复流程；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-217：全新环境、已有配置、异常退出恢复 E2E；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：原子性或时序错误导致数据损坏。故障注入覆盖断电、重复、乱序和部分写入。
- **建议提交**：`feat(tui): 实现 CLI 启动、首次运行与会话恢复入口`

#### M-10 TUI Spec/Permission

##### EP-1004 实现 TUI Spec/Approval/Skip 面板

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v0.1 / M-10 TUI Spec/Permission
- **需求追踪**：RQ-036–041；WI：WI-v0.1-68
- **目标**：交付 TUI spec UI，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 TUI Spec/Approval/Skip 面板”所需的最小闭环；主交付物为：TUI spec UI。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-1003/0408。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-tui/`、`crates/apex-spec/`、`crates/apex-permission/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：TUI spec UI；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-170`：审批失效/审计；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(tui): 实现 TUI Spec/Approval/Skip 面板`

##### EP-1005 实现 TUI Permission Ask/Allow/Deny UI

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v0.1 / M-10 TUI Spec/Permission
- **需求追踪**：RQ-047–054；WI：WI-v0.1-69
- **目标**：交付 TUI permission UI，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 TUI Permission Ask/Allow/Deny UI”所需的最小闭环；主交付物为：TUI permission UI。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0513、EP-1201、EP-1003。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-tui/`、`crates/apex-spec/`、`crates/apex-permission/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：TUI permission UI；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-171`：证据/不可绕过；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(tui): 实现 TUI Permission Ask/Allow/Deny UI`

### 9.2 v0.2

#### M-08 Context Engine

##### EP-1206 补齐前缀缓存固定与失效测试

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S6 / v0.2 / M-08 Context Engine
- **需求追踪**：RQ-074；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 prefix cache pin/evict/invalidate 测试夹具与证据，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“补齐前缀缓存固定与失效测试”所需的最小闭环；主交付物为：prefix cache pin/evict/invalidate 测试夹具与证据。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0602、EP-0801。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-context/`、`crates/apex-provider/`、`crates/apex-test-support/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：prefix cache pin/evict/invalidate 测试夹具与证据；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-219：稳定前缀命中、epoch 变化失效、预算边界；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`test(context): 补齐前缀缓存固定与失效测试`

#### M-09 TUI 核心

##### EP-1009 实现 TUI 共享逻辑终端 UI

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v0.2 / M-09 TUI 核心
- **需求追踪**：RQ-057/058；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 TUI terminal，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 TUI 共享逻辑终端 UI”所需的最小闭环；主交付物为：TUI terminal。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-1003/0520。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-tui/`、`crates/apex-tui/`、`crates/apex-client-sdk/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：TUI terminal；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-175`：channel/resize；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(tui): 实现 TUI 共享逻辑终端 UI`

#### M-11 Checkpoint

##### EP-0604 实现 60/70/80/90 watermark 状态

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S6 / v0.2 / M-11 Checkpoint
- **需求追踪**：RQ-074；WI：WI-v0.2-04
- **目标**：交付 Watermark store，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 60/70/80/90 watermark 状态”所需的最小闭环；主交付物为：Watermark store。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0601/0210。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-checkpoint/`、`crates/apex-storage/`、`crates/apex-tui/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Watermark store；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-98`：跨阈值只触发一次；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(checkpoint): 实现 60/70/80/90 watermark 状态`

##### EP-0605 实现 Tool-specific SnipHinter

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S6 / v0.2 / M-11 Checkpoint
- **需求追踪**：RQ-074；WI：WI-v0.2-05
- **目标**：交付 Snip strategies，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Tool-specific SnipHinter”所需的最小闭环；主交付物为：Snip strategies。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0516/0602。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-checkpoint/`、`crates/apex-storage/`、`crates/apex-tui/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Snip strategies；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-99`：错误/首尾/结构保留；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(checkpoint): 实现 Tool-specific SnipHinter`

##### EP-0606 实现 prune 引用占位与再取回

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S6 / v0.2 / M-11 Checkpoint
- **需求追踪**：RQ-074/077；WI：WI-v0.2-06
- **目标**：交付 ContextReference，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 prune 引用占位与再取回”所需的最小闭环；主交付物为：ContextReference。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0217/0602。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-checkpoint/`、`crates/apex-storage/`、`crates/apex-tui/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：ContextReference；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-100`：hash/引用有效；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(checkpoint): 实现 prune 引用占位与再取回`

##### EP-0607 实现独立摘要 Provider 与当前模型 fallback

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S6 / v0.2 / M-11 Checkpoint
- **需求追踪**：RQ-075；WI：WI-v0.2-07
- **目标**：交付 Summary adapter，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现独立摘要 Provider 与当前模型 fallback”所需的最小闭环；主交付物为：Summary adapter。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0801/0603。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-checkpoint/`、`crates/apex-storage/`、`crates/apex-tui/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Summary adapter；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-101`：失败/降级/专属 metadata；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(checkpoint): 实现独立摘要 Provider 与当前模型 fallback`

##### EP-0608 实现 Checkpoint Manifest schema

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S6 / v0.2 / M-11 Checkpoint
- **需求追踪**：RQ-076/077；WI：WI-v0.2-08
- **目标**：交付 checkpoint.md model，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Checkpoint Manifest schema”所需的最小闭环；主交付物为：checkpoint.md model。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0401/0602。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-checkpoint/`、`crates/apex-storage/`、`crates/apex-tui/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：checkpoint.md model；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-102`：预算/Active Intent；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：原子性或时序错误导致数据损坏。故障注入覆盖断电、重复、乱序和部分写入。
- **建议提交**：`feat(checkpoint): 实现 Checkpoint Manifest schema`

##### EP-0609 实现 Checkpoint chunk/attachment CAS writer

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S6 / v0.2 / M-11 Checkpoint
- **需求追踪**：RQ-077；WI：WI-v0.2-09
- **目标**：交付 CheckpointStore，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Checkpoint chunk/attachment CAS writer”所需的最小闭环；主交付物为：CheckpointStore。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0217/0608。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-checkpoint/`、`crates/apex-storage/`、`crates/apex-tui/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：CheckpointStore；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-103`：内容寻址/断块；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：原子性或时序错误导致数据损坏。故障注入覆盖断电、重复、乱序和部分写入。
- **建议提交**：`feat(checkpoint): 实现 Checkpoint chunk/attachment CAS writer`

##### EP-0610 接入 Turn/损处理/暂停/高风险写触发点

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S6 / v0.2 / M-11 Checkpoint
- **需求追踪**：RQ-076；WI：WI-v0.2-10
- **目标**：交付 Checkpoint hooks，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“接入 Turn/损处理/暂停/高风险写触发点”所需的最小闭环；主交付物为：Checkpoint hooks。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0307/0515/0609。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-checkpoint/`、`crates/apex-storage/`、`crates/apex-tui/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Checkpoint hooks；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-104`：四类触发全覆盖；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(checkpoint): 接入 Turn/损处理/暂停/高风险写触发点`

##### EP-0611 实现 Checkpoint reconstruction

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S6 / v0.2 / M-11 Checkpoint
- **需求追踪**：AC-010；WI：WI-v0.2-11
- **目标**：交付 ReconstructedSession，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Checkpoint reconstruction”所需的最小闭环；主交付物为：ReconstructedSession。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0212/0609/0610。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-checkpoint/`、`crates/apex-storage/`、`crates/apex-tui/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：ReconstructedSession；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-105`：无损重建；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：原子性或时序错误导致数据损坏。故障注入覆盖断电、重复、乱序和部分写入。
- **建议提交**：`feat(checkpoint): 实现 Checkpoint reconstruction`

##### EP-0612 实现 Checkpoint pin/120/365 retention

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S6 / v0.2 / M-11 Checkpoint
- **需求追踪**：RQ-078；WI：WI-v0.2-12
- **目标**：交付 Retention job，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Checkpoint pin/120/365 retention”所需的最小闭环；主交付物为：Retention job。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0222/0609。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-checkpoint/`、`crates/apex-storage/`、`crates/apex-tui/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Retention job；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-106`：Pinned GC root；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：原子性或时序错误导致数据损坏。故障注入覆盖断电、重复、乱序和部分写入。
- **建议提交**：`feat(checkpoint): 实现 Checkpoint pin/120/365 retention`

##### EP-1008 实现 TUI Checkpoint/Memory UI

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v0.2 / M-11 Checkpoint
- **需求追踪**：RQ-074–083；WI：WI-v0.6-09, WI-v0.2-23
- **目标**：交付 TUI context UI，形成可独立验证、可单独回滚的闭环。
- **范围**：仅保留 Checkpoint UI；Memory UI 移至 EP-1031。
- **非范围**：不包含：EP-1031；不顺带修改其他版本能力。
- **前置依赖**：EP-1003/0616。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-checkpoint/`、`crates/apex-storage/`、`crates/apex-tui/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：TUI context UI；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-174`：引用时机/删除导出；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：原子性或时序错误导致数据损坏。故障注入覆盖断电、重复、乱序和部分写入。
- **建议提交**：`feat(checkpoint): 实现 TUI Checkpoint/Memory UI`

#### M-12 内容快照

##### EP-0217 实现 CAS put/open/verify

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S2 / v0.2 / M-12 内容快照
- **需求追踪**：RQ-070/077；WI：WI-v0.2-02
- **目标**：交付 ContentStore，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 CAS put/open/verify”所需的最小闭环；主交付物为：ContentStore。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0201/0105。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-snapshot/`、`crates/apex-storage/`、`crates/apex-daemon/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：ContentStore；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-36`：hash/断块/幂等；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(snapshot): 实现 CAS put/open/verify`

##### EP-0218 实现文件事实索引与 reconcile marker

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S2 / v0.2 / M-12 内容快照
- **需求追踪**：RQ-025/026；WI：WI-v0.2-03
- **目标**：交付 file_sync_state，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现文件事实索引与 reconcile marker”所需的最小闭环；主交付物为：file_sync_state。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0212/0214/0217。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-snapshot/`、`crates/apex-storage/`、`crates/apex-daemon/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：file_sync_state；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-37`：DB/文件崩溃组合；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(snapshot): 实现文件事实索引与 reconcile marker`

##### EP-1202 实现 Turn 边界内容快照与恢复校验

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S2 / v0.2 / M-12 内容快照
- **需求追踪**：RQ-069、RQ-070；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 Turn pre/post Snapshot、PatchSet 绑定、混合时点拒绝与恢复校验，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Turn 边界内容快照与恢复校验”所需的最小闭环；主交付物为：Turn pre/post Snapshot、PatchSet 绑定、混合时点拒绝与恢复校验。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0217、EP-0218、EP-0307。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-snapshot/`、`crates/apex-storage/`、`crates/apex-daemon/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Turn pre/post Snapshot、PatchSet 绑定、混合时点拒绝与恢复校验；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-218：快照一致性、回滚、混合时点拒绝；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：原子性或时序错误导致数据损坏。故障注入覆盖断电、重复、乱序和部分写入。
- **建议提交**：`feat(snapshot): 实现 Turn 边界内容快照与恢复校验`

#### M-13 持久终端

##### EP-0206 实现进程树 supervisor Port

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S2 / v0.2 / M-13 持久终端
- **需求追踪**：RQ-057/058；WI：WI-v0.2-15
- **目标**：交付 ProcessTree Port，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现进程树 supervisor Port”所需的最小闭环；主交付物为：ProcessTree Port。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0201。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-terminal/`、`crates/apex-tool-gateway/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：ProcessTree Port；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-25`：子孙进程终止；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(terminal): 实现进程树 supervisor Port`

##### EP-0517 实现 Unix PTY 持久终端

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S5 / v0.2 / M-13 持久终端
- **需求追踪**：RQ-057/058；WI：WI-v0.2-16
- **目标**：交付 PTY adapter，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Unix PTY 持久终端”所需的最小闭环；主交付物为：PTY adapter。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0206/0515。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-terminal/`、`crates/apex-tool-gateway/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：PTY adapter；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-88`：输入/resize/kill tree；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(terminal): 实现 Unix PTY 持久终端`

##### EP-0518 实现 Windows ConPTY 持久终端

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S5 / v0.2 / M-13 持久终端
- **需求追踪**：RQ-057/058；WI：WI-v0.2-17
- **目标**：交付 ConPTY adapter，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Windows ConPTY 持久终端”所需的最小闭环；主交付物为：ConPTY adapter。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0206/0515。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-terminal/`、`crates/apex-tool-gateway/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：ConPTY adapter；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-89`：Job Object/编码；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(terminal): 实现 Windows ConPTY 持久终端`

##### EP-0520 实现共享逻辑终端与 Agent channel attribution

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S5 / v0.2 / M-13 持久终端
- **需求追踪**：RQ-058/073；WI：WI-v0.2-18
- **目标**：交付 LogicalTerminal，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现共享逻辑终端与 Agent channel attribution”所需的最小闭环；主交付物为：LogicalTerminal。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0517/0518。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-terminal/`、`crates/apex-tool-gateway/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：LogicalTerminal；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-91`：通道隔离/trace；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(terminal): 实现共享逻辑终端与 Agent channel attribution`

##### EP-0521 实现终端输出 ring buffer/backpressure

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S5 / v0.2 / M-13 持久终端
- **需求追踪**：RQ-058/114；WI：WI-v0.2-19
- **目标**：交付 Bounded stream，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现终端输出 ring buffer/backpressure”所需的最小闭环；主交付物为：Bounded stream。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0520/0219。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-terminal/`、`crates/apex-tool-gateway/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Bounded stream；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-92`：慢客户端/1GiB 输出；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(terminal): 实现终端输出 ring buffer/backpressure`

##### EP-0522 实现中断 Tool recovery 分类

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S5 / v0.2 / M-13 持久终端
- **需求追踪**：RQ-068/072；WI：WI-v0.2-20
- **目标**：交付 Interrupted/Unknown state，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现中断 Tool recovery 分类”所需的最小闭环；主交付物为：Interrupted/Unknown state。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0515/0222。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-terminal/`、`crates/apex-tool-gateway/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Interrupted/Unknown state；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-93`：幂等与未知副作用；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(terminal): 实现中断 Tool recovery 分类`

#### M-25a 发布工程

##### EP-0222 实现 120/365 天 Session 归档与只读挂载

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S2 / v0.2 / M-25a 发布工程
- **需求追踪**：RQ-106；WI：WI-v0.2-14
- **目标**：交付 ArchiveStore，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 120/365 天 Session 归档与只读挂载”所需的最小闭环；主交付物为：ArchiveStore。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0210/0217。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`.github/workflows/`、`xtask/`、`packaging/`、`docs/release/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：ArchiveStore；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-41`：归档/恢复/删除；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：门禁误报/漏报或产物不可复现。固定工具链，保留机器可读证据并提供可审计豁免。
- **建议提交**：`feat(release): 实现 120/365 天 Session 归档与只读挂载`

### 9.3 v0.3

#### M-14 AST 权限

##### EP-0202 实现 Home/config/key/runtime 权限诊断

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S2 / v0.3 / M-14 AST 权限
- **需求追踪**：RQ-091/109；WI：WI-v0.3-13
- **目标**：交付 PermissionDoctor，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Home/config/key/runtime 权限诊断”所需的最小闭环；主交付物为：PermissionDoctor。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0201。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-command-ast/`、`crates/apex-permission/`、`crates/apex-policy/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：PermissionDoctor；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-21`：0600/ACL 正负测试；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：误放行或敏感信息泄漏。默认拒绝；正例、负例和绕过语料必须同时覆盖。
- **建议提交**：`feat(policy): 实现 Home/config/key/runtime 权限诊断`

##### EP-0501 定义 CommandAst→CommandSemantics IR

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S5 / v0.3 / M-14 AST 权限
- **需求追踪**：RQ-050/051；WI：WI-v0.3-01
- **目标**：交付 AST semantic types，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“定义 CommandAst→CommandSemantics IR”所需的最小闭环；主交付物为：AST semantic types。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0104/0107。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-command-ast/`、`crates/apex-permission/`、`crates/apex-policy/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：AST semantic types；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-72`：IR golden fixture；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(policy): 定义 CommandAst→CommandSemantics IR`

##### EP-0502 实现 sh/bash/zsh tree-sitter parser

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S5 / v0.3 / M-14 AST 权限
- **需求追踪**：RQ-051；WI：WI-v0.3-02
- **目标**：交付 POSIX analyzer，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 sh/bash/zsh tree-sitter parser”所需的最小闭环；主交付物为：POSIX analyzer。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0501。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-command-ast/`、`crates/apex-permission/`、`crates/apex-policy/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：POSIX analyzer；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-73`：quote/pipeline/subshell；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(policy): 实现 sh/bash/zsh tree-sitter parser`

##### EP-0503 实现 PowerShell 7 parser adapter

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S5 / v0.3 / M-14 AST 权限
- **需求追踪**：RQ-051；WI：WI-v0.3-03
- **目标**：交付 PowerShell analyzer，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 PowerShell 7 parser adapter”所需的最小闭环；主交付物为：PowerShell analyzer。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0501。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-command-ast/`、`crates/apex-permission/`、`crates/apex-policy/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：PowerShell analyzer；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-74`：cmdlet/provider/scriptblock；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(policy): 实现 PowerShell 7 parser adapter`

##### EP-0504 实现 cmd.exe parser adapter

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S5 / v0.3 / M-14 AST 权限
- **需求追踪**：RQ-051；WI：WI-v0.3-04
- **目标**：交付 Cmd analyzer，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 cmd.exe parser adapter”所需的最小闭环；主交付物为：Cmd analyzer。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0501。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-command-ast/`、`crates/apex-permission/`、`crates/apex-policy/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Cmd analyzer；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-75`：expansion/redirect/call；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(policy): 实现 cmd.exe parser adapter`

##### EP-0505 实现 arity rule registry

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S5 / v0.3 / M-14 AST 权限
- **需求追踪**：RQ-050/052；WI：WI-v0.3-05
- **目标**：交付 Versioned arity rules，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 arity rule registry”所需的最小闭环；主交付物为：Versioned arity rules。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0501–0504。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-command-ast/`、`crates/apex-permission/`、`crates/apex-policy/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Versioned arity rules；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-76`：rm/git/curl/build fixture；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(policy): 实现 arity rule registry`

##### EP-0506 实现路径 canonicalization 与 Scope overlap

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S5 / v0.3 / M-14 AST 权限
- **需求追踪**：RQ-052/060；WI：WI-v0.3-06
- **目标**：交付 CanonicalPathScope，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现路径 canonicalization 与 Scope overlap”所需的最小闭环；主交付物为：CanonicalPathScope。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0201/0501。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-command-ast/`、`crates/apex-permission/`、`crates/apex-policy/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：CanonicalPathScope；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-77`：symlink/case/不存在；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(policy): 实现路径 canonicalization 与 Scope overlap`

##### EP-0507 实现网络目标规范化与重定向复核

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S5 / v0.3 / M-14 AST 权限
- **需求追踪**：RQ-052；WI：WI-v0.3-07
- **目标**：交付 NetworkResource，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现网络目标规范化与重定向复核”所需的最小闭环；主交付物为：NetworkResource。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0501。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-command-ast/`、`crates/apex-permission/`、`crates/apex-policy/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：NetworkResource；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-78`：DNS/private/redirect；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(policy): 实现网络目标规范化与重定向复核`

##### EP-0508 实现环境/凭据访问分类与清洗

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S5 / v0.3 / M-14 AST 权限
- **需求追踪**：RQ-052/092；WI：WI-v0.3-08
- **目标**：交付 Secret/env policy，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现环境/凭据访问分类与清洗”所需的最小闭环；主交付物为：Secret/env policy。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0202/0501。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-command-ast/`、`crates/apex-permission/`、`crates/apex-policy/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Secret/env policy；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-79`：Key/Token canary；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(policy): 实现环境/凭据访问分类与清洗`

##### EP-0509 实现 Trust→Mode→HardDeny 单调决策顺序

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S5 / v0.3 / M-14 AST 权限
- **需求追踪**：RQ-047–050/056；WI：WI-v0.3-09
- **目标**：交付 Policy pipeline，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Trust→Mode→HardDeny 单调决策顺序”所需的最小闭环；主交付物为：Policy pipeline。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0409/0501。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-command-ast/`、`crates/apex-permission/`、`crates/apex-policy/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Policy pipeline；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-80`：后层不得覆盖 Deny；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(policy): 实现 Trust→Mode→HardDeny 单调决策顺序`

##### EP-0510 实现 plan/ask/allow 模式矩阵

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S5 / v0.3 / M-14 AST 权限
- **需求追踪**：RQ-047–049；WI：WI-v0.3-10
- **目标**：交付 Mode evaluator，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 plan/ask/allow 模式矩阵”所需的最小闭环；主交付物为：Mode evaluator。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0509。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-command-ast/`、`crates/apex-permission/`、`crates/apex-policy/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Mode evaluator；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-81`：四类输入矩阵；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(policy): 实现 plan/ask/allow 模式矩阵`

##### EP-0523 接入可选 OS sandbox capability

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S5 / v0.3 / M-14 AST 权限
- **需求追踪**：RQ-055；WI：WI-v0.3-14
- **目标**：交付 Sandbox adapter，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“接入可选 OS sandbox capability”所需的最小闭环；主交付物为：Sandbox adapter。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0515/0206。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-command-ast/`、`crates/apex-permission/`、`crates/apex-policy/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Sandbox adapter；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-94`：不可用时降级/required 阻塞；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(policy): 接入可选 OS sandbox capability`

##### EP-0819 接入 AST 环境变量与秘密策略

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S5 / v0.3 / M-14 AST 权限
- **需求追踪**：RQ-048、RQ-049、RQ-062；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 环境变量读取分类、秘密引用识别与权限决策输入，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“接入 AST 环境变量与秘密策略”所需的最小闭环；主交付物为：环境变量读取分类、秘密引用识别与权限决策输入。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0505、EP-0508、EP-0817。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-command-ast/`、`crates/apex-permission/`、`crates/apex-policy/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：环境变量读取分类、秘密引用识别与权限决策输入；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-232：允许列表、秘密变量、动态展开与拒绝分支；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：误放行或敏感信息泄漏。默认拒绝；正例、负例和绕过语料必须同时覆盖。
- **建议提交**：`feat(policy): 接入 AST 环境变量与秘密策略`

#### M-15 验证规则

##### EP-0411 实现 PostToolUse 轻量安全/格式/语法门

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S4 / v0.3 / M-15 验证规则
- **需求追踪**：RQ-042；WI：WI-v0.3-15
- **目标**：交付 Lightweight gate，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 PostToolUse 轻量安全/格式/语法门”所需的最小闭环；主交付物为：Lightweight gate。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0409/0515。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-verification/`、`crates/apex-spec/`、`xtask/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Lightweight gate；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-67`：单文件修改失败阻断；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：误放行或敏感信息泄漏。默认拒绝；正例、负例和绕过语料必须同时覆盖。
- **建议提交**：`feat(verification): 实现 PostToolUse 轻量安全/格式/语法门`

##### EP-0412 实现增量批次重型检查编排

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S4 / v0.3 / M-15 验证规则
- **需求追踪**：RQ-043；WI：WI-v0.3-16
- **目标**：交付 Batch runner，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现增量批次重型检查编排”所需的最小闭环；主交付物为：Batch runner。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0410/0411。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-verification/`、`crates/apex-spec/`、`xtask/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Batch runner；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-68`：增量范围/完成门；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(verification): 实现增量批次重型检查编排`

##### EP-0413 实现受限自动修复子任务

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S4 / v0.3 / M-15 验证规则
- **需求追踪**：RQ-044；WI：WI-v0.3-17
- **目标**：交付 Repair plan，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现受限自动修复子任务”所需的最小闭环；主交付物为：Repair plan。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0411/0711。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-verification/`、`crates/apex-spec/`、`xtask/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Repair plan；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-69`：2 轮默认、范围不扩；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(verification): 实现受限自动修复子任务`

##### EP-0414 实现最终 Verification evidence 聚合

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S4 / v0.3 / M-15 验证规则
- **需求追踪**：RQ-040/046；WI：WI-v0.3-18
- **目标**：交付 Evidence aggregator，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现最终 Verification evidence 聚合”所需的最小闭环；主交付物为：Evidence aggregator。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0404/0412。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-verification/`、`crates/apex-spec/`、`xtask/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Evidence aggregator；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-70`：AC/覆盖率/风险映射；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(verification): 实现最终 Verification evidence 聚合`

##### EP-0415 实现用户确认/自动完成策略

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S4 / v0.3 / M-15 验证规则
- **需求追踪**：RQ-041；WI：WI-v0.3-19
- **目标**：交付 Completion decision，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现用户确认/自动完成策略”所需的最小闭环；主交付物为：Completion decision。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0414/0308。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-verification/`、`crates/apex-spec/`、`xtask/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Completion decision；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-71`：未确认不得完成；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(verification): 实现用户确认/自动完成策略`

#### M-17 项目信任

##### EP-0511 实现 Once/Run/Session/Project grant 存储

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S5 / v0.3 / M-17 项目信任
- **需求追踪**：RQ-054；WI：WI-v0.3-11
- **目标**：交付 Grant service，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Once/Run/Session/Project grant 存储”所需的最小闭环；主交付物为：Grant service。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0210/0509。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-trust/`、`crates/apex-platform/`、`crates/apex-permission/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Grant service；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-82`：过期/并发消费；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：原子性或时序错误导致数据损坏。故障注入覆盖断电、重复、乱序和部分写入。
- **建议提交**：`feat(trust): 实现 Once/Run/Session/Project grant 存储`

##### EP-0512 实现 Project Trust Gate

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S5 / v0.3 / M-17 项目信任
- **需求追踪**：RQ-056；WI：WI-v0.3-12
- **目标**：交付 Trust state，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Project Trust Gate”所需的最小闭环；主交付物为：Trust state。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0210/0509。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-trust/`、`crates/apex-platform/`、`crates/apex-permission/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Trust state；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-83`：确认前禁止读取；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(trust): 实现 Project Trust Gate`

### 9.4 v0.4

#### M-04 Provider 核心

##### EP-0810 接入 Session/Profile/DAG Provider 继承

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S8 / v0.4 / M-04 Provider 核心
- **需求追踪**：RQ-090；WI：WI-v0.4-10
- **目标**：交付 Route resolver，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“接入 Session/Profile/DAG Provider 继承”所需的最小闭环；主交付物为：Route resolver。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0701/0808。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-provider/`、`crates/apex-domain/`、`crates/apex-test-support/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Route resolver；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-143`：覆盖优先级；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：竞态、饥饿或无界队列。使用确定性时钟/调度器并验证取消后的资源回收。
- **建议提交**：`feat(provider): 接入 Session/Profile/DAG Provider 继承`

#### M-16 Subagent 与写路径

##### EP-0701 实现 AgentProfile 与 capability ceiling

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S7 / v0.4 / M-16 Subagent 与写路径
- **需求追踪**：RQ-090；WI：WI-v0.4-02
- **目标**：交付 Profile model，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 AgentProfile 与 capability ceiling”所需的最小闭环；主交付物为：Profile model。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0403/0808。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-agent-runtime/`、`crates/apex-storage/`、`crates/apex-tool-gateway/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Profile model；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-112`：继承/覆盖边界；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(agent): 实现 AgentProfile 与 capability ceiling`

##### EP-0702 实现父 Agent→Subagent Provider/model 继承

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S7 / v0.4 / M-16 Subagent 与写路径
- **需求追踪**：RQ-090；WI：WI-v0.4-03
- **目标**：交付 Route inheritance，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现父 Agent→Subagent Provider/model 继承”所需的最小闭环；主交付物为：Route inheritance。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0701/0809。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-agent-runtime/`、`crates/apex-storage/`、`crates/apex-tool-gateway/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Route inheritance；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-113`：DAG 显式覆盖；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(agent): 实现父 Agent→Subagent Provider/model 继承`

##### EP-0703 实现 exact_task_description/write_paths 校验

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S7 / v0.4 / M-16 Subagent 与写路径
- **需求追踪**：RQ-059/073；WI：WI-v0.4-04
- **目标**：交付 AgentExecutionSpec，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 exact_task_description/write_paths 校验”所需的最小闭环；主交付物为：AgentExecutionSpec。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0403/0506。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-agent-runtime/`、`crates/apex-storage/`、`crates/apex-tool-gateway/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：AgentExecutionSpec；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-114`：空任务/空路径拒绝；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(agent): 实现 exact_task_description/write_paths 校验`

##### EP-0707 实现全局/写 Agent/Provider 限流

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S7 / v0.4 / M-16 Subagent 与写路径
- **需求追踪**：RQ-063；WI：WI-v0.4-08, WI-v0.7-04
- **目标**：交付 Limiters，形成可独立验证、可单独回滚的闭环。
- **范围**：仅保留通用全局/Agent/Provider/写入限流原语；DAG 接入移至 EP-0723。
- **非范围**：不包含：EP-0723；不顺带修改其他版本能力。
- **前置依赖**：EP-0107、EP-0108。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-agent-runtime/`、`crates/apex-storage/`、`crates/apex-tool-gateway/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Limiters；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-118`：硬上限/动态下调；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：竞态、饥饿或无界队列。使用确定性时钟/调度器并验证取消后的资源回收。
- **建议提交**：`feat(agent): 实现全局/写 Agent/Provider 限流`

##### EP-0708 将 CanonicalPathScope 接入 Scheduler

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S7 / v0.4 / M-16 Subagent 与写路径
- **需求追踪**：RQ-060；WI：WI-v0.4-05
- **目标**：交付 Claim plan，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“将 CanonicalPathScope 接入 Scheduler”所需的最小闭环；主交付物为：Claim plan。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0506/0705。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-agent-runtime/`、`crates/apex-storage/`、`crates/apex-tool-gateway/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Claim plan；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-119`：父子重叠；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(agent): 将 CanonicalPathScope 接入 Scheduler`

##### EP-0709 实现 Claim lease TTL/fencing

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S7 / v0.4 / M-16 Subagent 与写路径
- **需求追踪**：RQ-060；WI：WI-v0.4-06
- **目标**：交付 WriteClaimService，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Claim lease TTL/fencing”所需的最小闭环；主交付物为：WriteClaimService。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0208/0708。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-agent-runtime/`、`crates/apex-storage/`、`crates/apex-tool-gateway/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：WriteClaimService；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-120`：过期 owner 不能提交；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(agent): 实现 Claim lease TTL/fencing`

##### EP-0710 实现父 Agent write_paths 预留

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S7 / v0.4 / M-16 Subagent 与写路径
- **需求追踪**：RQ-059；WI：WI-v0.4-07
- **目标**：交付 Parent reservation，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现父 Agent write_paths 预留”所需的最小闭环；主交付物为：Parent reservation。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0703/0709。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-agent-runtime/`、`crates/apex-storage/`、`crates/apex-tool-gateway/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Parent reservation；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-121`：嵌套 fail-fast；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(agent): 实现父 Agent write_paths 预留`

#### M-18 Activity Panel

##### EP-0313 实现 AgentActivityView durable/transient 投影

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S3 / v0.4 / M-18 Activity Panel
- **需求追踪**：RQ-073；WI：WI-v0.4-09
- **目标**：交付 Activity query，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 AgentActivityView durable/transient 投影”所需的最小闭环；主交付物为：Activity query。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0212/0304。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-activity/`、`crates/apex-tui/`、`apps/apex-desktop/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Activity query；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-55`：Skill/MCP/Subagent 展示；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(activity): 实现 AgentActivityView durable/transient 投影`

##### EP-1006 实现 TUI Agent/Skill/MCP/Subagent 活动面板

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v0.4 / M-18 Activity Panel
- **需求追踪**：RQ-073；WI：WI-v0.4-11, WI-v0.5-16
- **目标**：交付 TUI activity UI，形成可独立验证、可单独回滚的闭环。
- **范围**：仅保留 Activity Projection/Panel；扩展管理移至 EP-1030。
- **非范围**：不包含：EP-1030；不顺带修改其他版本能力。
- **前置依赖**：EP-1003/0313。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-activity/`、`crates/apex-tui/`、`apps/apex-desktop/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：TUI activity UI；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-172`：精确任务描述；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(activity): 实现 TUI Agent/Skill/MCP/Subagent 活动面板`

### 9.5 v0.5

#### M-18 Activity Panel

##### EP-1030 实现扩展管理面板

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v0.5 / M-18 Activity Panel
- **需求追踪**：RQ-083、RQ-087、RQ-091；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 Skill/MCP/Plugin 状态、启停、错误与诊断 UI，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现扩展管理面板”所需的最小闭环；主交付物为：Skill/MCP/Plugin 状态、启停、错误与诊断 UI。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0907、EP-0913、EP-0914、EP-1006。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-activity/`、`crates/apex-tui/`、`apps/apex-desktop/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Skill/MCP/Plugin 状态、启停、错误与诊断 UI；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-235：三类扩展生命周期、失败隔离与状态一致性；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(activity): 实现扩展管理面板`

#### M-19a Skills

##### EP-0901 实现 SkillSource/Scanner Trait

- **生命周期 / 状态 / 工作量**：Active / 未开始 / S
- **阶段 / 版本 / 模块**：S9 / v0.5 / M-19a Skills
- **需求追踪**：RQ-094；WI：WI-v0.5-01, WI-v0.5-08
- **目标**：交付 Skill scanner Port，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 SkillSource/Scanner Trait”所需的最小闭环；主交付物为：Skill scanner Port。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0110/0201。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-skill/`、`crates/apex-tool-gateway/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Skill scanner Port；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-150`：来源/错误隔离；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(skill): 实现 SkillSource/Scanner Trait`

##### EP-0902 实现 Claude user/project 扫描器

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S9 / v0.5 / M-19a Skills
- **需求追踪**：RQ-094；WI：WI-v0.5-02
- **目标**：交付 Claude catalog，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Claude user/project 扫描器”所需的最小闭环；主交付物为：Claude catalog。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0901。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-skill/`、`crates/apex-tool-gateway/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Claude catalog；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-151`：目录/标准 frontmatter；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(skill): 实现 Claude user/project 扫描器`

##### EP-0903 实现 Codex user/project 扫描器

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S9 / v0.5 / M-19a Skills
- **需求追踪**：RQ-094；WI：WI-v0.5-03
- **目标**：交付 Codex catalog，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Codex user/project 扫描器”所需的最小闭环；主交付物为：Codex catalog。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0901。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-skill/`、`crates/apex-tool-gateway/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Codex catalog；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-152`：兼容 fixture；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(skill): 实现 Codex user/project 扫描器`

##### EP-0904 实现 Apex user/project 扫描器

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S9 / v0.5 / M-19a Skills
- **需求追踪**：RQ-094；WI：WI-v0.5-04
- **目标**：交付 Apex catalog，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Apex user/project 扫描器”所需的最小闭环；主交付物为：Apex catalog。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0901/0201。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-skill/`、`crates/apex-tool-gateway/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Apex catalog；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-153`：优先级/冲突；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(skill): 实现 Apex user/project 扫描器`

##### EP-0905 实现 `apex:` frontmatter 阶段绑定

- **生命周期 / 状态 / 工作量**：Active / 未开始 / S
- **阶段 / 版本 / 模块**：S9 / v0.5 / M-19a Skills
- **需求追踪**：RQ-095；WI：WI-v0.5-05
- **目标**：交付 Extension schema，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 `apex:` frontmatter 阶段绑定”所需的最小闭环；主交付物为：Extension schema。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0401/0901。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-skill/`、`crates/apex-tool-gateway/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Extension schema；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-154`：未知字段保留；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(skill): 实现 apex: frontmatter 阶段绑定`

##### EP-0906 实现 Skill content hash/signature trust

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S9 / v0.5 / M-19a Skills
- **需求追踪**：RQ-096；WI：WI-v0.5-06
- **目标**：交付 Trust record，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Skill content hash/signature trust”所需的最小闭环；主交付物为：Trust record。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0217/0901。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-skill/`、`crates/apex-tool-gateway/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Trust record；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-155`：内容变化失信；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(skill): 实现 Skill content hash/signature trust`

##### EP-0907 将 Skill script/Tool 绑定 Tool Gateway

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S9 / v0.5 / M-19a Skills
- **需求追踪**：RQ-096；WI：WI-v0.5-07
- **目标**：交付 Skill activation，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“将 Skill script/Tool 绑定 Tool Gateway”所需的最小闭环；主交付物为：Skill activation。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0515/0906。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-skill/`、`crates/apex-tool-gateway/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Skill activation；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-156`：脚本不得绕权限；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(skill): 将 Skill script/Tool 绑定 Tool Gateway`

#### M-19b MCP

##### EP-0908 实现 McpSource/Config adapter Trait

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S9 / v0.5 / M-19b MCP
- **需求追踪**：RQ-097；WI：WI-v0.5-09
- **目标**：交付 MCP discovery Port，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 McpSource/Config adapter Trait”所需的最小闭环；主交付物为：MCP discovery Port。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0110/0207。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-mcp/`、`crates/apex-tool-gateway/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：MCP discovery Port；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-157`：未知配置保留；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(mcp): 实现 McpSource/Config adapter Trait`

##### EP-0909 实现 Claude/Cursor/VS Code/Codex/Apex scanner

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S9 / v0.5 / M-19b MCP
- **需求追踪**：RQ-097；WI：WI-v0.5-10
- **目标**：交付 MCP source catalog，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Claude/Cursor/VS Code/Codex/Apex scanner”所需的最小闭环；主交付物为：MCP source catalog。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0908。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-mcp/`、`crates/apex-tool-gateway/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：MCP source catalog；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-158`：五来源 fixture；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(mcp): 实现 Claude/Cursor/VS Code/Codex/Apex scanner`

##### EP-0910 实现 MCP fingerprint/provenance 合并

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S9 / v0.5 / M-19b MCP
- **需求追踪**：RQ-097/099；WI：WI-v0.5-11
- **目标**：交付 Catalog dedupe，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 MCP fingerprint/provenance 合并”所需的最小闭环；主交付物为：Catalog dedupe。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0909/0216。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-mcp/`、`crates/apex-tool-gateway/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Catalog dedupe；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-159`：冲突不静默合并；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(mcp): 实现 MCP fingerprint/provenance 合并`

##### EP-0911 实现 Apex enable override 与显式来源同步

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S9 / v0.5 / M-19b MCP
- **需求追踪**：RQ-099；WI：WI-v0.5-12
- **目标**：交付 Override store，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Apex enable override 与显式来源同步”所需的最小闭环；主交付物为：Override store。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0909/0214。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-mcp/`、`crates/apex-tool-gateway/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Override store；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-160`：hash conflict/回写 diff；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(mcp): 实现 Apex enable override 与显式来源同步`

##### EP-0912 实现 MCP start/stop/stdio 进程树生命周期

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S9 / v0.5 / M-19b MCP
- **需求追踪**：RQ-098；WI：WI-v0.5-13
- **目标**：交付 MCP supervisor，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 MCP start/stop/stdio 进程树生命周期”所需的最小闭环；主交付物为：MCP supervisor。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0206/0515/0911。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-mcp/`、`crates/apex-tool-gateway/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：MCP supervisor；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-161`：发现不启动/一键启停；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(mcp): 实现 MCP start/stop/stdio 进程树生命周期`

##### EP-0913 实现 MCP OAuth state/PKCE/loopback

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S9 / v0.5 / M-19b MCP
- **需求追踪**：RQ-097；WI：WI-v0.5-14
- **目标**：交付 OAuth flow，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 MCP OAuth state/PKCE/loopback”所需的最小闭环；主交付物为：OAuth flow。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0817、EP-0912。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-mcp/`、`crates/apex-tool-gateway/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：OAuth flow；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-162`：state/replay/Secret；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(mcp): 实现 MCP OAuth state/PKCE/loopback`

#### M-20 Plugin

##### EP-0914 实现 Plugin C ABI manifest/capability

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S9 / v0.5 / M-20 Plugin
- **需求追踪**：RQ-100；WI：WI-v0.5-15
- **目标**：交付 Plugin API，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Plugin C ABI manifest/capability”所需的最小闭环；主交付物为：Plugin API。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0107/0110。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-plugin/`、`crates/apex-tool-gateway/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Plugin API；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-163`：FFI 边界/ABI；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(plugin): 实现 Plugin C ABI manifest/capability`

### 9.6 v0.6

#### M-21 Memory

##### EP-0215 实现 watcher 防抖与自写去重

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S2 / v0.6 / M-21 Memory
- **需求追踪**：RQ-028；WI：WI-v0.6-02
- **目标**：交付 Watch service，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 watcher 防抖与自写去重”所需的最小闭环；主交付物为：Watch service。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0214。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-memory/`、`crates/apex-storage/`、`crates/apex-context/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Watch service；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-34`：外部/自身变更 fixture；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(memory): 实现 watcher 防抖与自写去重`

##### EP-0216 实现 Markdown AST 三方合并

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S2 / v0.6 / M-21 Memory
- **需求追踪**：RQ-029；WI：WI-v0.6-03
- **目标**：交付 Reconciler，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Markdown AST 三方合并”所需的最小闭环；主交付物为：Reconciler。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0214/0215。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-memory/`、`crates/apex-storage/`、`crates/apex-context/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Reconciler；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-35`：可合并/冲突/暂停；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：误放行或敏感信息泄漏。默认拒绝；正例、负例和绕过语料必须同时覆盖。
- **建议提交**：`feat(memory): 实现 Markdown AST 三方合并`

##### EP-0613 实现 Memory Markdown parser/writer

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S6 / v0.6 / M-21 Memory
- **需求追踪**：RQ-079/080；WI：WI-v0.6-01, WI-v0.6-08
- **目标**：交付 MemoryStore file side，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Memory Markdown parser/writer”所需的最小闭环；主交付物为：MemoryStore file side。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0214/0401。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-memory/`、`crates/apex-storage/`、`crates/apex-context/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：MemoryStore file side；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-107`：frontmatter/外部编辑；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(memory): 实现 Memory Markdown parser/writer`

##### EP-0614 实现 Memory sensitive proposal gate

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S6 / v0.6 / M-21 Memory
- **需求追踪**：RQ-081；WI：WI-v0.6-04
- **目标**：交付 MemoryWriteDecision，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Memory sensitive proposal gate”所需的最小闭环；主交付物为：MemoryWriteDecision。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0508/0613。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-memory/`、`crates/apex-storage/`、`crates/apex-context/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：MemoryWriteDecision；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-108`：默认阻止/逐次确认；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(memory): 实现 Memory sensitive proposal gate`

##### EP-0615 实现 FTS5 unicode61/jieba tokenizer adapter

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S6 / v0.6 / M-21 Memory
- **需求追踪**：RQ-082；WI：WI-v0.6-05
- **目标**：交付 FTS indexer，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 FTS5 unicode61/jieba tokenizer adapter”所需的最小闭环；主交付物为：FTS indexer。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0208/0613。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-memory/`、`crates/apex-storage/`、`crates/apex-context/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：FTS indexer；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-109`：中英文 fixture；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(memory): 实现 FTS5 unicode61/jieba tokenizer adapter`

##### EP-0616 实现召回排序、引用时机与 trace 记录

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S6 / v0.6 / M-21 Memory
- **需求追踪**：RQ-083；WI：WI-v0.6-06
- **目标**：交付 MemoryRecall，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现召回排序、引用时机与 trace 记录”所需的最小闭环；主交付物为：MemoryRecall。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0615/0307。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-memory/`、`crates/apex-storage/`、`crates/apex-context/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：MemoryRecall；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-110`：scope/score/explain；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(memory): 实现召回排序、引用时机与 trace 记录`

##### EP-0617 实现 Memory delete/export/tombstone

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S6 / v0.6 / M-21 Memory
- **需求追踪**：RQ-083；WI：WI-v0.6-07
- **目标**：交付 Delete/export flow，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Memory delete/export/tombstone”所需的最小闭环；主交付物为：Delete/export flow。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0613/0615。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-memory/`、`crates/apex-storage/`、`crates/apex-context/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Delete/export flow；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-111`：删除后不可召回；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(memory): 实现 Memory delete/export/tombstone`

##### EP-1031 实现 Memory 管理 UI

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v0.6 / M-21 Memory
- **需求追踪**：RQ-075、RQ-076；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 记忆检索、来源查看、编辑/删除与冲突反馈界面，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Memory 管理 UI”所需的最小闭环；主交付物为：记忆检索、来源查看、编辑/删除与冲突反馈界面。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0617、EP-1008。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-memory/`、`crates/apex-storage/`、`crates/apex-context/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：记忆检索、来源查看、编辑/删除与冲突反馈界面；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-236：检索、权限、并发冲突、删除恢复测试；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(memory): 实现 Memory 管理 UI`

### 9.7 v0.7

#### M-22 DAG

##### EP-0704 实现 workflow YAML schema

- **生命周期 / 状态 / 工作量**：Active / 未开始 / S
- **阶段 / 版本 / 模块**：S7 / v0.7 / M-22 DAG
- **需求追踪**：RQ-064/065；WI：WI-v0.7-01
- **目标**：交付 workflow-v1 schema，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 workflow YAML schema”所需的最小闭环；主交付物为：workflow-v1 schema。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0001/0403。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-dag/`、`crates/apex-agent-runtime/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：workflow-v1 schema；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-115`：未知字段/循环；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(dag): 实现 workflow YAML schema`

##### EP-0705 实现 tasks.md→VersionedDagIr 编译

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S7 / v0.7 / M-22 DAG
- **需求追踪**：RQ-064；WI：WI-v0.7-02
- **目标**：交付 DAG compiler，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 tasks.md→VersionedDagIr 编译”所需的最小闭环；主交付物为：DAG compiler。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0704/0403。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-dag/`、`crates/apex-agent-runtime/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：DAG compiler；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-116`：hash/依赖一致；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(dag): 实现 tasks.md→VersionedDagIr 编译`

##### EP-0706 实现 Ready Queue 稳定排序

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S7 / v0.7 / M-22 DAG
- **需求追踪**：RQ-063；WI：WI-v0.7-03
- **目标**：交付 Queue，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Ready Queue 稳定排序”所需的最小闭环；主交付物为：Queue。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0705。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-dag/`、`crates/apex-agent-runtime/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Queue；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-117`：同输入同选择；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(dag): 实现 Ready Queue 稳定排序`

##### EP-0711 实现路径扩展暂停/重新审批

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S7 / v0.7 / M-22 DAG
- **需求追踪**：RQ-062；WI：WI-v0.7-05
- **目标**：交付 Expansion proposal，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现路径扩展暂停/重新审批”所需的最小闭环；主交付物为：Expansion proposal。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0407/0709。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-dag/`、`crates/apex-agent-runtime/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Expansion proposal；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-122`：扩权被阻塞；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(dag): 实现路径扩展暂停/重新审批`

##### EP-0712 实现 DAG 显式 mailbox edge

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S7 / v0.7 / M-22 DAG
- **需求追踪**：RQ-066；WI：WI-v0.7-06
- **目标**：交付 AgentMailbox，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 DAG 显式 mailbox edge”所需的最小闭环；主交付物为：AgentMailbox。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0705/0210。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-dag/`、`crates/apex-agent-runtime/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：AgentMailbox；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-123`：未声明边拒绝；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：竞态、饥饿或无界队列。使用确定性时钟/调度器并验证取消后的资源回收。
- **建议提交**：`feat(dag): 实现 DAG 显式 mailbox edge`

##### EP-0713 实现父 Agent 结构化汇聚

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S7 / v0.7 / M-22 DAG
- **需求追踪**：RQ-066；WI：WI-v0.7-07
- **目标**：交付 NodeCompletion，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现父 Agent 结构化汇聚”所需的最小闭环；主交付物为：NodeCompletion。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0705/0712。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-dag/`、`crates/apex-agent-runtime/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：NodeCompletion；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-124`：schema/顺序；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(dag): 实现父 Agent 结构化汇聚`

##### EP-0714 实现受限 Merge Subagent 三方合并

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S7 / v0.7 / M-22 DAG
- **需求追踪**：RQ-067；WI：WI-v0.7-08
- **目标**：交付 Merge flow，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现受限 Merge Subagent 三方合并”所需的最小闭环；主交付物为：Merge flow。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0216/0713。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-dag/`、`crates/apex-agent-runtime/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Merge flow；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-125`：冲突/人工阻塞；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(dag): 实现受限 Merge Subagent 三方合并`

##### EP-0715 实现 Node 状态 reducer

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S7 / v0.7 / M-22 DAG
- **需求追踪**：RQ-063/068；WI：WI-v0.7-09
- **目标**：交付 Node state，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Node 状态 reducer”所需的最小闭环；主交付物为：Node state。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0106/0705。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-dag/`、`crates/apex-agent-runtime/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Node state；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-126`：非法迁移；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(dag): 实现 Node 状态 reducer`

##### EP-0716 实现 DAG pause/resume 安全点

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S7 / v0.7 / M-22 DAG
- **需求追踪**：RQ-067/068；WI：WI-v0.7-10
- **目标**：交付 DAG control，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 DAG pause/resume 安全点”所需的最小闭环；主交付物为：DAG control。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0610/0715。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-dag/`、`crates/apex-agent-runtime/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：DAG control；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-127`：暂停无新副作用；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：误放行或敏感信息泄漏。默认拒绝；正例、负例和绕过语料必须同时覆盖。
- **建议提交**：`feat(dag): 实现 DAG pause/resume 安全点`

##### EP-0717 实现崩溃恢复幂等分类

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S7 / v0.7 / M-22 DAG
- **需求追踪**：RQ-068；WI：WI-v0.7-11
- **目标**：交付 Recovery decision，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现崩溃恢复幂等分类”所需的最小闭环；主交付物为：Recovery decision。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0522/0611/0715。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-dag/`、`crates/apex-agent-runtime/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Recovery decision；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-128`：UnknownSideEffect 阻塞；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：原子性或时序错误导致数据损坏。故障注入覆盖断电、重复、乱序和部分写入。
- **建议提交**：`feat(dag): 实现崩溃恢复幂等分类`

##### EP-0723 将限流器接入 DAG 调度器

- **生命周期 / 状态 / 工作量**：Active / 未开始 / L
- **阶段 / 版本 / 模块**：S7 / v0.7 / M-22 DAG
- **需求追踪**：RQ-078、RQ-081；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 全局/Agent/Provider/写入配额在 DAG ready-queue 的统一准入，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“将限流器接入 DAG 调度器”所需的最小闭环；主交付物为：全局/Agent/Provider/写入配额在 DAG ready-queue 的统一准入。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0707、EP-0711、EP-0712。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-dag/`、`crates/apex-agent-runtime/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：全局/Agent/Provider/写入配额在 DAG ready-queue 的统一准入；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-229：公平性、饥饿保护、取消后配额归还；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：竞态、饥饿或无界队列。使用确定性时钟/调度器并验证取消后的资源回收。
- **建议提交**：`feat(dag): 将限流器接入 DAG 调度器`

##### EP-1007 实现 TUI DAG/Claim/Pause/Resume UI

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v0.7 / M-22 DAG
- **需求追踪**：RQ-059–069；WI：WI-v0.7-17
- **目标**：交付 TUI DAG UI，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 TUI DAG/Claim/Pause/Resume UI”所需的最小闭环；主交付物为：TUI DAG UI。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-1006/0715。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-dag/`、`crates/apex-agent-runtime/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：TUI DAG UI；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-173`：状态/冲突/恢复；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：竞态、饥饿或无界队列。使用确定性时钟/调度器并验证取消后的资源回收。
- **建议提交**：`feat(dag): 实现 TUI DAG/Claim/Pause/Resume UI`

#### M-23 Replay

##### EP-0718 将 Snapshot 接入 Tool/Node pre-write

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S7 / v0.7 / M-23 Replay
- **需求追踪**：RQ-069/070；WI：WI-v0.7-12
- **目标**：交付 Snapshot boundary，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“将 Snapshot 接入 Tool/Node pre-write”所需的最小闭环；主交付物为：Snapshot boundary。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0217/0515/0709。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-replay/`、`crates/apex-storage/`、`crates/apex-agent-runtime/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Snapshot boundary；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-129`：混合时间点拒绝；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(replay): 将 Snapshot 接入 Tool/Node pre-write`

##### EP-0719 实现状态确定性重放 executor

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S7 / v0.7 / M-23 Replay
- **需求追踪**：RQ-071；WI：WI-v0.7-13
- **目标**：交付 State replay，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现状态确定性重放 executor”所需的最小闭环；主交付物为：State replay。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0212/0715/0717。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-replay/`、`crates/apex-storage/`、`crates/apex-agent-runtime/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：State replay；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-130`：无副作用/projection hash；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(replay): 实现状态确定性重放 executor`

##### EP-0720 实现再执行重放副作用清单与整体确认

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S7 / v0.7 / M-23 Replay
- **需求追踪**：RQ-072；WI：WI-v0.7-14
- **目标**：交付 Reexecution plan，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现再执行重放副作用清单与整体确认”所需的最小闭环；主交付物为：Reexecution plan。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0719/0513。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-replay/`、`crates/apex-storage/`、`crates/apex-agent-runtime/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Reexecution plan；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-131`：不继承扩权；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(replay): 实现再执行重放副作用清单与整体确认`

##### EP-0721 实现补偿式部分回滚

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S7 / v0.7 / M-23 Replay
- **需求追踪**：RQ-069；WI：WI-v0.7-15
- **目标**：交付 Compensation，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现补偿式部分回滚”所需的最小闭环；主交付物为：Compensation。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0718/0719。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-replay/`、`crates/apex-storage/`、`crates/apex-agent-runtime/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Compensation；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-132`：历史事件不可删；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：原子性或时序错误导致数据损坏。故障注入覆盖断电、重复、乱序和部分写入。
- **建议提交**：`feat(replay): 实现补偿式部分回滚`

##### EP-0722 记录调度决定/limit snapshot/ready hash

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S7 / v0.7 / M-23 Replay
- **需求追踪**：RQ-071；WI：WI-v0.7-16
- **目标**：交付 Replay evidence，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“记录调度决定/limit snapshot/ready hash”所需的最小闭环；主交付物为：Replay evidence。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0706/0707/0719。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-replay/`、`crates/apex-storage/`、`crates/apex-agent-runtime/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Replay evidence；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-133`：重放选择一致；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(replay): 记录调度决定/limit snapshot/ready hash`

### 9.8 v0.8

#### M-24 多 Provider/多模态

##### EP-0805 实现 DeepSeek adapter

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S8 / v0.8 / M-24 多 Provider/多模态
- **需求追踪**：RQ-084；WI：WI-v0.8-01
- **目标**：交付 `apex-provider-deepseek`，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 DeepSeek adapter”所需的最小闭环；主交付物为：`apex-provider-deepseek`。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0801/0802。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-provider/`、`crates/apex-multimodal/`、`crates/apex-context/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：`apex-provider-deepseek`；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-138`：reasoning/Tool；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(multimodal): 实现 DeepSeek adapter`

##### EP-0806 实现 Kimi adapter

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S8 / v0.8 / M-24 多 Provider/多模态
- **需求追踪**：RQ-084；WI：WI-v0.8-02
- **目标**：交付 `apex-provider-kimi`，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Kimi adapter”所需的最小闭环；主交付物为：`apex-provider-kimi`。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0801/0802。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-provider/`、`crates/apex-multimodal/`、`crates/apex-context/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：`apex-provider-kimi`；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-139`：长上下文/文件；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(multimodal): 实现 Kimi adapter`

##### EP-0807 实现 OpenAI-Compatible adapter

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S8 / v0.8 / M-24 多 Provider/多模态
- **需求追踪**：RQ-085；WI：WI-v0.8-03
- **目标**：交付 Compatible adapter，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 OpenAI-Compatible adapter”所需的最小闭环；主交付物为：Compatible adapter。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0801/0802。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-provider/`、`crates/apex-multimodal/`、`crates/apex-context/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Compatible adapter；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-140`：base URL/capability override；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(multimodal): 实现 OpenAI-Compatible adapter`

##### EP-0811 实现默认关闭的 failover chain

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S8 / v0.8 / M-24 多 Provider/多模态
- **需求追踪**：RQ-089；WI：WI-v0.8-04
- **目标**：交付 Failover planner，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现默认关闭的 failover chain”所需的最小闭环；主交付物为：Failover planner。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0802/0810。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-provider/`、`crates/apex-multimodal/`、`crates/apex-context/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Failover planner；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-144`：retryable/不可迁移；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(multimodal): 实现默认关闭的 failover chain`

##### EP-0813 实现 Artifact MIME/大小/转码 Port

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S8 / v0.8 / M-24 多 Provider/多模态
- **需求追踪**：RQ-086/087；WI：WI-v0.8-05
- **目标**：交付 Attachment service，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Artifact MIME/大小/转码 Port”所需的最小闭环；主交付物为：Attachment service。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0217/0802。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-provider/`、`crates/apex-multimodal/`、`crates/apex-context/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Attachment service；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-146`：魔数/炸弹/原件保留；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(multimodal): 实现 Artifact MIME/大小/转码 Port`

##### EP-0815 实现视频文件抽取与实时视频硬禁

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S8 / v0.8 / M-24 多 Provider/多模态
- **需求追踪**：RQ-087/088；WI：WI-v0.8-06
- **目标**：交付 Video capability，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现视频文件抽取与实时视频硬禁”所需的最小闭环；主交付物为：Video capability。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0813/0802。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-provider/`、`crates/apex-multimodal/`、`crates/apex-context/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Video capability；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-148`：无实时视频入口；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(multimodal): 实现视频文件抽取与实时视频硬禁`

##### EP-0816 建立各 Adapter contract fixture/脱敏回放

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S8 / v0.8 / M-24 多 Provider/多模态
- **需求追踪**：RQ-084–092；WI：WI-v0.8-07
- **目标**：交付 Provider contract suite，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“建立各 Adapter contract fixture/脱敏回放”所需的最小闭环；主交付物为：Provider contract suite。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0803–0815。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-provider/`、`crates/apex-multimodal/`、`crates/apex-context/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Provider contract suite；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-149`：五 Adapter 同一测试集；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(multimodal): 建立各 Adapter contract fixture/脱敏回放`

### 9.9 v0.9

#### M-09 TUI 核心

##### EP-1023 完成中文/英文 message key 覆盖

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v0.9 / M-09 TUI 核心
- **需求追踪**：RQ-115；WI：WI-v0.9-26
- **目标**：交付 i18n resources，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“完成中文/英文 message key 覆盖”所需的最小闭环；主交付物为：i18n resources。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-1003、EP-1203。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-tui/`、`crates/apex-tui/`、`crates/apex-client-sdk/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：i18n resources；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-189`：key completeness；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(tui): 完成中文/英文 message key 覆盖`

#### M-20 Plugin

##### EP-0915 实现第三方 Plugin Host RPC/supervisor

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S9 / v0.9 / M-20 Plugin
- **需求追踪**：RQ-100/101；WI：WI-v0.9-21
- **目标**：交付 Plugin Host，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现第三方 Plugin Host RPC/supervisor”所需的最小闭环；主交付物为：Plugin Host。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0206/0914。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-plugin/`、`crates/apex-tool-gateway/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Plugin Host；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-164`：crash/越权隔离；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(plugin): 实现第三方 Plugin Host RPC/supervisor`

##### EP-0916 实现官方签名进程内 allowlist

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S9 / v0.9 / M-20 Plugin
- **需求追踪**：RQ-101；WI：WI-v0.9-22
- **目标**：交付 In-process policy，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现官方签名进程内 allowlist”所需的最小闭环；主交付物为：In-process policy。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0914/0915。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-plugin/`、`crates/apex-tool-gateway/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：In-process policy；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-165`：未签名绝不进程内；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：契约漂移或范围蔓延。先固定接口与失败语义，只修改声明位置和直接消费者。
- **建议提交**：`feat(plugin): 实现官方签名进程内 allowlist`

##### EP-0917 实现本地/Git/文件包安装与安全解包

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S9 / v0.9 / M-20 Plugin
- **需求追踪**：RQ-102；WI：WI-v0.9-23
- **目标**：交付 Extension installer，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现本地/Git/文件包安装与安全解包”所需的最小闭环；主交付物为：Extension installer。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0217/0914。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/apex-plugin/`、`crates/apex-tool-gateway/`、`crates/apex-storage/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Extension installer；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-166`：zip slip/submodule/hook；③ 运行 `cargo test --workspace` 及该 EP 的定向测试目标；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：误放行或敏感信息泄漏。默认拒绝；正例、负例和绕过语料必须同时覆盖。
- **建议提交**：`feat(plugin): 实现本地/Git/文件包安装与安全解包`

#### M-25a 发布工程

##### EP-0220 实现每日系统文本日志与 60 天清理

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S2 / v0.9 / M-25a 发布工程
- **需求追踪**：RQ-110；WI：WI-v0.9-01
- **目标**：交付 SystemLogSink，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现每日系统文本日志与 60 天清理”所需的最小闭环；主交付物为：SystemLogSink。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0201。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`.github/workflows/`、`xtask/`、`packaging/`、`docs/release/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：SystemLogSink；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-39`：日界线/分段/保留；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：门禁误报/漏报或产物不可复现。固定工具链，保留机器可读证据并提供可审计豁免。
- **建议提交**：`feat(release): 实现每日系统文本日志与 60 天清理`

##### EP-0221 实现日志 Ed25519 seal/verify/key rotation

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S2 / v0.9 / M-25a 发布工程
- **需求追踪**：RQ-109；WI：WI-v0.9-02
- **目标**：交付 Log verifier，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现日志 Ed25519 seal/verify/key rotation”所需的最小闭环；主交付物为：Log verifier。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0219。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`.github/workflows/`、`xtask/`、`packaging/`、`docs/release/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Log verifier；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-40`：篡改/断链/旧 key；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：门禁误报/漏报或产物不可复现。固定工具链，保留机器可读证据并提供可审计豁免。
- **建议提交**：`feat(release): 实现日志 Ed25519 seal/verify/key rotation`

##### EP-0223 实现升级/恢复前 SQLite+文件备份

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S2 / v0.9 / M-25a 发布工程
- **需求追踪**：RQ-105；WI：WI-v0.9-03
- **目标**：交付 Backup catalog，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现升级/恢复前 SQLite+文件备份”所需的最小闭环；主交付物为：Backup catalog。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0217/0222。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`.github/workflows/`、`xtask/`、`packaging/`、`docs/release/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Backup catalog；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-42`：备份完整性/恢复演练；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：原子性或时序错误导致数据损坏。故障注入覆盖断电、重复、乱序和部分写入。
- **建议提交**：`feat(release): 实现升级/恢复前 SQLite+文件备份`

##### EP-1101 建立 macOS x86/arm 构建流水线

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S11 / v0.9 / M-25a 发布工程
- **需求追踪**：RQ-004/005；WI：WI-v0.9-04/05/06, WI-v0.9-04
- **目标**：交付 macOS artifacts，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“建立 macOS x86/arm 构建流水线”所需的最小闭环；主交付物为：macOS artifacts。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-1028、EP-1029、EP-1205。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`.github/workflows/`、`xtask/`、`packaging/`、`docs/release/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：macOS artifacts；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-194`：签名/运行；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：门禁误报/漏报或产物不可复现。固定工具链，保留机器可读证据并提供可审计豁免。
- **建议提交**：`ci(release): 建立 macOS x86/arm 构建流水线`

##### EP-1102 建立 Windows x86/arm 构建流水线

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S11 / v0.9 / M-25a 发布工程
- **需求追踪**：RQ-004/005；WI：WI-v0.9-05
- **目标**：交付 Windows artifacts，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“建立 Windows x86/arm 构建流水线”所需的最小闭环；主交付物为：Windows artifacts。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-1028、EP-1029、EP-1205。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`.github/workflows/`、`xtask/`、`packaging/`、`docs/release/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Windows artifacts；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-195`：ACL/ConPTY；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：门禁误报/漏报或产物不可复现。固定工具链，保留机器可读证据并提供可审计豁免。
- **建议提交**：`ci(release): 建立 Windows x86/arm 构建流水线`

##### EP-1103 建立 Linux x86/arm 构建流水线

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S11 / v0.9 / M-25a 发布工程
- **需求追踪**：RQ-004/005；WI：WI-v0.9-06
- **目标**：交付 Linux artifacts，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“建立 Linux x86/arm 构建流水线”所需的最小闭环；主交付物为：Linux artifacts。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-1028、EP-1029、EP-1205。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`.github/workflows/`、`xtask/`、`packaging/`、`docs/release/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Linux artifacts；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-196`：UDS/包安装；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：门禁误报/漏报或产物不可复现。固定工具链，保留机器可读证据并提供可审计豁免。
- **建议提交**：`ci(release): 建立 Linux x86/arm 构建流水线`

##### EP-1104 实现安装/卸载/用户数据保留

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S11 / v0.9 / M-25a 发布工程
- **需求追踪**：RQ-004/008；WI：WI-v0.9-07
- **目标**：交付 Installers，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现安装/卸载/用户数据保留”所需的最小闭环；主交付物为：Installers。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-1101–1103。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`.github/workflows/`、`xtask/`、`packaging/`、`docs/release/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Installers；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-197`：fresh/upgrade/uninstall；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：门禁误报/漏报或产物不可复现。固定工具链，保留机器可读证据并提供可审计豁免。
- **建议提交**：`feat(release): 实现安装/卸载/用户数据保留`

##### EP-1105 实现 signed update manifest 与 SBOM

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S11 / v0.9 / M-25a 发布工程
- **需求追踪**：RQ-112；WI：WI-v0.9-08
- **目标**：交付 Release metadata，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 signed update manifest 与 SBOM”所需的最小闭环；主交付物为：Release metadata。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-1101–1103。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`.github/workflows/`、`xtask/`、`packaging/`、`docs/release/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Release metadata；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-198`：篡改/错误架构拒绝；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：门禁误报/漏报或产物不可复现。固定工具链，保留机器可读证据并提供可审计豁免。
- **建议提交**：`feat(release): 实现 signed update manifest 与 SBOM`

##### EP-1106 实现 Stable/Nightly/Development/Enterprise policy

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S11 / v0.9 / M-25a 发布工程
- **需求追踪**：RQ-112；WI：WI-v0.9-09
- **目标**：交付 Channel resolver，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Stable/Nightly/Development/Enterprise policy”所需的最小闭环；主交付物为：Channel resolver。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-1105/0223。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`.github/workflows/`、`xtask/`、`packaging/`、`docs/release/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Channel resolver；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-199`：下载/确认/安全点；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：门禁误报/漏报或产物不可复现。固定工具链，保留机器可读证据并提供可审计豁免。
- **建议提交**：`feat(release): 实现 Stable/Nightly/Development/Enterprise policy`

##### EP-1107 实现 apex-updater 安全点替换/回滚

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S11 / v0.9 / M-25a 发布工程
- **需求追踪**：RQ-112；WI：WI-v0.9-10
- **目标**：交付 Updater，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 apex-updater 安全点替换/回滚”所需的最小闭环；主交付物为：Updater。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0314/1105。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`.github/workflows/`、`xtask/`、`packaging/`、`docs/release/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Updater；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-200`：daemon/tool/DAG 中断；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：误放行或敏感信息泄漏。默认拒绝；正例、负例和绕过语料必须同时覆盖。
- **建议提交**：`feat(release): 实现 apex-updater 安全点替换/回滚`

##### EP-1108 完成同 Major old/new schema fixture

- **生命周期 / 状态 / 工作量**：Active / 未开始 / S
- **阶段 / 版本 / 模块**：S11 / v0.9 / M-25a 发布工程
- **需求追踪**：RQ-111；WI：WI-v0.9-11
- **目标**：交付 Compatibility matrix，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“完成同 Major old/new schema fixture”所需的最小闭环；主交付物为：Compatibility matrix。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0209/1105。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`.github/workflows/`、`xtask/`、`packaging/`、`docs/release/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Compatibility matrix；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-201`：未知字段/事件保留；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：门禁误报/漏报或产物不可复现。固定工具链，保留机器可读证据并提供可审计豁免。
- **建议提交**：`feat(release): 完成同 Major old/new schema fixture`

##### EP-1109 完成迁移中断/恢复/备份恢复演练

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S11 / v0.9 / M-25a 发布工程
- **需求追踪**：RQ-105/111；WI：WI-v0.9-12
- **目标**：交付 Migration runbook，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“完成迁移中断/恢复/备份恢复演练”所需的最小闭环；主交付物为：Migration runbook。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0223/1107。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`.github/workflows/`、`xtask/`、`packaging/`、`docs/release/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Migration runbook；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-202`：kill/resume/rollback；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：原子性或时序错误导致数据损坏。故障注入覆盖断电、重复、乱序和部分写入。
- **建议提交**：`feat(release): 完成迁移中断/恢复/备份恢复演练`

##### EP-1110 完成 60/120/365 retention scheduler

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S11 / v0.9 / M-25a 发布工程
- **需求追踪**：RQ-078/106/107/110；WI：WI-v0.9-13
- **目标**：交付 Retention jobs，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“完成 60/120/365 retention scheduler”所需的最小闭环；主交付物为：Retention jobs。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0220/0222/0612。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`.github/workflows/`、`xtask/`、`packaging/`、`docs/release/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Retention jobs；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-203`：时间边界/Pinned；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：门禁误报/漏报或产物不可复现。固定工具链，保留机器可读证据并提供可审计豁免。
- **建议提交**：`feat(release): 完成 60/120/365 retention scheduler`

##### EP-1111 完成 `apexd doctor --read-only`

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S11 / v0.9 / M-25a 发布工程
- **需求追踪**：RQ-113；WI：WI-v0.9-14
- **目标**：交付 Doctor command，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“完成 `apexd doctor --read-only`”所需的最小闭环；主交付物为：Doctor command。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0202/0208/0223。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`.github/workflows/`、`xtask/`、`packaging/`、`docs/release/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Doctor command；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-204`：损坏/权限/锁诊断；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：门禁误报/漏报或产物不可复现。固定工具链，保留机器可读证据并提供可审计豁免。
- **建议提交**：`feat(release): 完成 apexd doctor --read-only`

##### EP-1112 完成无遥测网络基线与诊断包

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S11 / v0.9 / M-25a 发布工程
- **需求追踪**：RQ-113；WI：WI-v0.9-15
- **目标**：交付 Privacy evidence，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“完成无遥测网络基线与诊断包”所需的最小闭环；主交付物为：Privacy evidence。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0817、EP-0818、EP-1108。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`.github/workflows/`、`xtask/`、`packaging/`、`docs/release/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Privacy evidence；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-205`：网络抓包/Secret canary；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：门禁误报/漏报或产物不可复现。固定工具链，保留机器可读证据并提供可审计豁免。
- **建议提交**：`feat(release): 完成无遥测网络基线与诊断包`

##### EP-1113 建立启动/Admission/Event/Page/FTS/RSS baseline

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S11 / v0.9 / M-25a 发布工程
- **需求追踪**：RQ-114；WI：WI-v0.9-16
- **目标**：交付 Benchmark suite，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“建立启动/Admission/Event/Page/FTS/RSS baseline”所需的最小闭环；主交付物为：Benchmark suite。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0213/0306/0615。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`.github/workflows/`、`xtask/`、`packaging/`、`docs/release/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Benchmark suite；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-206`：六项 P95/RSS；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：门禁误报/漏报或产物不可复现。固定工具链，保留机器可读证据并提供可审计豁免。
- **建议提交**：`feat(release): 建立启动/Admission/Event/Page/FTS/RSS baseline`

##### EP-1114 建立并发/限流/背压压力场景

- **生命周期 / 状态 / 工作量**：Active / 未开始 / S
- **阶段 / 版本 / 模块**：S11 / v0.9 / M-25a 发布工程
- **需求追踪**：RQ-063/114；WI：WI-v0.9-17
- **目标**：交付 Load fixture，形成可独立验证、可单独回滚的闭环。
- **范围**：仅保留 Core/TUI 并发、限流、背压负载；WebSocket/Web 负载移至 EP-1132。
- **非范围**：不包含：EP-1131、EP-1132；不顺带修改其他版本能力。
- **前置依赖**：EP-0305、EP-0307、EP-0315、EP-0707、EP-1001。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`.github/workflows/`、`xtask/`、`packaging/`、`docs/release/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Load fixture；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-207`：硬上限/无泄漏；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：竞态、饥饿或无界队列。使用确定性时钟/调度器并验证取消后的资源回收。
- **建议提交**：`feat(release): 建立并发/限流/背压压力场景`

##### EP-1115 建立 DB/文件/Tool/DAG/Provider chaos 场景

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S11 / v0.9 / M-25a 发布工程
- **需求追踪**：RQ-068/069/071；WI：WI-v0.9-18
- **目标**：交付 Chaos suite，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“建立 DB/文件/Tool/DAG/Provider chaos 场景”所需的最小闭环；主交付物为：Chaos suite。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0223/0522/0717/0812。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`.github/workflows/`、`xtask/`、`packaging/`、`docs/release/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Chaos suite；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-208`：恢复决策正确；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：竞态、饥饿或无界队列。使用确定性时钟/调度器并验证取消后的资源回收。
- **建议提交**：`feat(release): 建立 DB/文件/Tool/DAG/Provider chaos 场景`

##### EP-1131 建立 Core 与 TUI 负载及背压夹具

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S11 / v0.9 / M-25a 发布工程
- **需求追踪**：RQ-009、RQ-078、RQ-112；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 事件突发、模型流、工具并发、慢消费者与恢复负载夹具，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“建立 Core 与 TUI 负载及背压夹具”所需的最小闭环；主交付物为：事件突发、模型流、工具并发、慢消费者与恢复负载夹具。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0305、EP-0307、EP-0315、EP-0707、EP-1001。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`.github/workflows/`、`xtask/`、`packaging/`、`docs/release/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：事件突发、模型流、工具并发、慢消费者与恢复负载夹具；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-253：无无界队列、无事件丢失、背压指标可见；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：竞态、饥饿或无界队列。使用确定性时钟/调度器并验证取消后的资源回收。
- **建议提交**：`feat(release): 建立 Core 与 TUI 负载及背压夹具`

##### EP-1207 建立 Changelog 完整性 CI

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S11 / v0.9 / M-25a 发布工程
- **需求追踪**：RQ-111；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 用户可见变更与 Changelog 条目一致性检查，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“建立 Changelog 完整性 CI”所需的最小闭环；主交付物为：用户可见变更与 Changelog 条目一致性检查。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0003、EP-0006。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`.github/workflows/`、`xtask/`、`packaging/`、`docs/release/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：用户可见变更与 Changelog 条目一致性检查；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-220：缺失条目阻断、豁免需显式理由；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：门禁误报/漏报或产物不可复现。固定工具链，保留机器可读证据并提供可审计豁免。
- **建议提交**：`ci(release): 建立 Changelog 完整性 CI`

##### EP-1208 建立 design-before-code 门禁

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S11 / v0.9 / M-25a 发布工程
- **需求追踪**：RQ-036–041；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 需求、设计、EP、VAL 先于实现变更的 CI 检查，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“建立 design-before-code 门禁”所需的最小闭环；主交付物为：需求、设计、EP、VAL 先于实现变更的 CI 检查。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0001、EP-0003、EP-0006。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`.github/workflows/`、`xtask/`、`packaging/`、`docs/release/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：需求、设计、EP、VAL 先于实现变更的 CI 检查；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-221：无设计实现提交阻断、纯修复豁免审计；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：门禁误报/漏报或产物不可复现。固定工具链，保留机器可读证据并提供可审计豁免。
- **建议提交**：`feat(release): 建立 design-before-code 门禁`

#### M-25b 质量加固

##### EP-1123 执行 AST、路径与终端安全审计

- **生命周期 / 状态 / 工作量**：Active / 未开始 / L
- **阶段 / 版本 / 模块**：S11 / v0.9 / M-25b 质量加固
- **需求追踪**：RQ-047–RQ-058、RQ-110；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 命令解析、路径穿越、终端注入审计报告与修复证据，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“执行 AST、路径与终端安全审计”所需的最小闭环；主交付物为：命令解析、路径穿越、终端注入审计报告与修复证据。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0501、EP-0508、EP-0520、EP-0525、EP-0526。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/*/tests/`、`fuzz/`、`tests/e2e/`、`.github/workflows/`、`docs/security/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：命令解析、路径穿越、终端注入审计报告与修复证据；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-245：攻击语料、跨平台路径与终端转义回归；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：误放行或敏感信息泄漏。默认拒绝；正例、负例和绕过语料必须同时覆盖。
- **建议提交**：`test(quality): 执行 AST、路径与终端安全审计`

##### EP-1124 执行 Secret 与持久化数据安全审计

- **生命周期 / 状态 / 工作量**：Active / 未开始 / L
- **阶段 / 版本 / 模块**：S11 / v0.9 / M-25b 质量加固
- **需求追踪**：RQ-062、RQ-065、RQ-069、RQ-110；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 凭据、日志、事件、快照、CAS 数据泄漏审计与修复证据，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“执行 Secret 与持久化数据安全审计”所需的最小闭环；主交付物为：凭据、日志、事件、快照、CAS 数据泄漏审计与修复证据。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0217、EP-0817、EP-0818。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/*/tests/`、`fuzz/`、`tests/e2e/`、`.github/workflows/`、`docs/security/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：凭据、日志、事件、快照、CAS 数据泄漏审计与修复证据；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-246：静态扫描、动态注入与落盘抽检无明文秘密；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：误放行或敏感信息泄漏。默认拒绝；正例、负例和绕过语料必须同时覆盖。
- **建议提交**：`test(quality): 执行 Secret 与持久化数据安全审计`

##### EP-1125 执行扩展与供应链安全审计

- **生命周期 / 状态 / 工作量**：Active / 未开始 / L
- **阶段 / 版本 / 模块**：S11 / v0.9 / M-25b 质量加固
- **需求追踪**：RQ-083–RQ-092、RQ-110；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 Skill/MCP/Plugin 来源、权限、隔离与依赖供应链审计，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“执行扩展与供应链安全审计”所需的最小闭环；主交付物为：Skill/MCP/Plugin 来源、权限、隔离与依赖供应链审计。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0907、EP-0913、EP-0917、EP-1105、EP-1106。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/*/tests/`、`fuzz/`、`tests/e2e/`、`.github/workflows/`、`docs/security/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Skill/MCP/Plugin 来源、权限、隔离与依赖供应链审计；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-247：恶意包、篡改包、越权扩展与依赖风险测试；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：误放行或敏感信息泄漏。默认拒绝；正例、负例和绕过语料必须同时覆盖。
- **建议提交**：`test(quality): 执行扩展与供应链安全审计`

##### EP-1127 建立覆盖率与变异测试 Gate

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S11 / v0.9 / M-25b 质量加固
- **需求追踪**：RQ-112；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 关键 crate 覆盖率基线、变异阈值、例外登记与 CI 门禁，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“建立覆盖率与变异测试 Gate”所需的最小闭环；主交付物为：关键 crate 覆盖率基线、变异阈值、例外登记与 CI 门禁。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-1104、EP-1123、EP-1124、EP-1125。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/*/tests/`、`fuzz/`、`tests/e2e/`、`.github/workflows/`、`docs/security/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：关键 crate 覆盖率基线、变异阈值、例外登记与 CI 门禁；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-249：人为删除关键断言或分支时 Gate 失败；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：门禁误报/漏报或产物不可复现。固定工具链，保留机器可读证据并提供可审计豁免。
- **建议提交**：`test(quality): 建立覆盖率与变异测试 Gate`

##### EP-1128 建立 Fuzz 与属性测试 Gate

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S11 / v0.9 / M-25b 质量加固
- **需求追踪**：RQ-112；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 协议、AST、事件、Provider 流解析的 fuzz/property 测试集，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“建立 Fuzz 与属性测试 Gate”所需的最小闭环；主交付物为：协议、AST、事件、Provider 流解析的 fuzz/property 测试集。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0501、EP-0802、EP-1108。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/*/tests/`、`fuzz/`、`tests/e2e/`、`.github/workflows/`、`docs/security/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：协议、AST、事件、Provider 流解析的 fuzz/property 测试集；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-250：固定预算运行稳定且回归语料可复现；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：门禁误报/漏报或产物不可复现。固定工具链，保留机器可读证据并提供可审计豁免。
- **建议提交**：`test(quality): 建立 Fuzz 与属性测试 Gate`

##### EP-1129 建立 TUI v1.0 端到端发布 Gate

- **生命周期 / 状态 / 工作量**：Active / 未开始 / L
- **阶段 / 版本 / 模块**：S11 / v0.9 / M-25b 质量加固
- **需求追踪**：RQ-009、RQ-111、RQ-112；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 三平台 TUI 关键旅程、升级、回滚与证据包 Gate，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“建立 TUI v1.0 端到端发布 Gate”所需的最小闭环；主交付物为：三平台 TUI 关键旅程、升级、回滚与证据包 Gate。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-1023、EP-1101、EP-1102、EP-1103、EP-1127、EP-1128、EP-1131。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/*/tests/`、`fuzz/`、`tests/e2e/`、`.github/workflows/`、`docs/security/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：三平台 TUI 关键旅程、升级、回滚与证据包 Gate；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-251：干净机到完成首个工具回合全链路通过；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：门禁误报/漏报或产物不可复现。固定工具链，保留机器可读证据并提供可审计豁免。
- **建议提交**：`ci(quality): 建立 TUI v1.0 端到端发布 Gate`

### 9.10 v1.0

#### M-25b 质量加固

##### EP-1118 生成各 Feature 最终 verification.md

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S11 / v1.0 / M-25b 质量加固
- **需求追踪**：RQ-040/041；WI：WI-v1.0-01
- **目标**：交付 Verification reports，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“生成各 Feature 最终 verification.md”所需的最小闭环；主交付物为：Verification reports。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0404/1117。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/*/tests/`、`fuzz/`、`tests/e2e/`、`.github/workflows/`、`docs/security/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Verification reports；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-211`：证据 hash/用户确认；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：门禁误报/漏报或产物不可复现。固定工具链，保留机器可读证据并提供可审计豁免。
- **建议提交**：`feat(quality): 生成各 Feature 最终 verification.md`

##### EP-1119 生成 Release Candidate 与完整回滚包

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S11 / v1.0 / M-25b 质量加固
- **需求追踪**：全部 AC；WI：WI-v1.0-02
- **目标**：交付 RC artifacts，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“生成 Release Candidate 与完整回滚包”所需的最小闭环；主交付物为：RC artifacts。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-1101、EP-1102、EP-1103、EP-1104、EP-1105、EP-1106、EP-1107、EP-1108、EP-1109、EP-1110、EP-1111、EP-1112、EP-1113、EP-1115、EP-1118、EP-1123、EP-1124、EP-1125、EP-1127、EP-1128、EP-1129、EP-1131。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/*/tests/`、`fuzz/`、`tests/e2e/`、`.github/workflows/`、`docs/security/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：RC artifacts；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-212`：安装/升级/回滚；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：原子性或时序错误导致数据损坏。故障注入覆盖断电、重复、乱序和部分写入。
- **建议提交**：`feat(quality): 生成 Release Candidate 与完整回滚包`

##### EP-1120 执行独立发布评审并封存证据

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S11 / v1.0 / M-25b 质量加固
- **需求追踪**：G-8；WI：WI-v1.0-03
- **目标**：交付 Release decision，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“执行独立发布评审并封存证据”所需的最小闭环；主交付物为：Release decision。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-1119。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/*/tests/`、`fuzz/`、`tests/e2e/`、`.github/workflows/`、`docs/security/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Release decision；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-213`：无未处置高风险；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：门禁误报/漏报或产物不可复现。固定工具链，保留机器可读证据并提供可审计豁免。
- **建议提交**：`feat(quality): 执行独立发布评审并封存证据`

### 9.11 v1.1

#### M-26 Desktop

##### EP-0814 实现 Desktop/Web audio 与双向语音 Port

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S8 / v1.1 / M-26 Desktop
- **需求追踪**：RQ-088；WI：WI-v1.1-07
- **目标**：交付 Realtime audio，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Desktop/Web audio 与双向语音 Port”所需的最小闭环；主交付物为：Realtime audio。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0802/0813。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-desktop/`、`packages/apex-ui/`、`crates/apex-client-sdk/`、`packaging/desktop/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Realtime audio；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-147`：取消/VAD/无泄漏；③ 运行 `cargo test --workspace` 与前端单元/组件测试；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(desktop): 实现 Desktop/Web audio 与双向语音 Port`

##### EP-1011 建立 Vue domain stores/reducers

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v1.1 / M-26 Desktop
- **需求追踪**：RQ-017；WI：WI-v1.1-01
- **目标**：交付 TS adapter contract，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“建立 Vue domain stores/reducers”所需的最小闭环；主交付物为：TS adapter contract。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0111/0305。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-desktop/`、`packages/apex-ui/`、`crates/apex-client-sdk/`、`packaging/desktop/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：TS adapter contract；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-177`：durable/transient 分层；③ 运行 `cargo test --workspace` 与前端单元/组件测试；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(desktop): 建立 Vue domain stores/reducers`

##### EP-1012 实现共享 Platform Adapter interface

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v1.1 / M-26 Desktop
- **需求追踪**：RQ-017；WI：WI-v1.1-02
- **目标**：交付 TS adapter contract，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现共享 Platform Adapter interface”所需的最小闭环；主交付物为：TS adapter contract。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-1011。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-desktop/`、`packages/apex-ui/`、`crates/apex-client-sdk/`、`packaging/desktop/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：TS adapter contract；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-178`：Desktop/Web 等价；③ 运行 `cargo test --workspace` 与前端单元/组件测试；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(desktop): 实现共享 Platform Adapter interface`

##### EP-1013 实现 Tauri gRPC bridge

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v1.1 / M-26 Desktop
- **需求追踪**：RQ-009/017；WI：WI-v1.1-03
- **目标**：交付 Desktop transport，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Tauri gRPC bridge”所需的最小闭环；主交付物为：Desktop transport。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0302/1012。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-desktop/`、`packages/apex-ui/`、`crates/apex-client-sdk/`、`packaging/desktop/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Desktop transport；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-179`：WebView 不泄漏 socket；③ 运行 `cargo test --workspace` 与前端单元/组件测试；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(desktop): 实现 Tauri gRPC bridge`

##### EP-1015 实现共享 Session/Turn/Spec 页面

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v1.1 / M-26 Desktop
- **需求追踪**：AC-001/003；WI：WI-v1.1-04
- **目标**：交付 Vue feature slices，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现共享 Session/Turn/Spec 页面”所需的最小闭环；主交付物为：Vue feature slices。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-1011/1012。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-desktop/`、`packages/apex-ui/`、`crates/apex-client-sdk/`、`packaging/desktop/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Vue feature slices；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-181`：浏览器 E2E；③ 运行 `cargo test --workspace` 与前端单元/组件测试；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(desktop): 实现共享 Session/Turn/Spec 页面`

##### EP-1018 实现 Desktop/Web Checkpoint/Memory 页面

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v1.1 / M-26 Desktop
- **需求追踪**：RQ-077–083；WI：WI-v1.1-05
- **目标**：交付 Context UI，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Desktop/Web Checkpoint/Memory 页面”所需的最小闭环；主交付物为：Context UI。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-1015/0616。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-desktop/`、`packages/apex-ui/`、`crates/apex-client-sdk/`、`packaging/desktop/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Context UI；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-184`：恢复/导出；③ 运行 `cargo test --workspace` 与前端单元/组件测试；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：原子性或时序错误导致数据损坏。故障注入覆盖断电、重复、乱序和部分写入。
- **建议提交**：`feat(desktop): 实现 Desktop/Web Checkpoint/Memory 页面`

##### EP-1019 实现三端 Session/System Log 页面

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v1.1 / M-26 Desktop
- **需求追踪**：RQ-107/110；WI：WI-v1.1-06
- **目标**：交付 Log UI，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现三端 Session/System Log 页面”所需的最小闭环；主交付物为：Log UI。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-1015/0220/0221。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-desktop/`、`packages/apex-ui/`、`crates/apex-client-sdk/`、`packaging/desktop/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Log UI；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-185`：三端可浏览且脱敏；③ 运行 `cargo test --workspace` 与前端单元/组件测试；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(desktop): 实现三端 Session/System Log 页面`

##### EP-1020 实现 Desktop audio/file picker

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v1.1 / M-26 Desktop
- **需求追踪**：RQ-086/088；WI：WI-v1.1-07
- **目标**：交付 Tauri media bridge，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Desktop audio/file picker”所需的最小闭环；主交付物为：Tauri media bridge。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0813/0814/1013。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-desktop/`、`packages/apex-ui/`、`crates/apex-client-sdk/`、`packaging/desktop/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Tauri media bridge；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-186`：权限/取消；③ 运行 `cargo test --workspace` 与前端单元/组件测试；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(desktop): 实现 Desktop audio/file picker`

##### EP-1024 完成键盘/屏幕阅读器/颜色无关状态

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v1.1 / M-26 Desktop
- **需求追踪**：RQ-018/115；WI：WI-v1.1-08
- **目标**：交付 Accessibility，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“完成键盘/屏幕阅读器/颜色无关状态”所需的最小闭环；主交付物为：Accessibility。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-1002/1015。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-desktop/`、`packages/apex-ui/`、`crates/apex-client-sdk/`、`packaging/desktop/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Accessibility；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-190`：a11y smoke；③ 运行 `cargo test --workspace` 与前端单元/组件测试；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(desktop): 完成键盘/屏幕阅读器/颜色无关状态`

##### EP-1025 完成 Vue XSS/CSRF/URL/Secret 安全规则

- **生命周期 / 状态 / 工作量**：Active / 未开始 / S
- **阶段 / 版本 / 模块**：S10 / v1.1 / M-26 Desktop
- **需求追踪**：RQ-016/092；WI：WI-v1.1-09
- **目标**：交付 UI security gate，形成可独立验证、可单独回滚的闭环。
- **范围**：仅保留共享/Desktop UI 安全边界；Web 专属回归移至 EP-1037。
- **非范围**：不包含：EP-1037；不顺带修改其他版本能力。
- **前置依赖**：EP-1012、EP-1015。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-desktop/`、`packages/apex-ui/`、`crates/apex-client-sdk/`、`packaging/desktop/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：UI security gate；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-191`：静态+动态注入；③ 运行 `cargo test --workspace` 与前端单元/组件测试；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：误放行或敏感信息泄漏。默认拒绝；正例、负例和绕过语料必须同时覆盖。
- **建议提交**：`feat(desktop): 完成 Vue XSS/CSRF/URL/Secret 安全规则`

##### EP-1026 添加 TUI/Vue/Platform 单元组件测试

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v1.1 / M-26 Desktop
- **需求追踪**：RQ-046；WI：WI-v1.1-10
- **目标**：交付 Client unit tests，形成可独立验证、可单独回滚的闭环。
- **范围**：仅保留共享 Vue 与 Desktop 单元/组件测试；Web 测试移至 EP-1036。
- **非范围**：不包含：EP-1036；不顺带修改其他版本能力。
- **前置依赖**：EP-1011、EP-1012、EP-1013、EP-1015、EP-1018、EP-1019、EP-1020、EP-1024、EP-1025。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-desktop/`、`packages/apex-ui/`、`crates/apex-client-sdk/`、`packaging/desktop/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Client unit tests；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-192`：覆盖率阈值；③ 运行 `cargo test --workspace` 与前端单元/组件测试；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`test(desktop): 添加 TUI/Vue/Platform 单元组件测试`

##### EP-1032 实现 Desktop 视频文件引用界面

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v1.1 / M-26 Desktop
- **需求追踪**：RQ-096、RQ-097；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 桌面端视频选择、引用预览、能力降级与错误反馈，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Desktop 视频文件引用界面”所需的最小闭环；主交付物为：桌面端视频选择、引用预览、能力降级与错误反馈。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0814、EP-1015。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-desktop/`、`packages/apex-ui/`、`crates/apex-client-sdk/`、`packaging/desktop/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：桌面端视频选择、引用预览、能力降级与错误反馈；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-237：格式、大小、能力缺失与取消测试；③ 运行 `cargo test --workspace` 与前端单元/组件测试；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(desktop): 实现 Desktop 视频文件引用界面`

##### EP-1121 建立 Desktop 签名与安装包流水线

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S11 / v1.1 / M-26 Desktop
- **需求追踪**：RQ-098、RQ-106、RQ-111；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 Desktop 多平台构建、签名、公证、安装包与校验和，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“建立 Desktop 签名与安装包流水线”所需的最小闭环；主交付物为：Desktop 多平台构建、签名、公证、安装包与校验和。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-1013、EP-1026。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-desktop/`、`packages/apex-ui/`、`crates/apex-client-sdk/`、`packaging/desktop/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Desktop 多平台构建、签名、公证、安装包与校验和；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-243：干净机器安装、签名校验、升级与卸载；③ 运行 `cargo test --workspace` 与前端单元/组件测试；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(desktop): 建立 Desktop 签名与安装包流水线`

### 9.12 v1.2

#### M-25b 质量加固

##### EP-1126 执行 Web 与客户端边界安全审计

- **生命周期 / 状态 / 工作量**：Active / 未开始 / L
- **阶段 / 版本 / 模块**：S11 / v1.2 / M-25b 质量加固
- **需求追踪**：RQ-098–RQ-110；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 Origin/CSRF/XSS/URL/IPC/剪贴板边界审计与修复证据，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“执行 Web 与客户端边界安全审计”所需的最小闭环；主交付物为：Origin/CSRF/XSS/URL/IPC/剪贴板边界审计与修复证据。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0311、EP-1025、EP-1037。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`crates/*/tests/`、`fuzz/`、`tests/e2e/`、`.github/workflows/`、`docs/security/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Origin/CSRF/XSS/URL/IPC/剪贴板边界审计与修复证据；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-248：客户端边界攻击语料全量回归；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：误放行或敏感信息泄漏。默认拒绝；正例、负例和绕过语料必须同时覆盖。
- **建议提交**：`test(quality): 执行 Web 与客户端边界安全审计`

#### M-27 Web

##### EP-0303 实现 REST DTO 到 Application Command 映射

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S3 / v1.2 / M-27 Web
- **需求追踪**：RQ-012；WI：WI-v1.2-01
- **目标**：交付 Actix REST handlers，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 REST DTO 到 Application Command 映射”所需的最小闭环；主交付物为：Actix REST handlers。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0301。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-web/`、`packages/apex-ui/`、`crates/apex-web/`、`crates/apex-client-sdk/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Actix REST handlers；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-45`：等价错误/结果；③ 运行 `cargo test --workspace` 与前端单元/组件测试；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(web): 实现 REST DTO 到 Application Command 映射`

##### EP-0304 实现 WebSocket Subscribe/Close/错误帧

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S3 / v1.2 / M-27 Web
- **需求追踪**：RQ-012；WI：WI-v1.2-02
- **目标**：交付 WS endpoint，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 WebSocket Subscribe/Close/错误帧”所需的最小闭环；主交付物为：WS endpoint。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0301/0211。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-web/`、`packages/apex-ui/`、`crates/apex-web/`、`crates/apex-client-sdk/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：WS endpoint；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-46`：背压/断连；③ 运行 `cargo test --workspace` 与前端单元/组件测试；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(web): 实现 WebSocket Subscribe/Close/错误帧`

##### EP-0308 实现控制租约 acquire/renew/release

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S3 / v1.2 / M-27 Web
- **需求追踪**：RQ-021/022；WI：WI-v1.2-04
- **目标**：交付 ControlLeaseService，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现控制租约 acquire/renew/release”所需的最小闭环；主交付物为：ControlLeaseService。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0210/0302。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-web/`、`packages/apex-ui/`、`crates/apex-web/`、`crates/apex-client-sdk/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：ControlLeaseService；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-50`：FIFO/30 秒宽限；③ 运行 `cargo test --workspace` 与前端单元/组件测试；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(web): 实现控制租约 acquire/renew/release`

##### EP-0309 实现 force takeover 与旧 token fencing

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S3 / v1.2 / M-27 Web
- **需求追踪**：RQ-023；WI：WI-v1.2-05
- **目标**：交付 Takeover command，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 force takeover 与旧 token fencing”所需的最小闭环；主交付物为：Takeover command。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0308/0210。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-web/`、`packages/apex-ui/`、`crates/apex-web/`、`crates/apex-client-sdk/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Takeover command；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-51`：接管审计/旧 token 拒绝；③ 运行 `cargo test --workspace` 与前端单元/组件测试；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(web): 实现 force takeover 与旧 token fencing`

##### EP-0310 实现 TUI 自动 Web enable lease

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S3 / v1.2 / M-27 Web
- **需求追踪**：RQ-014/015；WI：WI-v1.2-06
- **目标**：交付 WebLeaseService，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 TUI 自动 Web enable lease”所需的最小闭环；主交付物为：WebLeaseService。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0204/0301/0308。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-web/`、`packages/apex-ui/`、`crates/apex-web/`、`crates/apex-client-sdk/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：WebLeaseService；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-52`：TUI 退出关闭 listener；③ 运行 `cargo test --workspace` 与前端单元/组件测试；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(web): 实现 TUI 自动 Web enable lease`

##### EP-0311 实现一次性 token exchange 与短 Cookie

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S3 / v1.2 / M-27 Web
- **需求追踪**：RQ-016；WI：WI-v1.2-07
- **目标**：交付 Web auth，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现一次性 token exchange 与短 Cookie”所需的最小闭环；主交付物为：Web auth。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0310。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-web/`、`packages/apex-ui/`、`crates/apex-web/`、`crates/apex-client-sdk/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Web auth；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-53`：token replay/过期；③ 运行 `cargo test --workspace` 与前端单元/组件测试；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(web): 实现一次性 token exchange 与短 Cookie`

##### EP-0312 实现 Origin/CSRF/CSP 校验

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S3 / v1.2 / M-27 Web
- **需求追踪**：RQ-016；WI：WI-v1.2-08
- **目标**：交付 Web security middleware，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Origin/CSRF/CSP 校验”所需的最小闭环；主交付物为：Web security middleware。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0311。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-web/`、`packages/apex-ui/`、`crates/apex-web/`、`crates/apex-client-sdk/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Web security middleware；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-54`：恶意 Origin/CSRF；③ 运行 `cargo test --workspace` 与前端单元/组件测试；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：误放行或敏感信息泄漏。默认拒绝；正例、负例和绕过语料必须同时覆盖。
- **建议提交**：`feat(web): 实现 Origin/CSRF/CSP 校验`

##### EP-1010 实现 TUI 自动 Web lease lifecycle

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v1.2 / M-27 Web
- **需求追踪**：RQ-014/015；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 TUI lease owner，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 TUI 自动 Web lease lifecycle”所需的最小闭环；主交付物为：TUI lease owner。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-1001/0310。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-web/`、`packages/apex-ui/`、`crates/apex-web/`、`crates/apex-client-sdk/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：TUI lease owner；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-176`：退出关闭 Web；③ 运行 `cargo test --workspace` 与前端单元/组件测试；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(web): 实现 TUI 自动 Web lease lifecycle`

##### EP-1014 实现 Web auth bootstrap/token cleanup

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v1.2 / M-27 Web
- **需求追踪**：RQ-012/016；WI：WI-v1.2-09
- **目标**：交付 Web auth entry，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Web auth bootstrap/token cleanup”所需的最小闭环；主交付物为：Web auth entry。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0311/1012。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-web/`、`packages/apex-ui/`、`crates/apex-web/`、`crates/apex-client-sdk/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Web auth entry；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-180`：fragment/Cookie/CSRF；③ 运行 `cargo test --workspace` 与前端单元/组件测试；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(web): 实现 Web auth bootstrap/token cleanup`

##### EP-1016 实现 Web Permission/Control takeover 页面

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v1.2 / M-27 Web
- **需求追踪**：RQ-023/047；WI：WI-v1.2-10
- **目标**：交付 Web control UI，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Web Permission/Control takeover 页面”所需的最小闭环；主交付物为：Web control UI。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-1015/0309/0510。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-web/`、`packages/apex-ui/`、`crates/apex-web/`、`crates/apex-client-sdk/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Web control UI；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-182`：接管确认/审计；③ 运行 `cargo test --workspace` 与前端单元/组件测试；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(web): 实现 Web Permission/Control takeover 页面`

##### EP-1017 实现 Web Agent/DAG/Activity 页面

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v1.2 / M-27 Web
- **需求追踪**：RQ-063/073；WI：WI-v1.2-11
- **目标**：交付 Web orchestration UI，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Web Agent/DAG/Activity 页面”所需的最小闭环；主交付物为：Web orchestration UI。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-1015/0313/0715。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-web/`、`packages/apex-ui/`、`crates/apex-web/`、`crates/apex-client-sdk/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Web orchestration UI；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-183`：实时事件；③ 运行 `cargo test --workspace` 与前端单元/组件测试；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：竞态、饥饿或无界队列。使用确定性时钟/调度器并验证取消后的资源回收。
- **建议提交**：`feat(web): 实现 Web Agent/DAG/Activity 页面`

##### EP-1021 实现 Web audio/file upload

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v1.2 / M-27 Web
- **需求追踪**：RQ-086/088；WI：WI-v1.2-12
- **目标**：交付 Browser media，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Web audio/file upload”所需的最小闭环；主交付物为：Browser media。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0813/0814/1014。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-web/`、`packages/apex-ui/`、`crates/apex-web/`、`crates/apex-client-sdk/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Browser media；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-187`：大小/MIME/CSRF；③ 运行 `cargo test --workspace` 与前端单元/组件测试；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(web): 实现 Web audio/file upload`

##### EP-1033 实现 Web 视频文件引用界面

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v1.2 / M-27 Web
- **需求追踪**：RQ-096、RQ-097、RQ-103；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 浏览器视频选择、上传引用、能力降级与错误反馈，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Web 视频文件引用界面”所需的最小闭环；主交付物为：浏览器视频选择、上传引用、能力降级与错误反馈。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0814、EP-1016。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-web/`、`packages/apex-ui/`、`crates/apex-web/`、`crates/apex-client-sdk/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：浏览器视频选择、上传引用、能力降级与错误反馈；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-238：Origin、大小、上传中断与能力缺失测试；③ 运行 `cargo test --workspace` 与前端单元/组件测试；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(web): 实现 Web 视频文件引用界面`

##### EP-1036 建立 Web 客户端单元与组件测试套件

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v1.2 / M-27 Web
- **需求追踪**：RQ-098–RQ-109；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 Web store、页面、重连、Origin 与浏览器边界测试，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“建立 Web 客户端单元与组件测试套件”所需的最小闭环；主交付物为：Web store、页面、重连、Origin 与浏览器边界测试。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-1014、EP-1016、EP-1017、EP-1021、EP-1033。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-web/`、`packages/apex-ui/`、`crates/apex-web/`、`crates/apex-client-sdk/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Web store、页面、重连、Origin 与浏览器边界测试；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-241：单元/组件测试覆盖 Web 独有失败路径；③ 运行 `cargo test --workspace` 与前端单元/组件测试；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`test(web): 建立 Web 客户端单元与组件测试套件`

##### EP-1037 建立 Web UI 安全回归套件

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v1.2 / M-27 Web
- **需求追踪**：RQ-100、RQ-101、RQ-103、RQ-110；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 Origin/CSRF、URL、XSS、Secret 展示与剪贴板安全回归，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“建立 Web UI 安全回归套件”所需的最小闭环；主交付物为：Origin/CSRF、URL、XSS、Secret 展示与剪贴板安全回归。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0311、EP-1016、EP-1025。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-web/`、`packages/apex-ui/`、`crates/apex-web/`、`crates/apex-client-sdk/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Origin/CSRF、URL、XSS、Secret 展示与剪贴板安全回归；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-242：恶意 Origin、注入内容、秘密复制均被阻断；③ 运行 `cargo test --workspace` 与前端单元/组件测试；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：误放行或敏感信息泄漏。默认拒绝；正例、负例和绕过语料必须同时覆盖。
- **建议提交**：`feat(web): 建立 Web UI 安全回归套件`

##### EP-1122 建立 Web 静态资源与嵌入式分发流水线

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S11 / v1.2 / M-27 Web
- **需求追踪**：RQ-098、RQ-100、RQ-111；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 Web 资源构建、完整性清单、Daemon 嵌入与独立部署产物，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“建立 Web 静态资源与嵌入式分发流水线”所需的最小闭环；主交付物为：Web 资源构建、完整性清单、Daemon 嵌入与独立部署产物。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-1017、EP-1036、EP-1037。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-web/`、`packages/apex-ui/`、`crates/apex-web/`、`crates/apex-client-sdk/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Web 资源构建、完整性清单、Daemon 嵌入与独立部署产物；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-244：缓存失效、资源完整性、两种部署模式；③ 运行 `cargo test --workspace` 与前端单元/组件测试；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(web): 建立 Web 静态资源与嵌入式分发流水线`

##### EP-1132 建立 WebSocket 与 Web 负载夹具

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S11 / v1.2 / M-27 Web
- **需求追踪**：RQ-100–RQ-104、RQ-112；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 多浏览器、断线重连、慢客户端、Origin 隔离与背压夹具，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“建立 WebSocket 与 Web 负载夹具”所需的最小闭环；主交付物为：多浏览器、断线重连、慢客户端、Origin 隔离与背压夹具。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0304、EP-0312、EP-1036、EP-1131。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`apps/apex-web/`、`packages/apex-ui/`、`crates/apex-web/`、`crates/apex-client-sdk/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：多浏览器、断线重连、慢客户端、Origin 隔离与背压夹具；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-254：并发连接、重放、限流与资源回收符合预算；③ 运行 `cargo test --workspace` 与前端单元/组件测试；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(web): 建立 WebSocket 与 Web 负载夹具`

### 9.13 v1.3

#### M-28 Trinity

##### EP-1027 添加三端等价性 E2E harness

- **生命周期 / 状态 / 工作量**：Active / 未开始 / M
- **阶段 / 版本 / 模块**：S10 / v1.3 / M-28 Trinity
- **需求追踪**：AC-001–020；WI：WI-v1.3-01, WI-v1.3-02, WI-v1.3-03, WI-v1.3-04, WI-v1.3-05
- **目标**：交付 Cross-client E2E，形成可独立验证、可单独回滚的闭环。
- **范围**：仅保留跨客户端领域状态哈希与同输入回放工具；能力矩阵与 Gate 关闭移出。
- **非范围**：不包含：EP-1034、EP-1035；不顺带修改其他版本能力。
- **前置依赖**：EP-0305、EP-1001、EP-1016、EP-1017、EP-1026、EP-1036。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`tests/trinity/`、`xtask/`、`docs/release/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：Cross-client E2E；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② `VAL-193`：同 Session/seq；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(trinity): 添加三端等价性 E2E harness`

##### EP-1034 实现 Trinity 能力矩阵验证器

- **生命周期 / 状态 / 工作量**：Active / 未开始 / L
- **阶段 / 版本 / 模块**：S10 / v1.3 / M-28 Trinity
- **需求追踪**：RQ-098–RQ-109；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 TUI/Desktop/Web 能力声明、探测结果与降级行为对比报告，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“实现 Trinity 能力矩阵验证器”所需的最小闭环；主交付物为：TUI/Desktop/Web 能力声明、探测结果与降级行为对比报告。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-0007、EP-1027、EP-1036。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`tests/trinity/`、`xtask/`、`docs/release/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：TUI/Desktop/Web 能力声明、探测结果与降级行为对比报告；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-239：支持/降级/不支持三态与声明一致；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`test(trinity): 实现 Trinity 能力矩阵验证器`

##### EP-1035 关闭 Trinity 等价性 Gate 与证据包

- **生命周期 / 状态 / 工作量**：Active / 未开始 / L
- **阶段 / 版本 / 模块**：S10 / v1.3 / M-28 Trinity
- **需求追踪**：RQ-098–RQ-109；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 G-7/G-8 等价性结果、例外清单、证据索引与签署记录，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“关闭 Trinity 等价性 Gate 与证据包”所需的最小闭环；主交付物为：G-7/G-8 等价性结果、例外清单、证据索引与签署记录。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-1027、EP-1034、EP-1130。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`tests/trinity/`、`xtask/`、`docs/release/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：G-7/G-8 等价性结果、例外清单、证据索引与签署记录；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-240：状态哈希、关键旅程、能力降级均可复核；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(trinity): 关闭 Trinity 等价性 Gate 与证据包`

##### EP-1130 建立 Trinity 跨客户端端到端 Gate

- **生命周期 / 状态 / 工作量**：Active / 未开始 / L
- **阶段 / 版本 / 模块**：S11 / v1.3 / M-28 Trinity
- **需求追踪**：RQ-098–RQ-109、RQ-112；WI：源文档未分配独立 WI；以本 EP 的 RQ/VAL 为执行依据
- **目标**：交付 TUI/Desktop/Web 同场景回放、状态哈希与降级一致性 Gate，形成可独立验证、可单独回滚的闭环。
- **范围**：实现“建立 Trinity 跨客户端端到端 Gate”所需的最小闭环；主交付物为：TUI/Desktop/Web 同场景回放、状态哈希与降级一致性 Gate。
- **非范围**：不实现下游 EP；不重写稳定上游契约；不做无关重构、UI 美化或提前性能优化。
- **前置依赖**：EP-1027、EP-1034、EP-1036、EP-1121、EP-1122。依赖未提供通过证据时不得开工；仅预研不算实现开始。
- **修改位置**：`tests/trinity/`、`xtask/`、`docs/release/`。最终路径以 EP-0101 workspace 清单为准，不新增平行重复 crate。
- **交付物**：TUI/Desktop/Web 同场景回放、状态哈希与降级一致性 Gate；同一提交集包含最小测试、错误语义、追踪表更新和验证证据索引。
- **验证**：① 先提交能失败的定向测试；② VAL-252：三客户端关键旅程可重复且差异有登记；③ 运行 `cargo xtask verify`（命令未落地前使用最小等价脚本）；④ 至少覆盖一个失败/取消/重复/越界反例；⑤ 非实现者从干净工作树复跑并记录证据。
- **DoD**：主要交付物唯一且可定位；公开接口和错误类型已文档化；正常、边界、故障路径通过；无被忽略测试；RQ→AC→EP→VAL 可追踪；fmt、Clippy、相关测试与 drift 检查通过；提交可独立回滚。
- **主要风险**：客户端状态漂移或平台差异。领域状态由事件重建，平台特例必须进入能力矩阵。
- **建议提交**：`feat(trinity): 建立 Trinity 跨客户端端到端 Gate`

## 10. Superseded EP 完整执行卡

### EP-0809 实现 SecretResolver 与 Provider Secret Firewall

- **生命周期 / 状态 / 工作量**：Superseded / 未开始 / 不再估算
- **原阶段 / 版本 / 模块**：S8 / v0.1 / M-04 Provider 核心
- **需求追踪**：RQ-092/093；原 VAL：`VAL-142`：Key 不入 DB/log/env。
- **目标**：仅保留历史编号和追踪关系，不再作为可领取任务。
- **范围**：记录旧设计意图、历史引用和迁移证据。
- **非范围**：不得提交新实现、测试或修复到此 EP。
- **替代 EP**：EP-0817, EP-0818, EP-0819。所有依赖方必须改指向满足真实需求的最小替代 EP。
- **修改位置**：仅允许 EP 注册表、追踪矩阵、迁移说明与历史链接。
- **交付物**：Superseded 标记与 `EP-0809 → EP-0817, EP-0818, EP-0819` 机器可读迁移记录。
- **验证**：EP-0010 确认无 Active EP 把 Superseded EP 当作唯一实现依赖；旧 ID 未复用；替代 EP 覆盖原 RQ/VAL。
- **DoD**：直接依赖已迁移；追踪闭包完整；旧编号可检索；执行看板不可领取。
- **主要风险**：依赖方误把旧编号当作已实现能力。CI 必须阻断新增实现依赖。
- **建议提交**：`docs(plan): 迁移 EP-0809 到替代 EP`

### EP-1022 实现 Desktop/Web 视频文件引用

- **生命周期 / 状态 / 工作量**：Superseded / 未开始 / 不再估算
- **原阶段 / 版本 / 模块**：S10 / v1.1 / M-26 Desktop
- **需求追踪**：RQ-086/087；原 VAL：`VAL-188`：实时视频无入口。
- **目标**：仅保留历史编号和追踪关系，不再作为可领取任务。
- **范围**：记录旧设计意图、历史引用和迁移证据。
- **非范围**：不得提交新实现、测试或修复到此 EP。
- **替代 EP**：EP-1032, EP-1033。所有依赖方必须改指向满足真实需求的最小替代 EP。
- **修改位置**：仅允许 EP 注册表、追踪矩阵、迁移说明与历史链接。
- **交付物**：Superseded 标记与 `EP-1022 → EP-1032, EP-1033` 机器可读迁移记录。
- **验证**：EP-0010 确认无 Active EP 把 Superseded EP 当作唯一实现依赖；旧 ID 未复用；替代 EP 覆盖原 RQ/VAL。
- **DoD**：直接依赖已迁移；追踪闭包完整；旧编号可检索；执行看板不可领取。
- **主要风险**：依赖方误把旧编号当作已实现能力。CI 必须阻断新增实现依赖。
- **建议提交**：`docs(plan): 迁移 EP-1022 到替代 EP`

### EP-1116 完成 AST/path/network/Secret/Plugin/Web 安全审计

- **生命周期 / 状态 / 工作量**：Superseded / 未开始 / 不再估算
- **原阶段 / 版本 / 模块**：S11 / v0.9 / M-25b 质量加固
- **需求追踪**：RQ-047–056/096/101；原 VAL：`VAL-209`：零 P0/逃逸。
- **目标**：仅保留历史编号和追踪关系，不再作为可领取任务。
- **范围**：记录旧设计意图、历史引用和迁移证据。
- **非范围**：不得提交新实现、测试或修复到此 EP。
- **替代 EP**：EP-1123, EP-1124, EP-1125, EP-1126。所有依赖方必须改指向满足真实需求的最小替代 EP。
- **修改位置**：仅允许 EP 注册表、追踪矩阵、迁移说明与历史链接。
- **交付物**：Superseded 标记与 `EP-1116 → EP-1123, EP-1124, EP-1125, EP-1126` 机器可读迁移记录。
- **验证**：EP-0010 确认无 Active EP 把 Superseded EP 当作唯一实现依赖；旧 ID 未复用；替代 EP 覆盖原 RQ/VAL。
- **DoD**：直接依赖已迁移；追踪闭包完整；旧编号可检索；执行看板不可领取。
- **主要风险**：依赖方误把旧编号当作已实现能力。CI 必须阻断新增实现依赖。
- **建议提交**：`docs(plan): 迁移 EP-1116 到替代 EP`

### EP-1117 完成覆盖率、mutation、fuzz、E2E 门

- **生命周期 / 状态 / 工作量**：Superseded / 未开始 / 不再估算
- **原阶段 / 版本 / 模块**：S11 / v0.9 / M-25b 质量加固
- **需求追踪**：RQ-046；原 VAL：`VAL-210`：90/80/E2E。
- **目标**：仅保留历史编号和追踪关系，不再作为可领取任务。
- **范围**：记录旧设计意图、历史引用和迁移证据。
- **非范围**：不得提交新实现、测试或修复到此 EP。
- **替代 EP**：EP-1127, EP-1128, EP-1129, EP-1130。所有依赖方必须改指向满足真实需求的最小替代 EP。
- **修改位置**：仅允许 EP 注册表、追踪矩阵、迁移说明与历史链接。
- **交付物**：Superseded 标记与 `EP-1117 → EP-1127, EP-1128, EP-1129, EP-1130` 机器可读迁移记录。
- **验证**：EP-0010 确认无 Active EP 把 Superseded EP 当作唯一实现依赖；旧 ID 未复用；替代 EP 覆盖原 RQ/VAL。
- **DoD**：直接依赖已迁移；追踪闭包完整；旧编号可检索；执行看板不可领取。
- **主要风险**：依赖方误把旧编号当作已实现能力。CI 必须阻断新增实现依赖。
- **建议提交**：`docs(plan): 迁移 EP-1117 到替代 EP`

## 11. 版本 Gate 与完成判定

| Gate | 适用版本 | 必须证据 | 失败处理 |
|---|---|---|---|
| G-0 计划完整性 | 全部 | EP-0002/0003/0010：编号唯一、依赖闭包、RQ/AC/VAL 完整 | 禁止进入实现 |
| G-1 Workspace | v0.1+ | Cargo metadata、fmt、Clippy、workspace test 可运行 | 先修 EP-0101–0103 |
| G-2 领域与持久化 | v0.1+ | 事件 round-trip、迁移、故障注入、恢复 | 禁止上层依赖合并 |
| G-3 安全边界 | v0.1+ | 默认拒绝、AST/路径/Secret 负例 | 禁止工具/Provider 发布 |
| G-4 客户端闭环 | 每个客户端版本 | 启动、重连、快照+增量、取消、恢复 | 禁止打包 |
| G-5 质量与供应链 | v0.9+ | 依赖、许可证、SBOM、覆盖率、fuzz、安全审计 | 禁止 RC |
| G-6 发布与回滚 | v1.0+ | 干净机安装、升级、回滚、校验和、签名 | 禁止 GA |
| G-7/G-8 Trinity | v1.3 | 状态哈希、关键旅程、能力矩阵和例外 | 禁止宣称等价 |

版本完成必须同时满足：该版本所有 Active EP 完成；不存在指向未来版本的硬依赖；Gate 证据可在干净环境复现；例外有 owner、范围、失效条件和后续 EP。

## 12. 执行纪律

1. 一次只领取一个 EP；分支、PR、提交和验证证据都带 EP ID。
1. 先 RED 再 GREEN；测试不得只验证 happy path。
1. 发现设计缺陷先更新 EP/依赖/VAL，再写实现；禁止代码先合并、文档后补。
1. 公共契约变化先通过 drift 检查，并列出所有直接消费者。
1. 存储、安全、权限、恢复 EP 必须使用故障注入或恶意输入 fixture。
1. Superseded EP 不得复活；范围仍过大时继续追加新编号。
1. 提交遵循中文 Conventional Commits；一个提交只表达一个 EP 的一个可审查状态跃迁。

## 13. 机器校验基线

- 正式 EP 总数：256。
- Active：252；Superseded：4。
- 原设计文档主表：215；补录历史 EP：8；新增正式 EP：33。
- 新增 VAL：VAL-222–VAL-254，连续且一一对应新增 EP。
- 模块规范：保留 M-19a/M-19b、M-25a/M-25b；客户端固定为 M-26/M-27/M-28。
- 必查：EP 标题唯一、总表/卡片集合一致、依赖目标存在、Superseded 依赖完成迁移、版本逆向依赖为零。
