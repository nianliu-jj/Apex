# Apex—— MCP、Skill、Hook与Plugin扩展系统详细设计

> 文档状态：详细设计基线（面向最终完整产品）  
> 适用版本：Apex v0.1 ～ v1.x（按 §交付阶段 分档启用；档位表以需求文档 §5.3 为准）  
> 上游文档：`Apex—— 需求分析文档.md`、`Apex—— 系统总体架构设计.md`、`Apex—— 领域模型与事件规范.md`、`Apex—— API与实时事件协议设计.md`、`Apex—— SQLite数据模型与迁移设计.md`、`Apex—— Agent Runtime与DAG调度器详细设计.md`、`Apex—— Tool Gateway与权限引擎详细设计.md`、`Apex—— Context与Checkpoint系统详细设计.md`、`Apex—— Rules与Verification Gate详细设计.md`  
> 参考实现：Claude Code、opencode、pi、CodeWhale、DeepSeek-Reasonix、DeepSeek-TUI、MiMo-Code 的项目内实现原理分析  
> 关键词：MCP、Skill、Hook、Plugin、Extension、Capability、Trust、Sandbox、Taint、Schema Revision、Supervision

---

## 0. 文档目的与范围

本文定义 Apex 最终完整产品中的统一扩展架构，回答以下核心问题：

1. Skill、MCP、Hook、Plugin 各自是什么，边界在哪里；
2. 外部扩展如何被发现、安装、校验、授权、启用、执行、更新和卸载；
3. 扩展如何接入 Agent Runtime、Context、Tool Gateway、Rules、Verification Gate、CredentialStore 与事件系统；
4. 如何在兼容现有生态的同时，避免第三方代码绕过 Core、权限、审计与工作区隔离；
5. 如何保证扩展版本、工具 schema、审批依据和运行结果可追溯、可恢复、可重放；
6. 如何将 Skill 的 token 经济性、MCP 的生态连接能力、Hook 的策略扩展能力和 Plugin 的产品扩展能力组合为一个可治理平台。

本文覆盖：

- Skill 包格式、发现、渐进式加载、触发和资源访问；
- MCP 配置发现、连接监督、能力注册、调用、重连和外部副作用对账；
- Hook 事件、匹配、顺序、输入输出、超时和失败策略；
- Plugin manifest、安装、签名、依赖、运行时、SDK 和版本兼容；
- 统一 Extension Registry、Capability、Trust、Sandbox、Taint 和可观测性；
- API、事件、持久化、崩溃恢复、测试和交付阶段。

本文不重新定义：

- Tool Gateway 的通用判权算法；
- Workspace Snapshot、Write Claim 和隔离工作区算法；
- Context/Checkpoint 的主体模型；
- RuleCheck、Gate、Repair Run 和 Waiver 的主体模型；
- CredentialStore 的具体加密与密钥轮换实现。

这些能力由上游详细设计负责，本文只定义扩展系统如何调用它们。

---

## 1. 核心架构结论

### 1.1 四类扩展不是四套孤立系统

Apex 使用统一的 Extension Control Plane 管理四类扩展，但保留各自不同的运行语义：

| 类型 | 核心职责 | 默认是否执行代码 | 是否可贡献 Tool | 是否进入 Context | 典型风险 |
|---|---|---:|---:|---:|---|
| Skill | 可复用知识、流程和资源包 | 否；脚本需另行调用 | 可选 | 是，渐进加载 | Prompt 注入、资源越界、脚本副作用 |
| MCP Server | 外部 Tool、Resource、Prompt 提供方 | 是，进程或远程服务 | 是 | 结果可进入 | 外部副作用、数据外发、schema 漂移 |
| Hook | 在受控事件点观察、阻断或提出诊断 | 是 | 否，除非属于 Plugin | 通常不直接注入 | 越权改写、死锁、循环触发 |
| Plugin | 打包并注册受限扩展点的产品模块 | 是 | 可选 | 可选 | 供应链、常驻代码、协议兼容 |

统一管理并不意味着统一权限。每次实际操作仍映射为明确的 Capability 和 ToolCall；安装、启用或加载扩展永远不等价于授权其执行任意动作。

### 1.2 Core 是唯一控制面

必须满足：

```text
UI / CLI / Agent / SubAgent / Skill / MCP / Hook / Plugin
                         │
                         ▼
                  Application Command
                         │
                         ▼
                 Apex Core authoritative state
                         │
       ┌─────────────────┼──────────────────┐
       ▼                 ▼                  ▼
Extension Registry   Tool Gateway     Context / Rules / DAG
       │                 │                  │
       └─────────────────┴──────────────────┘
                         │
                         ▼
                 Event + Audit + Projection
```

任何扩展都不得：

- 直接写 Apex 业务数据库；
- 直接改变 Session、Run、Gate、Permission 或 Checkpoint 的权威状态；
- 绕过 Tool Gateway 操作文件、Shell、Git、网络、MCP 或 Credential；
- 将外部文本提升为 system contract；
- 通过 Hook 静默放宽权限或伪造验证通过；
- 依赖 UI 状态作为正确性依据。

### 1.3 版本不可变，运行按摘要绑定

ExtensionDefinition 表示逻辑扩展；ExtensionRevision 表示不可变内容版本。运行、审批、缓存和审计均绑定 revision digest，而不是可变路径或显示版本号。

```text
extension_id = ext_...
revision_id  = exr_...
manifest_digest = sha256:...
content_digest  = sha256:...
registry_generation = 42
```

文件发生变化时创建新 revision。正在运行的 ToolCall、HookInvocation、SkillLoad 和 MCPCall 继续绑定旧 revision；新操作才使用已激活的新 revision。

### 1.4 权限只能收窄，不能继承放大

有效 Capability 为多层约束的交集：

```text
effective_capabilities =
    product_hard_ceiling
  ∩ organization_policy
  ∩ project_trust
  ∩ user_grant
  ∩ session_mode
  ∩ agent_profile
  ∩ extension_manifest_request
  ∩ extension_install_grant
  ∩ operation_specific_grant
  ∩ sandbox_backend_support
```

任一层拒绝即拒绝。Skill 的 `allowed-tools`、Plugin 的 manifest、MCP server 的能力声明均只用于进一步收窄或提出申请，不能提升父级权限。

### 1.5 第三方原生代码不得进 Core 进程

运行策略采用混合模型：

- **内置、随 Apex 发布且经过同一供应链的 Adapter**：允许在 Core 进程内运行；
- **纯计算、可移植、低权限 Plugin**：优先 Wasm/WASI 沙箱；
- **需要现有生态二进制、OS API、语言运行时或 stdio 的扩展**：受监督子进程；
- **远程 MCP**：经网络策略和 CredentialStore 访问；
- **任意第三方 native dynamic library**：v1 不允许加载到 Core 地址空间。

该结论解决总体架构中“Wasm 或受监督子进程”的 ADR：最终产品不是二选一，而是按扩展能力采用分级后端，默认不信任。

### 1.6 自动发现不等于自动信任

本地扫描只产生 Candidate：

```text
discovered → parsed → validated → awaiting_trust → enabled
```

Apex 可以自动发现 `.mcp.json`、兼容 Skill 目录和 Plugin 包，但不得自动启动未知进程、向远程主机连接、执行脚本或读取 Secret。用户/组织策略明确授予信任后才能启用。

---

## 2. 设计目标与非目标

### 2.1 设计目标

1. **生态兼容**：兼容 YAML frontmatter + Markdown body 的 Agent Skills；兼容常用 MCP 配置和标准协议能力。
2. **安全默认值**：未知来源禁用；Secrets 与配置分离；外部内容默认 tainted。
3. **统一治理**：所有工具、副作用、网络、Credential、工作区写入统一经 Tool Gateway。
4. **渐进披露**：大量 Skills 共存时仍保持 Context 可控。
5. **确定性**：Hook 顺序、版本选择、schema 绑定和审批依据稳定。
6. **可恢复**：Core 或扩展崩溃后可区分“未执行”“已执行”“结果未知”。
7. **可审计**：从一次模型决策追溯到 Skill、Hook、Plugin、MCP schema 和结果摘要。
8. **可演进**：稳定 protocol 与 SDK，允许扩展和 Core 独立升级。
9. **跨平台**：Windows、macOS、Linux 的进程监督、路径语义和终止策略一致。
10. **可观测**：UI 能展示加载、调用、token、延迟、健康、失败和权限状态。

### 2.2 非目标

- v1 不提供任意 Core 内存访问或 Rust ABI 插件；
- v1 不允许 Plugin 直接替换 Permission Engine、Scheduler 或 Event Store；
- 不保证无修改运行所有特定产品私有 Plugin；
- 不把 Skill 正文全部常驻系统提示；
- 不把 MCP server 声称的“只读”当作安全事实；
- 不以进程退出码单独证明外部副作用未发生；
- 不通过 Plugin 提供任意 SQL、任意 UI JavaScript 注入或任意动态链接库加载。

---

## 3. 术语、实体与边界

| 术语 | 定义 |
|---|---|
| Extension | 被 Apex 发现、治理和调用的扩展逻辑总称 |
| ExtensionDefinition | 逻辑身份，如 `com.example.rust-quality` |
| ExtensionRevision | 内容不可变的一次发布或本地文件快照 |
| ExtensionCandidate | 尚未被信任/安装的发现结果 |
| ExtensionInstance | 某 project/session 下启用 revision 后的运行实例 |
| Registry Generation | 当前可见扩展集合的单调递增快照号 |
| Capability | 可被授权和审计的最小能力标识 |
| Grant | 某主体在 scope、条件和期限内获得的 capability 子集 |
| Runtime Backend | builtin、Wasm、supervised process、remote MCP |
| SkillLoad | metadata/body/resource 的一次受控加载记录 |
| MCP Server Instance | 一个配置 revision 对应的受监督连接实例 |
| Schema Revision | MCP 工具/资源/Prompt 能力集合的不可变摘要 |
| Hook Subscription | Hook 对事件、matcher、优先级和阶段的声明 |
| Hook Invocation | 针对一个事件运行一次 Hook 的审计实体 |
| Plugin Package | 含 manifest、模块、资源、签名和锁文件的可安装包 |
| Taint | 数据来源及其不可信传播标签 |
| Quarantine | 扩展被发现但因风险、校验失败或策略阻断而隔离 |
| Reconcile | 外部调用结果未知时查询真实外部状态的对账动作 |

### 3.1 聚合关系

```text
ExtensionDefinition 1 ── * ExtensionRevision
ExtensionRevision   1 ── * ExtensionInstance
ExtensionRevision   1 ── * CapabilityRequest
ExtensionInstance   1 ── * HookSubscription
ExtensionInstance   1 ── * RegisteredTool
ExtensionInstance   1 ── * SkillDescriptor
ExtensionInstance   1 ── * MCPServerInstance

ToolCall ── optional SkillLoad
ToolCall ── optional MCPCall
ToolCall ── * HookInvocation
PluginRevision ── * SkillRevision / HookSubscription / ToolDescriptor
```

Plugin 是分发与运行容器，不吞并 Skill/MCP/Hook 的领域身份。例如，一个 Plugin 可捆绑两个 Skill、一个 Hook 和一个 MCP server template；它们分别注册、授权和观测。

---

## 4. 统一扩展分类与标识

### 4.1 扩展种类

```rust
pub enum ExtensionKind {
    Skill,
    McpServer,
    HookBundle,
    Plugin,
    BuiltinAdapter,
}

pub enum ExtensionSourceKind {
    Builtin,
    ProjectFile,
    UserFile,
    CompatibleDirectory,
    LocalPackage,
    ManagedRegistry,
    Marketplace,
    Git,
    RemoteUrl,
}
```

### 4.2 稳定身份

- `extension_id`：安装后稳定 ULID，不从路径推导；
- `canonical_name`：反向域名或 publisher/name，大小写归一；
- `revision_id`：每次内容变更新建；
- `source_locator`：只用于定位和展示，不作为授权主键；
- `manifest_digest`：规范化 manifest 的 SHA-256；
- `content_digest`：包内容 Merkle root；
- `publisher_id`：可为空，但 managed 安装源必须存在；
- `signature_set_digest`：签名集合摘要；
- `registry_generation`：解析完成后分配。

显示版本 `1.2.0` 不足以证明内容相同；两个同版本但 digest 不同的包必须被识别为不同 revision，并产生供应链告警。

### 4.3 命名规则

| 对象 | 规范 |
|---|---|
| Extension canonical name | `[a-z0-9][a-z0-9._-]{1,127}`，推荐反向域名 |
| Skill name | 兼容生态原名，同时生成规范化 lookup key |
| MCP server name | project 可见范围内唯一，规范化后用于命名空间 |
| MCP tool | `mcp__<server_slug>__<tool_slug>` |
| Hook | `<extension_name>/<hook_name>` |
| Plugin tool | `plugin__<plugin_slug>__<tool_slug>` |
| Capability | `<domain>.<action>.v<major>` |

