# Apex 需求基线与追踪矩阵

## 1. 文档状态

- 需求等级：L4（跨客户端、存储、安全、恢复、扩展与多 Agent 的系统级设计）。
- 确定性：用户已逐项澄清并明确确认，设计阶段无阻塞问题。
- 本轮范围：只生成目标架构文档，不实现代码，不兼容旧 Cargo 结构。
- 产品策略：直接设计完整产品；实施可分阶段，但不存在降低需求的 MVP 产品分支。

## 2. 产品目标

Apex 要成为一款本地优先、可审计、可恢复的编程 Agent。用户可以从 TUI、桌面端或 Web 端进入同一会话，观察 Skill、MCP、Subagent、Tool、权限与 DAG 的实时状态；所有编码工作受 Spec、静态权限、Checkpoint 和验证门控制。

## 3. 范围边界

### 3.1 范围内

- Rust 常驻服务、TUI、本地 gRPC、Actix REST/WebSocket、Tauri + Vue/TypeScript。
- 多 Provider、多模态、Skills、MCP、原生 Plugin、静态权限、持久终端。
- Spec、Rules、Checkpoint、Memory、DAG、Snapshot、重放、日志、归档和升级。
- macOS、Windows、Linux 的 x86_64 与 ARM64 发布。

### 3.2 明确不包含

- 云端 SaaS 控制面、组织/租户管理、Marketplace、自动遥测或自动崩溃上传。
- 实时视频、基于 LLM 的权限判断、QuickJS/任意调度脚本、Shadow Git Snapshot。
- 同一用户在多台机器之间的内建同步；当前“跨端”指同一机器上的三个客户端。
- 本轮代码实现、构建产物和数据库迁移。

## 4. 需求追踪矩阵

状态均为“已确认”。“落点”指该要求的主要权威设计文档，相关主题可能在其他文档中被引用。

### 4.1 产品、进程与客户端（RQ-001–RQ-024）

| ID | 已确认要求 | 主要落点 |
|---|---|---|
| RQ-001 | 目标为完整产品，不另设削减功能的 MVP 架构。 | [15](15-quality-risks-roadmap.md) |
| RQ-002 | 采用绿地重构，不受旧 Cargo Workspace 和旧实现兼容约束。 | [03](03-workspace-and-crates.md) |
| RQ-003 | 当前交付只生成文档，不进行代码开发。 | 本文、[15](15-quality-risks-roadmap.md) |
| RQ-004 | 支持 macOS、Windows、Linux。 | [14](14-install-upgrade-operations.md) |
| RQ-005 | 每个 OS 同时支持 x86_64 与 ARM64。 | [14](14-install-upgrade-operations.md) |
| RQ-006 | 每个 OS 用户只运行一个全局 `apexd`。 | [02](02-system-architecture.md) |
| RQ-007 | `apexd` 统一管理多项目，并使用一套用户级 SQLite。 | [07](07-storage-files-logging.md) |
| RQ-008 | Apex Home 在所有平台统一表示为 `~/.apex/`。 | [07](07-storage-files-logging.md) |
| RQ-009 | TUI 与 Tauri 桌面端通过本地 gRPC 访问 `apexd`。 | [06](06-protocol-and-clients.md) |
| RQ-010 | Unix 平台的本地 gRPC 使用 Unix Domain Socket。 | [06](06-protocol-and-clients.md) |
| RQ-011 | Windows 的本地 gRPC 使用 Named Pipe。 | [06](06-protocol-and-clients.md) |
| RQ-012 | Web 端使用 Actix REST + WebSocket。 | [06](06-protocol-and-clients.md) |
| RQ-013 | Web 监听只能绑定 localhost。 | [14](14-install-upgrade-operations.md) |
| RQ-014 | Actix Web 运行在 `apexd` 内，且只有 TUI 持有启用租约时才开放。 | [06](06-protocol-and-clients.md) |
| RQ-015 | TUI Web 租约失效后，`apexd` 必须关闭 Web 监听。 | [06](06-protocol-and-clients.md) |
| RQ-016 | Web 使用一次性令牌换短期 Cookie，并校验 Origin 与 CSRF。 | [06](06-protocol-and-clients.md) |
| RQ-017 | 桌面端与 Web 共用 Vue/TS 应用，以 Platform Adapter 区分传输。 | [06](06-protocol-and-clients.md) |
| RQ-018 | 三端核心功能等价，并明确能力差异。 | [06](06-protocol-and-clients.md) |
| RQ-019 | TUI 不提供日志查看能力。 | [06](06-protocol-and-clients.md)、[07](07-storage-files-logging.md) |
| RQ-020 | TUI 不支持音频与实时语音。 | [12](12-provider-multimodal.md) |
| RQ-021 | 会话控制权采用“先来先控制”的单控制租约。 | [06](06-protocol-and-clients.md) |
| RQ-022 | 控制端断线后保留 30 秒租约宽限。 | [06](06-protocol-and-clients.md) |
| RQ-023 | 其他客户端可以显式强制接管控制权，并留下审计记录。 | [06](06-protocol-and-clients.md) |
| RQ-024 | 控制端断开后默认继续执行，也可按项目策略在安全点暂停。 | [06](06-protocol-and-clients.md) |

