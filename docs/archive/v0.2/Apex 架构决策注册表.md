# Apex 架构决策注册表

ADR 解释“为什么这样选”，不重复主题文档的完整规范。状态为 `Accepted` 的决策若需改变，必须先标记 `Superseded` 并记录替代 ADR。

## 1. 注册表

| ADR | 决策 | 状态 | 主要规范 |
|---|---|---|---|
| ADR-001 | 每 OS 用户单一全局 `apexd` | Accepted | [总体架构](../02-system-architecture.md) |
| ADR-002 | Markdown 与 SQLite 分域事实源 | Accepted | [存储](../07-storage-files-logging.md) |
| ADR-003 | 原生客户端 gRPC，Web 使用 localhost REST/WS | Accepted | [协议](../06-protocol-and-clients.md) |
| ADR-004 | 运行生命周期使用事件+投影，配置/索引用普通表 | Accepted | [领域](../04-domain-model.md)、[存储](../07-storage-files-logging.md) |
| ADR-005 | 强制 Spec 流水线并允许显式审计跳过 | Accepted | [Spec](../08-spec-rules-verification.md) |
| ADR-006 | AST/arity/资源策略静态权限，禁止 LLM 判权 | Accepted | [权限](../09-tool-permission-terminal.md) |
| ADR-007 | Checkpoint-first 与四档有损兜底 | Accepted | [Context](../10-context-checkpoint-memory.md) |
| ADR-008 | 纯内容寻址 Snapshot，不使用 Shadow Git | Accepted | [Agent/DAG](../11-agent-dag-snapshot-replay.md) |
| ADR-009 | 共享工作区路径 Claim，必要时使用 worktree | Accepted | [Agent/DAG](../11-agent-dag-snapshot-replay.md) |
| ADR-010 | 四家独立 Provider Adapter + OpenAI-Compatible | Accepted | [Provider](../12-provider-multimodal.md) |
| ADR-011 | 外部 Skill 默认不信任，内容变化失效 | Accepted | [扩展](../13-skills-mcp-plugins.md) |
| ADR-012 | 官方签名 Plugin 可进程内，第三方必须 Plugin Host | Accepted | [扩展](../13-skills-mcp-plugins.md) |
| ADR-013 | Web 监听由 TUI 启用租约控制 | Accepted | [协议](../06-protocol-and-clients.md) |
| ADR-014 | 同一 Major Schema 只追加、旧版本容忍未知数据 | Accepted | [升级](../14-install-upgrade-operations.md) |
| ADR-015 | API Key 明文配置文件 + OS 权限，不用 SQLCipher | Accepted | [Provider](../12-provider-multimodal.md) |

## 2. 决策记录

### ADR-001：单一用户级 daemon

- 选择：每个 OS 用户一个 `apexd`，管理全部项目与客户端。
- 放弃：每项目 daemon（隔离更强但资源重复、跨项目会话与统一索引困难）；每客户端内嵌 Core（状态分叉）。
- 代价：必须提供单实例锁、崩溃恢复、资源配额与项目级隔离。
- 重审：若未来引入强租户隔离或系统服务账户，新增 ADR，不在本 ADR 内扩义。

### ADR-002：分域事实源

- 选择：Spec/Checkpoint/Memory/Verification 为文件事实；运行态为 SQLite 事实。
- 放弃：全部 SQLite（审计和 Git 评审差）；全部 Markdown（高频事件、索引和并发更新差）。
- 代价：跨域不存在天然 ACID，必须使用 generation、哈希、三方合并和 reconciliation。
- 重审：只有当产品放弃可编辑 Markdown 或改用具备事务文件语义的存储时。

### ADR-003：双传输

- 选择：TUI/Desktop 走本地 gRPC，Web 走 REST/WebSocket。
- 放弃：所有端统一 HTTP（原生流与类型契约较弱）；所有端统一 gRPC-Web（浏览器部署与安全复杂）。
- 代价：需共享应用命令模型，并对两种适配器做等价性契约测试。

### ADR-004：混合 SQLite 模型

- 选择：会话生命周期使用追加事件与 Reducer 投影；稳定配置和索引用普通表。
- 放弃：全量 Event Sourcing（配置读取和迁移负担过大）；全 CRUD（恢复/审计/重放不足）。
- 代价：需明确哪些表可直接更新，严禁把详细日志伪装成领域事件。