名称冲突不得使用不稳定的“最后加载覆盖”。Apex 采用确定性优先级并要求冲突诊断；受管来源和项目显式绑定优先于模糊自动发现。

---

## 5. Manifest 与包结构

### 5.1 统一 manifest 外壳

```yaml
apiVersion: apex.dev/v1
kind: Plugin
metadata:
  name: com.example.rust-quality
  displayName: Rust Quality Suite
  version: 1.2.0
  publisher: com.example
  license: Apache-2.0
spec:
  protocol: ">=1.0 <2.0"
  runtime:
    type: wasm
    entry: modules/main.wasm
  capabilities:
    required:
      - workspace.read.v1
    optional:
      - process.spawn.v1
  contributions:
    skills:
      - skills/rust-testing/SKILL.md
    hooks:
      - hooks/quality.yaml
    tools:
      - tools/manifest.yaml
  limits:
    memoryMiB: 128
    timeoutMs: 5000
    maxConcurrency: 2
  compatibility:
    apex: ">=1.0.0 <2.0.0"
    os: [windows, linux, macos]
```

统一外壳提供身份、来源、runtime、能力、贡献点、限制和兼容范围。各 kind 在 `spec` 下使用自己的 schema；未知字段按 protocol minor 兼容策略处理。

### 5.2 Plugin 包目录

```text
rust-quality.apx/
├── apex-plugin.yaml              # 必需
├── apex-plugin.lock              # 解析后的依赖与 digest
├── modules/
│   └── main.wasm
├── skills/
│   └── rust-testing/
│       ├── SKILL.md
│       ├── references/
│       ├── scripts/
│       └── assets/
├── hooks/
│   └── quality.yaml
├── schemas/
├── licenses/
├── signatures/
│   └── manifest.dsse.json
└── README.md
```

包内路径必须：

- 使用 UTF-8、正斜杠规范化；
- 拒绝绝对路径、`..`、设备路径和 ADS；
- 解包后仍位于 staging root；
- 对 symlink 解析后再次做 containment 校验；
- 受文件数、总大小、单文件大小和压缩比限制。

### 5.3 Manifest 校验阶段

```text
bytes limit
 → YAML/JSON safe parse
 → apiVersion/kind schema
 → canonicalization
 → path containment
 → digest/signature
 → dependency graph
 → compatibility
 → requested capability classification
 → policy evaluation
 → staged registration
```

Manifest 中的 description、README、Skill 文本、MCP tool description 都按 tainted content 处理；它们不能控制安装授权。

---

## 6. 发现源、优先级与冲突处理

### 6.1 Skill 发现路径

需求基线优先级：

1. `<project>/apex/skills/`；
2. `~/apex/skills/`；
3. `~/.claude/skills/`；
4. `~/.codex/skills/`；
5. `~/.agents/skills/`；
6. 已启用 Plugin 捆绑 Skills；
7. Apex 内置 Skills。

为了避免同名覆盖风险，最终选择规则为：

```text
project explicit pin
> project native source
> user explicit pin
> user native source
> managed plugin contribution
> compatible directories
> builtin fallback
```

同一优先级冲突时不按扫描顺序决定，使用 canonical path 排序并标记 `conflicted`；只有用户/策略选定后才进入 Agent 可见集合。

### 6.2 MCP 配置发现

扫描至少包括：

- `<project>/apex/mcp.json`；
- `<project>/.mcp.json`；
- `~/apex/mcp.json`；
- `~/.config/claude/claude_desktop_config.json`；
- 组织受管配置；
- Plugin 提供的 server template。

兼容导入器把不同格式转为 `McpServerCandidate`，保留原始来源、行号/JSON Pointer、导入器版本和安全配置摘要。项目中出现 `.mcp.json` 只触发发现，不触发启动。

### 6.3 文件监听与防抖

- watcher 只发送“可能变化”信号；
- Registry 重新读取并计算 digest；
- 相同 digest 不生成 revision；
- 默认防抖 300 ms，批量变化形成一个 scan batch；
- 文件写入中、解析失败时保留上一有效 revision，同时显示 diagnostic；
- 删除源文件将 revision 标记 unavailable，不中断已固定的运行；
- watcher 事件不能直接执行扩展代码。

### 6.4 远程来源

远程 Skill 索引、Marketplace、Git 或 URL 安装默认关闭；启用时要求：

- HTTPS 或受管内部 transport；
- origin allowlist；
- 下载大小与超时限制；
- immutable digest 或签名；
- staging 下载后原子激活；
- 禁止跨 origin 的未声明文件引用；
- 缓存与来源 metadata 分离；
- 下载内容不得在安装批准前执行。

---

## 7. Trust、签名与供应链

### 7.1 Trust 级别

```rust
pub enum ExtensionTrust {
    Builtin,
    Managed,
    VerifiedPublisher,
    UserApproved,
    ProjectApproved,
    Unverified,
    Quarantined,
    Revoked,
}
```

Trust 不是 Capability。高 trust 可以减少重复提示，但不能突破产品 hard deny；低 trust 也不代表永远不可用，只是需要更严格沙箱和逐次审批。

### 7.2 安装证据

每次安装或启用必须保存：

- source kind 与 locator；
- resolved commit/version/digest；
- manifest/content/signature digest；
- publisher identity 与验证链；
- scanner/validator 版本；
- requested 与 granted capabilities；
- 用户/组织决策主体和时间；
- 风险摘要；
- dependency lock；
- 最终 revision id。

### 7.3 签名策略

- Managed Registry：必须签名，且 publisher 在组织 trust root 中；
- 公共 Marketplace：建议签名，首次 publisher 需显式确认；
- Git/URL：必须 pin commit/digest；无签名则为 unverified；
- 本地 project/user 文件：以本地来源和 digest 审计，可由 project trust 决定是否启用；
- 内置：由 Apex release 签名覆盖。

签名仅证明内容和发布者，不证明安全。Capability、sandbox、静态检查和运行期治理仍必须执行。

### 7.4 撤销与隔离

发现以下情况进入 quarantine 或 revoked：

- 签名验证失败；
- 同版本内容替换；
- publisher trust root 被撤销；
- manifest 请求未声明的运行能力；
- 包路径逃逸或压缩炸弹；
- 连续崩溃、资源超限或协议违规达到阈值；
- 组织策略更新为禁止；
- 安全公告指向 exact digest。

隔离后停止新调用；运行中副作用型调用按安全策略取消或等待完成，不得通过强杀伪装为“未执行”。

---

## 8. 生命周期与状态机

### 8.1 Revision 生命周期

```text
candidate
  ├─ parse_error ───────────────→ rejected
  ▼
validated
  ├─ policy_denied ─────────────→ quarantined
  ▼
awaiting_trust
  ├─ user_denied ───────────────→ disabled
  ▼
staged
  ▼
enabled ── update found ───────→ superseded
  │  ├─ policy/revocation ──────→ quarantined
  │  ├─ user disable ───────────→ disabled
  │  └─ source removed ─────────→ unavailable
  ▼
retired ────────────────────────→ garbage_collectable
```

### 8.2 Instance 生命周期

```text
created → starting → ready → degraded → stopping → stopped
                    │          │
                    ├──────────┘
                    └→ failed → backoff → starting
```

Wasm Plugin 可以按调用实例化；受监督进程、Hook worker 和 MCP stdio server 通常具有 instance 生命周期。状态迁移必须由 Core actor 串行化并产生事件。

### 8.3 热更新语义

1. 扫描得到新 digest，创建新 revision；
2. 校验、依赖解析和授权差异评估在 staging 完成；
3. 若新增 capability，必须重新审批；
4. 原子切换 Registry active pointer，generation + 1；
5. 新调用固定新 revision；旧调用继续旧 revision；
6. 旧 instance drain，达到 deadline 后按操作类型处理；
7. Skill metadata cache、MCP schema cache、Hook subscription index 失效；
8. 使用旧 revision 产生但尚未完成的 RuleCheck/Gate 按规则标记 stale；
9. 更新失败回滚 active pointer，不覆盖旧 revision。

### 8.4 卸载语义

卸载是多阶段操作：disable → stop new calls → drain/reconcile → revoke grants → unregister projections → mark retired → 延迟清理文件。审计、历史 ToolCall、Gate、Event 和 digest 引用不得级联删除。

---

## 9. Capability、授权与沙箱模型

### 9.1 Capability 分层

Capability 必须是可枚举、可审计、可与规则绑定的字符串，而不是一个泛化的 `plugin.execute`：

| 层级 | 示例 | 说明 |
|---|---|---|
| 观察 | `session.observe.v1`、`event.subscribe.v1` | 读取已脱敏状态 |
| Context | `context.read.v1`、`context.contribute.v1` | 读取或贡献上下文片段 |
| 文件 | `workspace.read.v1`、`workspace.write.v1` | 必须附带路径/claim 条件 |
| 进程 | `process.spawn.v1` | 命令、环境、cwd、资源限制均需约束 |
| 网络 | `network.connect.v1`、`data.egress.v1` | host、port、协议和数据分类约束 |
| Credential | `credential.use.v1` | 只允许引用，不返回 Secret 原文 |
| Tool | `tool.register.v1`、`tool.invoke.v1` | 注册和实际调用分开 |
| MCP | `mcp.discover.v1`、`mcp.invoke.v1` | server 与 tool 作用域分离 |
| UI | `panel.register.v1`、`notification.emit.v1` | 不得成为控制面权威 |
| 管理 | `extension.install.v1`、`extension.update.v1` | 默认仅 Core/Admin 可授予 |

### 9.2 Capability 约束对象

Capability grant 至少包含：

```json
{
  "capability": "workspace.write.v1",
  "scope": {
    "project_id": "prj_123",
    "paths": ["src/**", "tests/**"]
  },
  "conditions": {
    "session_mode": ["implementation"],
    "requires_write_claim": true,
    "requires_approval": true
  },
  "expires_at_us": 1780000000000,
  "source": "user_decision",
  "bound_revision_digest": "sha256:..."
}
```

绑定到 revision digest 后，扩展内容变化不会继续复用旧授权。`always allow` 也必须有 scope、条件和过期策略，不能生成无界全局授权。

### 9.3 能力降级

当运行后端不支持所需 capability 时，Registry 不得“假装成功”：

```text
requested capability
  → backend capability set
  → policy intersection
  → granted capability set
  → missing capability diagnostic
```

可选 capability 缺失时注册为 degraded；必需 capability 缺失时不启用。MCP server 声明的工具不能自动获得 `data.egress.v1`；Skill 的 `requires-tools` 不能自动获得这些工具的执行权。

### 9.4 Sandbox profile

| Profile | 适用 | 默认权限 |
|---|---|---|
| `pure` | 纯函数 Wasm Plugin | 无网络、无文件、无进程 |
| `context` | Context/Skill 辅助 | 只读、大小限额、taint 保留 |
| `workspace-read` | 静态分析 | 指定工作区只读 |
| `tool-worker` | 受限工具 | 仍经 Tool Gateway |
| `process-worker` | MCP/生态二进制 | 受监督进程、最小环境 |
| `remote-client` | 远程 MCP | host allowlist、Credential 引用 |
| `managed` | 组织内扩展 | 仍受 hard ceiling，额外审计 |

Sandbox 不是权限替代品。沙箱内调用 Apex API 仍必须通过 capability check；沙箱外部操作也不能绕过 Tool Gateway。

---

## 10. 执行后端与资源管理

### 10.1 后端选择

```text
if builtin_adapter:
    in_process_trusted
else if wasm_module and no_os_integration:
    Wasm/WASI sandbox
else if stdio/existing_binary/OS_API:
    supervised subprocess
else if MCP remote endpoint:
    supervised remote client
else:
    reject unsupported backend
```

### 10.2 Wasm/WASI 后端

Wasm 适合：

- schema 转换、文本分类、静态诊断、数据格式适配；
- 没有任意文件/网络/进程需求的 Plugin；
- 需要可复现、跨平台、快速回收的短任务。

运行约束：

- 每次调用绑定 `instance_id`、revision digest 和 budget；
- memory/table/stack/fuel/time 分别限额；
- host function 按 Capability 暴露，不提供裸 filesystem；
- 超限产生结构化 `EXTENSION_RESOURCE_EXHAUSTED`；
- 不允许 Wasm 直接持有 Core 对象引用；
- host call 返回的外部内容保留 taint。

### 10.3 受监督子进程

进程启动由 `ProcessSupervisor` 负责：