### 4.2 数据权威与目录（RQ-025–RQ-035）

| ID | 已确认要求 | 主要落点 |
|---|---|---|
| RQ-025 | Spec、Checkpoint、Memory、最终验证报告以 Markdown/文件系统为事实源。 | [07](07-storage-files-logging.md) |
| RQ-026 | SQLite 保存会话、消息、Agent/Tool/Permission/DAG 状态、最小领域事件、投影和 FTS。 | [07](07-storage-files-logging.md) |
| RQ-027 | SQLite 事件不是日志；事件与文件日志通过 `event_id`/`trace_id` 关联。 | [07](07-storage-files-logging.md) |
| RQ-028 | Markdown 自动监听，同时提供显式重载。 | [07](07-storage-files-logging.md) |
| RQ-029 | 外部修改冲突优先三方合并；无法合并时暂停并等待人工处理。 | [07](07-storage-files-logging.md) |
| RQ-030 | 单项目 Spec 路径为 `specs/<feature>/{requirements,design,tasks,verification}.md`。 | [08](08-spec-rules-verification.md) |
| RQ-031 | 单项目运行文件位于 `.apex/{checkpoints,memory,snapshots,runtime}`。 | [07](07-storage-files-logging.md) |
| RQ-032 | 默认提交 Spec、验证报告和 Memory；忽略 Checkpoint、Snapshot、附件、缓存与日志。 | [07](07-storage-files-logging.md) |
| RQ-033 | 多根 Workspace 的 Spec/Checkpoint/工作流事实源位于 `~/.apex/workspaces/<workspace-id>/`。 | [07](07-storage-files-logging.md) |
| RQ-034 | 多根 Workspace 的每个根仍维护自己的 `.apex/memory/`。 | [10](10-context-checkpoint-memory.md) |
| RQ-035 | 多根 Workspace 必须指定审计根，并镜像 Spec 与最终验证报告。 | [07](07-storage-files-logging.md) |

### 4.3 Spec、Rules 与验证（RQ-036–RQ-046）

| ID | 已确认要求 | 主要落点 |
|---|---|---|
| RQ-036 | 强制执行需求 → 设计 → 任务 → 编码 → 验证流水线。 | [08](08-spec-rules-verification.md) |
| RQ-037 | 默认逐阶段审批；项目策略可改为三个 Spec 文档整体审批。 | [08](08-spec-rules-verification.md) |
| RQ-038 | 已批准 Spec 一旦变化立即失效，在下一安全点暂停并回改下游。 | [08](08-spec-rules-verification.md) |
| RQ-039 | `/skip-spec` 可跳阶段或全流程、作用于 Run/Session，并记录完整审计字段。 | [08](08-spec-rules-verification.md) |
| RQ-040 | 每个功能必须生成 `verification.md`。 | [08](08-spec-rules-verification.md) |
| RQ-041 | 默认由用户确认后完成；项目策略可允许自动验证通过即完成。 | [08](08-spec-rules-verification.md) |
| RQ-042 | 每次文件修改后同步执行轻量安全、格式和语法检查。 | [08](08-spec-rules-verification.md) |
| RQ-043 | 重型 lint/test/静态分析按增量批次执行，并在完成门统一强制。 | [08](08-spec-rules-verification.md) |
| RQ-044 | 增量自动修复默认 2 轮、可配置 1–5 轮，且不得扩大 `write_paths` 或权限。 | [08](08-spec-rules-verification.md) |
| RQ-045 | 内置规则覆盖 Rust、Go、Java、Python、TS/JS、Vue。 | [08](08-spec-rules-verification.md) |
| RQ-046 | 权限/调度/Spec/恢复覆盖率不低于 90%，其他 Rust 与 Vue/TS 不低于 80%，关键三端流程必须 E2E。 | [15](15-quality-risks-roadmap.md) |

