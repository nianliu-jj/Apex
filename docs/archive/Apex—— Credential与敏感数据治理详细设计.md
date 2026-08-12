# Apex—— Credential与敏感数据治理详细设计

> 文档状态：详细设计基线（面向最终完整产品）  
> 适用版本：Apex v0.1 ～ v1.x（按 §交付阶段 分档启用；档位表以需求文档 §5.3 为准）  
> 上游文档：`Apex—— 需求分析文档.md`、`Apex—— 系统总体架构设计.md`、`Apex—— 领域模型与事件规范.md`、`Apex—— API与实时事件协议设计.md`、`Apex—— SQLite数据模型与迁移设计.md`、`Apex—— Tool Gateway与权限引擎详细设计.md`、`Apex—— Rules与Verification Gate详细设计.md`、`Apex—— MCP、Skill、Hook与Plugin扩展系统详细设计.md`  
> 关键词：CredentialStore、CredentialRef、Secret、Data Classification、Redaction、Taint、Data Egress、Rotation、Revocation、Privacy Purge

---

## 0. 文档目的与范围

本文定义 Apex 最终产品对 Credential、Secret、敏感文件、敏感文本、外部数据和数据外发的统一治理方式，解决以下问题：

1. API key、Provider token、OAuth token、Cookie、SSH key、证书和密码应存在哪里；
2. Secret 如何导入、验证、绑定、使用、轮换、撤销和销毁；
3. Agent、Tool、MCP、Skill、Hook、Plugin、Provider 和 UI 在什么条件下可以使用敏感数据；
4. 敏感数据如何在 Prompt、ToolCall、事件、日志、Blob、Checkpoint、Memory、备份和 telemetry 中被隔离；
5. “使用 credential”与“向某个目的地外发数据”为什么必须是两个独立的权限维度；
6. 如何在可用性、兼容性、可审计性和 fail-closed 安全之间取得可实现的平衡。

本文覆盖：

- 数据分级、Secret 与 Credential 的领域模型；
- OS keyring、兼容 `~/apex/auth.json` 和未来远程 Vault 的抽象；
- CredentialRef、Secret Broker、临时注入信封和最小权限；
- Provider、MCP、Shell、Git、Browser、Plugin、Hook 的敏感数据路径；
- redaction、secret scan、taint、data lineage 和 data egress policy；
- API、事件、SQLite 表、备份、恢复、隐私清除和审计；
- 密钥轮换、撤销、泄漏响应、测试和交付路线。

本文不定义：

- 通用 Tool Gateway 的完整判权算法；
- OS keyring 内部实现和各平台密码库的全部细节；
- 企业外部 KMS/Vault 的具体部署产品；
- Prompt/Context 的主体压缩算法；
- 具体 Provider 的商业认证流程。

这些内容通过稳定 Port、CredentialRef、Policy 和事件协议接入。

---

## 1. 核心架构结论

### 1.1 Secret 不是普通字符串

Secret 是具有泄漏后果、生命周期、来源、作用域和使用条件的数据对象。任何看起来像 token 的字符串都不能简单通过字段名判断为 Secret；同时，字段名未知也不能成为放行理由。

```text
Secret value
  → CredentialStore encrypted material
  → Credential Broker short-lived lease
  → adapter/process isolated injection
  → operation-scoped use
  → output/log/event redaction
  → lease revoke/cleanup
```

### 1.2 CredentialRef 与 Secret Value 永久分离

Apex 的业务层只传递 `CredentialRef` 或 `SecretHandle`，不传递 Secret 原文：

```rust
pub struct CredentialRef {
    pub credential_id: CredentialId,
    pub kind: CredentialKind,
    pub scope: CredentialScope,
    pub version: CredentialVersion,
    pub provider: CredentialProvider,
    pub allowed_tools: Vec<ToolMatcher>,
    pub allowed_destinations: Vec<DestinationMatcher>,
    pub expires_at: Option<Timestamp>,
}
```

Secret 原文只在最后执行边界、由 Broker 按 operation scope 临时取得。UI、Agent、Skill、Hook、Plugin 和普通 Domain Event 永远只看到引用、状态、摘要和使用结果。

### 1.3 Credential use 与 data egress 分离

使用一个 Credential 不等于允许发送任意数据；允许向一个目的地发送数据也不等于允许使用任意 Credential。有效外发需要两个独立 Capability 的交集：

```text
credential.use.v1
∩ data.egress.v1
∩ network.connect.v1 / mcp.invoke.v1
∩ credential scope
∩ destination policy
∩ data classification policy
```

这可以防止“某个工具能够访问 GitHub token”被错误解释成“它可以把整个工作区发送到 GitHub”。

### 1.4 Core 是唯一 Secret 控制面

```text
Agent / SubAgent / Skill / Hook / Plugin / MCP / Provider / UI
                              │
                              ▼
                     Application Command
                              │
                              ▼
                      Core Policy Decision
                              │
            ┌─────────────────┼─────────────────┐
            ▼                 ▼                 ▼
      Credential Broker   Tool Gateway      Audit/Event Store
            │                 │                 │
            ▼                 ▼                 ▼
       CredentialStore    Adapter/Process   Redacted Projections
```

扩展不能：

- 读取 CredentialStore 的数据库或 keyring；
- 从 Core 对象中获取 Secret 指针；
- 通过日志、错误、环境快照、Panel 或 Hook input 间接取得 Secret；
- 自行判断 data egress 是否允许；
- 将 Secret 写入 Spec、Checkpoint、Memory、项目文件或普通 Blob。

### 1.5 默认安全边界

默认策略：

- 未声明、未识别或无法分类的数据按更高敏感级别处理；
- Secret 不进入 SQLite；
- 敏感文件默认只读或询问；
- 远程 MCP、网络外发和 Credential 使用默认需要独立授权；
- 不能完成脱敏时 fail closed；
- 无法确认外部操作状态时进入 reconcile，不以“失败”掩盖未知；
- telemetry 默认关闭，开启后只发送脱敏指标。

### 1.6 可用性边界

Apex 不承诺“任何命令都能无缝使用所有本机凭据”。当 OS keyring 不可用、凭据过期、目的地不匹配、策略要求重新认证或输出无法安全脱敏时，系统应该明确阻断并给出恢复动作，而不是静默降低安全级别。

---

## 2. 设计目标与非目标

### 2.1 设计目标

1. **最小暴露**：Secret 原文只在必要的最后边界、最短时间、最小进程范围内存在。
2. **可审计**：能够回答谁在什么 Run 中、以什么目的、向哪个目的地使用了哪个 Credential 版本。
3. **可撤销**：撤销后阻止新使用，尽可能终止活动 lease，并保留历史事实。
4. **可轮换**：同一逻辑 Credential 的版本可原子切换，旧版本可以 drain/revoke。
5. **可解释**：用户看到的是“使用哪个凭据访问哪个目的地并发送哪类数据”，而非不可理解的内部字符串。
6. **生态兼容**：支持现有 `~/apex/auth.json`，逐步迁移到 OS Credential Store。
7. **跨平台**：Windows ACL、macOS Keychain、Linux Secret Service/Keyring 的差异隐藏在 Port 后。
8. **可恢复**：Credential 元数据、lease、调用和撤销状态 crash 后可恢复；Secret 原文不从普通数据库恢复。
9. **可组合**：与 Tool Gateway、MCP、Rules、Gate、Context、Plugin capability 模型一致。
10. **隐私友好**：支持敏感内容清除、导出、保留期限和审计解释。

### 2.2 非目标

- 不在 Apex 中实现通用企业密码管理器；
- 不默认把所有环境变量导入 CredentialStore；
- 不通过“高熵字符串猜测”替代明确的 Secret 生命周期；
- 不承诺从已被第三方程序读取的 Secret 中恢复安全性；
- 不保证进程终止后 OS 内存中的字节立即物理不可恢复；
- 不把普通项目内容全部当作 Secret，避免系统不可用；
- 不允许项目配置覆盖产品 hard deny 或组织强制策略。

---

## 3. 数据分级与安全标签

### 3.1 四级持久化分级

与总体架构和 SQLite 设计保持一致：

| 分级 | 定义 | 示例 | 默认存储策略 |
|---|---|---|---|
| `public` | 泄露无明显安全/隐私影响 | 项目标题、工具名称、版本号、计数 | 普通 SQLite/事件 |
| `internal` | 项目或会话内部信息 | 状态、非敏感摘要、诊断元数据 | scope 授权 + 脱敏 |
| `confidential` | 泄露可推断项目结构或使用模式 | 任务参数摘要、文件路径、模型用量、规则详情 | scope 授权 + 脱敏，默认不进外部 telemetry |
| `sensitive` | 泄露可能暴露源码、隐私或业务信息 | Prompt 片段、源码、MCP 返回、环境变量名值、个人数据 | Blob/inline 均需授权和审计 |
| `secret_prohibited` | 可直接认证、签名或解密 | API key、token、cookie、私钥、密码、完整 Authorization | 不入 SQLite/事件/日志，交 CredentialStore |

`confidential` 与 `sensitive` 的分界：前者是**关于**工作的元信息（路径、参数摘要、用量），后者是工作**内容本身**（Prompt 正文、源码、外部响应）。可观测面板默认可展示 `confidential` 及以下，`sensitive` 需显式授权，`secret_prohibited` 永不展示。

> ADR-0022（跨文档一致性审查）：原为 4 级（`public/internal/sensitive/secret`）。Observability 详细设计 §2.3 使用 5 级并自称"沿用本文档"，实为不一致。现统一采纳 5 级：新增 `confidential` 层以区分元信息与内容本身（4 级会把二者都压进 `sensitive`，导致面板脱敏粒度过粗）；`secret` 更名 `secret_prohibited`，强调其为禁止落盘类别而非普通敏感级。

另设运行期标签，不改变持久化分级：

- `ephemeral_secret`：仅存在于一次 lease；
- `derived_sensitive`：由 Secret 或 sensitive 内容推导；
- `tainted_external`：来源为 Web/MCP/未信任扩展；
- `user_private`：用户个人数据；
- `regulated`：受组织/法规策略管理；
- `unknown`：无法分类，按最高适用策略处理。

### 3.2 数据标签模型

```rust
pub struct DataLabel {
    pub classification: DataClassification,
    pub categories: BTreeSet<DataCategory>,
    pub source: DataSource,
    pub taint: TaintSet,
    pub lineage: Vec<LineageRef>,
    pub retention: RetentionClass,
    pub redaction_profile: RedactionProfileId,
}
```

`classification` 表示泄露后果；`taint` 表示来源可信度；`lineage` 表示来源链。三者不能互相替代。例如，一个项目源码是 `sensitive`，即使来自 trusted workspace，也不等于可以外发；一个 MCP 结果可以是 `internal` 业务数据，但仍带 `tainted_external`。

### 3.3 Secret 类别

```rust
pub enum CredentialKind {
    ApiKey,
    BearerToken,
    OAuthAccessToken,
    OAuthRefreshToken,
    Password,
    Cookie,
    SshPrivateKey,
    TlsClientCertificate,
    TlsPrivateKey,
    WebhookSigningSecret,
    CloudRole,
    DatabaseCredential,
    EncryptionKey,
    CustomSecret,
}
```

`CustomSecret` 必须声明 provider、用途、scope、rotation policy 和 redaction profile；不能因选择 custom 而跳过安全扫描。

### 3.4 敏感文件默认策略

默认保护：