- 生成独立 process group/job object；
- 最小化环境变量和工作目录；
- stdin/stdout/stderr 使用有界管道；
- Windows 使用 Job Object，Unix 使用 process group/cgroup（可用时）；
- 记录 pid、树根、启动参数 digest，不在普通日志输出 Secret；
- 先优雅停止，再限时强制终止；
- 退出后回收全部子孙进程和临时目录；
- 进程重启使用指数退避和熔断，不无限重启。

### 10.4 Remote client

远程 MCP 连接具备：

- endpoint allowlist 和 DNS/IP 策略；
- TLS 校验、代理策略和连接超时；
- request/response 大小上限；
- CredentialStore 引用注入；
- 每次请求 operation_id、deadline 和审计摘要；
- 断线后能力 registry 进入 stale/degraded，不保留“看似 ready”的假状态。

### 10.5 资源预算

统一预算对象：

```rust
pub struct ExtensionBudget {
    pub wall_time_ms: u64,
    pub cpu_ms: u64,
    pub memory_bytes: u64,
    pub input_bytes: u64,
    pub output_bytes: u64,
    pub child_processes: u32,
    pub network_bytes: u64,
    pub context_tokens: u32,
}
```

预算由 product、project、extension、operation 多层取最小值。预算耗尽只允许生成明确的 timeout/resource result，不得自动扩大预算。

---

## 11. Extension Registry 与依赖解析

### 11.1 Registry 分层

```text
Source Scanner
   → Candidate Store
   → Validator
   → Revision Store
   → Dependency Resolver
   → Policy Evaluator
   → Active Registry (generation)
   → Runtime Projection
```

Active Registry 是某个 generation 的只读视图。Agent Run 开始时可固定 registry generation，避免一次运行中因热加载出现同名工具和 Skill 语义漂移。

### 11.2 依赖类型

- `runtime`: Plugin 运行时或 MCP client 依赖；
- `capability`: 声明所需 capability；
- `extension`: 依赖另一个 extension revision/version；
- `skill-resource`: Skill body 引用的受控资源；
- `tool`: 需要存在某个注册工具，但不自动授权；
- `provider`: 需要某个 Provider 能力。

依赖解析必须输出 lock：精确 revision digest、来源、版本、传递依赖和冲突解决结果。

### 11.3 冲突策略

遇到以下情况不得静默替换：

- 同名工具 schema 不同；
- 两个 Plugin 声明同一 Hook key 且无法排序；
- Skill 同名但来源/内容不同；
- MCP server name 相同但 endpoint/config digest 不同；
- capability 版本不兼容；
- 依赖形成环。

解决路径：显式别名、项目 pin、禁用一方或拒绝激活。诊断必须包含候选列表和稳定排序原因。

### 11.4 Runtime 注册

注册顺序固定为：

1. 内置 capability 和 hard policy；
2. 内置 tools/providers；
3. managed extensions；
4. project pinned extensions；
5. user extensions；
6. compatible discovered candidates；
7. session-local ephemeral contributions。

注册完成后生成 `registry_generation`。注册过程不是“执行扩展”，只能解析描述、schema 和静态 metadata；实际启动延迟到需要时，除非 MCP 被用户明确连接。

---

## 12. Skill 总体架构

### 12.1 Skill 的定位

Skill 是面向模型的知识、步骤和资源包，不是一个拥有无限权限的脚本容器。其主要输出是：

- 可注入 Context 的说明；
- 对任务分解和工具使用的建议；
- 受控引用资料；
- 可选的脚本/资产路径；
- 对 spec-phase、工具需求和适用范围的声明。

Skill 与 Plugin 的关系：Skill 可以独立存在，也可以由 Plugin 分发；由 Plugin 分发时，Skill 自身仍独立参与 discovery、load、taint 和审计。

### 12.2 Skill 最小格式

```markdown
---
name: rust-testing
description: Rust testing patterns with cargo test and property-based checks
allowed-tools: [Bash, Read]
spec-phase: implementation
requires-tools: [Bash]
version: 1.2.0
---

# Rust Testing

按项目测试策略执行……
```

最低要求：`name`、`description`、Markdown body。Apex 扩展字段全部可选，未知字段保留诊断但不允许改变安全策略。

### 12.3 Skill 目录约定

```text
skill-name/
├── SKILL.md
├── references/       # 按需加载的文档
├── scripts/          # 不因发现或 body 加载而自动执行
├── assets/           # 模板、图片、固定输入
└── tests/             # 可选，离线校验用
```

资源访问基于 `SKILL.md` 所在目录解析，经过路径 containment 和 allowlist 校验。Skill 不得通过相对路径逃逸到项目根之外；若要读取工作区内容，仍通过 Read/Workspace Capability。

### 12.4 Frontmatter 语义

| 字段 | Apex 语义 |
|---|---|
| `name` | 展示和显式调用名，不单独作为身份 |
| `description` | 触发提示，按 tainted metadata 处理 |
| `allowed-tools` | 请求性收窄列表，最终以 Permission 为准 |
| `disable-model-invocation` | 禁止模型自动触发，仅允许用户/策略显式调用 |
| `spec-phase` | 与 Agent Runtime 阶段匹配的提示过滤条件 |
| `requires-tools` | 依赖声明，不构成授权 |
| `version` | 展示/兼容信息，运行仍绑定 digest |
| `context` | 可请求 fork/isolated context，但必须由 Runtime 决定 |

### 12.5 Skill discovery 结果

```rust
pub struct SkillDescriptor {
    pub skill_id: SkillId,
    pub revision_id: RevisionId,
    pub name: String,
    pub description: String,
    pub source: SourceInfo,
    pub location: SafePath,
    pub content_digest: Digest,
    pub disable_model_invocation: bool,
    pub declared_tools: Vec<ToolName>,
    pub phase: Option<SpecPhase>,
    pub metadata_tokens: u32,
    pub diagnostics: Vec<DiagnosticRef>,
}
```

系统提示只注入经过长度截断和转义的 `name + description + source/usage hint`，必要时附带可审计的 stable handle，不直接暴露任意本地绝对路径给模型。

---

## 13. Skill 渐进式加载与调用

### 13.1 三层模型

严格采用：

```text
metadata → body → resources
```

| 层 | 时机 | 内容 | 默认上限 |
|---|---|---|---:|
| metadata | discovery/每次 run | name、description、来源、摘要 | 约 100～200 tokens/skill |
| body | 触发或显式调用 | SKILL.md 正文 | 由 context budget 限制，建议 < 5k tokens |
| resources | 明确引用/受控读取 | references、scripts、assets | 单次和累计预算限制 |

Scripts 只属于 resources；加载 body 不执行 scripts。metadata 解析失败的 Skill 不应进入可触发清单。

### 13.2 触发方式

1. 模型依据 metadata 选择 Skill；
2. 用户 `/skill <name>` 显式触发；
3. Spec phase 或 Workflow 节点按 allowlist 自动选择；
4. 已加载 Skill 建议另一个 Skill，但必须重新匹配和授权；
5. Plugin/API 发出 `InvokeSkill` command。

`disable-model-invocation: true` 只影响第 1、4 类，不绕过权限和 trust。

### 13.3 Load 流程

```text
resolve skill handle + registry_generation
 → check enabled/trust/phase
 → create SkillLoad(idempotency_key)
 → read body via Tool Gateway
 → parse bounded markdown
 → attach source + taint + revision
 → append Context contribution
 → emit SkillBodyLoaded
```

资源加载再次创建 `SkillResourceLoad`，必须声明 resource path、目的、大小预算和上下文目标。相同 revision + path + checkpoint 可复用只读缓存，但 cache hit 仍记录观测。

### 13.4 Context 注入

Skill body 进入 Context 的 `extension_guidance` 或 `volatile_suffix` 区域，不进入不可变 system contract。推荐包装：

```text
<apex_skill name="rust-testing" revision="sha256:...">
  <instructions taint="extension_content">...</instructions>
  <source>project-skill</source>
</apex_skill>
```

Skill 文本不能直接创建 PermissionDecision、Gate pass 或 system policy。它可以建议动作，实际动作仍形成 ToolCall。

### 13.5 Token 预算与淘汰

- discovery 时估算 metadata token；
- body 注入前检查 Context budget；
- 超预算时要求用户选择、摘要或只加载指定章节；
- resources 默认不注入全文，使用受控 Read 或摘要；
- checkpoint 恢复时重新验证 revision digest；
- Skill revision 变化后旧 body cache 标记 stale；
- UI 展示 metadata/body/resource 各层 token 消耗。

### 13.6 Skill 工具与脚本

Skill 不直接调用操作系统。以下动作都必须转成 ToolCall：

- 运行 `scripts/*`；
- 读取 workspace；
- 访问网络或 MCP；
- 写入文件；
- 生成报告或调用外部命令。

脚本默认使用 skill staging cwd，环境变量最小化；脚本命令、digest、输入文件和输出都写入 Tool Gateway 审计。

---

## 14. MCP 总体架构

### 14.1 MCP 的定位

MCP Server 是外部能力提供方，不是 Apex Core 的一部分。它可以提供：

- Tools：可产生计算或外部副作用；
- Resources：外部文档、数据或资源引用；
- Prompts：可复用 Prompt 模板；
- sampling/通知等协议扩展（按版本和策略支持）。

MCP server 的能力声明是描述，不是授权；返回结果是外部数据，默认 tainted。

### 14.2 连接管线

```text
config discovery
 → import safe config
 → trust/schema validation
 → explicit enable/connect
 → server supervisor
 → initialize + capability discovery
 → schema revision freeze
 → namespace registration
 → Tool Gateway invocation
 → timeout/taint/audit
 → health/reconcile/projection
```

### 14.3 配置与 Secret 分离

`safe_config_json` 中允许：

- transport 类型；
- command 的安全 argv 模板；
- endpoint；
- server display name；
- capability allowlist；
- timeout、reconnect、health 参数。

以下内容不得写入普通配置、manifest、事件 payload 或 UI：

- API key、Bearer token、Cookie；
- client secret、private key；
- 完整 Authorization header。

配置只保存 `credential_ref`，实际 Secret 由 CredentialStore 在连接/调用时短时注入。

### 14.4 MCP server instance

```rust
pub struct McpServerInstance {
    pub server_id: McpServerId,
    pub revision_id: RevisionId,
    pub transport: McpTransport,
    pub state: McpState,
    pub capability_revision: u64,
    pub process_handle: Option<ProcessHandle>,
    pub endpoint_policy: EndpointPolicy,
    pub credential_refs: Vec<CredentialRef>,
    pub last_heartbeat_at_us: Option<i64>,
}
```

一个配置 revision 只能有一个 active instance；需要并行隔离时，必须生成不同 instance scope 和审计关系，不能共享不可控的 transport 状态。

---

## 15. MCP 发现、连接与能力注册

### 15.1 本地自动发现

扫描配置时只读取安全字段并展示：

- server name；
- source path；
- transport；
- command/endpoint 的脱敏摘要；
- 是否需要 Credential；
- 请求 capability；
- 上次健康状态；
- 信任/策略状态。

UI 可提供“一键启用”，但一键启用仍生成明确的 EnableMcpServer command 和审计事件。

### 15.2 stdio 生命周期

```text
spawn process group
 → send initialize
 → validate protocol/version
 → list capabilities
 → freeze schema revision
 → ready
```

退出处理：

- 记录 exit code、signal、stderr digest 和最后消息；
- 清理 process tree；
- 将未完成调用标记为 `unknown`，不得简单标记 failed；
- 根据 restart policy 进入 backoff；
- 连接稳定性差时触发 circuit breaker；
- 进程启动参数和 env 经过 redaction 后再审计。

### 15.3 SSE/HTTP 生命周期

- connect timeout、handshake timeout、idle timeout 分开配置；
- 仅允许策略批准的 scheme/host/port；
- 证书、代理和 DNS 策略由 Network Policy 管理；
- server capability notification 触发新 schema generation；
- remote disconnect 时保留旧 schema 但标记 stale，不允许新副作用调用；
- 重连成功后重新 initialize，不能假定 session state 未变化。

### 15.4 Schema Revision

能力注册结果规范化后计算 digest：

```text
schema_revision = hash(
  protocol_version,
  server_info,
  tools[name, description, input_schema, annotations],
  resources[uri_template, mime_type],
  prompts[name, arguments]
)
```

ToolCall、审批、Rules exception、Gate 证据均绑定 `schema_revision`。发生以下情况必须创建新 revision：

- tool input schema 变化；
- tool 名称/描述变化（描述变化也可能改变模型行为）；
- tool annotations 变化；
- resource/prompt 能力变化；
- server protocol version 变化。

schema revision 变化会让尚未执行的旧审批失效；正在执行的调用继续使用原绑定，但结果按旧 revision 解释。

### 15.5 工具命名与安全映射