### 4.4 Tool、权限与终端（RQ-047–RQ-058）

| ID | 已确认要求 | 主要落点 |
|---|---|---|
| RQ-047 | `plan` 模式只读且无副作用。 | [09](09-tool-permission-terminal.md) |
| RQ-048 | `ask` 模式对白名单内操作自动放行，其余询问。 | [09](09-tool-permission-terminal.md) |
| RQ-049 | `allow` 模式对静态策略允许项自动放行，硬禁止不可绕过。 | [09](09-tool-permission-terminal.md) |
| RQ-050 | 权限判断必须零 Token，不允许调用 LLM。 | [09](09-tool-permission-terminal.md) |
| RQ-051 | 完整解析 sh/bash/zsh、PowerShell 7 与 cmd.exe。 | [09](09-tool-permission-terminal.md) |
| RQ-052 | 权限覆盖 Tool、文件读写、命令/程序/参数语义、网络目标、凭据/环境变量。 | [09](09-tool-permission-terminal.md) |
| RQ-053 | AST 未知/失败时：plan 拒绝，ask 询问，allow 也降级为询问。 | [09](09-tool-permission-terminal.md) |
| RQ-054 | 授权期限支持单次、Run、Session、Project；不提供用户级全局授权。 | [09](09-tool-permission-terminal.md) |
| RQ-055 | OS 沙箱是可选增强，默认安全基础为静态策略。 | [09](09-tool-permission-terminal.md) |
| RQ-056 | 未信任项目在用户确认前连读取都禁止。 | [09](09-tool-permission-terminal.md) |
| RQ-057 | 默认创建持久 PTY/ConPTY，也支持一次性非交互命令。 | [09](09-tool-permission-terminal.md) |
| RQ-058 | UI 展示一个共享逻辑终端；并发 Agent 使用隔离通道并按 Agent/Task/trace 归因。 | [09](09-tool-permission-terminal.md) |

### 4.5 Agent、DAG、Snapshot 与重放（RQ-059–RQ-073）

| ID | 已确认要求 | 主要落点 |
|---|---|---|
| RQ-059 | 可写 Subagent 必须声明 `write_paths`。 | [11](11-agent-dag-snapshot-replay.md) |
| RQ-060 | 默认共享工作区，通过规范化路径 Claim/租约实现互斥。 | [11](11-agent-dag-snapshot-replay.md) |
| RQ-061 | 高风险任务可切换到隔离 worktree。 | [11](11-agent-dag-snapshot-replay.md) |
| RQ-062 | 扩展写路径必须暂停、修改 `tasks.md`/工作流并重新审批。 | [11](11-agent-dag-snapshot-replay.md) |
| RQ-063 | 默认并发为全局 `min(8, CPU)`、写 Agent 4、单 Provider 4，硬上限 `min(32, 2×CPU)`。 | [11](11-agent-dag-snapshot-replay.md) |
| RQ-064 | DAG 来源为已批准 `tasks.md` 与 `.apex/workflows/*.yaml`。 | [11](11-agent-dag-snapshot-replay.md) |
| RQ-065 | 不使用 QuickJS 或任意调度脚本。 | [11](11-agent-dag-snapshot-replay.md) |
| RQ-066 | Subagent 默认由父 Agent 汇聚；仅显式 DAG 通信边允许持久邮箱。 | [11](11-agent-dag-snapshot-replay.md) |
| RQ-067 | 汇聚冲突由受限 Merge Subagent 尝试三方合并，失败转人工。 | [11](11-agent-dag-snapshot-replay.md) |
| RQ-068 | 崩溃后只自动继续可证明幂等节点；未知副作用保持阻塞。 | [11](11-agent-dag-snapshot-replay.md) |
| RQ-069 | 部分回滚使用补偿式恢复，历史事件不可删除。 | [11](11-agent-dag-snapshot-replay.md) |
| RQ-070 | Snapshot 使用纯内容寻址文件快照，不用 Shadow Git，也不污染用户 Git。 | [11](11-agent-dag-snapshot-replay.md) |
| RQ-071 | 确定性状态重放复用已记录结果，不重新执行副作用。 | [11](11-agent-dag-snapshot-replay.md) |
| RQ-072 | 再执行重放可重新调用 LLM/Tool，继承原权限，展示副作用清单并整体确认，只承诺尽力复现。 | [11](11-agent-dag-snapshot-replay.md) |
| RQ-073 | 面板实时展示 Skill 名称、MCP 服务名称和 Subagent 的具体任务描述。 | [06](06-protocol-and-clients.md) |