### ADR-005：强制 Spec，可审计跳过

- 选择：默认硬门，用户可以显式 `/skip-spec`，但无法静默跳过审计。
- 放弃：仅提示式 Spec（无法保证）；绝不允许跳过（应急和探索体验过差）。
- 代价：需要安全点、审批失效传播和 Skip Scope 状态机。

### ADR-006：静态权限

- 选择：tree-sitter AST、arity 语义规则与资源策略组合，权限判断不调用 LLM。
- 放弃：Prompt 判权（非确定、耗 Token、可被注入）；纯命令字符串正则（无法理解复合 Shell）。
- 代价：复杂动态语法会保守询问；三个 Shell 家族需独立解析和契约测试。

### ADR-007：Checkpoint-first

- 选择：无损 Checkpoint 为主，snip/prune/摘要只是兜底。
- 放弃：仅摘要（丢信息）；无限 Context（成本和模型上限不可控）。
- 代价：需要内容寻址片段、附件、章节预算、保留与 GC。

### ADR-008：内容寻址 Snapshot

- 选择：以文件内容块和 Manifest 捕获/恢复，不创建隐藏 Git 分支或提交。
- 放弃：Shadow Git（成熟但污染/依赖 Git 语义）；全目录复制（空间和延迟高）。
- 代价：需自行处理权限位、符号链接、大小写、原子恢复和 GC。

### ADR-009：共享工作区 Claim

- 选择：默认共享项目目录，调度器对规范化 `write_paths` 租约互斥；高风险可用 worktree。
- 放弃：所有任务 worktree（合并成本高、用户不可实时看到）；全串行（吞吐低）。
- 代价：路径重叠、symlink、大小写折叠、父子 Agent 预留与公平队列都必须严格定义。

### ADR-010：Provider 专属优化通道

- 选择：Anthropic/OpenAI/DeepSeek/Kimi 独立 crate，共享最小核心抽象；其他先走兼容端点。
- 放弃：单一 OpenAI 格式抽象全部 Provider（会丢失 reasoning、cache、realtime 等能力）。
- 代价：适配器数量和契约测试矩阵扩大。

### ADR-011：Skill 内容信任

- 选择：外部 Skill 初始不信任；路径、哈希/签名和来源构成信任记录，内容变化即失效。
- 放弃：按目录永久信任（供应链风险）；完全禁止外部 Skill（生态不兼容）。
- 代价：升级后可能需要再次确认，UI 必须解释变化。

### ADR-012：Plugin 隔离

- 选择：只有 Apex 官方签名 Plugin 可进程内；第三方动态库在独立 Host 加载。
- 放弃：全部进程内（无法限制内存破坏/崩溃）；全部 WASM（不符合原生 Rust Plugin 要求且生态能力受限）。
- 代价：Plugin Host IPC、ABI 版本、进程监督和能力代理更复杂。

### ADR-013：TUI 控制 Web 生命周期

- 选择：Web server 位于 `apexd`，由 TUI 的可续租启用租约控制。
- 放弃：daemon 永久监听（扩大攻击面）；Web 独立进程（状态/安装复杂）。
- 代价：TUI 崩溃时必须可靠过期租约并关闭 listener，浏览器会话随之失联。

### ADR-014：同 Major 前向容忍

- 选择：Schema/事件/字段只追加；旧版本忽略并保留未知信息，新能力只读或隐藏。
- 放弃：严格版本锁（回滚困难）；旧版本可写所有新状态（可能破坏语义）。
- 代价：同一 Major 不能清理旧结构，写入需 feature ownership 与 min-writer-version 门。

### ADR-015：明文 Key + 文件权限

- 选择：Key 保存在 `~/.apex/config/providers.toml`，用 0600/用户 ACL 保护，不进入 SQLite。
- 放弃：OS Keychain（跨平台和自动化复杂，用户已选择配置文件）；SQLCipher（不解决运行进程读取，迁移复杂）。
- 代价：磁盘被同用户恶意进程读取时无法防护；必须提供权限诊断、Secret Firewall 与高风险提示。