外部名称不能直接作为内部全局 key。映射：

```text
mcp__<server_slug>__<tool_slug>
```

slug 冲突时加稳定短 digest，而不是按加载顺序编号。原始 server/tool name 保存在 metadata，UI 展示友好名，审计使用完整 identity。

---

## 16. MCP 调用、数据外发与 Taint

### 16.1 MCP ToolCall

一次 MCP 工具调用至少关联：

- `tool_call_id`；
- `operation_id`；
- `mcp_server_id`、server revision；
- `schema_revision`；
- tool name；
- request digest 和 redacted summary；
- input taint/classification；
- permission decision；
- credential refs；
- timeout/deadline；
- response blob 和 reconcile state。

调用流程：

```text
Agent proposal
 → tool schema validation
 → taint/data classification
 → Capability check
 → Permission/Rules check
 → optional user approval
 → credential resolution
 → MCP send
 → bounded response parse
 → taint propagation
 → operation receipt/reconcile
```

### 16.2 外部数据外发

以下示例要求至少同时满足：

```text
mcp.invoke.v1
∩ credential.use.v1 (若需认证)
∩ data.egress.v1
∩ network.connect.v1
∩ destination allowlist
∩ data classification policy
∩ user approval / saved rule
```

MCP server manifest 或 tool description 声称“不会保存数据”不能替代 data egress policy。敏感字段在发送前由 Data Policy/Redaction 处理；无法证明脱敏成功时 fail closed。

### 16.3 Taint 传播

外部 MCP description、resource、prompt、tool result 都带来源：

```json
{
  "taint": {
    "source": "mcp",
    "server_id": "mcp_123",
    "schema_revision": 7,
    "untrusted": true,
    "data_class": "external_content"
  }
}
```

Taint 传播规则：

- 进入 Context 的 MCP 内容放在 volatile/extension 区域；
- 生成的 ToolCall 若依据 tainted 内容，保留 provenance；
- tainted 内容不能改变 Capability、Permission、Rule 或 Gate 状态；
- 写入工作区前必须经过规则和用户/系统目标校验；
- 输出到 UI 时转义，防止伪造 Apex 控制消息；
- 进入 Prompt 的内容不得使用控制平面保留标记伪造 system/assistant 消息。

### 16.4 Resources 与 Prompts

MCP Resource 读取不是普通文件读取：

- URI 必须经过 server scope 校验；
- 大小、分页、mime type 有界；
- 结果存 Blob/引用，避免把大内容直接塞进事件；
- 可选地进入 Context，但标记 server/source/revision；
- Prompt 模板当作外部建议，不直接拼接成 system contract。

### 16.5 Sampling 与反向能力

如果未来支持 MCP server 请求 Apex sampling 或反向调用：

- 必须为独立 Capability；
- 明确 max tokens、model/provider、成本和数据范围；
- 默认禁止访问完整 Session；
- 请求必须关联 parent operation_id；
- server 不得获得用户隐式授权；
- sampling 结果仍 tainted 并进入普通上下文区。

---

## 17. MCP 超时、重试与 Reconcile

### 17.1 三种结果状态

| 状态 | 含义 | 是否可自动重试 |
|---|---|---:|
| `failed_before_send` | 已在本地确认未发送 | 可按策略重试 |
| `failed_confirmed` | 外部明确拒绝/失败且无副作用 | 受策略限制重试 |
| `unknown` | 发送、断线或超时导致真实状态不明 | 不得盲目重试 |

### 17.2 超时算法

默认 MCP timeout 为 30 秒，但按 server/tool/operation 可进一步收窄。超时后：

1. 停止等待，不代表远端停止；
2. 记录 `MCP_CALL_TIMEOUT_UNKNOWN`；
3. 尝试 transport-level cancel（若协议支持）；
4. 查询 server health/status；
5. 若有声明的 idempotency key，按 reconcile protocol 查询；
6. 生成 `ReconcileRequired`；
7. 外部副作用未对账前，阻止依赖该结果的 Gate pass 或重复动作。

### 17.3 重试条件

只有同时满足以下条件才可自动重试：

- Tool/Server 明确标记幂等，或提供 idempotency key；
- 失败状态可以确认未产生副作用；
- 原 capability/approval 仍有效；
- schema revision 未变化；
- 未超过 retry budget；
- Rules/Policy 允许。

网络错误不能自动推断“未执行”。

### 17.4 Reconcile Gate

外部副作用型操作的最终状态必须满足：

```text
ToolCall finished
 → receipt exists
 → external state query/reconcile succeeded
 → result classified
 → dependent RuleCheck/Gate may continue
```

若 server 不支持查询且操作可能有副作用，状态为 `reconcile_unsupported`，交由用户或人工流程处理。对应错误包括：`MCP_CALL_TIMEOUT_UNKNOWN`、`MCP_RECONCILIATION_UNSUPPORTED`、`MCP_SCHEMA_CHANGED`。

### 17.5 MCP 健康状态

```text
unknown → connecting → ready
ready → degraded → reconnecting → ready
ready → stale_schema
ready → failed → circuit_open
```

健康状态不等于业务调用成功。面板必须同时展示 transport health、schema freshness、最近调用和副作用对账状态。

---

## 18. Hook 事件模型

### 18.1 Hook 定位

Hook 是 Core 在确定事件点发出的受控扩展调用。Hook 可以：

- 观察事件；
- 返回诊断；
- 收窄或拒绝某项动作；
- 请求用户审批；
- 提出参数改写建议；
- 启动异步、不阻塞主流程的辅助检查。

Hook 不可以：

- 授予 Capability；
- 绕过 hard deny；
- 静默更改 ToolCall 参数、PermissionDecision、RuleCheck 或 Gate；
- 直接写 workspace 或数据库；
- 隐藏事实、伪造工具结果；
- 把失败改写成成功。

### 18.2 首批事件

| 事件 | 时机 | 是否可阻断 | 典型用途 |
|---|---|---:|---|
| `PreToolUse` | ToolCall 判权前/执行前受控阶段 | 是 | 风险诊断、拒绝、请求审批 |
| `PostToolUse` | ToolResult 已解析、提交前 | 是，阻止流程继续 | 输出验证、敏感数据检查 |
| `PermissionRequest` | 生成权限请求后 | 否/仅诊断 | 风险摘要补充 |
| `SpecStageChanged` | Spec 阶段转换事务后 | 否 | 通知、外部索引 |
| `AgentStop` | Agent Run 即将结束 | 是，可要求修复 | 检查遗漏、生成诊断 |
| `SessionStop` | Session 停止/归档 | 否 | 清理、汇总 |
| `RuleCheckRequested` | RuleCheck 排队前 | 否/可诊断 | 注册 custom checker |
| `CheckpointCreated` | checkpoint committed 后 | 否 | 外部镜像、度量 |
| `ExtensionChanged` | registry generation 变化 | 否 | 缓存失效 |

需求中的 `Stop` 在 Apex 中按作用域具体化为 `AgentStop` 和 `SessionStop`；兼容导入器可将外部 `Stop` 映射到 `AgentStop`，并记录兼容语义。

### 18.3 执行阶段

`PreToolUse` 不是一个任意位置，而是分阶段：

```text
Core hard validation
 → manifest/schema validation
 → PreToolUse policy hooks
 → Permission Engine
 → optional approval
 → PreToolUse final hooks (only narrow)
 → execute
```

Core hard validation 永远先于外部 Hook。final hooks 只能收窄已经批准的动作；若提出参数变化，必须创建新的 ToolCall revision 并重新走完整流程。

`PostToolUse`：

```text
raw result
 → bounded parse + redaction
 → PostToolUse hooks
 → Rule/verification implications
 → commit ToolResult
 → continue Agent
```

PostTool Hook 发现违规时，可以阻止结果被视为可接受完成，但不能删除原始审计证据。

### 18.4 Hook 输入信封

```json
{
  "protocol_version": "1.0",
  "invocation_id": "hki_...",
  "hook": "com.example/security/pre-write",
  "event_name": "PreToolUse",
  "event_id": "evt_...",
  "occurred_at_us": 1780000000000,
  "scope": {
    "project_id": "prj_...",
    "session_id": "ses_...",
    "run_id": "run_...",
    "agent_id": "agt_..."
  },
  "registry_generation": 42,
  "input": {
    "tool_call_ref": "tcl_...",
    "safe_summary": "write src/lib.rs",
    "argument_ref": "blob://redacted/..."
  },
  "taint": [],
  "deadline_at_us": 1780000005000
}
```

Hook 只获得 manifest 声明且 grant 允许的字段；默认使用 safe summary 和引用，不把完整 transcript、Secrets 或未经授权的文件内容发送给 Hook。

---

## 19. Hook 匹配、顺序与结果协议

### 19.1 Matcher

支持结构化 matcher，而非默认接受任意正则：

```yaml
on: PreToolUse
match:
  toolNames: [Write, Edit]
  toolNamespace: builtin
  paths: ["src/**"]
  specPhases: [implementation]
  riskAtLeast: medium
priority: 300
```

兼容配置可接受精确、or、glob 和有限正则，但正则必须有长度、复杂度和运行时间限制，避免 ReDoS。

### 19.2 确定性顺序

排序键：

```text
phase
→ policy_tier (builtin > managed > project > user > session)
→ priority ascending
→ extension canonical name
→ revision digest
→ hook name
```

同一个 event + hook revision 最多成功执行一次；数据库使用唯一键去重。顺序包含在 invocation batch digest 中，便于重放。

### 19.3 HookResult

```rust
pub enum HookDecision {
    Continue,
    Deny,
    RequestApproval,
    BlockCompletion,
    DiagnosticOnly,
    ProposeRewrite,
    AsyncCheckScheduled,
}

pub struct HookResult {
    pub decision: HookDecision,
    pub reason_code: String,
    pub user_message: Option<String>,
    pub diagnostics: Vec<Diagnostic>,
    pub proposed_patch: Option<ContentRef>,
    pub cache_ttl_ms: Option<u64>,
    pub output_taint: Vec<TaintLabel>,
}
```

`continue` 只表示该 Hook 不反对，不代表 Permission 或 Gate 通过。`request_approval` 只能增加审批要求。`proposed_patch`/参数 rewrite 不就地生效，而是创建新请求。

### 19.4 参数改写

禁止共享可变 output 对象或静默 mutation。安全模型：

```text
Hook proposes rewrite
 → Core validates patch schema
 → create ToolCallRevision N+1
 → recompute risk/capability/rules
 → invalidate old approval
 → user/agent confirms
```

这样保留完整 before/after、责任主体和审批依据。

### 19.5 合并规则

多个结果合并使用最严格语义（取值同 §19.4 `HookDecision`，此处按 wire 形式的 snake_case 书写）：

```text
deny > block_completion > request_approval > propose_rewrite
     > async_check_scheduled > diagnostic_only > continue
```

诊断全部保留并去重；rewrite 相互冲突时不自动选择，生成 `HOOK_REWRITE_CONFLICT`。

> ADR-0021（跨文档一致性审查）：原优先级链写作 `diagnostics` 且遗漏 `async_check_scheduled`，与 §19.4 的七值枚举不一致。现补齐并统一为 `diagnostic_only`。

---

## 20. Hook 失败、重入与异步执行

### 20.1 超时和失败策略

Hook 必须声明 `failurePolicy`：

- `fail_closed`：安全/合规 Hook；超时或协议错误阻断动作；
- `fail_open_with_diagnostic`：通知/度量 Hook；继续但记录告警；
- `defer`：结果转入异步检查，不阻塞当前非关键操作。

安全敏感 `PreToolUse`、`PostToolUse`、`AgentStop` 默认 fail closed；一般事件观察 Hook 默认 fail open。用户级 Hook 不能把组织级 fail closed 改为 fail open。

### 20.2 超时默认值

| 类型 | 默认 | 最大建议 |
|---|---:|---:|
| Wasm policy Hook | 500 ms | 2 s |
| 本地 command Hook | 5 s | 60 s |
| Prompt/LLM Hook | 15 s | 30 s |
| async observer | 30 s | 5 min |

Prompt-based Hook 被视为不确定性判决器，只能增加诊断/审批/拒绝，不能单独授予高风险操作。

### 20.3 重入保护

每次 invocation 包含 `hook_chain` 和 depth：

- 默认最大深度 4；
- Hook 自身调用 Tool 时触发新的 ToolCall，但默认排除发起它的同一 Hook；
- 同一 `(event_id, hook_revision)` 唯一；
- 检测 `A → B → A` 环并终止；
- Hook 不得同步等待由自己阻断的动作完成；
- Plugin dispose/stop 阶段不接受新 Hook。

### 20.4 异步 Hook

长耗时审查可返回 `AsyncCheckScheduled`：