### 4.6 Context、Checkpoint 与 Memory（RQ-074–RQ-083）

| ID | 已确认要求 | 主要落点 |
|---|---|---|
| RQ-074 | Context 阈值为 60% 软提示、70% snip、80% prune、90% LLM 摘要。 | [10](10-context-checkpoint-memory.md) |
| RQ-075 | 摘要可配置独立模型，未配置时回退当前模型。 | [10](10-context-checkpoint-memory.md) |
| RQ-076 | 每 Turn 成功结束、任何有损处理前、暂停/退出前、高风险写前强制 Checkpoint。 | [10](10-context-checkpoint-memory.md) |
| RQ-077 | `checkpoint.md` 为清单，引用内容寻址片段和多模态附件，共同支持无损重建。 | [10](10-context-checkpoint-memory.md) |
| RQ-078 | Checkpoint 活跃期全保留，120 天归档，365 天删除，Pinned 永久。 | [10](10-context-checkpoint-memory.md) |
| RQ-079 | Memory 同时支持项目级 `.apex/memory/` 与全局 `~/.apex/memory/`。 | [10](10-context-checkpoint-memory.md) |
| RQ-080 | Agent 可自动写 Memory，但必须记录来源、理由与作用域。 | [10](10-context-checkpoint-memory.md) |
| RQ-081 | 疑似敏感 Memory 默认阻止，必须逐次确认后写入。 | [10](10-context-checkpoint-memory.md) |
| RQ-082 | FTS5 tokenizer 可选 `unicode61`/`jieba-rs`，中文默认 jieba。 | [10](10-context-checkpoint-memory.md) |
| RQ-083 | UI 支持查看 Memory 引用时机、删除和导出。 | [10](10-context-checkpoint-memory.md) |

### 4.7 Provider 与多模态（RQ-084–RQ-093）

| ID | 已确认要求 | 主要落点 |
|---|---|---|
| RQ-084 | Anthropic、OpenAI、DeepSeek、Kimi 各自拥有独立适配 crate。 | [12](12-provider-multimodal.md) |
| RQ-085 | 通义、智谱及其他首版使用 OpenAI-Compatible，并保留后续专属适配通道。 | [12](12-provider-multimodal.md) |
| RQ-086 | 支持文本、Tool、推理、图片、文件、音频、实时双向语音和视频文件。 | [12](12-provider-multimodal.md) |
| RQ-087 | 不支持实时视频。 | [12](12-provider-multimodal.md) |
| RQ-088 | 桌面/Web 支持音频和实时语音；TUI 不支持音频。 | [12](12-provider-multimodal.md) |
| RQ-089 | 默认不自动切换 Provider，但支持配置故障转移链路。 | [12](12-provider-multimodal.md) |
| RQ-090 | Subagent 默认继承父模型，Agent Profile 或 DAG 节点可覆盖 Provider/模型。 | [12](12-provider-multimodal.md) |
| RQ-091 | API Key 明文保存于 `~/.apex/config/providers.toml`，Unix 0600，Windows 当前用户 ACL。 | [12](12-provider-multimodal.md) |
| RQ-092 | API Key 不得进入 SQLite、日志、Spec、Checkpoint 或 Memory。 | [12](12-provider-multimodal.md) |
| RQ-093 | SQLite 不启用 SQLCipher。 | [07](07-storage-files-logging.md) |

### 4.8 Skills、MCP 与 Plugin（RQ-094–RQ-102）