```text
.env
.env.*
*.key
*.pem
*.p12
*.pfx
credentials*
secrets/**
**/.aws/credentials
**/.ssh/id_*
```

规则：

- Read 默认返回元数据或脱敏摘要，完整读取需要显式确认/策略；
- Write/Edit/Delete 默认 `ask` 或 `deny`；
- sensitive file 不得自动进入 Prompt、Memory 或外发请求；
- 使用专用 credential adapter 优先于直接读取文件；
- 项目规则不能声明“忽略敏感文件保护”来提升权限。

### 3.5 分类优先级

当多个来源给出不同分类时，采用更严格结果：

```text
secret > sensitive > internal > public
```

组织 policy、用户显式标记、Schema 字段声明、已知 Credential 值、文件路径规则、内容 scanner 的结果按优先级合并。降级必须是受控 Command，绑定 Actor、reason、scope 和 TTL。

---

## 4. Credential 领域模型

### 4.1 逻辑 Credential 与版本

```text
CredentialDefinition
  ├─ credential_id: cred_...
  ├─ logical_name: github-work
  ├─ kind: OAuthAccessToken
  ├─ owner/scope
  ├─ provider metadata
  └─ active_version_id

CredentialVersion
  ├─ version_id: crv_...
  ├─ secret_ref in external store
  ├─ fingerprint (non-reversible)
  ├─ valid_from / expires_at
  ├─ status
  ├─ rotation lineage
  └─ allowed use policy
```

Credential 逻辑身份在轮换后保持不变，版本改变。使用审计绑定 version fingerprint，用户可理解为“github-work 的 2026-08-08 版本”，但永远看不到原值。

### 4.2 CredentialRef

```json
{
  "credential_id": "cred_github_work",
  "version": "active",
  "kind": "oauth_access_token",
  "scope": "user",
  "allowed_tools": ["mcp__github__*"],
  "allowed_destinations": ["api.github.com"],
  "purpose": "repository_issue_management"
}
```

`version: active` 只允许在命令解析阶段解析为当前有效版本；一旦 Operation 开始，必须固定为具体 `credential_version_id`。审批、重试和对账不能跨版本悄悄复用。

### 4.3 Credential Scope

```rust
pub enum CredentialScope {
    User,
    Project(ProjectId),
    Workspace(WorkspaceId),
    Session(SessionId),
    Extension(ExtensionRevisionId),
    Operation(OperationId),
}
```

作用域越窄越优先。Project Credential 不自动成为用户全局 Credential；Extension-scoped Credential 不能被另一个 Plugin 枚举或转借。

### 4.4 Credential 状态

```text
candidate → imported → validation_pending → active
active → expiring → rotation_pending → rotated
active → revoked
active → expired
active → quarantined
candidate/imported → rejected
```

状态由 Core 维护；外部 Provider 返回 unauthorized 只改变健康/验证状态，不自动删除 Secret。用户可选择重新认证或切换版本。

### 4.5 Credential 与 Provider 连接

Provider Adapter 不接收全局 Secret map。它收到的是 operation-scoped `CredentialLease`：

```rust
pub struct CredentialLease {
    pub lease_id: LeaseId,
    pub credential_version_id: CredentialVersionId,
    pub operation_id: OperationId,
    pub injection_plan: InjectionPlan,
    pub expires_at: Timestamp,
    pub revocation_epoch: u64,
}
```

Lease 到期、撤销 epoch 变化或 Operation 终结时，Broker 负责清理和失效。

---

## 5. CredentialStore 抽象与存储后端

### 5.1 CredentialStore Port

```rust
#[async_trait]
pub trait CredentialStore: Send + Sync {
    async fn put_secret(
        &self,
        input: StoreCredentialInput,
    ) -> Result<CredentialVersionRef, CredentialStoreError>;

    async fn get_secret_handle(
        &self,
        request: SecretAccessRequest,
    ) -> Result<SecretHandle, CredentialStoreError>;

    async fn revoke_version(
        &self,
        version_id: CredentialVersionId,
        reason: RevokeReason,
    ) -> Result<(), CredentialStoreError>;

    async fn rotate_metadata(
        &self,
        input: RotateCredentialInput,
    ) -> Result<CredentialVersionRef, CredentialStoreError>;

    async fn health(&self) -> Result<CredentialStoreHealth, CredentialStoreError>;
}
```

返回的 `SecretHandle` 只能交给 Broker/Adapter Host，不提供 `String` 或 `Vec<u8>` 给业务层。

### 5.2 后端优先级

1. Windows Credential Manager / DPAPI + ACL；
2. macOS Keychain；
3. Linux Secret Service/系统 keyring；
4. 受组织策略管理的外部 Vault/KMS；
5. 兼容 `~/apex/auth.json` 的文件后端；
6. 明文配置：默认禁用，只允许开发测试 profile，且必须显示强告警。

后端选择是 deployment capability，不改变领域模型。切换后端必须通过迁移 Command，不能在启动时静默复制 Secret。

### 5.3 `~/apex/auth.json` 兼容策略

需求文档要求兼容 `~/apex/auth.json`，但最终安全基线为：

- 优先从 OS Credential Store 读取；
- 发现 `auth.json` 时只读取结构和字段名，先进行权限检查；
- Unix 要求文件 mode 为 0600；Windows 要求 ACL 仅当前用户/受管主体；
- 首次使用可导入到 OS store，导入成功后建议删除或重命名旧文件；
- 文件中的 Secret 不进入 SQLite、日志、诊断和 UI；
- 旧文件变更由 watcher 触发 revalidation，不能直接覆盖 active version；
- 文件后端可被组织 policy 禁止。

### 5.4 文件后端格式

如果使用兼容文件后端，文件必须是版本化、完整性校验的受保护容器，而不是随意 TOML/JSON 明文：

```json
{
  "format": "apex-auth-v2",
  "store_id": "store_...",
  "entries": [
    {
      "credential_id": "cred_openai",
      "kind": "api_key",
      "ciphertext": "...",
      "nonce": "...",
      "key_ref": "os-keyring:apex/auth-file-key",
      "fingerprint": "hmac-sha256:..."
    }
  ],
  "integrity": "..."
}
```

文件加密主密钥必须来自 OS store 或外部 KMS，不能与文件放在同一目录。若没有可用密钥保护，Apex 不应创建该文件后端。

### 5.5 Store health

CredentialStore health 至少包含：

- backend kind/version；
- locked/unlocked；
- available/temporarily unavailable；
- keychain access policy；
- clock/expiry support；
- last successful access；
- migration pending；
- security warning。

health 不返回 Secret、credential count 的过度详细信息或 keychain 错误原文。

---

## 6. 导入、注册与验证

### 6.1 导入流程

```text
用户显式选择来源
 → 读取候选元数据
 → 检查文件/环境/OS 权限
 → Secret 进入受保护输入通道
 → 创建 CredentialVersion
 → 计算不可逆 fingerprint
 → 可选执行最小验证
 → 写 metadata + audit
 → 清理输入 buffer
```

环境变量导入默认不自动扫描全量环境。支持显式变量名、Provider profile 或用户粘贴。命令行参数导入必须避免在 shell history/进程列表中出现。

### 6.2 规范化

导入时只做不会改变认证语义的规范化：

- 去除明确允许的首尾换行；
- 识别 `Bearer ` 前缀但不把 token 写入日志；
- 解析 PEM/JSON/JWT 结构的 metadata；
- 记录 issuer、audience、subject、expiry 等非秘密声明；
- 不自动截断、重编码或猜测缺失字符。

### 6.3 Fingerprint

fingerprint 用于识别同一版本或检测轮换，不可逆：

```text
fingerprint = HMAC(store_scoped_fingerprint_key, secret_bytes)
```

不能使用裸 SHA-256 作为低熵密码的唯一保护。UI 只显示短指纹后缀或“已配置/已变化”。fingerprint key 轮换时需保留可验证历史 lineage，不能暴露旧值。

### 6.4 最小验证

验证按 CredentialKind 适配器执行：

- API key：调用 provider 的身份/余额最小端点；
- OAuth：校验 issuer、expiry、scope，必要时刷新；
- SSH key：只做本地格式/公钥一致性验证，连接测试另行审批；
- TLS cert：解析链、有效期和 key match；
- Cookie：只在明确 Provider adapter 中验证，不通用发送；
- webhook secret：只做长度/格式检查，不能“在线验证”而触发副作用。

验证本身可能外发数据或消耗额度，必须产生 Operation 并使用对应 Network/Credential capability。

### 6.5 导入失败

失败原因分类：

```text
format_invalid
permission_insecure
store_unavailable
provider_rejected
expired
scope_insufficient
policy_denied
validation_timeout_unknown
```

失败消息不回显候选值；不把 Provider 的完整响应直接展示给模型或写入日志。

---

## 7. Secret Broker 与最后时刻注入

### 7.1 Broker 职责

`CredentialBroker` 是 Tool Gateway 与 CredentialStore 之间的唯一运行期桥梁，负责：

- 校验 CredentialRef、operation、tool、destination 和 capability；
- 向 CredentialStore 请求 SecretHandle；
- 生成注入计划；
- 创建 lease、expiry、revocation epoch；
- 绑定进程/transport/adapter；
- 监控使用时间和泄漏事件；
- 在完成、取消、超时、崩溃时清理；
- 向审计提供不含原文的使用证据。

### 7.2 Access request

```rust
pub struct SecretAccessRequest {
    pub operation_id: OperationId,
    pub tool_call_id: ToolCallId,
    pub principal: PrincipalId,
    pub credential_ref: CredentialRef,
    pub destination: Option<DestinationIdentity>,
    pub purpose: PurposeCode,
    pub required_capabilities: CapabilitySet,
    pub data_labels: DataLabelSet,
    pub deadline: Timestamp,
}
```

Broker 不接受缺少 operation_id、purpose、destination（对网络外发）或 data labels 的请求。目的地未知时不能假设为本地。

### 7.3 Injection channel 优先级

允许的注入通道（按优先级）：

1. OS keychain/credential API handle；
2. 受控 stdin/pipe 或协议级 auth channel；
3. 临时文件（严格 ACL、随机名、no-follow、执行后清理）；
4. 子进程私有环境变量。

**命令行参数是硬禁止通道**，不存在"经审批后可用"的例外。理由：进程命令行对同机任意进程可见（Linux `/proc/<pid>/cmdline`、`ps`，Windows WMI/进程枚举），审批无法改变该可见性；且这与 §9 hard deny 条件"注入通道会把 Secret 暴露在命令行或公共日志"直接冲突。工具确需命令行传参时，应改用临时文件或 stdin，由 Adapter 负责转换。

环境变量不是天然安全：应记录变量名而不记录值，防止子进程 dump、诊断和崩溃报告泄漏。

> ADR-0019（跨文档一致性审查）：本节原第 5 项"命令参数仅在无替代方案且有额外审批时使用"已删除。上游需求文档 §4.2 规定密钥不进入命令行参数属硬约束。


### 7.4 Lease

```text
requested → policy_check → issued → injected → active
active → consumed → revoked
active → expired
active → operation_finished → cleanup_pending → cleaned
```

Lease 默认一次 Operation、一次 ToolCall、单一 destination。跨 ToolCall 复用必须显式声明并重新检查条件；Plugin/MCP 不能把 lease 转发给另一个扩展。

### 7.5 清理

清理动作：