1. Core 创建 background verification task；
2. 主流程仅在策略允许时继续；
3. 任务完成后发 `HookAsyncResultReady`；
4. 若结果影响未结束 Run，可向 Agent 注入结构化诊断；
5. 若 Run 已结束，则进入 Session inbox/面板；
6. 异步结果不能篡改历史 ToolResult，只能触发 Repair Run、RuleCheck 或通知。

### 20.5 熔断

按 extension revision + hook name 维护：

- 连续 timeout/protocol violation；
- 最近窗口失败率；
- p95 延迟；
- 资源超限。

触发熔断后，安全 Hook 按 fail closed，观察 Hook 按 fail open with diagnostic；面板显示 circuit open 和下一次 probe 时间。

---

## 21. Plugin 架构与贡献点

### 21.1 Plugin 定位

Plugin 是可安装、版本化、可授权的扩展包和运行单元。它可以贡献：

- Skills；
- Hook subscriptions；
- 受限 Tools；
- MCP server templates/adapters；
- Provider adapters；
- Rules/Verification checkers；
- Commands；
- 只读 Panel 数据源和安全 UI 描述；
- 导入/导出格式适配器。

Plugin 不可以贡献：

- 任意 Core state mutation；
- 任意 SQL；
- 任意前端 JavaScript 注入；
- 权限放宽器；
- 替换审计、事件存储或加密实现；
- 未声明网络/文件/进程访问。

### 21.2 稳定贡献点

```rust
pub enum ContributionKind {
    Skill,
    Tool,
    Hook,
    Command,
    McpTemplate,
    ProviderAdapter,
    VerificationChecker,
    PanelDescriptor,
    Importer,
    Exporter,
}
```

每个贡献点有独立 schema major version。Plugin protocol 兼容不意味着所有贡献点都兼容；Registry 逐项校验并可部分降级。

### 21.3 Tool 注册

Plugin Tool descriptor 包括：

- 稳定 tool key；
- input/output JSON Schema；
- side-effect class；
- risk hints；
- required capabilities；
- timeout/idempotency/reconcile contract；
- safe model description；
- handler endpoint。

注册时 Core 计算 schema digest；调用时仍走 Tool Gateway。Plugin handler 返回数据不直接成为 ToolResult，需经 schema、大小、taint 和 redaction 校验。

### 21.4 Verification Checker

Plugin checker 必须：

- revision/hash 固定；
- 在隔离后端执行；
- 输入为明确 artifact/workspace snapshot 引用；
- 输出版本化结构化 receipt；
- 声明 deterministic/side-effect-free；
- 不能自行将 Gate 标记 pass；
- 不能读取未授权 Secret；
- hot update 后相关 RuleCheck/Gate 变 stale。

### 21.5 Panel 扩展

v1 只允许声明式 Panel：

- Core 提供组件白名单；
- Plugin 提供 schema + data projection；
- UI 根据安全组件渲染；
- 不执行 Plugin 提供的任意 JS；
- 用户动作映射为 Application Command；
- Panel 不能直接访问 filesystem/network/Credential。

后续若支持 WebView，必须独立进程、严格 CSP、无 Node integration，并通过稳定消息协议接入。

---

## 22. Plugin Protocol 与 SDK

### 22.1 Protocol 原则

- 使用稳定、版本化的 wire types；
- 不暴露 Rust 私有类型和内存布局；
- 可用 protobuf/JSON/CBOR 承载，语义由 protocol 定义；
- 所有请求带 protocol version、operation id、deadline；
- 未知字段按 minor 兼容，未知 enum 保留但不执行；
- breaking change 提升 major；
- 错误使用稳定 code，不要求 Plugin 解析文本。

### 22.2 Handshake

```json
{
  "type": "extension.hello",
  "protocol": "1.2",
  "extension_revision": "exr_...",
  "manifest_digest": "sha256:...",
  "runtime": "wasm",
  "requested_host_apis": ["context.read.v1", "tool.invoke.v1"]
}
```

Core 返回：

```json
{
  "type": "host.welcome",
  "protocol": "1.2",
  "instance_id": "exi_...",
  "registry_generation": 42,
  "granted_host_apis": ["context.read.v1"],
  "limits": {"timeout_ms": 5000, "max_message_bytes": 1048576}
}
```

握手中的 requested host APIs 不能超出 manifest；Core 返回值是当前实例有效授权，不是永久授权。

### 22.3 Host API

初始稳定 API：

- `extension.get_metadata`；
- `context.read_fragment`；
- `context.propose_contribution`；
- `tool.propose_call`；
- `event.emit_diagnostic`；
- `blob.read_bounded` / `blob.write_bounded`；
- `checkpoint.resolve_ref`；
- `project.get_safe_metadata`；
- `credential.request_use`（只发起请求，不返回 Secret）；
- `operation.reconcile_status`。

所有 host call 都执行 capability check 和审计。

### 22.4 SDK 形态

官方 SDK 提供：

- Rust SDK：Wasm 与 process 两种 transport；
- TypeScript SDK：supervised process/MCP adapter；
- JSON Schema/protobuf definitions；
- manifest validator；
- local test harness；
- fake Core、fake Tool Gateway 和 deterministic clock；
- signing/packaging CLI；
- compatibility test suite。

SDK 的便捷 API 不扩大协议能力。SDK 版本和 protocol version 解耦。

### 22.5 ABI 策略

v1 不承诺 Rust/C/C++ 动态库 ABI。Wasm 使用组件/host function ABI；进程后端使用 wire protocol。这样避免编译器、标准库和平台升级导致 Core 崩溃。

---

## 23. Plugin 安装、更新与回滚

### 23.1 安装事务

```text
download/copy to staging
 → unpack safely
 → validate manifest
 → verify digest/signature
 → resolve + lock dependencies
 → static scan
 → compute capability diff
 → user/admin approval
 → insert immutable revision
 → atomic activate
 → emit events
```

任何失败都不污染 active registry。staging 目录可回收，安装审计保留。

### 23.2 Capability diff

更新时展示：

- 新增/删除 capabilities；
- 新增 network hosts、workspace paths、process commands；
- 新增 Hooks/Tools/Panels；
- sandbox backend 变化；
- publisher/signature 变化；
- schema/side-effect class 变化。

新增高风险能力必须重新审批；仅减少能力可在策略允许时自动更新。

### 23.3 原子激活

- revision 与文件先完整提交；
- registry pointer 在单事务中切换；
- runtime instance 采用 blue/green 或 stop/start；
- 新实例 ready 后接收新调用；
- 旧实例 drain；
- 激活失败恢复旧 pointer；
- DB revision 与磁盘目录用 intent/finalize 模式处理崩溃窗口。

### 23.4 回滚

回滚不是覆盖文件，而是重新激活历史 revision。若旧 revision 的 publisher 被 revoked、依赖不可用或 policy 不再允许，则拒绝回滚并给出原因。回滚后 registry generation 仍递增。

### 23.5 卸载与数据

Plugin 卸载不删除：

- ToolCall/HookInvocation/MCPCall 历史；
- Event、Gate、RuleCheck 证据；
- 使用过的 manifest/content digest；
- 用户明确保存的 Plugin 产物。

Plugin 私有 cache、临时目录、非权威 projection 可延迟 GC。

---

## 24. 依赖、组合与扩展间调用

### 24.1 依赖图

Dependency Resolver 生成 DAG；循环依赖直接拒绝。可选依赖缺失时 contribution 降级，必需依赖缺失时 revision 不激活。

### 24.2 扩展间调用

扩展不能直接拿到另一个扩展的进程句柄。调用方式：

```text
Plugin A → Host API propose ToolCall → Tool Gateway → Plugin B Tool
```

这样 Permission、timeout、taint、operation journal 和审计保持完整。内部优化可复用连接，但语义不得绕过 ToolCall。

### 24.3 Skill 组合

多个 Skill 可同时加载，但：

- 每个 body 单独记录来源与 revision；
- Context budget 按优先级分配；
- 冲突指令不按加载顺序隐式覆盖；
- system contract 与 project rules 始终更高优先级；
- 互相引用必须解析为 stable skill handle；
- 最大嵌套触发深度和总 Skill 数有限制。

### 24.4 Hook 组合

Hook 采用确定性串行合并，必要时允许同一阶段并行运行纯观察 Hook，但结果按预先确定顺序归并。可阻断 Hook 默认串行，避免竞态和不可解释决策。

### 24.5 Plugin 捆绑 MCP

Plugin 可以提供 MCP server template，但不得在安装时自动启动。用户选择配置、Credential 和 scope 后，生成独立 `McpServerRevision`。Plugin 更新不会静默替换正在使用的 MCP config。

---

## 25. 安全威胁模型

### 25.1 威胁分类

| 威胁 | 示例 | 主要防线 |
|---|---|---|
| Prompt injection | Skill/MCP 描述要求忽略策略 | taint、Context 分层、Core authority |
| Capability escalation | Plugin 请求裸文件/网络 | manifest + grant 交集、Tool Gateway |
| Supply chain | 同版本替换、恶意依赖 | digest、签名、lock、staging |
| Secret exfiltration | Hook/MCP 获取 token | Credential ref、data egress policy |
| Path traversal | Skill resource `../../` | containment、symlink 复核 |
| Process escape | MCP 子进程残留 | Job Object/process group、最小环境 |
| Schema bait-and-switch | 审批后 MCP schema 改变 | schema revision binding |
| Result spoofing | 扩展伪造 Gate pass | 结构化 receipt、Core commit |
| DoS | Hook 卡死、输出无限 | deadline、budget、circuit breaker |
| Confused deputy | 高权 Agent 调低信扩展 | operation-specific capability |
| UI spoofing | MCP 输出伪造权限弹窗 | 转义、声明式 UI、控制消息隔离 |
| Replay | 重放外部副作用 | idempotency key、operation journal |

### 25.2 Prompt 注入边界

必须把以下信息区分：

- control plane：Core policy、Permission、Rules、Gate；
- trusted project instruction：已信任项目配置；
- extension guidance：Skill/Plugin 文本；
- external data：MCP/resource/tool result；
- user content；
- model-generated proposal。

低层内容不得改变高层控制状态。即使 Skill 声明“你已经获得写权限”，Core 也只把它当文本。

### 25.3 Secret 使用

CredentialStore 只向受监督 transport 注入 Secret：

- Plugin/MCP 只拿 opaque ref；
- Secret 不进入 Prompt、Hook input、普通日志或 SQLite JSON；
- 注入范围限定到 server/host/tool；
- 使用前评估 data egress；
- stdout/stderr/response 做泄漏扫描与 redaction；
- Credential revoke 后实例重连或停止。

### 25.4 外部副作用

对发送消息、创建 issue、支付、删除云资源等外部动作，tool descriptor 必须标明 side-effect class。未知或缺失标注按高风险处理，不按只读处理。

### 25.5 安全更新

组织可下发 digest denylist、publisher revoke、最低 protocol/SDK 版本和强制更新策略。紧急撤销优先停止新调用，并生成正在运行调用清单供人工处置。

---

## 26. 与 Rules、Gate、Context、DAG 的集成

### 26.1 Rules 集成

Extension manifest 可以声明：

- Rule/Hook trigger；
- custom checker；
- capability；
- schema；
- timeout；
- sandbox；
- publisher。

Rules 编译器基于 revision digest 生成依赖。扩展更新时相关 RuleCheck 标记 stale。Hook 只能收窄、拒绝或增加 diagnostic，不能生成 Permission grant。

### 26.2 Gate 集成

- Plugin checker 输出 receipt，不直接 pass Gate；
- MCP 外部副作用需 Reconcile Gate；
- Skill/Plugin 提出的修复必须创建 Repair Run；
- Hook `BlockCompletion` 会让对应 Gate 保持 blocked/inconclusive，而不是伪造 failed；
- extension unavailable 时，Gate 根据必需性变为 inconclusive 或 blocked；
- Waiver 必须绑定 exact extension/checker revision 和 diagnostic fingerprints。

### 26.3 Context 集成

所有扩展贡献含：

```text
source identity
revision digest
registry generation
taint labels
token estimate
expiry/staleness rule
checkpoint relation
```

Skill body、MCP result、Plugin context contribution 都不能进入 immutable system contract。Checkpoint 恢复时若 revision 不可用，保留历史引用并提示 degraded，不静默替换为最新版。

### 26.4 DAG 集成

DAG node 可声明：

- required skills；
- required MCP servers/tools；
- required plugins/checkers；
- Hook policy profile；
- Capability scope；
- registry generation pin。

节点排队时验证依赖；运行中扩展崩溃按 retry/reconcile 语义处理。并行节点共享 MCP server 时，由 per-server concurrency limit 和 Tool Gateway 控制，不允许越过全局副作用串行约束。