| ID | 已确认要求 | 主要落点 |
|---|---|---|
| RQ-094 | Skill 扫描器可插拔，首版完整兼容 Claude 与 Codex Skills 目录/格式。 | [13](13-skills-mcp-plugins.md) |
| RQ-095 | Skill frontmatter 支持 Apex 扩展字段以绑定流水线阶段。 | [13](13-skills-mcp-plugins.md) |
| RQ-096 | 外部 Skill 默认不信任，哈希/签名变化使信任失效，脚本必须经过 Tool Gateway。 | [13](13-skills-mcp-plugins.md) |
| RQ-097 | MCP 自动扫描 Claude Desktop/Code、Cursor、VS Code、Codex 与 Apex 配置。 | [13](13-skills-mcp-plugins.md) |
| RQ-098 | MCP 只发现不自动启动，面板支持一键启停。 | [13](13-skills-mcp-plugins.md) |
| RQ-099 | Apex 默认只保存启用覆盖，只有用户显式操作才回写来源配置。 | [13](13-skills-mcp-plugins.md) |
| RQ-100 | Plugin 是原生 Rust 动态库，支持进程内和独立 Plugin Host。 | [13](13-skills-mcp-plugins.md) |
| RQ-101 | 仅 Apex 官方签名 Plugin 可进程内；第三方 Plugin 必须独立进程。 | [13](13-skills-mcp-plugins.md) |
| RQ-102 | 不建设 Marketplace，只支持本地目录、Git 与文件包安装。 | [13](13-skills-mcp-plugins.md) |

### 4.9 SQLite、日志与归档（RQ-103–RQ-111）

| ID | 已确认要求 | 主要落点 |
|---|---|---|
| RQ-103 | SQLite 采用“运行生命周期事件+投影，配置/索引普通表”的混合模型。 | [07](07-storage-files-logging.md) |
| RQ-104 | WAL 默认 `synchronous=NORMAL`，Checkpoint/审批等关键事务临时使用 FULL。 | [07](07-storage-files-logging.md) |
| RQ-105 | 仅在升级、迁移和高风险恢复前自动备份。 | [14](14-install-upgrade-operations.md) |
| RQ-106 | 会话运行数据 120 天后归档并移出主库、查询时只读挂载、继续时恢复、365 天删除。 | [07](07-storage-files-logging.md) |
| RQ-107 | 会话日志为按 Session 分割的 JSONL，文件名含时间和 Session ID，10 MiB 轮转，保留 120 天，Desktop/Web 可看而 TUI 不可看。 | [07](07-storage-files-logging.md) |
| RQ-108 | 会话日志默认仅记录元数据/摘要/长度/哈希；单会话全文调试须显式开启并高风险提示。 | [07](07-storage-files-logging.md) |
| RQ-109 | 每条会话日志形成哈希链，每段由 `~/.apex/keys/` 中的 Ed25519 密钥签名。 | [07](07-storage-files-logging.md) |
| RQ-110 | 系统日志为人类可读文本、每日一个逻辑文件、10 MiB 分段并保留 60 天。 | [07](07-storage-files-logging.md) |
| RQ-111 | 同一 Major 内旧版本可打开最新 Schema：保留未知字段/表/事件，新功能只读/不可见，禁止删除、改名或改变既有语义。 | [14](14-install-upgrade-operations.md) |

### 4.10 发布、隐私与非功能需求（RQ-112–RQ-115）

| ID | 已确认要求 | 主要落点 |
|---|---|---|
| RQ-112 | 发布通道为 Stable、Nightly、Development、Enterprise，并按已确认策略提示/下载/安全点安装；Enterprise 可用管理员私有更新源但无组织管理。 | [14](14-install-upgrade-operations.md) |
| RQ-113 | 无遥测、无自动崩溃上传，只提供手动生成的脱敏诊断包。 | [14](14-install-upgrade-operations.md) |
| RQ-114 | 满足启动、命令确认、跨端事件、分页、Memory 搜索和空闲内存六项性能目标。 | [15](15-quality-risks-roadmap.md) |
| RQ-115 | 简体中文和英文完整支持，其他语言通过语言包扩展。 | [06](06-protocol-and-clients.md) |

## 5. 产品验收标准