- 从注入环境/临时文件移除；
- 关闭 pipe/handle；
- 清空 Broker 内部缓冲区（尽最大努力）；
- 标记 lease revoked/cleaned；
- 清理 staging 和临时 stdout；
- 对进程强杀后进行 child tree reconciliation；
- 若发现疑似泄漏，立即提升 incident severity 并触发 Credential revoke 建议。

Apex 不对 OS、第三方 runtime 或进程 core dump 提供绝对内存擦除保证，但必须避免主动持久化和重复传播。

### 7.6 不允许的注入

禁止：

- 写入项目 `.env` 作为“临时配置”；
- 把 Secret 放入 Tool arguments JSON；
- 把 Secret 拼入 shell command string；
- 把 Secret 放入 Prompt 或 Skill body；
- 把完整 Secret 写入 Hook stdin；
- 把 Secret 作为 Plugin Panel 数据返回；
- 把 Secret 放入 operation journal、event payload、trace baggage。

---

## 8. Credential 使用授权与策略

### 8.1 双重授权模型

Credential 使用必须同时满足两层：

```text
CredentialGrant
  → 允许“使用哪个逻辑凭据/版本”

OperationGrant
  → 允许“在这次操作中做什么、发往哪里、发送什么数据”
```

CredentialGrant 不能代替 Tool permission；Tool permission 不能代替 CredentialGrant。

### 8.2 有效决策输入

```rust
pub struct CredentialPolicyInput {
    pub principal: PrincipalId,
    pub project_id: Option<ProjectId>,
    pub session_id: Option<SessionId>,
    pub run_id: Option<RunId>,
    pub extension_revision: Option<ExtensionRevisionId>,
    pub tool_revision: ToolRevision,
    pub credential_ref: CredentialRef,
    pub destination: Option<DestinationIdentity>,
    pub purpose: PurposeCode,
    pub labels: DataLabelSet,
    pub trust: TrustContext,
    pub approval_context: ApprovalContext,
}
```

判权顺序：

```text
hard deny
 → credential existence/status
 → scope ownership
 → tool/extension binding
 → destination binding
 → data classification/egress
 → project trust
 → user/org rules
 → one-time approval / reauth
 → issue lease
```

### 8.3 Hard deny

以下情况不可通过用户“总是允许”覆盖：

- Credential 已 revoked/expired；
- 目标不在组织禁止清单；
- Extension revision 不可信或签名撤销；
- 工具尝试把 Secret 写入普通文件、Prompt 或 Event；
- 外发数据 classification 超过 policy ceiling；
- 注入通道会把 Secret 暴露在命令行或公共日志；
- 目的地无法建立身份或网络边界；
- 请求缺少 operation、purpose 或 audit context；
- 项目未授信却要求读取项目外凭据。

### 8.4 Approval 视图

审批界面只显示服务端生成的安全摘要：

```text
将使用 credential “github-work” 的当前版本
通过 MCP server “github”
调用 “create_issue”
目的地：api.github.com
数据：repository_content（已脱敏）
项目：apex
有效期：本次操作
```

不显示：

- Secret 原文、完整 token、Cookie；
- 未授权的完整 Prompt/文件内容；
- 可重放的认证 header；
- 未经过 redaction 的第三方错误文本。

审批决定绑定 `approval_summary_digest`、tool/schema revision、credential version、destination policy revision 和 data classification policy revision。

### 8.5 Re-authentication

高风险操作需要 freshness proof：

- 添加或导出 Credential；
- 使用生产、支付或删除类凭据；
- 访问 SSH private key 或 TLS private key；
- 将 sensitive/regulated 数据发送到新目的地；
- 禁用 redaction 或执行 privacy purge；
- 修改组织级 Credential policy。

重新认证由客户端触发，Core 只接收短期 proof，不接收密码或生物特征原文。

### 8.6 Saved rule

Saved permission rule 必须绑定：

- credential_id 或 kind；
- tool matcher；
- destination matcher；
- data classification ceiling；
- project/session scope；
- extension revision（若来自 Plugin/MCP）；
- TTL/expiry；
- policy revision。

“允许使用 GitHub token”不能自动涵盖任意 host、任意工具和任意数据。

---

## 9. Destination 与 Data Egress 治理

### 9.1 Destination Identity

```rust
pub struct DestinationIdentity {
    pub kind: DestinationKind,
    pub canonical_host: Option<String>,
    pub port: Option<u16>,
    pub scheme: Option<String>,
    pub server_id: Option<McpServerId>,
    pub provider_id: Option<ProviderId>,
    pub account_hint: Option<SafeText>,
    pub policy_revision: PolicyRevision,
}
```

同一个显示名称不等于同一个目的地。MCP server 使用 `server_id + config_digest + endpoint`，Provider 使用 adapter/provider/account 组合，Shell 使用命令和进程边界。

### 9.2 Destination canonicalization

- host 小写化、国际化域名规范化；
- 默认端口显式化；
- URL 去除 fragment，敏感 query 参数不进入摘要；
- IP、DNS、代理和重定向分别校验；
- MCP stdio destination 是本地 command digest + executable identity；
- 重定向到新 host 触发重新判权；
- `localhost`、loopback、link-local、private network 依然需要网络策略。

### 9.3 Data Egress Policy

策略输入包括：

```text
source project/workspace/session
source paths/artifacts
data labels and lineage
redaction profile/result
destination identity
credential identity/version
extension/tool revision
user/org policy
purpose and retention
```

策略输出：

```rust
pub enum EgressDecision {
    AllowRedacted,
    AllowWithApproval,
    Deny,
    RequireSanitization,
    RequireDestinationTrust,
    UnknownNeedsReview,
}
```

### 9.4 外发分类

| 外发等级 | 允许条件 |
|---|---|
| `public` | 可按网络和工具规则自动放行 |
| `internal` | 目的地属于项目/组织 allowlist，可能需要审批 |
| `sensitive` | 必须 redaction、目的地信任和明确 purpose；默认 ask |
| `secret` | 默认 deny；只允许协议级 Credential injection，不允许作为普通数据发送 |
| `regulated` | 按组织特定策略，默认 deny 或强制审批 |

即使数据经过脱敏，也保留 `redacted_from=sensitive` lineage，避免下游把它误认为 public。

### 9.5 Redaction 不是授权

Redaction 成功只能证明某些模式被遮盖，不代表：

- 所有敏感信息都已消除；
- 语义重识别风险为零；
- 目的地可信；
- 发送操作已获授权。

Egress policy 必须同时检查 redaction result、classification、destination、purpose 和 approval。

### 9.6 重定向与批量外发

- HTTP 3xx、MCP proxy、Provider routing 产生新目的地时重新判权；
- 批量文件/目录外发按合并后的最高 classification；
- 无法列举实际文件时不能用目录名称替代数据范围；
- 大型 payload 使用 BlobRef，但判权基于 Blob manifest/labels，不是因为引用化就降低敏感级别。

---

## 10. Redaction 与 Secret Scanner

### 10.1 统一 Redaction Pipeline

所有可能进入非 Secret 存储或输出的内容经过：

```text
raw bytes
 → encoding/size validation
 → known secret exact match
 → structured field rules
 → provider/MCP patterns
 → private key/cookie/token detectors
 → entropy heuristic (辅助)
 → path/content classification
 → replacement + lineage
 → bounded output
```

覆盖路径：

- Provider request/response；
- Tool arguments、stdout、stderr；
- MCP resource/tool result；
- Hook input/output；
- Diagnostic、Event、Error、trace；
- Export、backup manifest、support bundle；
- Panel projection 和浏览器下载。

### 10.2 Redaction replacement

推荐稳定替换格式：

```text
<APEX_REDACTED kind=api_key id=red_... fingerprint=fp_...>
```

对模型上下文可使用更短的 `<redacted:credential>`；对审计和诊断保留不可逆 redaction id、规则 revision、命中类别和 lineage。替换文本不能模拟 JSON control field、system message 或 permission response。

### 10.3 Known secret registry

Broker 在 lease 建立时向 scanner 注册短期匹配材料：

- Secret 原文不出 Broker/Scanner 受控内存；
- Scanner 可使用 keyed matcher 或安全进程隔离；
- 注册有 expiry 和 operation scope；
- operation 终结后移除；
- 只返回命中类别、位置摘要和 redaction id。

若安全实现暂时无法避免 scanner 看到原文，必须将 scanner 置于独立受限 worker，并禁止持久化、网络和日志。

### 10.4 模式检测

检测器包括：

- Provider 特定前缀和长度；
- JWT header/payload 结构；
- PEM private key/certificate；
- Authorization、Cookie、Set-Cookie 等结构化 header；
- cloud credential 文件字段；
- database URL 中的 password；
- Slack/GitHub/OpenAI 等已知 token 格式；
- 高熵字符串和近似编码变体。

模式匹配只能产生候选标签；真正的 known secret match 由 CredentialStore/Broker 提供的 fingerprint/安全 matcher 加强。

### 10.5 False positive 与 false negative

- false positive：允许用户标记为非 Secret，但只在精确字段/operation scope 生效；
- false negative：任何无法完成 scanner 的结果不得视为 clean；
- scanner 版本更新后可重扫 Blob metadata 和 active outputs；
- 误报率/漏报率指标只记录统计，不记录原始值；
- 对 private key、token、Cookie 使用更严格的 fail closed。

### 10.6 输出处理顺序

```text
Adapter output
 → process-level bounded capture
 → secret redaction
 → schema normalization
 → taint classification
 → sensitive Blob commit
 → safe summary projection
```

即使 Adapter 声称“已脱敏”，Core 仍执行自己的 scanner。ToolResult 成功与 redaction/Policy violation 分离：副作用可能已发生，但 Run 进入 `succeeded_with_violations` 或 `reconcile_required`。

---

## 11. Taint、Lineage 与传播规则

### 11.1 Taint 标签

```rust
pub enum TaintSource {
    Repository,
    UserInput,
    ModelGenerated,
    ShellOutput,
    Web,
    Mcp,
    Skill,
    Plugin,
    CredentialDerived,
    Redacted,
}

pub struct TaintLabel {
    pub source: TaintSource,
    pub source_id: String,
    pub untrusted: bool,
    pub data_class: DataClassification,
    pub created_at_us: i64,
}
```

### 11.2 传播规则

| 传播动作 | 结果 |
|---|---|
| sensitive + public | 至少 sensitive，除非经过受控 redaction |
| secret + 任意内容 | 不得自动拼成普通内容 |
| MCP/Skill/Plugin 文本进入 Context | 保留外部/扩展 taint |
| 外部文本生成 ToolCall | ToolCall 保存 provenance，重新判权 |
| redaction 后 sensitive | 可降低发送 payload 分类，但保留 lineage |
| Secret 派生摘要 | 默认 `derived_sensitive`，不得当作 public |
| Gate receipt 使用 tainted input | Gate 记录 source/revision，不能自动 pass |

### 11.3 Control Plane 禁止升级

以下数据永远不能仅凭内容自身改变：

- system contract；
- ProjectTrust；
- Capability grant；
- PermissionDecision；
- Ruleset/Gate verdict；
- Credential status；
- Reconcile result。

如果需要将外部结果纳入控制面，必须经过 Core schema validator、来源校验、策略检查和明确的 commit command。

### 11.4 Lineage

Lineage 采用有界 DAG 引用：

```text
user message
  → model proposal
  → selected files
  → redaction result
  → MCP request
  → MCP response
  → generated issue
```

只保存引用、摘要、digest、classification 和 operation relation。超过深度/节点预算时合并为 lineage checkpoint，不能把原始敏感内容复制到 lineage。