### 26.5 SubAgent 集成

子 Agent 继承扩展可见性的受限子集：

```text
subagent_extensions ⊆ parent_visible_extensions
subagent_capabilities ⊆ parent_effective_capabilities
```

Agent profile 可以禁用某 Skill/MCP/Plugin，但不能增加父级未授予能力。子 Agent 调用扩展的事件和成本单独归因。

---

## 27. 可观测性与产品面板

### 27.1 统一指标

每个 extension revision 至少采集：

- discovered/enabled/disabled/quarantined 状态；
- invocation count、success/failure/timeout；
- p50/p95/p99 latency；
- active concurrency、queue time；
- input/output bytes；
- Context token 使用；
- resource/sandbox 超限；
- capability deny/ask/allow；
- crash/restart/circuit state；
- taint/data egress 分类；
- revision/schema change。

### 27.2 Skill 面板

展示：

- 已发现、已启用、冲突、解析失败；
- source、version、digest、trust；
- metadata/body/resource 加载层级；
- 当前调用者 Agent/Run；
- 调用次数和 token 消耗；
- `disable-model-invocation`、spec phase；
- 请求工具与实际有效权限；
- 最近修改和 stale cache；
- 一键显式加载、禁用、查看诊断。

### 27.3 MCP 面板

展示：

- server 连接状态、transport、来源；
- process/endpoint 安全摘要；
- protocol、schema revision；
- tools/resources/prompts 列表；
- 调用延迟、错误、超时和并发；
- reconnect/backoff/circuit；
- Credential 引用状态（不显示 Secret）；
- data egress 和审批；
- `reconcile_required` 队列；
- 启用、断开、重载和诊断操作。

### 27.4 Hook 面板

展示订阅事件、matcher、顺序、failure policy、最近 invocation、deny/ask 数量、超时、熔断和输入字段权限。用户可查看一次 ToolCall 被哪些 Hook 评估及合并结果。

### 27.5 Plugin 面板

展示 publisher、签名、来源、版本、capability diff、contributions、运行后端、依赖、更新、健康、资源使用和撤销状态。高风险 grant 使用醒目标识。

### 27.6 日志与追踪

一次跨扩展调用共享：

```text
trace_id
operation_id
tool_call_id
extension_revision_id
hook_invocation_id / skill_load_id / mcp_call_id
registry_generation
```

日志默认结构化和脱敏；原始大输出存 Blob，由 ACL 控制下载。

---

## 28. API 与实时事件

### 28.1 Command 服务

在既有 `ExtensionCommandService` 基础上扩展：

```proto
service ExtensionCommandService {
  rpc DiscoverExtensions(DiscoverExtensionsRequest) returns (CommandResponse);
  rpc InstallPlugin(InstallPluginRequest) returns (CommandResponse);
  rpc EnableExtension(EnableExtensionRequest) returns (CommandResponse);
  rpc DisableExtension(DisableExtensionRequest) returns (CommandResponse);
  rpc UpdateExtension(UpdateExtensionRequest) returns (CommandResponse);
  rpc RollbackExtension(RollbackExtensionRequest) returns (CommandResponse);
  rpc UninstallExtension(UninstallExtensionRequest) returns (CommandResponse);

  rpc DiscoverSkills(DiscoverSkillsRequest) returns (CommandResponse);
  rpc InvokeSkill(InvokeSkillRequest) returns (CommandResponse);
  rpc LoadSkillResource(LoadSkillResourceRequest) returns (CommandResponse);

  rpc ConnectMcpServer(ConnectMcpServerRequest) returns (CommandResponse);
  rpc DisconnectMcpServer(DisconnectMcpServerRequest) returns (CommandResponse);
  rpc ReloadMcpServer(ReloadMcpServerRequest) returns (CommandResponse);
  rpc ReconcileMcpCall(ReconcileMcpCallRequest) returns (CommandResponse);
}
```

### 28.2 Query 服务

```proto
service ExtensionQueryService {
  rpc ListExtensions(ListExtensionsRequest) returns (ExtensionList);
  rpc GetExtension(GetExtensionRequest) returns (ExtensionView);
  rpc ListSkills(ListSkillsRequest) returns (SkillList);
  rpc ListMcpServers(ListMcpServersRequest) returns (McpServerList);
  rpc GetMcpSchema(GetMcpSchemaRequest) returns (McpSchemaView);
  rpc ListHookSubscriptions(ListHookSubscriptionsRequest) returns (HookList);
  rpc ListExtensionInvocations(ListInvocationsRequest) returns (InvocationList);
  rpc GetExtensionMetrics(GetExtensionMetricsRequest) returns (MetricsView);
}
```

Query 返回摘要和引用，不默认返回完整 Secret、完整命令行或无限制 tool result。

### 28.3 REST 映射

| Method | Path | 用途 |
|---|---|---|
| `POST` | `/api/v1/extensions:discover` | 扫描候选 |
| `POST` | `/api/v1/extensions:install` | 安装 Plugin |
| `POST` | `/api/v1/extensions/{id}:enable` | 启用 revision |
| `POST` | `/api/v1/extensions/{id}:disable` | 禁用 |
| `POST` | `/api/v1/extensions/{id}:update` | 更新 |
| `POST` | `/api/v1/extensions/{id}:rollback` | 回滚 |
| `GET` | `/api/v1/extensions` | 列表 |
| `GET` | `/api/v1/extensions/{id}` | 详情 |
| `POST` | `/api/v1/skills:invoke` | 调用 Skill |
| `POST` | `/api/v1/mcp/servers/{id}:connect` | 连接 MCP |
| `POST` | `/api/v1/mcp/calls/{id}:reconcile` | 对账 |
| `GET` | `/api/v1/extensions/events` | 实时事件 |

### 28.4 事件

事件名称建议：

```text
ExtensionCandidateDiscovered
ExtensionRevisionValidated
ExtensionTrustChanged
ExtensionEnabled
ExtensionDisabled
ExtensionActivated
ExtensionRolledBack
ExtensionQuarantined
ExtensionCrashed
ExtensionCircuitOpened
SkillMetadataLoaded
SkillBodyLoaded
SkillResourceLoaded
SkillInvocationStarted
SkillInvocationFinished
McpServerConnecting
McpServerReady
McpSchemaChanged
McpServerDisconnected
McpCallStarted
McpCallFinished
McpCallTimeoutUnknown
McpReconcileRequired
HookInvocationStarted
HookInvocationFinished
HookBlocked
HookCircuitOpened
PluginContributionRegistered
```

事件 payload 只包含稳定 ID、摘要、digest、状态和引用；敏感输入/输出放在受控 Blob，不直接广播给所有客户端。

### 28.5 幂等与并发

所有写 command 带 `CommandMeta.idempotency_key`。并发规则：

- 同一 extension revision 同时 enable 只有一个提交者成功；
- 同一 MCP server connect 使用 server scope mutex；
- 同一 Hook event + revision unique；
- update/rollback 使用 registry compare-and-swap generation；
- 过期 command 返回 `REGISTRY_GENERATION_CONFLICT`，要求重新读取。

---

## 29. SQLite 数据模型与迁移

### 29.1 设计原则

与既有 `Skills、MCP、Memory、Hook 与 Plugin 表族` 对齐，并补充不可变 revision、权限、schema 和运行后端信息。文件/包是扩展内容权威；SQLite 是注册、授权、观测、事件和 projection 权威。

### 29.2 统一扩展表

```sql
CREATE TABLE extensions (
    extension_id TEXT PRIMARY KEY,
    canonical_name TEXT NOT NULL,
    kind TEXT NOT NULL,
    source_kind TEXT NOT NULL,
    scope_type TEXT NOT NULL,
    scope_id TEXT,
    publisher_id TEXT,
    state TEXT NOT NULL,
    active_revision_id TEXT,
    created_at_us INTEGER NOT NULL,
    updated_at_us INTEGER NOT NULL,
    UNIQUE(canonical_name, scope_type, scope_id)
);

CREATE TABLE extension_revisions (
    revision_id TEXT PRIMARY KEY,
    extension_id TEXT NOT NULL REFERENCES extensions(extension_id),
    display_version TEXT,
    manifest_digest TEXT NOT NULL,
    content_digest TEXT NOT NULL,
    manifest_json TEXT NOT NULL CHECK(json_valid(manifest_json)),
    source_locator TEXT,
    source_commit TEXT,
    trust_state TEXT NOT NULL
        CHECK (trust_state IN ('untrusted','discovered','pinned','signed','managed','legacy_unpinned','revoked')),
    runtime_backend TEXT NOT NULL,
    protocol_major INTEGER NOT NULL,
    protocol_minor INTEGER NOT NULL,
    requested_capabilities_json TEXT NOT NULL CHECK(json_valid(requested_capabilities_json)),
    dependency_lock_json TEXT NOT NULL CHECK(json_valid(dependency_lock_json)),
    signature_set_digest TEXT,
    created_at_us INTEGER NOT NULL,
    retired_at_us INTEGER,
    UNIQUE(extension_id, content_digest)
);

CREATE TABLE extension_generations (
    registry_generation INTEGER PRIMARY KEY,
    active_revision_set_digest TEXT NOT NULL,
    reason TEXT NOT NULL,
    created_at_us INTEGER NOT NULL
);
```

### 29.3 授权与实例

```sql
CREATE TABLE extension_grants (
    grant_id TEXT PRIMARY KEY,
    revision_id TEXT NOT NULL REFERENCES extension_revisions(revision_id),
    subject_type TEXT NOT NULL,
    subject_id TEXT NOT NULL,
    capability TEXT NOT NULL,
    scope_json TEXT NOT NULL CHECK(json_valid(scope_json)),
    conditions_json TEXT NOT NULL CHECK(json_valid(conditions_json)),
    decision_id TEXT,
    expires_at_us INTEGER,
    state TEXT NOT NULL,
    created_at_us INTEGER NOT NULL,
    revoked_at_us INTEGER
);

CREATE TABLE extension_instances (
    instance_id TEXT PRIMARY KEY,
    revision_id TEXT NOT NULL REFERENCES extension_revisions(revision_id),
    project_id TEXT,
    session_id TEXT,
    backend TEXT NOT NULL,
    state TEXT NOT NULL,
    process_identity_json TEXT,
    started_at_us INTEGER,
    stopped_at_us INTEGER,
    last_error_json TEXT,
    created_at_us INTEGER NOT NULL
);
```

### 29.4 Skill 增补表

在既有 `skills`、`skill_loads` 上补充：

```sql
CREATE TABLE skill_revisions (
    skill_revision_id TEXT PRIMARY KEY,
    skill_id TEXT NOT NULL REFERENCES skills(skill_id),
    extension_revision_id TEXT REFERENCES extension_revisions(revision_id),
    frontmatter_json TEXT NOT NULL CHECK(json_valid(frontmatter_json)),
    body_digest TEXT NOT NULL,
    metadata_tokens INTEGER,
    body_tokens INTEGER,
    state TEXT NOT NULL,
    created_at_us INTEGER NOT NULL
);

CREATE TABLE skill_resource_loads (
    resource_load_id TEXT PRIMARY KEY,
    skill_revision_id TEXT NOT NULL REFERENCES skill_revisions(skill_revision_id),
    run_id TEXT,
    relative_path TEXT NOT NULL,
    resource_digest TEXT NOT NULL,
    resource_kind TEXT NOT NULL,
    bytes_read INTEGER NOT NULL,
    token_estimate INTEGER,
    taint_json TEXT NOT NULL CHECK(json_valid(taint_json)),
    state TEXT NOT NULL,
    created_at_us INTEGER NOT NULL
);
```

### 29.5 MCP 增补表

在既有 `mcp_servers`、`mcp_tools`、`mcp_calls` 上补充：

```sql
CREATE TABLE mcp_schema_revisions (
    schema_revision_id TEXT PRIMARY KEY,
    mcp_server_id TEXT NOT NULL REFERENCES mcp_servers(mcp_server_id),
    revision_number INTEGER NOT NULL,
    protocol_version TEXT,
    schema_digest TEXT NOT NULL,
    tools_json TEXT NOT NULL CHECK(json_valid(tools_json)),
    resources_json TEXT NOT NULL CHECK(json_valid(resources_json)),
    prompts_json TEXT NOT NULL CHECK(json_valid(prompts_json)),
    state TEXT NOT NULL,
    discovered_at_us INTEGER NOT NULL,
    retired_at_us INTEGER,
    UNIQUE(mcp_server_id, schema_digest)
);

ALTER TABLE mcp_calls ADD COLUMN schema_revision_id TEXT REFERENCES mcp_schema_revisions(schema_revision_id);
ALTER TABLE mcp_calls ADD COLUMN reconcile_state TEXT NOT NULL DEFAULT 'not_required';
ALTER TABLE mcp_calls ADD COLUMN idempotency_key TEXT;
```