| AC | 场景 | Given | When | Then |
|---|---|---|---|---|
| AC-001 | 三端共享会话 | 任一客户端创建会话 | 其他客户端连接同一 `apexd` | 250 ms 内可查询/订阅到同一权威状态 |
| AC-002 | Web 租约 | TUI 未持有 Web 租约 | 探测 Web 端口 | 无监听；获得租约后才开放 localhost |
| AC-003 | Spec 编码门 | Spec 未批准或批准已失效 | Agent 请求写代码 | 在安全点前被拒绝/暂停并显示原因 |
| AC-004 | Skip 审计 | 用户显式执行 `/skip-spec` | Run/Session 继续 | 范围、理由、操作者、时间、需求、trace 均可查 |
| AC-005 | Markdown 事实源 | 外部编辑 Spec/Checkpoint/Memory | watcher 对账 | 投影更新；冲突三方合并或人工阻塞，不静默覆盖 |
| AC-006 | 静态权限 | 命令含文件、网络或凭据副作用 | 权限引擎评估 | 不调用 LLM，并按模式/AST/白名单确定 allow/ask/deny |
| AC-007 | 控制租约 | 两个客户端同时请求控制 | 先到客户端获得租约 | 后到客户端只读，除非显式强制接管并审计 |
| AC-008 | 路径互斥 | 并行写 Agent 的规范化路径重叠 | 调度器分配任务 | 冲突任务不并发，非冲突任务不受队首阻塞 |
| AC-009 | 崩溃恢复 | daemon 在 Tool 或 DAG 运行时崩溃 | 重启并恢复 | 幂等节点可继续，未知副作用节点阻塞且历史完整 |
| AC-010 | Checkpoint 重建 | Context 经 snip/prune/摘要 | 从最新 Checkpoint 恢复 | 用户原始意图、消息、Tool 结果、附件与状态可无损重建 |
| AC-011 | 状态重放 | 选择确定性重放 | 回放历史 | 不重跑外部副作用；投影结果与已记录事实一致 |
| AC-012 | 再执行重放 | 选择重新调用 LLM/Tool | 用户确认副作用清单 | 使用原权限边界尽力复现，并生成新 Run/trace |
| AC-013 | Memory 召回 | 存在中英文项目/全局记忆 | 新 Turn 关键词匹配 | 可检索、可解释引用时机、可删除与导出 |
| AC-014 | Provider 可替换 | 同一 Agent Profile 切换兼容模型 | 执行文本/Tool 流程 | 核心循环不依赖厂商类型，专属能力通过 capability 协商 |
| AC-015 | 多模态能力 | Desktop/Web 选择受支持 Provider | 上传/流式输入 | 图片、文件、音频、语音或视频文件按能力降级；无实时视频入口 |
| AC-016 | 扩展信任 | 外部 Skill/MCP/Plugin 首次发现或内容变化 | 用户尝试启用 | 未信任内容不自动执行，脚本/第三方插件处于受控边界 |
| AC-017 | 日志完整性 | 会话产生日志并发生轮转 | 离线验证日志段 | 哈希链与 Ed25519 签名可验证，且 trace/event 可关联 |
| AC-018 | 归档生命周期 | 会话超过 120/365 天 | 运行清理任务 | 120 天归档可查/可恢复，365 天删除；Pinned Checkpoint 保留 |
| AC-019 | 版本兼容 | 同一 Major 的旧 Apex 打开新 Schema | 读取未知结构 | 不破坏数据，未知结构被保留，新能力只读或不可见 |
| AC-020 | 完成门 | 所有实现任务结束 | 触发最终验证 | 生成 `verification.md`，覆盖率/E2E/NFR/风险证据满足策略后才可完成 |

## 6. 设计通过条件

- 115 个 `RQ` 均在目标文档中有明确落点。
- 所有核心状态、ID、Trait、Wire 类型和错误语义只有一个权威定义。
- L4 所需总体架构图、部署图、流程图、时序图、ER 图、状态机和异常恢复图齐备。
- 风险清单覆盖跨平台 IPC、文件/数据库双事实域、静态命令分析、第三方扩展、重放副作用、Schema 兼容与日志密钥。
- 本文经用户审核后，方可把后续实现计划转入编码阶段；当前文档任务不会进入编码。