### 11.5 Context 与 Memory

- Secret 永远不得写入 Memory；
- sensitive Context 默认进入 volatile suffix 或授权 Blob；
- 自动 Memory 创建前必须 redaction + secret scan；
- Checkpoint 保存内容引用和 redaction metadata，不保存 Secret；
- MCP/Plugin 内容进入 Context 时标记 taint；
- compaction/summary 也必须重新扫描，不能因为是模型摘要就当作安全。

---

## 12. Provider Credential 治理

### 12.1 Provider 配置

Provider profile 只保存：

- provider/adapter id；
- model defaults；
- endpoint 的安全摘要；
- credential_id/version policy；
- organization/project/account hint；
- timeout、retry、cost policy；
- allowed data classification；
- proxy/network policy。

API key、OAuth secret、client secret 和完整 Authorization header 只存 CredentialStore。

### 12.2 LLM 请求边界

Provider request 在发送前必须有：

```text
provider.invoke.v1
∩ credential.use.v1 (若 provider 需要认证)
∩ network.connect.v1
∩ data.egress.v1
∩ model/provider allowlist
∩ context data policy
```

LLM Provider 通常接收用户 Prompt、项目内容或工具结果，因此 data classification 不应默认为 public。Provider profile 必须声明数据处理级别和组织允许范围。

### 12.3 Provider 不得返回 Credential

Adapter 只能返回：

- response text/content；
- usage；
- provider request id；
- rate/limit metadata；
- normalized error code；
- safe headers subset。

不得返回请求 headers、CredentialRef 之外的认证内容、完整 URL query 或原始 SDK exception。

### 12.4 OAuth 生命周期

```text
authorization_start
 → user/browser approval
 → code exchange in callback boundary
 → store access/refresh versions
 → use access token lease
 → refresh before expiry
 → rotate version lineage
 → revoke/logout
```

浏览器回调：

- state/PKCE 由 Core 生成并短时存储；
- callback 只接受匹配 state、redirect origin 和 session；
- code 不进入 URL 日志、事件或 WebView history；
- token exchange 在 Core/受信 Provider adapter 完成；
- WebView 不读取 keyring 或 daemon 配置。

### 12.5 多 Provider 与账户切换

同一 provider 可以有多个逻辑 Credential，例如个人、组织、只读机器人账户。选择必须显式显示 account hint、scope、expiry 和目的地；不能仅按“最近使用”隐式切换到高权限账户。

### 12.6 Token 续传与流断线

Provider 流断线重试时：

- 保持相同 operation_id；
- Credential lease 可续期但不能跨 policy expiry；
- 重新发送完整 Prompt 前重新执行 egress policy；
- 已接收内容保留 redaction/taint；
- 无法确认 Provider 是否处理请求时按 unknown，不能把重新发送视为无副作用。

---

## 13. MCP、Network 与 Credential 组合

### 13.1 MCP config/token 分离

MCP 配置只保存 Credential 引用，不保存完整认证值。规范化结构示例：

```json
{
  "server": "github",
  "transport": "http",
  "url": "https://api.github.com/mcp",
  "headers": {
    "Authorization": {
      "credential_ref": "cred_github_work",
      "format": "Bearer"
    }
  }
}
```

兼容导入器遇到带有明文 token 的配置时：

1. 不把原配置原样写入 SQLite 或普通日志；
2. 将候选 Secret 导入 staging credential flow；
3. 成功存入 CredentialStore 后重写为 ref；
4. 原文件是否清除由用户确认和文件策略决定；
5. 生成 `MCP_CONFIG_SECRET_FOUND` 安全诊断。

### 13.2 MCP server scope

MCP Credential grant 绑定：

- `mcp_server_id`；
- config/revision digest；
- endpoint/server identity；
- allowed tool matcher；
- data egress classification；
- extension/plugin revision（若由 Plugin 提供）；
- operation purpose。

同一 Credential 不能因为两个 MCP server 都叫 `github` 就互相复用。

### 13.3 HTTP header 注入

- header 名称必须由 adapter schema allowlist 控制；
- Authorization/Cookie/Proxy-Authorization 不允许通过普通 Tool arguments 传入；
- 注入前校验 destination；
- 重定向、代理和重连变更时重新校验；
- 错误和 trace 只显示 header 名，不显示值；
- response 中的 Set-Cookie 默认 secret，不能进入 ToolResult。

### 13.4 MCP 结果处理

MCP description/resource/tool result：

- 视为 `tainted_external`；
- 经过大小、schema、redaction 和 prompt-injection boundary；
- 不得直接写入 CredentialStore、Permission Rule 或 Gate；
- 若包含 token，Result Normalizer 必须将其隐藏并产生诊断；
- 外部结果触发新 ToolCall 时保存 lineage 和 source server/schema revision。

### 13.5 Network destination policy

DNS、代理和 TLS 证书验证属于 Network Policy；Credential Policy 只负责“是否允许该 Credential 绑定该目的地”。两者都通过后才允许 Broker 发放 lease。

### 13.6 外部副作用与对账

MCP/HTTP 发送使用 Credential 后，如果状态未知：

- lease 不因请求超时立即视为未使用；
- Operation 进入 `reconcile_required`；
- 只有对账完成或人工处置后才能 revoke/重试；
- Provider/MCP 不支持对账时记录 `reconcile_unsupported`；
- 审计中区分 credential 已注入、请求已发送、结果已确认和业务状态已确认。

---

## 14. Shell、Git、Browser 与本地工具

### 14.1 Shell

Credential-aware Shell 的安全边界：

- 不把 Secret 拼入 command string；
- 使用 stdin/环境变量/临时文件时记录 injection plan；
- 命令解析器检查 `env`, `printenv`, `set`, `history`, `ps`, 重定向、管道和子进程传播；
- 默认禁止把 Credential 注入到任意 shell；
- 只允许声明的 executable、cwd、args shape 和 destination；
- shell 输出经过 scanner，防止命令回显 Secret。

### 14.2 Git

Git Credential 使用分为：

- remote fetch/push authentication；
- commit signing；
- SSH key；
- HTTPS token；
- credential helper。

Apex 优先使用 Git 原生 credential helper/SSH agent 的受控接口，不读取 private key 原文。Push/remote 修改还需 Git/Network/Workspace 权限，不能因 Credential 可用而自动允许。

### 14.3 Browser

- Cookie/session token 不进入普通 Browser ToolResult；
- WebView 不能把 cookie jar 暴露给 Agent/Plugin；
- 页面内容是 external/sensitive，仍需 egress policy；
- 下载文件和上传文件分别判权；
- URL query、referrer、crash report 进行 Secret scan；
- 浏览器缓存、session storage 和 WebView cache 使用 no-store/受控清理策略。

### 14.4 Cloud CLI

Cloud CLI 的 credential helper 必须声明：

- provider/account；
- executable；
- allowed service/action；
- region/host；
- whether it may create/delete/modify resources；
- reconcile method；
- output redaction profile。

未知 CLI 默认按高风险 process tool，不自动继承当前用户 shell 环境中的所有 Credential。

### 14.5 本地 Secret 文件

直接读取 `.env`、private key 或 cloud credentials 文件：

- 创建独立 sensitive read ToolCall；
- 默认不进入模型上下文；
- 如仅为配置解析，优先使用结构化 parser 并返回字段状态；
- Secret 字段变为 CredentialRef；
- 原文件内容不写 Memory、Checkpoint、日志或外发 Blob。

---

## 15. Plugin、Skill、Hook 的敏感数据边界

### 15.1 Plugin capability

Plugin 只能获得 manifest 和 grant 明确授予的 capability：

```text
credential.discover_metadata.v1
credential.use.v1
network.connect.v1
data.egress.v1
secret.scan.v1
```

`credential.discover_metadata.v1` 只能读取名称、kind、状态、expiry、scope 摘要，不能读取 Secret。`credential.use.v1` 只能通过 Host API 请求 Broker lease。

### 15.2 Wasm Plugin

Wasm Plugin：

- 默认无 keyring、文件、网络和环境变量访问；
- Host API 不提供 Secret bytes；
- 若确需使用认证，Host 注入 opaque operation handle；
- 所有网络/工具操作回到 Tool Gateway；
- Wasm memory 在 instance 终止后按 runtime 能力清理；
- 输出仍经 scanner 和 taint pipeline。

### 15.3 Supervised subprocess Plugin

进程 Plugin：

- 环境变量按 allowlist 注入；
- stdin/pipe 优先于 argv；
- process dump、crash dump 和 debug logging 默认关闭；
- 子进程不得访问 CredentialStore socket；
- supervisor 清理临时目录和环境 staging；
- 进程输出不能直接广播给 UI。

### 15.4 Skill

Skill 只能声明 `requires-tools` 或 `requires-credentials` 作为请求；

- body 不含真实 Secret；
- resources 不得包含未加密 credential file；
- script 使用 Credential 必须通过 Tool Gateway；
- model invocation 不自动取得 Credential；
- Skill description 中的“需要 token”不能触发自动授权。

### 15.5 Hook

Hook input 默认只含安全摘要、引用和 classification。Hook：

- 不能读取完整 Secret；
- 不能改变 Credential grant；
- 可针对即将发生的 credential use 返回 deny/ask/diagnostic；
- 参数 rewrite 不能把 CredentialRef 改成另一个 Credential；
- PostToolUse 发现疑似泄漏时可 block completion，但不能修改历史原文。

### 15.6 MCP Server

MCP server 只得到为其配置的 Credential 注入；不提供 Apex 全局凭据枚举 API。MCP server 返回的文本永远不能要求 Apex 再提供新的 Credential，除非产生新 Command 并重新授权。

---

## 16. Secret 生命周期：轮换、过期与撤销

### 16.1 Rotation 模型

```text
active(v1)
  → create v2
  → validate v2
  → dual-valid overlap
  → switch active pointer to v2
  → drain leases using v1
  → revoke v1
  → retain metadata lineage
```

切换 active pointer 是数据库/metadata 原子操作；正在运行的 Operation 是否完成由 lease policy 决定，不能在 HTTP 请求中间无条件换 token。

### 16.2 轮换来源

- 用户手动 rotate；
- Provider OAuth refresh；
- 到期前自动提醒；
- 组织策略强制 rotate；
- 泄漏响应紧急 rotate；
- Provider 返回 invalid/expired；
- Plugin/MCP config 更新要求新 credential version。

### 16.3 Overlap

有些服务需要旧/新 token 短时同时有效。Overlap 必须：

- 有明确 start/end；
- 限定 destination 和 tool；
- 不延长旧 lease 的无界寿命；
- UI 显示两个版本状态；
- 到期自动 revoke 旧版本；
- crash 后由 startup reconciler 重新计算。

### 16.4 Expiry

Expiry 分为：

- provider declared expiry；
- CredentialStore expiry；
- policy expiry；
- lease expiry；
- approval expiry。

有效期限取最早值。expiry 时间不应依赖模型或扩展自行计算；采用 Core canonical clock，并处理时钟回拨/漂移诊断。

### 16.5 Revocation

撤销来源：用户、管理员、Provider webhook、泄漏 scanner、签名/组织策略、异常使用检测。撤销流程：

1. 原子更新 credential/version state；
2. 增加 revocation epoch；
3. 阻止新 Broker lease；
4. 通知 active adapters/processes；
5. 尝试取消可取消 Operation；
6. 对未知外部操作进入 reconcile；
7. 创建审计事件和 incident reference。