### 29.6 Hook 增补表

```sql
CREATE TABLE hook_subscriptions (
    hook_subscription_id TEXT PRIMARY KEY,
    revision_id TEXT NOT NULL REFERENCES extension_revisions(revision_id),
    hook_name TEXT NOT NULL,
    event_name TEXT NOT NULL,
    matcher_json TEXT NOT NULL CHECK(json_valid(matcher_json)),
    priority INTEGER NOT NULL DEFAULT 500,
    failure_policy TEXT NOT NULL,
    timeout_ms INTEGER NOT NULL,
    state TEXT NOT NULL,
    created_at_us INTEGER NOT NULL
);

ALTER TABLE hook_invocations ADD COLUMN revision_id TEXT REFERENCES extension_revisions(revision_id);
ALTER TABLE hook_invocations ADD COLUMN sequence_no INTEGER;
ALTER TABLE hook_invocations ADD COLUMN decision_json TEXT CHECK(json_valid(decision_json));
```

### 29.7 迁移与兼容

- SQLite migration 每次有单调版本；
- `ALTER TABLE` 后校验旧安装的列存在性；
- 既有 `plugins` 表可通过 backfill 建立 `extensions(kind=plugin)` 和 `extension_revisions`；
- 既有 Skill/MCP ID 保留，避免历史引用断裂；
- backfill 失败进入 migration diagnostic，不删除旧数据；
- projection 可从权威 revision/event 重建；
- 迁移期间禁止加载不完整 registry generation。

---

## 30. 崩溃恢复、进程清理与一致性

### 30.1 启动恢复顺序

```text
open SQLite WAL
 → recover operation journal
 → recover extension install intents
 → mark orphan instances stopping/unknown
 → reconcile process supervisor inventory
 → load active registry generation
 → validate revision files/digests
 → rebuild runtime projections
 → resume eligible invocations
```

### 30.2 安装 intent

安装/更新使用：

```text
intent(created)
 → staging ready
 → revision committed
 → registry switched
 → intent finalized
```

启动发现未 finalize 的 intent 时，根据磁盘 digest 和 DB 状态选择 finalize、rollback 或 quarantine，不通过文件时间猜测。

### 30.3 MCP 崩溃

MCP server 崩溃后：

- 对未发送调用标记 `failed_before_send`（必须有 transport 证据）；
- 对已写入 transport 但无响应调用标记 `unknown`；
- 清理所有子孙进程；
- 保留 schema revision，但 state = degraded/stale；
- 根据 policy 只对可证明幂等操作重试；
- 外部副作用调用进入 reconcile queue。

### 30.4 Plugin/Hook 崩溃

- Wasm instance 直接回收，调用得到 resource/crash error；
- 受监督 Plugin 进程按 failure policy 处理；
- 不把 Plugin crash 传播为 Core panic；
- 连续失败触发 circuit breaker；
- 未提交的 proposed ToolCall 不进入主控制流；
- 已产生的诊断事件保留。

### 30.5 Session 与 checkpoint 恢复

恢复 Run 时固定的 extension revision、MCP schema revision 和 registry generation 作为历史事实。若无法启动旧后端：

- 只读历史 Context 可展示；
- 新调用必须重新解析/审批；
- 旧 Hook 不能被最新版替代后声称重放相同语义；
- Gate/RuleCheck 依赖不可用时按 stale/inconclusive 处理。

---

## 31. 性能、缓存与容量规划

### 31.1 性能目标

初始工程目标（以中等开发机、无网络等待为基线）：

| 操作 | 目标 |
|---|---:|
| 1000 个 Skill metadata discovery | p95 < 1 s（热缓存） |
| 单 Skill metadata refresh | p95 < 50 ms |
| Skill body bounded load | p95 < 100 ms（本地文件） |
| Wasm Hook invocation | p95 < 20 ms（不含 host call） |
| Plugin process handshake | p95 < 2 s |
| MCP local initialize | p95 < 5 s |
| registry generation query | p95 < 20 ms |
| Hook matching 1000 subscriptions | p95 < 5 ms |

这些目标不包括外部网络、用户审批和模型延迟。

### 31.2 缓存层

- Discovery cache：path/digest → parsed metadata；
- Skill body cache：revision/path → bounded body；
- Resource cache：revision/resource digest → Blob reference；
- MCP schema cache：server config digest + handshake fingerprint；
- Hook matcher index：event/tool/path/risk 维度；
- Plugin module cache：content digest → verified staging artifact。

缓存永远不能改变授权结论。grant、trust、policy、revision 或 registry generation 变化必须使相关缓存失效。

### 31.3 容量限制

默认限制：

- 单项目 active extensions：500；
- 单 registry generation Skill descriptors：5000；
- 单 Plugin 包：256 MiB；
- 单 Skill body：1 MiB；
- 单 MCP response：16 MiB（可按策略调整）；
- 单 Hook output：1 MiB；
- 单 extension process child：8；
- 单 server 并发调用：由 manifest/policy 最小值确定；
- 单 session extension context：由 token budget 决定。

达到上限时拒绝或降级，不能无提示截断控制信息。

### 31.4 并发

- Registry 写操作单 project 串行；
- 不同 project 可并行扫描；
- MCP server 每实例有 semaphore；
- Hook blocking chain 串行；observer 可受控并行；
- Wasm instance 池按 revision 隔离；
- Process/MCP 资源受全局 supervisor budget 限制。

---

## 32. 测试、仿真与故障注入

### 32.1 单元测试

覆盖：

- manifest canonicalization、path safety、digest；
- Skill frontmatter、metadata/body/resources 解析；
- source precedence/conflict；
- capability intersection；
- matcher、priority、Hook merge；
- MCP namespace/schema digest；
- retry/idempotency/reconcile state；
- taint propagation；
- dependency resolver；
- migration/backfill。

### 32.2 契约测试

每个 Plugin/MCP/Hook SDK 必须通过：

- handshake version matrix；
- unknown field/enum；
- timeout/cancel；
- oversized input/output；
- malformed schema/result；
- secret redaction；
- protocol error；
- crash and restart；
- registry hot update；
- capability denial。

### 32.3 集成测试

场景包括：

1. 项目 Skill 覆盖兼容 Skill 的冲突诊断；
2. Skill body 引用 `../` 被拒绝；
3. MCP `.mcp.json` 被发现但未自动启动；
4. MCP tool schema 改变使旧审批失效；
5. MCP timeout 后进入 `reconcile_required`；
6. PreToolUse Hook deny 不能被 Plugin 改成 allow；
7. PostToolUse Hook 阻止 completion 但保留原结果；
8. Plugin 新增 `data.egress.v1` 触发重新授权；
9. Wasm 资源超限不影响 Session Actor；
10. MCP 子进程子孙进程被完整回收；
11. 热更新期间旧 Run 固定旧 revision；
12. Rules Gate 对 custom checker 更新标记 stale。

### 32.4 故障注入

可注入：

- 启动中断、DB crash、磁盘满；
- 下载中断、签名错误、同 digest 内容替换；
- stdio half-close、远程 TCP reset、DNS 变化；
- Hook hang、无限输出、恶意重入；
- Plugin OOM、panic、非法 host call；
- schema notification 在调用中到达；
- Credential revoke；
- process tree 脱离父进程；
- clock skew、deadline 过期。

### 32.5 安全测试

- zip slip、symlink race、ADS、UNC path；
- YAML parser bomb、regex DoS；
- prompt injection 控制标记伪造；
- tool schema poisoning；
- taint label 丢失；
- capability confused deputy；
- output/log Secret 泄漏；
- publisher key revoke 与 rollback；
- UI payload XSS/command injection。

---

## 33. Rust 模块与接口边界

建议模块：

```text
crates/
├── extension-core/
│   ├── domain.rs
│   ├── registry.rs
│   ├── revision.rs
│   ├── manifest.rs
│   ├── trust.rs
│   ├── dependency.rs
│   ├── capability.rs
│   └── policy.rs
├── extension-discovery/
│   ├── sources.rs
│   ├── skill_scanner.rs
│   ├── mcp_importer.rs
│   └── plugin_scanner.rs
├── extension-runtime/
│   ├── backend.rs
│   ├── wasm.rs
│   ├── process.rs
│   ├── remote.rs
│   └── supervisor.rs
├── extension-skill/
│   ├── frontmatter.rs
│   ├── progressive.rs
│   ├── resource.rs
│   └── invocation.rs
├── extension-mcp/
│   ├── protocol.rs
│   ├── connection.rs
│   ├── schema.rs
│   ├── call.rs
│   └── reconcile.rs
├── extension-hook/
│   ├── subscription.rs
│   ├── matcher.rs
│   ├── chain.rs
│   └── result.rs
├── extension-plugin/
│   ├── package.rs
│   ├── signature.rs
│   ├── sdk_protocol.rs
│   └── contributions.rs
├── extension-observability/
│   ├── metrics.rs
│   ├── events.rs
│   └── projections.rs
└── extension-api/
    ├── commands.rs
    ├── queries.rs
    └── events.rs
```

### 33.1 Ports

```rust
#[async_trait]
pub trait ExtensionRegistryPort {
    async fn active_view(&self, project_id: ProjectId) -> Result<RegistryView>;
    async fn activate(&self, expected_generation: u64, set: ActiveRevisionSet) -> Result<Generation>;
}

#[async_trait]
pub trait ExtensionRuntimePort {
    async fn start(&self, request: StartExtension) -> Result<InstanceHandle>;
    async fn invoke(&self, handle: &InstanceHandle, request: ExtensionRequest) -> Result<ExtensionResponse>;
    async fn stop(&self, handle: &InstanceHandle, reason: StopReason) -> Result<()>;
}

#[async_trait]
pub trait McpTransportPort {
    async fn initialize(&mut self, request: InitializeRequest) -> Result<InitializeResult>;
    async fn list_capabilities(&mut self) -> Result<CapabilitySnapshot>;
    async fn call_tool(&mut self, request: McpToolRequest) -> Result<McpToolResponse>;
    async fn cancel(&mut self, request_id: RequestId) -> Result<CancelResult>;
}

#[async_trait]
pub trait HookRunnerPort {
    async fn invoke(&self, invocation: HookInvocationRequest) -> Result<HookResult>;
}
```

这些 port 只暴露协议对象，不暴露 Core 内部 actor、repository 或数据库连接。

### 33.2 Actor 边界

- `ExtensionRegistryActor`：扫描结果、revision、generation；
- `ExtensionSupervisorActor`：实例、进程、重启、资源；
- `McpServerActor`：单 server transport/schema/call 序列；
- `HookDispatcherActor`：事件订阅、顺序、并发和熔断；
- `SkillIndexActor`：发现、metadata、body/resource cache；
- `PluginInstallActor`：staging、签名、依赖、激活；
- `ExtensionProjectionActor`：面板查询模型和指标。

Session Actor 只依赖这些 actor 的命令/查询接口，不直接管理外部进程。

---

## 34. 交付阶段与迁移路线

> ADR-0001（跨文档一致性审查）：本节原使用 v0.4 / v0.6 / v0.8 三个基线路线图中不存在的档位。现收编为五档基线（v0.1/v0.3/v0.5/v0.7/v1.0）内的子阶段：原 v0.4 基础能力并入 v0.5 前段（Skill 基础），原 v0.6 Trust+Reconcile 并入 v0.5 后段，原 v0.8 Wasm/SDK 并入 v0.7 后段。

### 34.1 v0.5 阶段一：Skill 基础能力

- 统一 ExtensionDefinition/Revision；
- Skill metadata/body/resources 解析；
- project/user/兼容目录发现；
- Tool Gateway 接入 Skill resource/script；
- 结构化事件和基础 Skill 面板。

### 34.2 v0.5 阶段二：MCP + Skills

- 本地 MCP 自动发现；
- `.mcp.json` 兼容导入；
- stdio 与 SSE/HTTP；
- server supervisor、进程树清理；
- schema revision；
- MCP 面板和基础重连；
- Skill 显式/模型触发、token 观测；
- Memory 索引接入（与基线 v0.5 的 Memory 能力同期）。

### 34.3 v0.5 阶段三：Trust + Reconcile

- extension revision、签名/digest；
- Capability grant；
- credential ref；
- MCP timeout unknown、reconcile queue；
- external taint 和 data egress policy；
- 崩溃恢复。

### 34.4 v0.7 阶段一：Hook

- `PreToolUse`、`PostToolUse`、`AgentStop`/`SessionStop`（外部 `Stop` 经兼容导入器映射到 `AgentStop`）；
- deterministic matcher/order；
- fail closed/open；
- Hook panel、熔断、重入保护；
- Rules/Gate integration。