### 16.6 泄漏响应

疑似泄漏证据包括：

- Secret 命中日志/输出；
- 外发到未授权目的地；
- 进程 argv、crash dump 或文件发现 Secret；
- Plugin/MCP protocol violation；
- 用户报告或 Provider 告警。

默认响应等级：

```text
observe → warn → restrict → revoke_pending → revoked
```

生产 Credential、private key 和 signing secret 可直接进入 revoke_pending，需要用户/组织确认的恢复动作不能延迟阻断新使用。

### 16.7 删除与销毁

逻辑删除不等于物理销毁：

- Credential metadata 保留审计 lineage；
- Secret material 从 backend 删除/撤销；
- 文件后端执行 secure replacement + key destruction（受 OS 能力限制）；
- 备份/缓存/临时目录按 retention policy 清除；
- 不承诺第三方 Provider 已撤销，需保存 revoke request 状态；
- destroy operation 本身不包含原 Secret。

---

## 17. Credential 健康、测试与可用性

### 17.1 健康状态

```text
unknown → configured → verified → healthy
healthy → expiring → refresh_pending → healthy
healthy → provider_rejected → needs_reauth
healthy → store_unavailable → degraded
healthy → revoked/expired/quarantined
```

健康只表示最近验证结果，不代表当前 Operation 自动被授权。

### 17.2 不在启动时批量验证

Apex 启动时只读取 metadata/状态，不批量调用 Provider 验证所有 Credential，以避免：

- 无意外发起网络请求；
- 触发 rate limit；
- 使用已撤销但尚未清理的 token；
- 把用户 offline 启动变成强制在线。

需要时执行最小范围 `ValidateCredential` Command。

### 17.3 使用前检查

Broker 发放 lease 前检查：

- Store available；
- version active；
- expiry 足够覆盖 deadline；
- scope 与 tool/destination 匹配；
- policy revision 未过期；
- provider/account 状态非 blocked；
- reauth freshness 满足；
- data egress 判定允许。

### 17.4 失败分类

```text
CREDENTIAL_NOT_FOUND
CREDENTIAL_STORE_UNAVAILABLE
CREDENTIAL_REVOKED
CREDENTIAL_EXPIRED
CREDENTIAL_SCOPE_DENIED
CREDENTIAL_DESTINATION_DENIED
CREDENTIAL_REAUTH_REQUIRED
CREDENTIAL_INJECTION_UNSUPPORTED
CREDENTIAL_LEASE_EXPIRED
CREDENTIAL_LEAK_SUSPECTED
CREDENTIAL_PROVIDER_REJECTED
```

错误响应只能返回安全摘要和用户动作，不包含底层 Secret 或完整 HTTP authorization response。

### 17.5 离线模式

Offline 模式允许：

- 查看 credential metadata；
- 使用不需 Secret 的本地工具；
- 读取已缓存的非 Secret Provider metadata；
- 生成待执行操作。

Offline 模式不允许新的远程 Credential use；已有本地 lease 到期后不续租。用户可显式切回 online 并重新进行 network policy 判断。

---

## 18. 审计、日志与可观测性

### 18.1 Credential 使用审计

每次 Credential use 记录：

- credential_id；
- credential_version_id/fingerprint；
- principal/actor/client；
- project/session/run/operation/tool；
- extension/MCP/provider revision；
- destination identity；
- purpose；
- data labels；
- approval decision/summary digest；
- lease created/used/revoked timestamps；
- result status、redaction summary、reconcile status；
- policy revisions；
- error code。

不记录：

- Secret 原文；
- 完整 Authorization/Cookie header；
- 未脱敏的命令行、Prompt、文件内容；
- 可重放的 refresh code。

### 18.2 安全日志等级

| 事件 | 默认等级 | 内容 |
|---|---|---|
| Credential metadata read | internal | id、kind、scope、actor |
| Lease issued | sensitive | id、operation、destination 摘要 |
| Secret injected | sensitive | channel、process/adapter identity，不含值 |
| Secret scanner hit | high | category、location 摘要、redaction id |
| Leak suspected | high | credential/version、incident、action |
| Rotate/revoke | high | reason、old/new version fingerprint |
| Store failure | internal/high | backend code、恢复建议 |

### 18.3 Metrics

只提供聚合指标：

- active credential count（按 scope/状态粗粒度）；
- lease issuance success/failure；
- store latency；
- rotation/expiry/revocation count；
- redaction hit count/category；
- egress allow/ask/deny；
- unknown/reconcile count；
- provider unauthorized rate；
- scanner false positive review count。

不按 credential value、token prefix 或具体敏感文件内容做 telemetry 标签。

### 18.4 UI 面板

Credential Panel 展示：

- logical name、kind、scope、provider、状态；
- version fingerprint 后缀、expiry、last used（粗粒度）；
- allowed tools/destinations；
- rotation/revoke/reauth 操作；
- 最近使用摘要；
- store backend health；
- 泄漏告警和需要处理的 incident。

UI 不显示 Secret 值、完整 Cookie、private key、refresh token 或从 Provider 返回的原始认证错误。

### 18.5 Trace 传播

`trace_id`、`operation_id`、`tool_call_id` 可以传播；以下字段禁止进入 trace baggage：

- Prompt；
- 源码/文件内容；
- token、cookie、credential；
- 完整 URL query；
- Secret scanner 原文命中；
- 用户密码或 reauth proof。

---

## 19. SQLite、Blob、备份与恢复

### 19.1 SQLite 权威关系

CredentialStore 是 Secret material 权威；SQLite 只保存：

- Credential definition/version metadata；
- provider/scope/status/fingerprint；
- policy/grant/lease/rotation lineage；
- audit/event/projection；
- redaction/taint/lineage reference。

SQLite 不保存：

- Secret bytes；
- keyring export；
- OAuth authorization code；
- refresh token；
- private key；
- full cookie jar。

### 19.2 推荐表

```sql
CREATE TABLE credentials (
    credential_id TEXT PRIMARY KEY,
    owner_type TEXT NOT NULL,
    owner_id TEXT NOT NULL,
    logical_name TEXT NOT NULL,
    kind TEXT NOT NULL,
    provider_id TEXT,
    scope_json TEXT NOT NULL CHECK(json_valid(scope_json)),
    state TEXT NOT NULL,
    active_version_id TEXT,
    policy_revision TEXT NOT NULL,
    created_at_us INTEGER NOT NULL,
    updated_at_us INTEGER NOT NULL,
    UNIQUE(owner_type, owner_id, logical_name)
);

CREATE TABLE credential_versions (
    credential_version_id TEXT PRIMARY KEY,
    credential_id TEXT NOT NULL REFERENCES credentials(credential_id),
    backend_kind TEXT NOT NULL,
    backend_locator TEXT NOT NULL,
    fingerprint TEXT NOT NULL,
    version_number INTEGER NOT NULL,
    valid_from_us INTEGER,
    expires_at_us INTEGER,
    state TEXT NOT NULL,
    rotation_parent_id TEXT REFERENCES credential_versions(credential_version_id),
    source TEXT NOT NULL,
    created_at_us INTEGER NOT NULL,
    revoked_at_us INTEGER,
    UNIQUE(credential_id, version_number),
    UNIQUE(credential_id, fingerprint)
);
```

`backend_locator` 只能是 keyring/Vault 的 opaque locator，不得包含文件中的明文值。fingerprint 不是 Secret 的替代备份。

### 19.3 Grants、leases 与使用

```sql
CREATE TABLE credential_grants (
    grant_id TEXT PRIMARY KEY,
    credential_id TEXT NOT NULL REFERENCES credentials(credential_id),
    subject_type TEXT NOT NULL,
    subject_id TEXT NOT NULL,
    tool_matcher_json TEXT NOT NULL CHECK(json_valid(tool_matcher_json)),
    destination_matcher_json TEXT NOT NULL CHECK(json_valid(destination_matcher_json)),
    data_classification_ceiling TEXT NOT NULL,
    purpose TEXT NOT NULL,
    conditions_json TEXT NOT NULL CHECK(json_valid(conditions_json)),
    state TEXT NOT NULL,
    expires_at_us INTEGER,
    created_at_us INTEGER NOT NULL,
    revoked_at_us INTEGER
);

CREATE TABLE credential_leases (
    lease_id TEXT PRIMARY KEY,
    credential_version_id TEXT NOT NULL REFERENCES credential_versions(credential_version_id),
    operation_id TEXT NOT NULL REFERENCES operation_journal(operation_id),
    tool_call_id TEXT,
    destination_json TEXT CHECK(json_valid(destination_json)),
    injection_channel TEXT NOT NULL,
    state TEXT NOT NULL,
    revocation_epoch INTEGER NOT NULL,
    issued_at_us INTEGER NOT NULL,
    expires_at_us INTEGER NOT NULL,
    revoked_at_us INTEGER,
    cleanup_at_us INTEGER
);

CREATE TABLE credential_usages (
    usage_id TEXT PRIMARY KEY,
    lease_id TEXT NOT NULL REFERENCES credential_leases(lease_id),
    credential_id TEXT NOT NULL,
    credential_version_id TEXT NOT NULL,
    operation_id TEXT NOT NULL,
    destination_digest TEXT,
    purpose TEXT NOT NULL,
    data_labels_json TEXT NOT NULL CHECK(json_valid(data_labels_json)),
    approval_summary_digest TEXT,
    redaction_summary_json TEXT CHECK(json_valid(redaction_summary_json)),
    result_state TEXT NOT NULL,
    created_at_us INTEGER NOT NULL
);
```

### 19.4 Redaction/lineage metadata

```sql
CREATE TABLE redaction_records (
    redaction_id TEXT PRIMARY KEY,
    operation_id TEXT,
    source_ref TEXT NOT NULL,
    profile_id TEXT NOT NULL,
    scanner_revision TEXT NOT NULL,
    matched_categories_json TEXT NOT NULL CHECK(json_valid(matched_categories_json)),
    replacement_count INTEGER NOT NULL,
    source_digest TEXT NOT NULL,
    output_digest TEXT NOT NULL,
    created_at_us INTEGER NOT NULL
);

CREATE TABLE data_lineage_edges (
    edge_id TEXT PRIMARY KEY,
    operation_id TEXT,
    parent_ref TEXT NOT NULL,
    child_ref TEXT NOT NULL,
    relation TEXT NOT NULL,
    labels_json TEXT NOT NULL CHECK(json_valid(labels_json)),
    created_at_us INTEGER NOT NULL
);
```

### 19.5 Blob policy

Blob 可以保存 sensitive 内容，但必须：

- 在 commit 前执行 secret scanner；
- 保存 classification、taint、purpose、retention；
- 使用 BlobRef 而不是原文进入 Event/Prompt；
- 下载需要 capability 和 purpose；
- 浏览器下载 `Cache-Control: no-store`；
- 不保存 Provider key、cookie、CredentialStore 内容；
- 高敏 Blob 加密/隔离策略由 deployment policy 决定。

### 19.6 备份

默认备份包括 Credential metadata 和审计，但不包含 SecretStore material。若用户/组织请求迁移 Credential：

- 使用显式 encrypted export Command；
- 目标设备/公钥/有效期明确；
- export 包不进入普通 Blob、事件或日志；
- 导出后立即记录一次性使用/撤销状态；
- import 设备重新建立 Credential version，不复制旧 grant；
- backup manifest 不保存解密密钥。

### 19.7 启动恢复

```text
open DB
 → migrate credential metadata
 → detect unfinished rotate/revoke/lease intents
 → query CredentialStore health
 → mark leases with unknown cleanup
 → reconcile active processes/adapters
 → rebuild redaction/scanner projections
 → publish CredentialRecoveryCompleted
```

恢复不得自动读取并广播 Secret。对未完成 lease，只能依据 supervisor/adapter evidence 标记 cleanup、unknown 或 revoke_pending。

---

## 20. 隐私保留、导出与清除

### 20.1 Retention 分类

| 数据 | 默认策略 |
|---|---|
| Credential metadata | 长期保留，支持审计和 lineage |
| Secret material | 由 CredentialStore/Provider 生命周期控制 |
| Lease metadata | 按审计政策保留，不含原文 |
| Redaction record | 长期或合规要求保留 |
| Sensitive Blob | TTL + 引用计数 + 用户 pin |
| Tool/Provider raw payload | 仅按项目 policy 保留 |
| Trace/log | 最小化、短期、脱敏 |
| Incident evidence | 按安全/合规策略保留 |

策略变更不改变已经发生的事件含义；GC operation 必须记录当时采用的 retention policy revision。

### 20.2 Privacy purge

用户请求删除敏感内容时使用独立受控 Command：

1. 创建 `PrivacyPurgeOperation` 和范围快照；
2. 校验 Actor、项目边界、备份和组织保留要求；
3. 将可删除内容替换为不可逆 redaction marker；
4. 删除/重加密 Blob、缓存、FTS 和临时文件；
5. 对 CredentialStore 执行 revoke/destroy 或仅清除本地引用；
6. 追加 `PrivacyPurged` 补偿事件；
7. 重建受影响 projection、lineage 和面板；
8. 生成清除报告，列出保留原因和不可恢复范围。

### 20.3 不可承诺的物理清除

普通数据库 DELETE、操作系统文件删除、进程内存清零和第三方 Provider 删除都不等价于密码学擦除。Apex 必须明确区分：

- logical revoked；
- local reference removed；
- backend material destroyed；
- provider revoke requested；
- provider revoke confirmed；
- backup/cache cleanup completed。

### 20.4 导出安全视图

用户导出项目或 Session 时：

- 默认排除 Credential material；
- sensitive 内容按 export profile redaction；
- event 保留 id/seq/type/digest lineage；
- Credential 使用记录显示 logical name/version fingerprint，不显示值；
- MCP headers、cookies、Provider raw errors 都脱敏；
- 导出包写入临时受保护目录，完成后清理；
- 分享给 support 前强制执行二次 scanner。

---

## 21. 事件、API 与实时协议

### 21.1 Command 服务

```proto
service CredentialCommandService {
  rpc DiscoverCredentialSources(DiscoverCredentialSourcesRequest) returns (CommandResponse);
  rpc ImportCredential(ImportCredentialRequest) returns (CommandResponse);
  rpc ValidateCredential(ValidateCredentialRequest) returns (CommandResponse);
  rpc RotateCredential(RotateCredentialRequest) returns (CommandResponse);
  rpc RevokeCredential(RevokeCredentialRequest) returns (CommandResponse);
  rpc DeleteCredential(DeleteCredentialRequest) returns (CommandResponse);
  rpc CreateCredentialGrant(CreateCredentialGrantRequest) returns (CommandResponse);
  rpc RevokeCredentialGrant(RevokeCredentialGrantRequest) returns (CommandResponse);
  rpc ReauthenticateCredential(ReauthenticateCredentialRequest) returns (CommandResponse);
  rpc PurgeSensitiveData(PurgeSensitiveDataRequest) returns (CommandResponse);
}
```

Command body 只接收安全输入引用或受控 secret input channel，不接收任意 JSON 字段中的 Secret。所有命令带 `CommandMeta.idempotency_key`。

### 21.2 Query 服务

```proto
service CredentialQueryService {
  rpc ListCredentials(ListCredentialsRequest) returns (CredentialList);
  rpc GetCredential(GetCredentialRequest) returns (CredentialView);
  rpc ListCredentialUsages(ListCredentialUsagesRequest) returns (CredentialUsageList);
  rpc GetCredentialHealth(GetCredentialHealthRequest) returns (CredentialHealthView);
  rpc GetRedactionProfile(GetRedactionProfileRequest) returns (RedactionProfileView);
  rpc GetEgressDecisionSummary(GetEgressDecisionSummaryRequest) returns (EgressDecisionView);
}
```

Query 返回 scope-filtered metadata；不提供“下载 Secret”的普通 Query 方法。

### 21.3 REST 映射

| Method | Path | 用途 |
|---|---|---|
| `GET` | `/api/v1/credentials` | 列表 |
| `GET` | `/api/v1/credentials/{id}` | 元数据 |
| `POST` | `/api/v1/credentials:import` | 导入 |
| `POST` | `/api/v1/credentials/{id}:validate` | 验证 |
| `POST` | `/api/v1/credentials/{id}:rotate` | 轮换 |
| `POST` | `/api/v1/credentials/{id}:revoke` | 撤销 |
| `POST` | `/api/v1/credentials/{id}/grants` | 授权 |
| `DELETE` | `/api/v1/credentials/{id}` | 删除逻辑身份 |
| `POST` | `/api/v1/privacy:purge` | 隐私清除 |
| `GET` | `/api/v1/credentials/events` | 实时事件 |

浏览器端不得接触 keyring、Broker socket、SecretHandle 或 file backend key。

### 21.4 实时事件

```text
CredentialDiscovered
CredentialImportStarted
CredentialImported
CredentialValidationStarted
CredentialValidationFinished
CredentialExpiring
CredentialRotationStarted
CredentialVersionActivated
CredentialVersionRevoked
CredentialGrantCreated
CredentialGrantRevoked
CredentialLeaseIssued
CredentialLeaseRevoked
CredentialLeaseCleanupFailed
CredentialUseStarted
CredentialUseFinished
CredentialStoreUnavailable
CredentialLeakSuspected
CredentialQuarantineStarted
CredentialReauthRequired
DataEgressEvaluated
DataEgressDenied
RedactionApplied
SecretScanFailed
PrivacyPurgeStarted
PrivacyPurged
```

事件 payload 只包含 id、kind、scope 摘要、状态、policy revision、fingerprint 后缀、destination digest 和引用。Secret 原文、完整 token、Cookie、private key 和未脱敏内容使用 `secret_prohibited` 规则禁止进入事件。

### 21.5 Event view

原始审计事件与客户端 EventView 分离：

- Core 内部保留最小必要事实；
- UI 根据 principal/capability 生成 view；
- `event_id/global_seq/event_type` 保持一致；
- `redacted_fields[]` 明确说明缺失字段；
- 未授权订阅者不能通过事件顺序、长度或错误文本推断 Secret。

---

## 22. Rust 模块与接口边界

建议新增/调整模块：

```text
crates/
├── apex-credential/
│   ├── domain.rs
│   ├── refs.rs
│   ├── policy.rs
│   ├── broker.rs
│   ├── lease.rs
│   ├── rotation.rs
│   └── revocation.rs
├── apex-credential-store/
│   ├── port.rs
│   ├── windows.rs
│   ├── macos.rs
│   ├── linux.rs
│   ├── file_compat.rs
│   └── vault.rs
├── apex-secret-scan/
│   ├── scanner.rs
│   ├── known_secret.rs
│   ├── patterns.rs
│   ├── redaction.rs
│   └── profiles.rs
├── apex-data-policy/
│   ├── classification.rs
│   ├── destination.rs
│   ├── egress.rs
│   └── lineage.rs
├── apex-tool-gateway/
│   ├── credential_plan.rs
│   ├── preflight.rs
│   └── result.rs
└── apex-storage/
    ├── credential_repo.rs
    ├── lease_repo.rs
    └── redaction_repo.rs
```

### 22.1 Broker Port

```rust
#[async_trait]
pub trait CredentialBrokerPort {
    async fn authorize_and_issue(
        &self,
        request: SecretAccessRequest,
    ) -> Result<CredentialLease, CredentialError>;

    async fn revoke_lease(
        &self,
        lease_id: LeaseId,
        reason: LeaseRevokeReason,
    ) -> Result<(), CredentialError>;

    async fn reconcile_lease(
        &self,
        lease_id: LeaseId,
        evidence: CleanupEvidence,
    ) -> Result<LeaseState, CredentialError>;
}
```

### 22.2 SecretHandle Port

```rust
#[async_trait]
pub trait SecretHandle {
    async fn inject(
        &self,
        plan: InjectionPlan,
        target: InjectionTarget,
    ) -> Result<InjectionReceipt, InjectionError>;

    async fn revoke(&self) -> Result<(), InjectionError>;
}
```

`SecretHandle` 不提供 `read_bytes`。需要将 Secret 交给第三方时，必须通过明确的 InjectionTarget 和 operation-scoped channel。

### 22.3 Data Policy Port

```rust
#[async_trait]
pub trait DataEgressPolicyPort {
    async fn classify(&self, input: ClassificationInput) -> Result<DataLabelSet, PolicyError>;
    async fn evaluate(&self, input: EgressPolicyInput) -> Result<EgressDecision, PolicyError>;
    async fn explain(&self, decision_id: DecisionId) -> Result<EgressExplanation, PolicyError>;
}
```

### 22.4 Actor 边界

- `CredentialRegistryActor`：定义、版本、状态和元数据；
- `CredentialBrokerActor`：lease、注入、撤销、清理；
- `CredentialStoreActor`：后端连接和迁移；
- `SecretScannerActor`：短期 matcher、redaction、scanner worker；
- `DataPolicyActor`：分类、目的地、外发判定；
- `RotationActor`：轮换、overlap、provider refresh；
- `PrivacyActor`：purge、retention 和 export；
- `CredentialProjectionActor`：Panel/Query 视图。

Session Actor、Plugin 和 UI 只能调用 Command/Query/Port，不直接访问上述内部 actor。

---

## 23. 与 Tool Gateway、Rules、Gate、Context 的集成

### 23.1 Tool Gateway

Tool Gateway 在 preflight 阶段生成 `CredentialInjectionPlan`：

```text
ToolDefinition
 → normalize arguments
 → identify credential refs
 → classify data/destination
 → evaluate capability + permission + egress
 → issue Broker lease
 → inject at adapter boundary
 → execute
 → scan/redact result
 → revoke/cleanup lease
```

Tool arguments 中只允许 CredentialRef，不允许 Secret literal。若扫描发现 literal Secret，ToolCall 进入 deny 或 `requires_sanitization`。

### 23.2 Rules

Rules 可以声明：

- 某路径/工具禁止读取 Secret；
- 某 Credential 只能访问指定 destination；
- 某 data class 必须 redaction；
- 某 provider 必须重新认证；
- 某 extension revision 不得使用 Credential；
- 某时间段禁止外发。

Rules 只能进一步收紧；Credential Policy 不会把 Rule 变成 Permission grant。

### 23.3 Verification Gate

Credential 相关 Gate：

- 证明敏感文件未被写入项目；
- 证明外发 payload 已经过指定 redaction profile；
- 证明外部副作用已对账；
- 证明使用的 credential version/extension revision 与审批一致；
- 证明 Secret scanner 完成且无高风险命中。

Gate receipt 记录证据引用和 scanner/policy revision，不包含 Secret。未知/无法扫描应为 inconclusive 或 blocked，不能 pass。

### 23.4 Context/Checkpoint