### 34.5 v0.7 阶段二：Wasm 与 SDK

- Wasm/WASI backend；
- Plugin protocol v1；
- manifest validator/packager；
- fake Core 与 contract tests；
- declaration-only Panel contribution。

### 34.6 v1.0 Plugin API

- 受监督 process Plugin；
- Wasm + process hybrid；
- install/update/rollback/marketplace/managed policy；
- Provider、checker、importer、panel API；
- 供应链撤销和企业分发；
- 完整 observability、audit、migration、fault injection。

### 34.7 兼容迁移

旧版只记录 `plugins`、`skills`、`mcp_servers` 的系统应通过 backfill 建立统一 extension/revision，但不得改变历史审计语义。无法确定旧内容 digest 时标记 `trust_state=legacy_unpinned`，禁止自动复用高风险 grant。

> ADR-0029（跨文档一致性审查）：`legacy_unpinned` 原只在本节散文中出现，未进入任何枚举清单。现已加入 `extension_revisions.trust_state` 的 CHECK 约束（§29 表定义），迁移产生的状态值必须先在枚举中登记才能使用。

---

## 35. ADR：关键架构决策

### ADR-EXT-001：统一 Registry，保留四类运行语义

**决定**：Skill、MCP、Hook、Plugin 共享 discovery/trust/revision/capability/observability，但分别由独立 domain runtime 管理。  
**原因**：减少重复治理，同时避免把 Skill 当脚本、把 MCP 当普通 Tool、把 Hook 当可变 middleware。  
**替代方案**：全部统一为 Plugin；被否决，因为会破坏生态兼容和最小权限边界。

### ADR-EXT-002：采用混合 Wasm + 受监督子进程

**决定**：纯计算 Plugin 优先 Wasm/WASI；生态二进制、MCP stdio、OS API 使用 supervised subprocess；第三方 native 不进 Core。  
**原因**：同时满足可移植性、生态兼容、资源隔离和平台集成。  
**替代方案**：只用 Wasm；无法覆盖现有 MCP 和系统工具。只用进程；隔离和可复现性成本更高。

### ADR-EXT-003：本地自动发现不自动启用

**决定**：发现只产生 Candidate，连接和脚本执行必须显式启用/授权。  
**原因**：本地配置和第三方目录可能包含未经审查的网络、进程和 Secret 引用。

### ADR-EXT-004：Skill 三层渐进加载

**决定**：`metadata → body → resources`，正文和资源按需加载。  
**原因**：兼容 Claude Code/pi/opencode 的 token 经济性，避免大量 Skill 侵占 Context。

### ADR-EXT-005：MCP schema revision 绑定审批

**决定**：tool/resource/prompt 能力计算不可变 schema revision；schema 变化使未执行旧审批失效。  
**原因**：名称不变也可能通过输入 schema/description/annotations 改变风险。

### ADR-EXT-006：Hook 不允许静默 mutation

**决定**：Hook 修改只能通过 `ProposeRewrite → new ToolCall revision → revalidate`。  
**原因**：保留责任链、审批依据和可重放性。

### ADR-EXT-007：外部结果默认 tainted

**决定**：MCP、未信任 Skill、Plugin 输出不能进入控制平面；必须保留 provenance。  
**原因**：防止 Prompt injection 和 confused deputy。

### ADR-EXT-008：未知外部效果必须 reconcile

**决定**：超时/断线不能推断未执行；副作用型 MCP 操作进入 reconcile_required。  
**原因**：避免重复创建、重复发送或数据不一致。

### ADR-EXT-009：声明式 Panel 优先

**决定**：v1 Plugin UI 只注册安全 schema，禁止任意 JS 注入。  
**原因**：保持 Core/UI/权限边界，减少 XSS 和控制面伪造。

---

## 36. 实现前审查清单

### 36.1 领域与协议

- [ ] 四类扩展是否有清晰边界；
- [ ] 每个 revision 是否不可变且有 digest；
- [ ] Tool/Hook/MCP/Skill 是否可追溯到 operation/run；
- [ ] protocol major/minor 是否明确；
- [ ] 错误是否使用稳定 code。

### 36.2 权限与安全

- [ ] Capability 是否按最小粒度声明；
- [ ] effective capability 是否只取交集；
- [ ] hard deny 是否无法被 Hook/Plugin 绕过；
- [ ] Secret 是否只存 CredentialStore；
- [ ] 外部内容是否标记 taint；
- [ ] `.mcp.json` 是否发现而不自动启动；
- [ ] path/symlink/压缩包是否安全。

### 36.3 生命周期

- [ ] 安装、更新、回滚是否原子；
- [ ] hot update 是否 pin 旧运行；
- [ ] process tree 是否完整清理；
- [ ] unknown MCP call 是否进入 reconcile_required；
- [ ] crash recovery 是否可区分未发送/已发送/未知；
- [ ] quarantine/revoke 是否阻断新调用。

### 36.4 Context/Rules/Gate

- [ ] 是否实现 `metadata → body → resources`；
- [ ] Skill/MCP/Plugin 内容是否不能改变 system contract；
- [ ] schema revision 是否绑定审批和 Gate；
- [ ] custom checker 是否只能返回 receipt；
- [ ] Hook deny/block 是否保留原始证据；
- [ ] extension update 是否让依赖检查 stale。

### 36.5 产品体验

- [ ] Skill 面板是否显示加载层级和 token；
- [ ] MCP 面板是否显示连接、schema、延迟和对账；
- [ ] Hook 面板是否可解释顺序和决策；
- [ ] Plugin 面板是否展示 capability diff；
- [ ] 用户是否知道来源、publisher、风险和版本；
- [ ] 失败是否给出下一步可执行动作。

---

## 37. 结论与后续文档

Apex 的扩展系统应当被实现为一个受 Core 控制的 Extension Control Plane，而不是一组散落的“读取目录、启动进程、拼接 Prompt”的工具函数。其关键约束是：

1. 所有扩展都以不可变 revision 运行；
2. 所有实际能力都通过 Capability、Tool Gateway 和 Rules/Permission 链路；
3. Skill 通过 `metadata → body → resources` 控制 Context 成本；
4. MCP 通过 server supervision、schema revision、taint 和 reconcile 管理外部世界；
5. Hook 只能观察、收窄、诊断或提出新请求；
6. Plugin 通过稳定 protocol、Wasm/受监督子进程和声明式贡献接入；
7. 自动发现、安装、启用、更新、撤销、恢复和审计都由 Core 原子化管理；
8. UI 只呈现 projection，不成为权限或状态权威。

下一份建议文档为：

> **`Apex—— Credential与敏感数据治理详细设计.md`**

该文档将把本文中的 `credential.use.v1`、`data.egress.v1`、Secret 注入、脱敏、外部数据分类、审计留存、密钥轮换与组织策略落成独立的最终产品设计。

---

# 附录 A：Skill Manifest 示例

```yaml
apiVersion: apex.dev/v1
kind: Skill
metadata:
  name: rust-testing
  version: 1.2.0
spec:
  entry: SKILL.md
  sourceCompatibility:
    - agentskills/v1
  invocation:
    model: true
    user: true
    disableModelInvocation: false
  allowedTools:
    - Read
    - Bash
  requiresTools:
    - Bash
  phase: implementation
  limits:
    bodyBytes: 1048576
    resourceBytes: 8388608
    contextTokens: 5000
```

# 附录 B：MCP Server 导入后的规范结构

```json
{
  "api_version": "apex.dev/mcp/v1",
  "server_id": "mcp_01J...",
  "name": "github",
  "source": {
    "kind": "project_file",
    "path": ".mcp.json",
    "pointer": "/mcpServers/github"
  },
  "transport": {
    "kind": "stdio",
    "command": "npx",
    "args": ["-y", "@example/github-mcp"]
  },
  "credential_refs": ["cred_github_token"],
  "policy": {
    "allow": false,
    "requires_confirmation": true
  },
  "config_digest": "sha256:..."
}
```

# 附录 C：Hook 配置示例

```yaml
apiVersion: apex.dev/v1
kind: HookBundle
metadata:
  name: com.example.security
  version: 1.0.0
spec:
  hooks:
    - name: prevent-sensitive-write
      event: PreToolUse
      matcher:
        toolNames: [Write, Edit]
        paths: [".env", "**/*.pem"]
      runtime:
        type: wasm
        entry: modules/security.wasm
      decision:
        allowed: [continue, deny, request_approval, diagnostic_only]
      failurePolicy: fail_closed
      timeoutMs: 500
```

# 附录 D：Extension 错误码

| 错误码 | 语义 |
|---|---|
| `EXTENSION_MANIFEST_INVALID` | manifest 不符合 schema |
| `EXTENSION_DIGEST_MISMATCH` | 内容摘要不一致 |
| `EXTENSION_SIGNATURE_INVALID` | 签名无效 |
| `EXTENSION_TRUST_REQUIRED` | 需要显式信任 |
| `EXTENSION_POLICY_DENIED` | 策略拒绝 |
| `EXTENSION_DEPENDENCY_CONFLICT` | 依赖冲突或循环 |
| `EXTENSION_PROTOCOL_UNSUPPORTED` | protocol 不兼容 |
| `EXTENSION_SANDBOX_UNAVAILABLE` | 无可用安全后端 |
| `EXTENSION_RESOURCE_EXHAUSTED` | 资源预算耗尽 |
| `EXTENSION_CIRCUIT_OPEN` | 扩展熔断 |
| `SKILL_FRONTMATTER_INVALID` | Skill frontmatter 无效 |
| `SKILL_RESOURCE_PATH_DENIED` | Skill 资源路径越界 |
| `SKILL_CONTEXT_BUDGET_EXCEEDED` | Skill Context 超预算 |
| `MCP_SERVER_DISABLED` | MCP server 未启用 |
| `MCP_SCHEMA_CHANGED` | schema revision 已改变 |
| `MCP_CALL_TIMEOUT_UNKNOWN` | MCP 调用超时且状态未知 |
| `MCP_RECONCILIATION_UNSUPPORTED` | 外部状态无法对账 |
| `MCP_EGRESS_DENIED` | 外发数据策略拒绝 |
| `HOOK_TIMEOUT` | Hook 超时 |
| `HOOK_REENTRANCY_LIMIT` | Hook 重入深度超限 |
| `HOOK_REWRITE_CONFLICT` | Hook 提议改写冲突 |
| `PLUGIN_CONTRIBUTION_INVALID` | Plugin contribution 无效 |
| `REGISTRY_GENERATION_CONFLICT` | Registry generation 过期 |
| `EXTENSION_REVISION_UNAVAILABLE` | 固定 revision 不可用 |

# 附录 E：关键不变量

1. **不可变版本不变量**：运行、审批、缓存、Gate 证据绑定 revision digest；
2. **权限交集不变量**：扩展不能把任何父级 Capability 变大；
3. **Core 权威不变量**：扩展不能直接修改控制面状态；
4. **渐进加载不变量**：Skill metadata、body、resources 具有独立加载与审计记录；
5. **schema 绑定不变量**：MCP schema 变化使未执行旧审批失效；
6. **taint 不升级不变量**：外部内容不得自动升级为 trusted control data；
7. **未知效果不重试不变量**：MCP unknown 状态不得无条件重试；
8. **Hook 收窄不变量**：Hook 只能继续、诊断、拒绝、请求审批或提出新请求；
9. **进程隔离不变量**：第三方 native 代码不进入 Core 地址空间；
10. **审计保留不变量**：卸载/撤销不删除历史证据；
11. **热更新不变量**：旧运行继续使用其固定 revision；
12. **面板非权威不变量**：UI 任何动作都必须转为 Command。

# 附录 F：与已有文档的交叉引用

| 主题 | 本文依赖/实现位置 |
|---|---|
| 总体模块边界 | `Apex—— 系统总体架构设计.md` §9、§10 |
| 领域事件 | `Apex—— 领域模型与事件规范.md` 的 Event/Operation 约定 |
| Command/Query/Event | `Apex—— API与实时事件协议设计.md` §7.8、实时事件章节 |
| SQLite | `Apex—— SQLite数据模型与迁移设计.md` §13 及本文 §29 |
| Agent/DAG | `Apex—— Agent Runtime与DAG调度器详细设计.md` 的 Tool/Run/Node 约束 |
| 权限与 Tool Gateway | `Apex—— Tool Gateway与权限引擎详细设计.md` §16、§19、§29 |
| Context/Checkpoint | `Apex—— Context与Checkpoint系统详细设计.md` 的 taint、volatile suffix、恢复语义 |
| Rules/Gate | `Apex—— Rules与Verification Gate详细设计.md` 的 Extension Manifest、Hook、Checker、Reconcile Gate |
| 下一主题 | `Apex—— Credential与敏感数据治理详细设计.md` |