- Context builder 在加入文件、MCP、Provider、Skill 内容时携带 DataLabel/Taint；
- Secret 不进入 prompt assembly；
- sensitive content 通过 BlobRef/volatile suffix 控制；
- compaction summary 重新做 scan；
- checkpoint 只保存 refs、digest、classification、redaction summary；
- 恢复时重新确认 active policy，不自动恢复 Secret lease。

### 23.5 Extension System

与扩展设计对齐：

- `credential.use.v1` 只通过 Broker；
- `data.egress.v1` 独立判定；
- Skill、Hook、Plugin 不能提升权限；
- MCP schema revision 绑定 credential/destination approval；
- Extension update 或 Credential policy update 会让依赖中的 authorization/egress decision stale。

### 23.6 DAG 与 SubAgent

DAG node 和 SubAgent profile 可声明 `requested_credentials`，但最终：

```text
subagent_credentials ⊆ parent_effective_credentials
node_egress ⊆ workflow_egress_policy
```

并行节点不能共享无界 lease；需要共享时必须绑定同一 operation group、destination、purpose 和 concurrency policy。

---

## 24. 威胁模型与防护

### 24.1 威胁分类

| 威胁 | 示例 | 防线 |
|---|---|---|
| Prompt injection | 外部内容要求打印 token | taint、Core authority、secret prohibited |
| Log leak | token 出现在 stderr/trace | scanner、redaction、bounded capture |
| Process leak | token 出现在 argv/环境 dump | pipe/handle、最小环境、dump policy |
| Config leak | `.mcp.json` 带明文 header | importer、migration、diagnostic |
| Confused deputy | 低信扩展借用高权 Agent Credential | extension binding、operation grant |
| Destination spoofing | DNS/redirect/代理换 host | canonicalization、TLS/network policy |
| Schema bait-and-switch | MCP tool 变更参数后复用审批 | schema revision binding |
| Rotation race | 旧 token 在 revoke 后继续发送 | epoch/lease/reconcile |
| Backup leak | auth/DB backup 包含 Secret | metadata-only backup、encrypted export |
| UI inference | 通过长度/错误推断 token | safe views、constant-ish summaries |
| Scanner bypass | 编码、分片、压缩绕过 | decode/bounded scan/lineage |
| Malicious plugin | 读取 keyring 或外发 | Wasm/process isolation、Host API |
| Memory disclosure | crash dump/core dump | dump policy、process supervisor |
| Supply chain | Plugin 截获 credential | digest/signature/capability binding |

### 24.2 Secret-adjacent 数据

项目配置、私有仓库 URL、用户邮箱、云账户 ID、路径和错误响应可能不是 Secret，但具备敏感性。默认按 `sensitive` 或 `internal` 处理，不因不属于 CredentialStore 而自动 public。

### 24.3 泄漏检测后的动作

scanner 命中 Secret 时：

1. 阻止内容进入普通事件/Prompt/Blob projection；
2. 记录 redaction/incident metadata；
3. 如果已进入外部操作，标记 operation violation/unknown；
4. 对高价值 Credential 触发 revoke_pending；
5. 发送安全告警；
6. 提供轮换、清除、导出审查和对账动作。

### 24.4 隔离级别

组织可按风险选择：

- `standard`：本地 keyring、redaction、普通审批；
- `strict`：禁止文件 backend、强制 reauth、所有 sensitive egress ask；
- `regulated`：外部 Vault、强制审计、禁止 Prompt 中 sensitive、deny 未签名扩展；
- `air_gapped`：禁 network egress，只允许本地 provider/credential adapter。

---

## 25. 测试、仿真与故障注入

### 25.1 单元测试

覆盖：

- DataLabel 合并和最高等级；
- CredentialRef scope/expiry；
- destination canonicalization；
- capability intersection；
- rotation lineage；
- fingerprint 稳定性和不可逆接口；
- redaction pattern、编码变体；
- lease 状态机；
- retention/purge selection；
- migration/backfill。

### 25.2 契约测试

CredentialStore backend 必须通过：

- put/get/revoke/health；
- access denied；
- backend unavailable；
- keychain locked；
- version rotation；
- crash after write；
- no Secret in error/log/metadata；
- clock expiry；
- concurrent lease/revoke；
- migration from `auth.json`。

### 25.3 集成测试

至少覆盖：

1. CredentialRef 可以被 Tool Gateway 解析，但 Secret 不进入 ToolCall JSON；
2. 用户批准 MCP 调用但目的地重定向时重新判权；
3. `credential.use.v1` 允许而 `data.egress.v1` 拒绝时不发出请求；
4. MCP timeout 后 lease 状态与 reconcile 状态一致；
5. Hook 无法将 CredentialRef 改成高权 Credential；
6. Plugin 只能看到 metadata，不能直接访问 store；
7. Skill script 不能通过环境变量继承全局 Secret；
8. Provider stream retry 保留 operation_id，且重新检查 egress；
9. scanner 在 stdout、stderr、JSON、gzip/base64 路径命中；
10. rotation v2 激活后旧 lease 按 overlap 规则结束；
11. revoke 后新 lease 立即拒绝；
12. privacy purge 删除内容但保留最小审计 lineage。

### 25.4 故障注入

- keyring locked/unavailable；
- `auth.json` 权限错误/被替换；
- CredentialStore 写入后 Core 崩溃；
- Broker 发 lease 后 Adapter 崩溃；
- 进程被强杀，无法确认清理；
- provider 401/429/5xx/超时；
- HTTP redirect 到新目的地；
- scanner OOM/timeout；
- 日志/diagnostic Blob 写入失败；
- rotation 在 active pointer 切换前后崩溃；
- revoke 与 network send 并发；
- backup/export 中途取消；
- OS 时间回拨。

### 25.5 安全回归

- Secret 是否进入 SQLite/WAL/Blob/FTS；
- secret 是否出现在 command line、history、env dump；
- redaction 是否可通过 Unicode/URL/base64/JSON escape 绕过；
- UI 是否显示完整 token 或异常原文；
- event subscription 是否发生 side-channel；
- Plugin/MCP 是否能访问未声明 Credential；
- provider retry 是否重复外发；
- stale policy/credential grant 是否被缓存复用。

---

## 26. 性能与容量

### 26.1 目标

| 操作 | 目标 |
|---|---:|
| Credential metadata 查询 | p95 < 20 ms |
| 本地 grant 判定 | p95 < 10 ms |
| Broker lease issuance（热 store） | p95 < 100 ms |
| 小输出 secret scan | p95 < 20 ms |
| 1 MiB 文本 redaction | p95 < 200 ms |
| destination canonicalization | p95 < 5 ms |
| revoke 传播到本地 active adapter | p95 < 1 s |
| startup metadata recovery | p95 < 2 s，不读取所有 Secret |

不包括外部 Provider、keyring 解锁、用户审批和网络延迟。

### 26.2 限制

- 单用户 Credential definitions：1000；
- 单 project active grants：1000；
- 单 operation leases：16；
- 单 lease lifetime：默认 5 min，可按工具收紧；
- 单 scanner input：16 MiB，超出转 Blob streaming scanner；
- 单 redaction output：输入大小 + bounded expansion；
- 单 lineage edge：10000，超出 checkpoint 聚合；
- 单 export 包：由 policy/Blob quota 限制。

### 26.3 缓存

可缓存：

- Credential metadata；
- provider capability/expiry metadata；
- destination policy；
- redaction profile；
- public certificate metadata。

不可缓存为长期值：

- Secret bytes；
- 无 scope 的授权决策；
- revocation 之前的 lease；
- 包含敏感 payload 的普通日志。

任何 credential state、grant、policy、destination 或 scanner revision 变化都要使相关缓存失效。

---

## 27. 交付阶段与迁移路线

> ADR-0001（跨文档一致性审查）：本节原使用 v0.6 / v0.8 两个基线路线图中不存在的档位。现收编为五档基线内的子阶段：原 v0.6 Data Egress 并入 v0.5 后段，原 v0.8 Rotation/Revoke 并入 v0.7 后段。另：Provider API key 治理是 v0.1 双 Provider 的前置条件，故安全基础提前到 v0.1。

### 27.1 v0.1 安全基础

- 数据分类 public/internal/confidential/sensitive/secret_prohibited；
- `.env`、`*.key`、`*.pem`、`credentials*` 等敏感文件保护；
- 统一 redaction 和 secret scanner；
- ToolResult taint/summary；
- Provider API key 的最小可用存取路径（OS keyring 优先）；
- `auth.json` 权限检查和不入日志。

### 27.2 v0.5 CredentialStore + Broker

- OS keyring adapter；
- CredentialRef/CredentialVersion；
- Credential Broker 最后时刻注入；
- Provider API key/OAuth；
- lease、使用审计和基础撤销；
- API/Panel。

### 27.3 v0.5 阶段二：Data Egress

- Destination Identity；
- `credential.use.v1` 与 `data.egress.v1` 分离；
- MCP/Provider/Network 联动；
- sensitive payload redaction；
- schema/policy revision binding；
- unknown/reconcile 集成。

### 27.4 v0.7 阶段一：Extension 安全集成

- Plugin/Skill/Hook/MCP capability binding；
- Wasm/process injection boundary；
- MCP config 明文 token 导入迁移；
- 外部数据 taint/lineage；
- redaction panel 和 incident 基础能力。

### 27.5 v0.7 阶段二：Rotation/Revoke

- OAuth refresh；
- dual-valid overlap；
- Provider revoke；
- 泄漏响应；
- revocation epoch 和 active adapter 通知；
- keyring migration。

### 27.6 v1.0 Privacy/Enterprise

- 外部 Vault/KMS adapter；
- 组织级 data classification policy；
- strict/regulated/air-gapped profile；
- encrypted credential export/import；
- privacy purge、retention、support bundle scanner；
- 完整 audit、incident 和合规报表。

### 27.7 兼容迁移

旧版本可能把 API key 写入 `~/apex/auth.json`。迁移策略：

1. 读取文件权限和结构；
2. 解析候选 Credential metadata；
3. 用户确认导入目标；
4. 写入 OS CredentialStore；
5. 创建新 CredentialVersion；
6. 重写 Provider/MCP 配置为 CredentialRef；
7. 保留旧文件备份的加密/清理状态；
8. 旧引用标记 `legacy_file_backend`，不自动授予新高风险 capability。

---

## 28. ADR：关键架构决策

### ADR-CRED-001：Secret 不进入普通业务存储

**决定**：SQLite、Domain Event、日志、Prompt、Checkpoint、Memory、普通 Blob 不保存 Secret 原文。  
**原因**：降低复制面，避免 WAL、备份、Projection 和 UI 泄漏。  
**替代方案**：数据库加密后存储；被否决为默认方案，因为授权面和恢复面仍过大。

### ADR-CRED-002：OS Credential Store 优先，保留 auth.json 兼容

**决定**：优先 Windows Credential Manager、macOS Keychain、Linux keyring；兼容 `~/apex/auth.json`，逐步迁移。  
**原因**：满足既有需求，同时提高本地保护能力。  
**替代方案**：只使用 auth.json；被否决，因为文件权限、备份和同步风险更高。

### ADR-CRED-003：最后时刻注入

**决定**：Credential 只在 Tool Gateway 已完成判权、即将执行时由 Broker 创建短期 lease 并注入。  
**原因**：避免 Agent Context、工具 schema、Plugin 常驻内存和日志接触 Secret。

### ADR-CRED-004：Credential use 与 data egress 分离

**决定**：`credential.use.v1` 与 `data.egress.v1` 必须独立判定。  
**原因**：认证能力不应自动等于数据外发能力。

### ADR-CRED-005：Redaction 是硬门和持续过程

**决定**：所有非 Secret 输出路径都经过统一 scanner/redaction，Adapter 自称脱敏不构成豁免。  
**原因**：第三方 adapter、MCP、Shell 输出不可信，且可能出现编码/结构化泄漏。

### ADR-CRED-006：未知状态不等于未使用

**决定**：Credential 已注入且外部状态未知时，Operation 进入 reconcile_required；不盲目重试或立即标记未使用。  
**原因**：避免重复副作用和错误审计。

### ADR-CRED-007：Plugin 不可直接获得 Secret bytes

**决定**：Plugin 通过受限 Host API/opaque handle 使用 Credential，禁止直接读取 keyring。  
**原因**：保护 Core 和其他 Credential，支持 Wasm/process 两类后端。

### ADR-CRED-008：用户可见的是安全摘要

**决定**：审批、Panel、Error 和 Event 只提供 Credential/目的地/数据类别摘要。  
**原因**：帮助用户做真实决策，同时避免展示可重放认证信息。

---

## 29. 实现前审查清单

### 29.1 存储与生命周期

- [ ] Secret 是否只存在 CredentialStore；
- [ ] SQLite 是否只保存 metadata/reference/fingerprint；
- [ ] OS keyring adapter 是否实现 health、revoke、rotation；
- [ ] `auth.json` 是否检查 0600/ACL；
- [ ] Credential version 是否不可变；
- [ ] active pointer、rotation、revoke 是否可恢复；
- [ ] lease 是否有 expiry、operation scope 和 cleanup evidence。

### 29.2 权限与外发

- [ ] Credential use 与 data egress 是否分离；
- [ ] destination 是否规范化和绑定；
- [ ] tool/extension/MCP revision 是否参与判权；
- [ ] hard deny 是否不可被 saved rule 覆盖；
- [ ] 高风险是否要求 reauth；
- [ ] provider redirect/proxy/DNS 变更是否重新判权；
- [ ] unknown external effect 是否进入 reconcile_required。

### 29.3 脱敏与传播

- [ ] Provider、MCP、Shell、Hook、Plugin、Export 全部经过 scanner；
- [ ] Secret 是否禁止进入 Prompt/Event/Trace/Memory/Checkpoint；
- [ ] redaction 是否保留 lineage 和规则版本；
- [ ] scanner 失败是否 fail closed；
- [ ] taint 是否能跨 Context/ToolCall/Blob/Rule/Gate 传播；
- [ ] UI/错误是否避免长度和内容侧信道。

### 29.4 扩展和进程

- [ ] Skill script 是否只能通过 Tool Gateway；
- [ ] Plugin 是否不能读取 keyring socket；
- [ ] Wasm Host API 是否无 Secret bytes；
- [ ] subprocess 是否无 argv 注入；
- [ ] MCP config 是否禁止明文 token 持久化；
- [ ] process crash 后是否清理 lease 和临时文件；
- [ ] Credential revoke 是否通知 active adapter。

### 29.5 隐私和运维

- [ ] backup 是否排除 Secret material；
- [ ] export 是否加密、一次性、可审计；
- [ ] privacy purge 是否区分 logical revoke 和物理清除；
- [ ] support bundle 是否二次扫描；
- [ ] telemetry 是否默认关闭且不含敏感标签；
- [ ] incident 是否有轮换、撤销、对账和清除动作。

---

## 30. 结论与后续文档

Apex 的 Credential 与敏感数据治理必须以“Secret 原文不进入普通控制面”为第一原则，以 CredentialStore、Credential Broker、Tool Gateway、Data Egress Policy 和统一 Redaction Pipeline 形成完整闭环：

1. Credential 是有身份、版本、作用域和生命周期的领域实体；
2. Secret value 与 CredentialRef 永久分离；
3. OS Credential Store 是默认安全后端，`~/apex/auth.json` 只作为兼容迁移路径；
4. Credential use 与 data egress 必须独立授权；
5. Secret 只在最后时刻通过短期 lease 注入；
6. Provider、MCP、Shell、Plugin、Hook 和 Browser 都不能绕过 Broker；
7. 所有非 Secret 输出都必须经过 scanner/redaction/taint 处理；
8. 外部状态未知时必须 reconcile，不能用失败或未使用掩盖事实；
9. 轮换、撤销、隐私清除和备份都必须保留可审计 lineage；
10. UI 只能展示安全摘要，不成为 Secret 访问入口。

下一份建议文档为：

> **`Apex—— Observability、审计与运维控制面详细设计.md`**

该文档将整合 Session、Run、Tool、MCP、Credential、Hook、Plugin、Rules、Gate 的日志、指标、Tracing、事件投影、告警、审计查询、故障恢复和运维操作。

---

# 附录 A：Credential Manifest 示例

```yaml
apiVersion: apex.dev/v1
kind: CredentialProfile
metadata:
  name: github-work
  owner: user
spec:
  kind: oauth_access_token
  provider: github
  scope: user
  allowedTools:
    - mcp__github__issues_*
  allowedDestinations:
    - host: api.github.com
      scheme: https
  dataClassificationCeiling: sensitive
  purpose:
    - repository_issue_management
  rotation:
    refreshBeforeSeconds: 300
    overlapSeconds: 60
  storage:
    backend: os-keyring
```

# 附录 B：Credential Usage Receipt 示例

```json
{
  "usage_id": "cuse_01J...",
  "credential_id": "cred_github_work",
  "credential_version_id": "crv_03",
  "operation_id": "op_...",
  "tool_call_id": "tcl_...",
  "destination": {
    "kind": "mcp_http",
    "server_id": "mcp_github",
    "host": "api.github.com"
  },
  "purpose": "repository_issue_management",
  "input_labels": ["sensitive", "repository_content"],
  "redaction": {
    "profile": "github_issue_v1",
    "state": "applied",
    "record_id": "red_..."
  },
  "lease": {
    "channel": "protocol_header_injection",
    "issued_at_us": 1780000000000,
    "revoked_at_us": 1780000008123
  },
  "result_state": "confirmed"
}
```

# 附录 C：Egress Decision 示例

```json
{
  "decision_id": "eg_...",
  "decision": "allow_with_approval",
  "source": {
    "project_id": "prj_...",
    "paths": ["src/**"],
    "classification": "sensitive"
  },
  "destination": {
    "host": "api.github.com",
    "server_id": "mcp_github"
  },
  "credential": {
    "credential_id": "cred_github_work",
    "version_fingerprint": "fp_7a91"
  },
  "requirements": [
    "credential.use.v1",
    "data.egress.v1",
    "mcp.invoke.v1",
    "redaction_profile:github_issue_v1"
  ],
  "summary": "将脱敏后的项目内容发送到已批准的 GitHub MCP 目的地"
}
```

# 附录 D：错误码

| 错误码 | 语义 |
|---|---|
| `CREDENTIAL_NOT_FOUND` | Credential 不存在或不可见 |
| `CREDENTIAL_STORE_UNAVAILABLE` | CredentialStore 不可用 |
| `CREDENTIAL_STORE_LOCKED` | 本地安全存储锁定 |
| `CREDENTIAL_PERMISSION_INSECURE` | 文件/ACL 权限不安全 |
| `CREDENTIAL_REVOKED` | Credential 已撤销 |
| `CREDENTIAL_EXPIRED` | Credential 或 lease 已过期 |
| `CREDENTIAL_SCOPE_DENIED` | 作用域不匹配 |
| `CREDENTIAL_DESTINATION_DENIED` | 目的地不允许 |
| `CREDENTIAL_REAUTH_REQUIRED` | 需要重新认证 |
| `CREDENTIAL_INJECTION_UNSUPPORTED` | 当前 adapter 不支持安全注入 |
| `CREDENTIAL_LEASE_EXPIRED` | Lease 超时 |
| `CREDENTIAL_LEAK_SUSPECTED` | 疑似 Secret 泄漏 |
| `CREDENTIAL_PROVIDER_REJECTED` | Provider 拒绝认证 |
| `DATA_CLASSIFICATION_UNKNOWN` | 数据无法分类 |
| `DATA_EGRESS_DENIED` | 数据外发被拒绝 |
| `DATA_EGRESS_REDACTION_REQUIRED` | 必须先脱敏 |
| `DESTINATION_CHANGED` | 目的地发生变化 |
| `SECRET_SCAN_FAILED` | Secret scanner 失败 |
| `SECRET_FOUND_IN_ARGUMENTS` | Tool arguments 中发现 Secret |
| `SECRET_FOUND_IN_OUTPUT` | 输出中发现 Secret |
| `ROTATION_CONFLICT` | 轮换版本冲突 |
| `REVOCATION_IN_PROGRESS` | 撤销正在进行 |
| `PRIVACY_PURGE_BLOCKED` | 隐私清除被保留策略阻断 |
| `CREDENTIAL_RECONCILIATION_REQUIRED` | Credential 使用状态未知 |

# 附录 E：关键不变量

1. **不落库不变量**：Secret 原文不进入 SQLite、Event、Prompt、Memory、Checkpoint、Trace 或普通 Blob；
2. **引用分离不变量**：业务层只传 CredentialRef/opaque handle；
3. **最后注入不变量**：Secret 只在执行边界按 operation scope 注入；
4. **双授权不变量**：Credential use 与 data egress 必须分别通过；
5. **硬拒绝不变量**：revoked/expired/不可信扩展/不安全注入不可被 saved rule 覆盖；
6. **脱敏门不变量**：所有非 Secret 输出必须经过 scanner/redaction；
7. **taint 不升级不变量**：外部/敏感内容不能自动成为 control plane 数据；
8. **目的地绑定不变量**：redirect/proxy/schema/config 变化触发重新判权；
9. **版本绑定不变量**：审批、lease、usage 绑定 credential version 和 policy revision；
10. **未知不重试不变量**：Credential 已注入但外部状态未知时必须 reconcile；
11. **扩展隔离不变量**：Plugin/Skill/Hook/MCP 不得直接读取全局 CredentialStore；
12. **撤销传播不变量**：revoke 阻止新 lease，并尽快通知 active adapters；
13. **审计保留不变量**：删除 Secret material 不删除最小使用事实和 lineage；
14. **面板非权威不变量**：UI 只显示安全 projection，操作必须转为 Command。

# 附录 F：与已有文档的交叉引用

| 主题 | 对应文档 |
|---|---|
| 敏感文件、auth.json、项目安全 | `Apex—— 需求分析文档.md` §4.2、§4.3 |
| CredentialStore、最后注入、redaction | `Apex—— Tool Gateway与权限引擎详细设计.md` §20、§21、ADR-TG-009 |
| ToolResult、Taint、Reconcile | `Apex—— Tool Gateway与权限引擎详细设计.md` §5、§21、§29 |
| 数据分级、Secret 不入 SQLite | `Apex—— SQLite数据模型与迁移设计.md` §19 |
| 扩展 capability、MCP egress、taint | `Apex—— MCP、Skill、Hook与Plugin扩展系统详细设计.md` §9、§16、§25 |
| Context/Checkpoint 中的敏感内容 | `Apex—— Context与Checkpoint系统详细设计.md` |
| Rules/Gate 与外部证据 | `Apex—— Rules与Verification Gate详细设计.md` |
| 后续主题 | `Apex—— Observability、审计与运维控制面详细设计.md` |
