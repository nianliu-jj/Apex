# Apex—— Deployment、升级与灾备详细设计

> 文档状态：详细设计基线（面向最终完整产品）  
> 适用版本：Apex v0.1 ～ v1.x（按 §交付阶段 分档启用；档位表以需求文档 §5.3 为准）  
> 编制日期：2026-08-08  
> 上游文档：`Apex—— 需求分析文档.md`、`Apex—— 系统总体架构设计.md`、`Apex—— 领域模型与事件规范.md`、`Apex—— API与实时事件协议设计.md`、`Apex—— SQLite数据模型与迁移设计.md`、`Apex—— Agent Runtime与DAG调度器详细设计.md`、`Apex—— Tool Gateway与权限引擎详细设计.md`、`Apex—— Rules与Verification Gate详细设计.md`、`Apex—— MCP、Skill、Hook与Plugin扩展系统详细设计.md`、`Apex—— Credential与敏感数据治理详细设计.md`、`Apex—— Observability、审计与运维控制面详细设计.md`  
> 关键词：Deployment、apexd、Tauri、Actix、Release、Migration、Backup、Restore、Disaster Recovery、Rollback、Compatibility、Supply Chain

---

## 0. 文档目的与范围

本文将 Apex 的总体架构和各领域详细设计落到可部署、可升级、可恢复的产品拓扑。最终产品既要支持个人开发者在一台电脑上运行，也要支持显式启用的 Web 服务部署；两者必须共享同一 Core、同一事件语义、同一安全边界和同一数据恢复原则。

本文覆盖：

- 单机模式、Desktop 模式、Web 模式和后续远程团队模式；
- `apexd`、TUI、Tauri Desktop、Actix Web Gateway、Vue UI、Provider、MCP、Tool、Credential Store 和存储的部署关系；
- Windows、macOS、Linux 的进程、目录、权限和本地 IPC 约束；
- Release artifact、版本矩阵、协议协商、数据库 schema、事件 schema、Projection revision 和插件兼容；
- 安装、首次启动、升级、降级、回滚、迁移、备份、恢复、灾备和发布中止；
- SQLite、Markdown、Blob、Shadow Git、Credential metadata 和 OS Credential Store 的一致性；
- 单机灾难恢复、可选远程备份、RPO/RTO、演练和安全供应链；
- v0.1～v1.x 的交付拓扑与上线门槛。

本文不规定：

- 云厂商具体产品、Kubernetes Helm Chart 或某个托管数据库的实现细节；
- Provider、MCP Server、操作系统 keyring 的内部协议；
- 业务领域状态机和 Tool 权限算法；
- UI 的视觉设计。

这些对象必须通过既有 Protocol、Port、Event Registry、Migration、Backup Manifest 和 Capability 接入。

---

## 1. 最终部署架构结论

### 1.1 单机模块化单核是默认产品形态

Apex 默认采用一个 OS 用户对应一个 `apexd` 实例：

```text
┌──────────────────────────────────────────────────────────────┐
│                         用户设备                             │
│                                                              │
│  ┌──────────────┐   Native IPC / loopback   ┌──────────────┐ │
│  │ TUI / CLI    │◄─────────────────────────►│              │ │
│  └──────────────┘                           │              │ │
│  ┌──────────────┐   Tauri commands/events   │   apexd      │ │
│  │ Tauri Shell  │◄─────────────────────────►│ Core Runtime │ │
│  │ Vue WebView  │                           │              │ │
│  └──────────────┘                           │              │ │
│  ┌──────────────┐ HTTP/WS  ┌─────────────┐  │              │ │
│  │ Browser Vue  │◄─────────►│ Actix Web   │◄─┤              │ │
│  └──────────────┘           │ Gateway     │  └──────┬───────┘ │
│                             └─────────────┘         │         │
│                  ┌──────────────────────────────────┼──────┐  │
│                  │ SQLite WAL │ Files │ Blobs │ Git │ OS   │  │
│                  │            │       │       │     │Keyring│  │
│                  └──────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
                              │
                              ▼
              Provider API / MCP / Shell / Test / Browser
```

`apexd` 是唯一业务核心、唯一 SQLite writer、唯一执行控制面和唯一事件提交者。三类客户端不直接访问 SQLite、工作区、Shadow Git、Provider 或 MCP。

### 1.2 Web 不是第二个 Core

Web 模式分为两种：

1. **本机 Web**：浏览器连接本机 Actix Gateway，由 Gateway 连接同机 `apexd`；
2. **远程 Web（显式启用）**：浏览器连接部署在服务器上的 Actix Gateway/Core，必须增加 TLS、身份系统、租户隔离、远程 Credential 策略和备份边界。

两种模式都遵守同一 Application Command、Query、Event Subscription 和 Capability 语义。Actix Gateway 不复制领域逻辑，不直接调用 Provider、Tool 或数据库。

### 1.3 三种部署模式

| 模式 | 目标 | 进程 | 默认暴露面 | 数据位置 |
|---|---|---|---|---|
| Local Headless | CLI/TUI/自动化 | `apexd` | 本机 IPC | 用户 Apex Home |
| Desktop | 普通桌面用户 | `apexd` + Tauri shell | Tauri IPC/本机连接 | 用户 Apex Home |
| Local Web | 浏览器访问本机 | `apexd` + Actix Gateway | loopback HTTP/WS | 用户 Apex Home |
| Remote Single-User | 服务器个人实例 | `apexd` + Actix Gateway | TLS/反向代理 | 服务用户数据目录 |
| Remote Team（后续） | 多用户协作 | Gateway + Core 实例/租户边界 | TLS/身份系统 | 受控服务存储 |

v1 之前不把 Remote Team 的多租户能力隐含到 Local 模式中；任何远程监听都需要显式配置、启动告警和安全预检。

**各形态的版本归属（ADR-0026）**：

| 形态 / 能力 | 版本 | 说明 |
|---|---|---|
| Local Headless、Desktop | v0.1 / v0.3 | 基线产品形态 |
| Local Web | v0.3 | 随三端共享交付 |
| Windows Service / launchd / systemd unit | v1.0 | 服务化托管属发行工程，不进 MVP |
| 签名安装器与受控自动更新（Development/Nightly/Stable/Enterprise 四通道） | v1.0 | 与供应链签名、撤销名单一同交付 |
| 容器部署（`/data/apex` 持久卷、liveness 探针） | v1.x | **非默认支持路径**；仅用于 Remote Single-User 场景，需显式启用 |
| Remote Single-User | v1.x | 需 TLS、身份、备份边界；不隐含多租户 |
| Remote Team | v1.x+ | 需另行设计租户隔离与审计模型 |

需求文档与系统总体架构把 Apex 定位为**本机优先**产品：默认只暴露本机 IPC/loopback，不将代码执行能力暴露到公网。容器化与远程形态是该定位之上的可选扩展，其存在不改变本机形态的安全默认值——特别是容器场景下 Credential Store 无法依赖 OS keyring，必须显式配置加密文件后端或外部 KMS，不得回退到明文环境变量。

> ADR-0026（跨文档一致性审查）：容器部署、服务化托管、四发布通道与自动更新、Remote Single-User 转正等形态原散落于本文各节且未标注版本归属，读者易误认为属 MVP 范围。现集中给出版本归属与非默认路径标注。

### 1.4 进程隔离原则

默认进程关系：

```text
apexd
 ├─ Runtime Supervisor
 ├─ SQLite StorageWriter
 ├─ Read Query workers
 ├─ Event Bus / Projectors / Outbox
 ├─ Provider adapters
 ├─ MCP supervisors
 ├─ Tool subprocess supervisors
 └─ Credential Broker client

Tauri shell / Actix Gateway / TUI / CLI
 └─ protocol clients, no direct domain side effects
```

Provider、MCP、Shell、Hook 和 Plugin 必须由 Supervisor 以独立子进程或受限 adapter 运行；它们崩溃不应使 `apexd` 失去已提交事实或破坏其他项目。

---

## 2. 支持平台与运行时边界

### 2.1 平台目标

| 平台 | Core | TUI | Desktop | Local Web | 本机 IPC |
|---|---|---|---|---|---|
| Windows 10/11 x64 | 支持 | 支持 | 支持 | 支持 | Named Pipe 优先 |
| macOS 13+ Apple Silicon/Intel | 支持 | 支持 | 支持 | 支持 | Unix Domain Socket 优先 |
| Linux x64 | 支持 | 支持 | 可选 | 支持 | Unix Domain Socket 优先 |

具体最低版本、签名和安装包格式在发布清单中版本化；代码不能把某个发行版的路径、shell 或 keyring 当作跨平台事实。

### 2.2 Rust 运行时

- `apexd` 使用 Tokio 运行异步 supervisor、协议服务、外部 adapter 和后台任务；
- SQLite writer 仍是单独受控的同步写入 actor，避免事务跨 `await`；
- TUI 使用 ratatui，仅持有 Query projection 和事件缓存；
- Tauri 使用 Rust shell 管理本机连接、窗口、OS 能力和敏感视图；
- Actix 仅承担 Web Gateway、认证、限流、协议转换和连接生命周期；
- Provider、MCP 和 Plugin SDK 版本通过稳定 protocol 与 Core 解耦。

### 2.3 不支持的隐式部署

以下部署不能作为默认支持路径：

- 将 SQLite、WAL 或 Shadow Git 放在 SMB/NFS/云同步目录；
- 多个 `apexd` 进程共享同一 Apex Home；
- 浏览器直接打开 `apex.db` 或调用本机工具；
- 通过反向代理把本机未认证 loopback 端口暴露到公网；
- 把 Credential Store 原文放进容器镜像、安装包或普通环境变量；
- 在升级期间同时运行新旧两个 writer；
- 只恢复 SQLite 而不校验 Blob、Snapshot 和 Credential metadata manifest。

---

## 3. Release Artifact 与供应链

### 3.1 发布物分类

每个版本至少生成以下 artifact：

```text
apexd-{version}-{target}.zip/tar.gz
apex-cli-{version}-{target}
apex-tui-{version}-{target}
apex-desktop-{version}-{platform}.{msi|dmg|AppImage|deb|rpm}
apex-web-assets-{version}.tar.gz
apex-migrations-{schema_revision}.json
apex-event-registry-{protocol_family}.json
apex-release-manifest-{version}.json
```

`apexd`、CLI、TUI、Tauri shell 和 Web Gateway 采用明确的 Protocol/Schema 兼容矩阵，不根据文件名猜测兼容关系。

### 3.2 Release Manifest

```json
{
  "product": "apex",
  "version": "0.5.0",
  "release_id": "rel_01...",
  "commit": "git-sha",
  "build_profile": "release",
  "targets": ["windows-x86_64", "macos-aarch64", "linux-x86_64"],
  "minimum_database_format": 5,
  "maximum_database_format": 5,
  "protocol_family": "apex.v1",
  "protocol_range": {"min": 1, "max": 3},
  "event_registry_revision": "events-v8",
  "projection_revisions": {
    "overview": "overview-v3",
    "mcp": "mcp-v2"
  },
  "migration_plan_digest": "sha256:...",
  "artifact_digests": {},
  "signature": {
    "algorithm": "ed25519",
    "key_id": "release-key-2026"
  }
}
```

Manifest 不包含 Credential、用户路径、Prompt 或项目数据。安装器和 `apexd` 启动器在执行前验证 manifest、签名、目标平台和最低版本。

### 3.3 可复现与构建信息

CI 至少保存：

- 源码 commit、依赖 lockfile、Rust toolchain、SQLite bundled 版本；
- 编译 target、feature flags、是否包含调试能力；
- 构建脚本版本、生成的 Protocol/事件注册表 digest；
- SBOM、漏洞扫描结果、签名和发布审批记录。

生产二进制不能以“未签名 debug build”替代正式发布物。开发构建可以连接本地 fake Provider，但必须在启动 Banner 和事件中标明 build profile。

### 3.4 供应链验证

安装、升级和远程启动前依次验证：

1. artifact 是否属于当前产品和目标平台；
2. manifest 签名是否通过；
3. artifact digest 是否匹配；
4. 版本是否满足数据库、协议和插件兼容矩阵；
5. 安装包是否来自允许的渠道；
6. 是否存在撤销或阻断名单；
7. 失败时进入安全停止，不执行迁移、不启动外部 Tool。

---

## 4. Apex Home 与目录布局

### 4.1 用户级目录

默认用户级 Apex Home：

```text
Windows: %APPDATA%\Apex\
macOS:   ~/Library/Application Support/Apex/
Linux:   ${XDG_STATE_HOME:-~/.local/state}/apex/
```

可通过 `APEX_HOME` 覆盖，但启动时必须验证目录位于本地文件系统、权限可控、不是明显的共享目录或同步目录。

```text
<APEX_HOME>/
├── apex.db
├── apex.db-wal
├── apex.db-shm
├── config/
│   ├── instance.toml
│   ├── policies.toml
│   └── providers.toml
├── rules/                          # 用户级规则（项目级在 <project>/apex/rules/）
├── skills/                         # 用户级 Skill（项目级在 <project>/apex/skills/）
├── mcp.json                        # 用户级 MCP 配置
├── sockets/
│   ├── apexd.sock                 # Unix；Windows 使用 named pipe 标识
│   └── endpoint.json              # 非 Secret 的发现信息
├── logs/
│   ├── apexd.log
│   ├── audit.log
│   └── crash/
├── blobs/
│   ├── objects/sha256/ab/cd/<digest>
│   ├── tmp/
│   └── quarantine/
├── projects/
│   └── <project-id>/
│       ├── worktrees/              # Apex 管理的隔离工作树
│       ├── cache/                  # 可重建的派生数据
│       └── project.toml            # 本机绑定信息（非团队资产）
├── snapshots/                      # 影子 Git：<project_hash>/<worktree_hash>/.git
├── backups/
├── exports/
├── quarantine/
├── support-bundles/
├── diagnostics/
└── runtime/
    ├── pid
    ├── readiness.json
    └── locks/
```

### 4.1.1 项目级目录（团队可提交资产）

Apex Home 只保存**本机运行时状态**。以下资产属于项目仓库，随代码一起提交、评审和分发：

```text
<project>/apex/
├── specs/<feature-name>/           # requirements/design/tasks/verification
├── rules/                          # 项目编码规范
├── skills/                         # 项目级 Skill
├── memory/                         # 项目记忆
├── checkpoints/<session-id>/       # 会话 Checkpoint 的 Markdown 镜像
├── mcp.json
└── config.toml
```

划分依据：**是否应当随仓库分发并被人类评审**。Spec、Rules、Skills、Memory、Checkpoint 的 Markdown 镜像是团队资产（需求文档 §3.1.3、§3.3.2；系统总体架构 §16.2）；数据库、Blob、影子 Git、日志、Credential 和隔离工作树是本机状态，不入项目仓库。

Spec 权威源仍是 SQLite artifact revision，`<project>/apex/specs/` 是可编辑、可提交的 Markdown 镜像，由 §10.3 的 materialization 协议保持一致。

> ADR-0002（跨文档一致性审查）：原 §4.1 把 `specs/`、`checkpoints/`、`memory/`、`snapshots/` 移入 `<APEX_HOME>/projects/<project-id>/`，全文未出现 `<project>/apex/`。该改动会取消"Spec 随仓库提交、可 code review"这项在需求文档中定义的产品能力。现采用混合方案：用户级目录保留本节的平台原生路径（符合 OS 规范、利于打包发行），项目级可提交资产回归 `<project>/apex/`。同时补回原清单遗漏的 `rules/`、`skills/`、`mcp.json`、`diagnostics/`。


### 4.2 Credential Store 目录

Credential 原文不放入普通 Apex Home。默认顺序：

1. OS Credential Store/keyring；
2. 由用户明确启用的加密文件后端；
3. 兼容 `~/apex/auth.json` 的迁移读取器，仅作为导入/兼容边界。

`auth.json` 若存在，必须按平台权限要求保护（Unix `0600`；Windows ACL 仅当前用户和系统可读），并且不被日志、备份 manifest、诊断包和项目文件自动收集。迁移完成后可生成 tombstone 或由用户确认删除。

### 4.3 项目目录

项目工作区不等于 Apex Home。项目可以位于任意受支持的本地目录，但 Core 保存其 canonical path、workspace identity、文件系统类型、snapshot backend 和 policy digest。项目目录内的 `.env`、`*.key`、`*.pem` 等路径默认受保护。

### 4.4 路径解析与符号链接

安装器、启动器、备份和恢复模块必须：

- 使用绝对路径 canonicalize 后再比较范围；
- 拒绝跨边界的符号链接、junction、reparse point 或 mount；
- 删除、移动和 GC 前重新校验最终路径；
- 不通过拼接字符串执行递归删除；
- 将路径以 `workspace-relative`、`outside-workspace` 或 `sensitive-path` 分类后再进入事件/日志。

---

## 5. 进程生命周期与所有权

### 5.1 `apexd` 启动器

启动器职责：

- 定位可执行文件、Apex Home 和配置；
- 检查是否已有实例；
- 验证 release/schema/protocol；
- 建立进程级 single-instance lock；
- 创建本机 endpoint 和短期握手 token；
- 启动 `apexd`，等待 readiness；
- 将启动失败分类并输出安全诊断。

启动器不直接执行业务 Command，不持有 SQLite connection，不访问 Credential 原文。

### 5.2 `apexd` 生命周期

```text
created
  → booting
  → preflight
  → migrating
  → recovering
  → ready
  → degraded
  → quiescing
  → stopped
```

`failed` 是启动尝试结果，不是可继续接受 Command 的运行态。所有状态变化发布 `process.*` 或 `recovery.*` 事件并写安全日志。

### 5.3 优雅关闭

```text
1. 停止接受新的普通 Command
2. 保留 cancel、approval、security 和 recovery 通道
3. 通知客户端 server_restart/going_away
4. 停止新建外部 Operation
5. 等待可安全完成的 StorageWriter 事务
6. 将不可确认的外部操作写为 interrupted/unknown
7. 保存 projection/outbox watermark
8. checkpoint WAL（不以删除 WAL 为成功条件）
9. 关闭 adapters、读连接和 writer
10. 删除短期 endpoint/token/运行锁
```

强制退出后由下一次启动执行完整 Recovery；不得把进程退出代码为 0 当作所有 Run 成功完成。

### 5.4 多实例保护

同一 Apex Home 默认只允许一个 `apexd`。如确需多个实例，必须为每个实例使用不同 `APEX_HOME`、端口/IPC 名称、数据库、日志和 Credential scope，并在启动 Banner 中显示实例 ID。

---

## 6. 本机通信与远程通信

### 6.1 Native IPC

优先级：

```text
macOS/Linux: Unix Domain Socket → loopback TCP fallback
Windows: Named Pipe → loopback TCP fallback
```

Native endpoint 发现文件只保存：`instance_id`、PID、协议范围、endpoint kind、启动时间、证书/握手信息的引用和 expires_at。短期 handshake token 不写入普通日志，不返回给任意浏览器页面。

### 6.2 握手

```text
Client → Hello(protocol_range, client_kind, client_version, nonce)
apexd  → Challenge(instance_id, server_nonce, capabilities_hint)
Client → Authenticate(short_lived_token, proof, requested_scope)
apexd  → Ready(connection_id, negotiated_protocol, event_store_id, watermark)
```

服务端再次根据 actor、client、project 和 Capability 判定权限。协议协商成功不等于获得读写或执行能力。

### 6.3 Tauri 通信

Tauri Rust shell 负责：

- 连接同机 `apexd`；
- 将 typed Command 转发给 Core；
- 接收 gRPC/persistent event 并转换为有界 Tauri event；
- 保存必要的连接游标和窗口状态；
- 处理 OS 文件选择器、通知和安全存储引用。

WebView 不直接持久化带 session cookie 的敏感响应，不直接访问数据库、文件系统、Credential Store 或本机工具。

### 6.4 Actix Gateway

Actix Gateway 负责：

- HTTP/REST、WebSocket、静态资源和请求大小限制；
- 本机或远程 TLS、身份认证、CSRF/CORS、限流；
- REST DTO 与 Core protocol 转换；
- 连接级订阅、心跳、重连和压缩；
- 统一错误映射和审计上下文注入。

Gateway 不得：

- 直接写 SQLite；
- 直接调用 Provider/MCP/Tool；
- 自己计算 Permission、Approval Summary 或 Domain 状态；
- 在日志中打印完整 request body。

### 6.5 远程暴露预检

`listen != loopback` 时启动必须明确警告并要求配置：

- TLS 证书和私钥引用；
- 身份提供方或本地管理员账号；
- 反向代理可信头列表；
- CSRF/CORS 允许源；
- 多用户项目范围和数据隔离；
- Credential 使用与远程外发策略；
- 备份、审计保留和入侵响应策略。

未满足预检时，Core 只允许 loopback/IPC，不启动公网监听。

---

## 7. 安装与首次启动

### 7.1 安装阶段

安装器执行：

1. 验证安装包签名和目标平台；
2. 安装 `apexd`、CLI、TUI、Tauri shell 和资源文件；
3. 创建必要的用户级目录；
4. 注册桌面快捷方式/命令路径（可选）；
5. 不创建明文 Credential；
6. 不自动把项目目录复制到安装目录；
7. 记录安装版本和 artifact digest。

安装器失败可安全重试，不应删除用户 Apex Home 或项目工作区。

### 7.2 Bootstrap

首次启动执行：

```text
validate platform and filesystem
  → create Apex Home
  → create OS permissions
  → initialize SQLite application_id/schema
  → create event_store_id and instance_id
  → install event registry/projection registry
  → create default config with safe values
  → initialize local endpoint
  → run quick_check
  → show onboarding state
```

Bootstrap 失败时保留失败报告和 quarantine 内容，不留下半初始化状态被误判为可用实例。

### 7.3 首次运行安全默认值

- 只监听本机；
- 外部 telemetry 关闭；
- `.env`、`*.key`、`*.pem`、Credential 文件默认保护；
- 新工具和 MCP 调用按权限策略处理；
- Shell/Plugin/Hook 默认最小 capability；
- 自动更新不在后台静默替换正在使用的 Core；
- 备份目标未配置时提示用户但不上传数据；
- 所有 Provider token 通过 Credential Store 导入。

### 7.4 首次项目导入

项目导入只记录项目路径、canonical identity、配置 digest 和扫描摘要。导入过程不应把所有文件内容读入 SQLite；需要索引或 Memory 的内容按规则、大小和数据分类处理。发现敏感目录、符号链接越界或工作区锁冲突时，项目进入 `needs_review`。

---

## 8. 配置、环境变量与密钥

### 8.1 配置优先级

```text
安全硬规则（编译/平台级）
  > 启动参数（仅非敏感或 Credential 引用）
  > instance.toml
  > project.toml
  > policy profile
  > session temporary option
```

低层配置不能覆盖安全硬规则，例如禁止把 `secret_prohibited` 写入日志、禁止公网无 TLS、禁止多个 writer 共享数据库。

### 8.2 环境变量

允许环境变量用于：

- `APEX_HOME`、日志级别、开发端口、Provider 类型和非敏感调试开关；
- Credential Store backend 的定位信息；
- CI 中临时注入的 Credential 引用，而非 Secret 原文。

禁止在启动诊断、进程列表、Crash dump、Telemetry 和支持包中收集全部环境变量。启动器只选择性读取注册表内的变量名。

### 8.3 Provider 配置

Provider 配置保存 endpoint、模型名、能力、超时、重试、usage 估算和 CredentialRef。配置变更通过 Command 生成版本和审计；Provider adapter 在调用前检查策略、Credential lease、Data Egress 和当前版本。

### 8.4 配置热更新

允许热更新：日志级别、面板刷新、告警阈值、某些非破坏性 Provider 参数。需要重启或 quiesce：监听地址、TLS、数据库路径、Credential backend、协议重大版本和 runtime sandbox。热更新使用 config revision；旧 Run 保留启动时的配置快照引用。

---

## 9. 版本矩阵与兼容性

### 9.1 版本维度

Apex 同时管理以下版本，不允许只用一个产品版本号代替：

| 维度 | 示例 | 作用 |
|---|---|---|
| Product version | `0.5.0` | 用户可见发行版本 |
| Build/release ID | `rel_...` | 二进制和供应链追踪 |
| Protocol family | `apex.v1` | Client/Core 协商 |
| Protocol range | `1..=3` | 前后兼容窗口 |
| Database format | `5` | SQLite 打开/写入能力 |
| Migration revision | `2026_08_08_001` | 迁移顺序和 checksum |
| Event schema | `tool.call_finished` | 事实兼容 |
| Event registry | `events-v8` | 事件策略 |
| Projection revision | `mcp-v2` | 查询结果结构 |
| Plugin API | `plugin.v1` | 扩展兼容 |
| Config revision | `config-v4` | 配置解析与默认值 |

### 9.2 Client/Core 矩阵

| Client | Core 版本 | 行为 |
|---|---|---|
| 同版本 | 交集存在 | 正常连接 |
| 旧客户端 | Core 仍支持 | 使用兼容 DTO，隐藏未知字段 |
| 新客户端 | Core 过旧 | 只读降级或明确 `VERSION_INCOMPATIBLE` |
| 协议无交集 | 任意 | 拒绝连接，不循环重试 |
| Projection revision 不兼容 | 任意 | Query refresh/旧视图适配 |

### 9.3 兼容规则

- 协议新增字段优先可选；
- Domain Event 新语义创建新版本；
- Migration forward-only，旧二进制不能打开新 schema 写入；
- Projection 采用 rebuild/swap，不在在线查询中混用半成品；
- 插件不链接 Core 私有类型，只依赖稳定协议；
- 降级主要依赖备份恢复或 Portable Archive Import，不支持未经验证的原地降级。

---

## 10. Release、安装包与渠道

### 10.1 渠道

| 渠道 | 适用 | 更新策略 | 风险控制 |
|---|---|---|---|
| Development | 开发测试 | 手动 | 标记 debug，允许 fake adapter |
| Nightly/Preview | 内测 | 可选自动下载 | 独立 channel、备份前置 |
| Stable | 普通用户 | 用户确认/受控自动更新 | 签名、灰度、回滚 |
| Enterprise/Offline | 受限环境 | 离线导入 | manifest、审批、内部镜像 |

### 10.2 Desktop 安装包

Windows 生成 MSI/安装包并使用代码签名；macOS 生成已签名和公证的 app/dmg；Linux 提供 AppImage 及发行版包。安装包内不包含用户数据库、项目文件、Credential 原文、测试 token 或开发日志。

### 10.3 Server 包

服务器部署至少包含：

```text
apexd binary
apex-cli binary
web assets
migration manifest
event registry
projection registry
release manifest
system service template
health/readiness command
```

服务文件只引用受控配置路径和用户身份，不把 Secret 写入 unit file、容器镜像或命令行参数。

### 10.4 发布签名与撤销

签名密钥在 CI/发布服务的受控环境中使用。客户端维护允许的公钥/密钥 ID 和撤销列表；发现撤销版本时停止安装或升级，并把原因写入本地安全日志和 Audit。

---

## 11. 升级总流程

### 11.1 升级原则

升级必须是一个可观察、可中止、可恢复的 MaintenanceRun，而不是安装器覆盖文件后“顺便启动”。原则如下：

1. 先确认二进制、manifest、备份目标和磁盘空间；
2. 停止普通写入，保留取消、安全和恢复通道；
3. 生成升级前 SQLite/Blob/Snapshot manifest；
4. 预检查 schema、event registry、projection 和插件兼容；
5. 执行 Expand/Backfill/Contract 迁移；
6. 运行 quick check、integrity check 和 projection health；
7. 以新 binary 启动并执行 Recovery；
8. 只在 readiness 通过后恢复新 Run；
9. 发现不可恢复问题时回滚到旧 binary 或从备份恢复；
10. 记录完整升级审计和报告。

### 11.2 两阶段升级拓扑

```text
old apexd ──quiesce──► migration helper / maintenance mode
                             │
                             ├── backup + manifest
                             ├── schema migration
                             ├── backfill / validation
                             └── mark compatible
                                      │
                                      ▼
                           new apexd ──recovery──► ready
```

单机模式可以由启动器自动完成；远程模式必须由部署编排器或管理员明确触发。旧 Core 不得与新 Core 同时争抢 writer lock。

### 11.3 升级前置检查

```text
binary signature and target
→ current database format and migration state
→ active runs/operations
→ write claims and child processes
→ disk free / WAL / blob bytes
→ backup destination and encryption policy
→ credential backend availability
→ plugin/skill/MCP compatibility
→ protocol client compatibility
→ maintenance lock availability
```

任一阻断项失败，升级进入 `blocked`，不应部分替换数据库或删除旧安装。

### 11.4 活跃运行处理

默认策略是：

- 不为升级新建 Run；
- 对可安全暂停的 Agent/Workflow 写入 `pause_requested`；
- 等待已提交的 StorageWriter 事务；
- 外部 operation 仍在执行但无法确认时标记 `interrupted/unknown`；
- 释放或续租 Write Claim；
- 迁移完成后由 Recovery/Reconciler 决定是否可继续，不能依赖旧内存。

用户明确选择“强制升级”时，必须显示可能的未知副作用和未保存文件风险，并生成高等级 Audit。

---

## 12. 数据库 Migration 设计

### 12.1 Migration 包

每个迁移包含：

```text
migration_id
from_format
into_format
checksum
preconditions
expand_sql / rust step
backfill_plan
validation_queries
projection_impact
rollback_or_restore_plan
estimated_duration
requires_quiesce
```

Migration checksum 写入 `schema_migrations`，运行时发现相同 ID 内容变化必须阻止启动。

### 12.2 Expand / Backfill / Contract

采用三阶段策略：

```text
Expand
  新增表/列/索引，保持旧 binary 可读（必要时）
Backfill
  分批填充、记录 checkpoint、可暂停、可重试
Contract
  删除旧结构或切换必填约束，必须在兼容窗口结束后执行
```

SQLite 的表重建、索引创建和 FTS rebuild 可能产生较大临时空间，预检必须按实际文件和临时目录估算水位。发现异常行不得静默跳过；任务需进入 failed/blocked 并保留报告。

### 12.3 迁移事务

- 小型 schema 变更可在单个短事务中完成；
- 大型 backfill 不持有长事务，按事件/主键/字节分批；
- 每批写入 `maintenance_run_steps` 或等价进度；
- 迁移过程中禁止调用 Provider、MCP、Shell 或 Credential Broker；
- 文件和数据库跨介质变化通过 write intent、manifest 和 Recovery 对账；
- 迁移结束必须执行 schema/application_id/integrity/事件序列检查。

### 12.4 事件与投影迁移

Domain Event 原文通常不改写：

- 事件 schema 变化通过新版本和纯 Upcaster；
- Event Registry 提供旧版本读取策略；
- Projection 可从旧事件重建为新 revision；
- 不因投影字段变更而生成伪造的领域事件；
- 迁移结果记录 `projection_revision`、`event_seq_from/to` 和校验摘要。

### 12.5 Migration failure

迁移失败时按阶段处理：

| 阶段 | 处理 |
|---|---|
| Preflight | 不改业务数据，返回 blocked |
| Expand | 若事务未提交则回滚，若已提交由下次启动继续/修复 |
| Backfill | 保存 batch cursor，可重入或恢复到备份 |
| Contract | 视为高风险，默认只允许从备份恢复 |
| Validation | 进入 read-only/degraded，不宣称升级成功 |

升级器不得删除唯一旧数据库、唯一 Blob manifest 或唯一可恢复备份。

---

## 13. Projection、FTS 与缓存升级

### 13.1 Projection 双版本

投影升级优先采用并行 revision：

```text
projection_old (serving)
projection_new (rebuilding)
        │
        └── catch up to event_store_head_seq
                         │
                         ▼
              atomic registry switch
```

切换前检查：事件范围、行数、关键聚合、redaction revision、unknown event 数量和查询契约。切换后旧投影保留一段可回滚窗口，之后由 GC 删除。

### 13.2 FTS 升级

FTS 是可重建索引，不是事实来源。升级期间可以：

- 暂时将搜索标记为 stale；
- 从安全 Artifact/Memory/Event 派生源重建；
- 对敏感文本执行相同分类和清除策略；
- 记录 token/doc count、索引版本和抽样结果。

### 13.3 内存缓存与客户端缓存

所有进程内缓存都带：

```text
source_revision
as_of_global_seq
schema/protocol revision
expiry
```

升级或恢复时，缓存默认失效并从 Core Query 重建。Tauri WebView cache、浏览器 IndexedDB 和 TUI 本地状态都不能作为业务恢复依据。

### 13.4 升级后的查询兼容

如果客户端请求旧 Projection view：

- 能安全转换则由 Core/Protocol Adapter 返回旧字段；
- 无法转换则返回明确 `PROJECTION_VERSION_INCOMPATIBLE`；
- 不能以空值静默伪装“没有数据”；
- 响应必须包含 `projection_revision` 和 warning。

---

## 14. 插件、Skill、MCP 与 Provider 升级

### 14.1 Extension registry

每个扩展以 manifest、digest、签名状态、API 版本、registry generation 和 capability 集合注册。升级流程：

```text
download/import
  → verify signature/digest
  → inspect manifest
  → compare API/capability
  → install in versioned directory
  → run isolated compatibility check
  → update registry generation
  → enable or keep disabled
```

扩展升级不会静默扩大 capability。新增 capability 必须重新确认、重新审计。

### 14.2 Skill 升级

Skill 内容属于有版本的输入源：

- 保存 source/layer/version/digest；
- 活跃 Run 固定 Skill snapshot；
- 新版本不回写正在运行的 Context；
- 升级后触发 registry generation 变化和面板提示；
- Skill 加载失败不应破坏历史事件或已完成 Run。

### 14.3 MCP Server 升级

MCP Server 升级前：

- 停止新调用；
- 记录连接、工具 registry 和版本；
- 等待可安全结束的 operation；
- 未知副作用写入 `interrupted/unknown`；
- 新进程通过 handshake/health check 后才恢复调用；
- 工具集合变化重新执行 capability 和 policy 检查。

### 14.4 Provider adapter 升级

Provider API、模型名或 token 计费字段变化时：

- adapter version 和 model capability 写入请求快照；
- 历史 usage 不因新 adapter 重算覆盖；
- 新旧 adapter 可在兼容窗口并存，但同一个 operation 绑定单一 adapter revision；
- 认证失败、限流和 schema 变化以稳定错误码归一化。

---

## 15. 桌面端、TUI 与 Web 发布

### 15.1 TUI/CLI

TUI/CLI 可独立升级，但连接时必须执行 Hello 协商。升级后的 CLI 不应自动修改 Core 数据库，除非通过明确 Maintenance Command。CLI 的 stdout 默认不打印 Secret、完整错误栈或未经授权的事件 payload。

### 15.2 Tauri Desktop

Tauri 发布包包含：

```text
Tauri Rust shell
Vue static assets
protocol client
icons/resources
release manifest
```

更新策略：

- 下载到临时目录；
- 验证签名和 digest；
- 先确认当前 `apexd` 可升级或保持兼容；
- 替换 shell/UI 不等于升级数据库；
- WebView 资源失败可回退旧资源；
- shell 与 Core 无协议交集时显示升级指引，不循环重启。

### 15.3 Web Gateway 与 Vue

Web 静态资源采用 content hash 和版本 manifest。Gateway 与 UI 版本不一致时：

- 允许安全的向后兼容窗口；
- 禁止旧 UI 发送未知高风险 Command；
- Gateway 返回 `server_version`、`protocol_range` 和 `ui_compatibility`；
- 发布期间保留旧资源或维护页，避免用户得到半套 UI。

### 15.4 多端同时更新

推荐顺序：

```text
1. 备份并 quiesce Core
2. 升级/验证 apexd
3. 启动新 Core 并 readiness
4. 升级 Tauri/CLI/TUI/Web assets
5. 客户端重新协商协议和 projection
6. 恢复普通 Run
```

如果客户端先升级，必须能只读连接旧 Core；如果 Core 先升级，必须保留旧客户端的兼容协议窗口。

---

## 16. 发布与回滚流程

### 16.1 灰度发布

Stable 发布建议分为：

```text
internal validation
  → canary users/projects
  → limited stable
  → broad stable
```

每阶段观察：启动失败、迁移耗时、SQLite 错误、Projection lag、Outbox lag、Provider/MCP 错误、Secret scanner、崩溃率、恢复成功率和用户回滚请求。

### 16.2 回滚等级

| 等级 | 条件 | 动作 |
|---|---|---|
| UI rollback | 仅静态资源异常 | 回退 Tauri/Web assets |
| Client rollback | 协议或客户端异常 | 回退 TUI/CLI/shell |
| Binary rollback | Core 行为错误但 schema 未升级 | 停止新 Run，恢复旧 binary |
| Database restore | schema/数据不可逆异常 | 从升级前 backup 恢复 |
| Portable import | 原库不可用但事件/资产可读 | 新 event_store 导入 |
| Disaster recovery | 设备/目录丢失 | 新设备恢复 backup/archive |

### 16.3 回滚前检查

- 当前数据库 format 是否仍被旧 binary 支持；
- 是否执行了 Contract migration；
- 是否已经写入旧 binary 无法解释的新事件；
- 是否存在未确认外部副作用；
- 是否需要先导出新版本产生的事件和资产；
- Credential Store 是否仍保持独立版本和可用性。

不能把“把 exe 换回旧文件”称为完整回滚。数据库和外部 Blob/Snapshot 必须共同考虑。

### 16.4 回滚审计

每次回滚记录：版本、触发 Incident、operator、确认 token、数据库/manifest digest、恢复点、未知操作数、验证结果和最终 readiness。回滚后仍可查询旧版本产生的安全事件；若旧 binary 无法解析，则保留原始事件并通过新工具导出安全摘要。

---

## 17. 备份架构

### 17.1 备份对象边界

完整 Apex 项目恢复点至少包含：

```text
SQLite consistent backup
  ├── schema/database format
  ├── event_store_id + global_seq range
  ├── Domain Events
  ├── projections metadata/cursors
  ├── maintenance/audit metadata
  └── redaction/retention metadata
Blob manifest
  ├── object digest
  ├── size/media type
  ├── classification
  ├── reference count
  └── encryption metadata
Project artifact manifest
  ├── Spec revisions
  ├── Checkpoints
  ├── Memory documents
  ├── Workspace/Snapshot refs
  └── file digest summaries
Shadow Git/Snapshot metadata
Credential metadata
  ├── CredentialRef
  ├── provider/scope/version/status
  └── no secret material
```

Credential Store 原文默认不进入项目备份；恢复后需重新绑定或由独立的受控 Secret backup 恢复。

### 17.2 备份类型

| 类型 | 频率 | 内容 | 目标 |
|---|---|---|---|
| Pre-migration | 每次高风险迁移前 | DB + manifest | 本地备份目录 |
| Scheduled | 每日/阈值触发 | DB + referenced blobs | 本地或用户配置目的地 |
| Incremental manifest | 高频 | 新增 digest/事件范围 | 低成本历史 |
| User export | 用户命令 | Portable archive | 用户指定位置 |
| Crash quarantine | 完整性失败 | 原库、WAL、报告 | 隔离目录 |

### 17.3 在线备份

SQLite 使用 backup API 或等价一致性快照。不得直接把活动 `apex.db-wal` 当作独立备份；备份完成后计算：

- 数据库 digest；
- schema/database format；
- event_store_id、seq 起止；
- Blob manifest digest；
- Snapshot/Artifact 清单；
- 创建版本、策略和加密状态。

备份 manifest 与数据一起校验，但不写入加密密钥本身。

### 17.4 备份加密

- 本地备份依赖 OS 磁盘加密和文件权限；
- 用户启用加密归档时，密钥来自 Credential Store/外部 KMS 引用；
- 加密密钥轮换不能覆盖唯一旧备份；
- 加密失败进入 failed，不产生“成功但不可恢复”的标记；
- 备份下载、复制和删除都有 Audit。

### 17.5 备份验证

备份创建成功不等于可恢复。定期执行：

```text
restore to temporary directory
  → verify signature/checksum
  → open SQLite read-only
  → integrity_check
  → event sequence/hash validation
  → verify blob manifest
  → rebuild selected projections
  → sample Query
  → destroy temporary restore
```

验证结果保存为 MaintenanceRun report，至少包含恢复耗时、数据量、seq、Blob 缺失数和抽样结果。

---

## 18. 恢复与灾备等级

### 18.1 恢复语义

分为两种：

- **同库恢复**：恢复原 `event_store_id` 的一致性备份，客户端 cursor 可回退到备份水位；恢复动作产生新恢复上下文和审计；
- **导入恢复**：从 Portable Archive 创建全新数据库和新 `event_store_id`，通过 Import Command 建立新的事实链，并保留来源 manifest。

不能只恢复 Current State 而假装拥有完整历史；如果只有摘要或导出数据，必须明确数据缺失范围。

### 18.2 个人单机灾备

默认目标：

```text
RPO：最近一次成功备份之后的可恢复事件/资产范围
RTO：设备可用后完成启动和完整性预检的目标时间
```

产品不隐含承诺云端备份。首次配置时应提示用户选择本地外置盘、加密归档或其他受控目的地，并显示备份失败时的风险。

### 18.3 Remote Single-User 灾备

远程实例需要：

- 数据目录与备份目录隔离；
- 至少一个不同存储介质/区域的备份目标；
- TLS/身份配置和 Credential Store 恢复材料分开管理；
- 备份、恢复、权限和管理员操作审计；
- 监控备份 freshness、完整性和恢复演练结果。

### 18.4 灾难场景

| 场景 | 恢复路径 |
|---|---|
| Core 进程崩溃 | 重启 + SQLite WAL recovery + Domain recovery |
| SQLite 损坏 | 只读隔离 + 最近备份/Portable Export |
| Apex Home 误删 | 新 Home + 备份/归档恢复 |
| 工作区误改 | Snapshot/Shadow Git/Checkpoint 对账和回滚 |
| Blob 缺失 | manifest 检查，进入 degraded/readonly |
| Credential Store 丢失 | 元数据恢复，Credential 重新绑定 |
| 发布升级失败 | 回滚 binary 或恢复升级前备份 |
| 设备完全丢失 | 新设备安装 + 签名备份恢复 |
| 外部 Provider 未知副作用 | Operation reconcile，不盲目重放 |

---

## 19. Restore 详细流程

### 19.1 Restore Preflight

恢复命令先返回影响范围：

```json
{
  "restore_id": "restore_01...",
  "source_manifest_digest": "sha256:...",
  "target_instance_id": "inst_...",
  "target_event_store_id": "evtstore_...",
  "will_replace_database": true,
  "will_replace_project_files": false,
  "missing_blobs": 0,
  "credential_rebind_required": true,
  "unknown_external_operations": 3,
  "required_capabilities": ["maintenance.restore.v1"],
  "confirmation_token": "short_lived_one_time_token"
}
```

恢复默认不覆盖工作区文件，除非用户明确选择并通过 Workspace/Snapshot 的独立恢复流程确认。

### 19.2 同库恢复

```text
quiesce Core
  → acquire restore/db_write locks
  → preserve current DB, WAL and runtime metadata
  → verify source backup
  → restore DB into temp path
  → verify schema/event/blob/snapshot manifest
  → rebuild required projections/FTS
  → atomically switch database path
  → launch new Core in recovery mode
  → reconcile active operations/claims
  → readiness and post-restore checks
```

原数据库保留在 quarantine 或 rollback 目录，直到新库通过验证并超过回滚保留期。

### 19.3 Portable Import

Portable archive 导入到新实例时：

- 生成新的 `instance_id`、`event_store_id` 和导入 Command；
- 原始事件保存 `source_event_id/source_event_store_id` 引用；
- 不能把新实例中的导入结果伪装为原始连续事件；
- CredentialRef 默认变为 `needs_rebind`；
- 外部路径、项目 ID、Snapshot 和 Blob capability 重新核对；
- 导入报告显示不可迁移字段和丢失内容。

### 19.4 恢复后验证

恢复完成前必须验证：

1. SQLite integrity_check；
2. event_store_id、seq、hash chain；
3. Projection cursor 和 registry；
4. Outbox 是否有待处理项目；
5. Blob manifest 引用完整性；
6. Workspace/Snapshot 版本和路径边界；
7. Credential metadata 与 Credential Store 状态；
8. active Run/Agent/Tool 的未知副作用；
9. Query/API/IPC readiness；
10. 安全日志、审计和支持包路径可用。

---

## 20. 数据一致性与跨介质恢复

### 20.1 SQLite 与文件系统

SQLite 事务和文件 rename 无法形成跨介质原子操作。所有 Artifact、Checkpoint、Snapshot 和文件变更遵循：

```text
1. 生成 content digest 和临时文件
2. fsync 临时文件及父目录（平台支持时）
3. 写入 DB write_intent/artifact_pending
4. 执行受控 rename/materialize
5. 校验最终 digest 和 canonical path
6. 在 SQLite 事务中提交 artifact ready/domain event
7. Recovery 扫描 pending/ready 不一致项
```

任何中间态都必须可识别、可清理、可重试，不能静默覆盖用户文件。

### 20.2 SQLite 与 Shadow Git

Snapshot 使用 manifest、commit/ref 和 workspace identity 关联：

```text
snapshot_id
workspace_id
shadow_repo_id
commit_id
parent_snapshot_id
file_manifest_digest
created_event_id
```

恢复时先校验 Shadow Git 对象和 manifest，再决定是否 materialize 到工作区。Shadow Git 缺对象时项目进入 degraded，不自动把不完整 Snapshot 当作当前工作区。

### 20.3 SQLite 与 Blob

Blob 使用内容寻址：

```text
blob_ref = class + digest + size + media_type + encryption_revision
```

数据库只保存引用、用途、分类和生命周期。Blob 写入失败不能提交“已完成”的事件；事件已提交但 Blob 后续缺失时，必须创建数据完整性 Incident 并阻止相关下载/Run。

### 20.4 SQLite 与 Credential Store

Credential Store 不是同库事务参与者：

- 业务事件只保存 CredentialRef、version、status 和 safe usage summary；
- Credential rotation/revocation 以独立事件收敛；
- 备份恢复后重新验证 Credential version 和 lease；
- OS keyring 锁定、丢失或权限错误导致依赖该 Credential 的操作 fail-closed；
- 不能把恢复备份中的 auth.json 自动当作可信 Secret。

---

## 21. Credential、密钥和证书升级

### 21.1 CredentialRef 稳定性

业务数据只引用：

```text
credential_id
credential_kind
provider
scope
version
status
expires_at
```

升级、备份和迁移不复制 Secret value。Credential Store 通过自己的 backend 维护 material、keyring item、rotation 和 revocation。

### 21.2 轮换流程

```text
create new credential version
  → validate/provider probe（若策略允许）
  → mark pending/active
  → new operations acquire new version
  → old leases drain or revoke
  → emit rotation completed
  → retain metadata according to policy
```

正在运行的 operation 使用启动时获得的短期 lease；轮换不能中途把 Secret 原文写入日志或强行替换进程环境。高风险 rotation 可要求 Approval 和 break-glass 记录。

### 21.3 证书与 TLS

远程 Web、外部 Provider 或 MCP 的证书配置保存引用和摘要。证书更新前执行：

- 证书链和 hostname 校验；
- 有效期和即将过期告警；
- 新旧连接的受控切换；
- 私钥不进入 Event/Log/Support Bundle；
- 失败时保留旧配置并回滚。

### 21.4 密钥丢失和撤销

Credential Store 不可用、版本撤销或证书过期时：

- 立即阻断新使用；
- 已建立连接按策略继续、排空或关闭；
- 生成 Credential health、Audit 和 Incident；
- UI 显示引用、状态、原因和重新绑定入口；
- 不在错误消息中返回后端 Secret Store 的原始详情。

---

## 22. 安全加固与本地权限

### 22.1 文件权限

Unix：

```text
Apex Home                 0700
config/                   0700
apex.db / backups         0600 或目录继承限制
logs/                     0700，文件 0600
sockets/                  0700
Credential compatibility  0600
```

Windows 使用仅当前用户/服务身份可读写的 ACL；安装器和启动器必须检测权限，不满足时进入安全提示或拒绝启动。

### 22.2 进程权限

- 不以 root/Administrator 运行 Core，除非用户明确选择且有强警告；
- 子进程使用尽可能低权限用户、工作目录和环境白名单；
- Tool/MCP/Plugin 只获得 operation-scoped capability；
- 外部命令不继承整个父进程环境；
- 网络、文件、进程和 Credential 权限分别判定；
- Crash handler 不读取 Secret 内存并只生成安全摘要。

### 22.3 网络边界

默认：

- Core 只接受本机 IPC/loopback；
- Provider/MCP 的出站目标由 allowlist、Data Egress Policy 和 Credential scope 共同决定；
- Web Gateway 远程监听必须 TLS；
- 回调、Webhook 和浏览器功能使用短期 state/nonce，禁止把 token 放 URL；
- 外部返回值标记为 untrusted，不能直接改变 Policy/配置或执行命令。

### 22.4 数据最小化

部署、升级、崩溃和支持包默认收集：版本、错误码、计数、耗时、digest、ID 引用和状态，不收集 Prompt、项目文件正文、Credential 原文和完整外部响应。用户启用额外诊断时必须看到范围、用途、TTL 和下载/上传目标。

---

## 23. 资源、容量和磁盘保护

### 23.1 目录级配额

建议为以下对象分别统计：

```text
SQLite database bytes
SQLite WAL bytes
Blob bytes
Snapshot/Shadow Git bytes
Checkpoint/Memory bytes
Logs/Traces bytes
Backups bytes
Temporary/quarantine bytes
```

指标只输出数量和大小，不输出文件内容。项目级和实例级配额由 Core 预检并在写入/快照/备份前判断。

### 23.2 磁盘水位

```text
normal
  → warn: show warning and schedule cleanup
  → stop_optional: stop nonessential snapshots/log export
  → stop_uploads: block external uploads and support bundle export
  → stop_writes: reject new runs and large writes
  → read_only_recovery: allow integrity/backup/restore only
```

水位调整需经过配置 Command 和审计。GC 不得删除仍被事件、Checkpoint、Snapshot 或 Incident 引用的数据。

### 23.3 WAL 与备份容量

Apex 禁用自动 checkpoint，由维护/后台策略根据：

- WAL 字节数；
- 活跃读事务；
- 写入速率；
- 磁盘剩余空间；
- 最近备份和恢复状态；

决定 passive/full/restart checkpoint。长读事务必须有 deadline，不能永久阻塞 checkpoint。

### 23.4 临时空间

Migration、Projection Rebuild、FTS、Backup 和 Restore 预估临时空间。若估算不足，任务进入 blocked，不先创建巨大临时文件再失败。临时目录与目标库应尽量位于同一受控本地文件系统，以便安全 rename。

---

## 24. 运行服务编排

### 24.1 Windows

可选 Windows Service 或用户登录启动项。服务账户必须明确，服务目录与用户项目目录分离时要通过配置绑定。Named Pipe ACL 只允许当前用户/服务身份；停止、升级和恢复由 Service Controller 配合 MaintenanceRun。

### 24.2 macOS

可选 launchd user agent。Tauri Desktop 通常按需启动 `apexd`，退出最后一个客户端后可按 idle timeout 保持或停止。Unix socket、应用支持目录和 keychain access group 必须匹配签名身份。

### 24.3 Linux

推荐 user-level systemd service：

```ini
[Service]
ExecStart=/opt/apex/bin/apexd --home=%h/.local/state/apex
Restart=on-failure
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=read-only
```

生产模板必须按实际项目目录和 Credential Store 能力调整；不能用过宽的 `ReadWritePaths=/` 或把 Secret 通过 `Environment=` 传入。

### 24.4 容器/服务器

容器模式只在显式部署文档和镜像中支持：

- `/data/apex` 是持久卷，禁止 ephemeral-only 运行生产实例；
- `/backup` 与数据库卷分离；
- Credential Store 使用外部 secret manager 或绑定受控 socket；
- 容器内不运行用户提供的任意命令作为 root；
- health/readiness/liveness 分离；
- 优雅终止时间覆盖 StorageWriter drain 和外部 operation reconcile；
- 镜像 digest 固定，运行时不自动下载未经验证的扩展。

---

## 25. 高可用边界与多实例演进

### 25.1 v1 默认不做 SQLite 多写者 HA

单实例 SQLite writer 是 v1 的一致性边界。可以有多个只读客户端、多个外部连接和多个后台 worker，但不允许多个 Core 同时对同一数据库写入。

### 25.2 故障转移策略

在不引入分布式数据库前，故障转移采用：

```text
primary instance stopped/fenced
  → verify storage lock
  → restore or attach database on standby
  → run recovery/integrity
  → issue new endpoint
  → clients reconnect by event_store_id + cursor
```

若数据库被复制到新实例，必须确认复制一致性、WAL 状态、Blob/Snapshot manifest 和 Credential Store 绑定。未完成 fencing 时，禁止 standby 写入。

### 25.3 未来远程团队模式

未来若支持多用户/多租户：

- Core 业务边界仍是唯一事实提交者；
- 租户/项目 ID 成为强制隔离字段；
- Event Store、Blob、Credential、Audit 和日志均不能跨租户查询；
- 远程 Gateway 负责身份映射，不能自己解释领域权限；
- SQLite 可以替换为兼容 Storage Port 的分布式后端，但 Event/Projection/Outbox 语义不变；
- RPO/RTO、审计归档和密钥管理必须重新评审。

---

## 26. 监控、发布门禁与运维指标

### 26.1 部署健康指标

```text
apex_process_ready
apex_process_restarts_total
apex_startup_duration_ms
apex_migration_duration_ms
apex_migration_failures_total
apex_backup_last_success_timestamp
apex_backup_freshness_seconds
apex_backup_verify_failures_total
apex_restore_duration_ms
apex_restore_failures_total
apex_db_integrity_failures_total
apex_db_writer_queue_depth
apex_event_store_head_seq
apex_projection_lag_events
apex_outbox_pending_events
apex_disk_free_bytes
apex_blob_missing_refs
apex_unknown_external_operations
apex_protocol_version_mismatch_total
```

### 26.2 发布门禁

Stable 发布前必须满足：

- 所有目标平台 artifact 签名和 digest 验证通过；
- Release Manifest、SBOM、迁移 checksum、事件注册表和 Projection revision 已发布；
- 全量单元、集成、恢复、升级、回滚和安全回归测试通过；
- 至少完成一次真实备份恢复演练；
- `secret_prohibited` telemetry 测试为零泄漏；
- 关键错误码、readiness、health 和 operator audit 可查询；
- 迁移在代表性数据库规模上满足窗口和临时空间要求；
- 发布说明明确是否支持旧客户端、是否需要重启、是否会阻断写入。

### 26.3 发布观察窗口

升级后的观察窗口至少覆盖：

```text
启动成功率
迁移/恢复耗时
DB busy/locked/full/ioerr
事件提交延迟
Projection/Outbox lag
Provider/MCP 成功率
客户端重连和版本不兼容
Crash/Restart
Disk/WAL/Blob 增长
Secret scanner / redaction blocked
```

异常达到回滚阈值时，自动停止扩大灰度范围并创建 Incident；自动回滚仅适用于已验证的低风险路径。

---

## 27. 备份、恢复与灾备演练

### 27.1 演练周期

- 每个 Release：临时目录恢复和完整性校验；
- 每月：代表性用户数据库、Blob、Snapshot 的完整恢复；
- 每季度：设备/实例丢失后的新环境恢复；
- 每次重大 schema/存储变更：升级前后双向恢复验证；
- 每次 Credential backend 变更：metadata restore + rebind 流程验证。

### 27.2 演练记录

每次演练保存：

- 场景、版本、数据库格式和数据规模；
- 备份来源、digest、event seq 和 Blob manifest；
- 预估与实际 RPO/RTO；
- 缺失对象、未知 operation、投影重建耗时；
- 发现的问题、Incident、修复责任人和截止日期；
- 是否允许继续发布。

演练数据必须使用脱敏或合成数据。不能把生产 Secret 复制到测试恢复环境。

### 27.3 恢复验收

恢复验收至少包含：

```text
can start apexd
can query project/session/run
can replay event cursor
can rebuild panel projections
can inspect audit safely
can see missing/rebound credentials
can avoid executing unknown external effects
can create a new run only when readiness allows
```

---

## 28. 故障处置 Runbook

### 28.1 `apexd` 无法启动

1. 检查 release/build manifest；
2. 检查 single-instance lock、路径权限和 endpoint；
3. 读取安全启动日志和 readiness 报告；
4. 运行只读 SQLite quick check；
5. 若处于 migration，按 MaintenanceRun 恢复或继续；
6. 若完整性失败，隔离原库/WAL，不原地修复；
7. 使用最近验证过的备份恢复；
8. 完成 post-restore health 和 Audit。

### 28.2 数据库磁盘满

1. 进入 stop_optional/stop_writes；
2. 停止外部 telemetry、Snapshot、非关键日志扩张；
3. 检查 WAL、Blob、临时目录、备份和 GC candidates；
4. 只删除已过期且无引用对象；
5. 若无法释放，保留只读查询和恢复能力；
6. 记录 Incident，不建议用户手工删除 `apex.db-wal`。

### 28.3 升级迁移失败

1. 停止新写入并保留失败报告；
2. 判断是否在 Expand/Backfill/Contract；
3. 运行 migration status，不直接重复执行未知步骤；
4. 若可重入，使用相同 checksum 和 cursor 继续；
5. 若不可逆或验证失败，从 pre-migration backup 恢复；
6. 启动旧/新 binary 做兼容确认；
7. 关闭发布灰度并创建 Incident。

### 28.4 外部副作用未知

1. 保留 operation journal、external_operation_id 和 trace/audit 引用；
2. 禁止自动重试；
3. 尝试 provider/MCP 查询状态，或请求人工确认；
4. 更新为 completed/failed/cancelled/unknown；
5. 若产生补偿动作，创建新的明确 Command；
6. 在会话和运维面板显示未决风险。

### 28.5 Credential Store 故障

1. 标记 Credential backend degraded；
2. 阻断需要 Secret 的新操作；
3. 保留不需要 Credential 的 Query/本地分析；
4. 检查 OS keyring/外部 KMS 连接和权限；
5. 修复后重新验证 Credential version 和 lease；
6. 不通过环境变量或数据库明文临时绕过。

---

## 29. Rust Workspace 与部署代码组织

推荐补充以下 crate/目录：

```text
crates/
  apex-bootstrap/
    src/preflight.rs
    src/instance_lock.rs
    src/endpoint.rs
    src/platform_paths.rs
  apex-release/
    src/manifest.rs
    src/signature.rs
    src/compatibility.rs
    src/channel.rs
  apex-migration/
    src/runner.rs
    src/plan.rs
    src/backfill.rs
    src/validation.rs
  apex-recovery/
    src/startup.rs
    src/reconcile.rs
    src/quarantine.rs
  apex-backup/
    src/manifest.rs
    src/sqlite_backup.rs
    src/blob_manifest.rs
    src/restore.rs
    src/verify.rs
  apex-runtime/
    src/supervisor.rs
    src/shutdown.rs
    src/fencing.rs
  packaging/
    windows/
    macos/
    linux/
  deploy/
    systemd/
    launchd/
    windows-service/
    container/
```

### 29.1 Port

```rust
#[async_trait]
pub trait ReleasePort {
    async fn verify_manifest(
        &self,
        artifact: ArtifactRef,
    ) -> Result<ReleaseVerification, ReleaseError>;

    async fn compatibility(
        &self,
        target: ReleaseTarget,
    ) -> Result<CompatibilityReport, ReleaseError>;
}

#[async_trait]
pub trait BackupPort {
    async fn create(
        &self,
        request: BackupRequest,
        auth: AuthContext,
    ) -> Result<BackupReport, BackupError>;

    async fn verify(
        &self,
        backup: BackupRef,
        auth: AuthContext,
    ) -> Result<RestoreVerification, BackupError>;

    async fn restore_preflight(
        &self,
        request: RestoreRequest,
        auth: AuthContext,
    ) -> Result<RestorePlan, BackupError>;
}
```

### 29.2 依赖方向

```text
Domain / Application
        ↓
Release / Migration / Recovery / Backup Ports
        ↓
SQLite / Filesystem / OS Service / Keyring / Process adapters
```

部署代码不能反向依赖 Vue、Actix handler 或 Provider SDK。安装器只调用 bootstrap/release Port，不能直接修改领域表。

---

## 30. 测试与质量门

### 30.1 安装测试

- 干净系统安装、升级安装、卸载保留数据、重新安装；
- 权限不足、路径包含 Unicode、路径过长、共享目录、符号链接和磁盘不足；
- Windows Named Pipe、macOS UDS、Linux systemd/user service；
- Tauri 首次启动、Core 尚未启动、Core 已存在、端口冲突；
- 安装包签名错误、digest 错误、平台错误和撤销版本。

### 30.2 迁移测试

- 每个 migration 从最小支持版本连续升级；
- Expand/Backfill/Contract 中断后恢复；
- 重复执行相同 checksum；
- 中途断电、进程崩溃、磁盘满、WAL 膨胀；
- 旧客户端、新客户端与新 Core 的协议矩阵；
- Projection/FTS rebuild 和未知事件处理。

### 30.3 备份恢复测试

- 数据库、WAL、Blob、Snapshot、Checkpoint 和 Artifact manifest 一致性；
- 缺 Blob、损坏 Blob、错误签名、错误 schema、错误 event_store_id；
- 同库恢复和 Portable Import 的 ID、cursor、审计和 Credential rebind；
- 恢复后不自动执行未知 Tool/MCP/Provider；
- 恢复后创建新 Run 的 readiness 条件。

### 30.4 安全测试

- 安装包和更新器的签名、路径穿越和临时文件权限；
- 远程监听无 TLS、错误代理头、CSRF、CORS 和身份绕过；
- 日志、Crash、备份、诊断包、环境变量和安装器输出 Secret 扫描；
- 多实例/多项目/多用户边界；
- Credential Store 丢失、撤销、轮换和过期；
- 恶意 MCP/Plugin/Provider 返回值和升级 manifest。

### 30.5 混沌测试

```text
kill apexd at each startup/upgrade checkpoint
kill projector/outbox/mcp/provider child
fill disk and corrupt WAL copy
interrupt backup/restore
expire maintenance lease/fence
drop client connection during replay
return duplicate/late external result
remove one blob from backup
rotate credential during active operation
```

每个场景都要验证：事件事实、审计证据、状态终态、恢复动作、用户提示和安全阻断。

---

## 31. 交付阶段与最终拓扑

> ADR-0001（跨文档一致性审查）：本节原把"单机可用基线"标为 v0.5、"三端共享"标为 v0.7，与需求文档 §5.3、系统总体架构 §18、API 协议 §17 和项目开发计划 §2.1 的五档路线图（v0.1/v0.3/v0.5/v0.7/v1.0）整体右移两档。现按基线重标。

### 31.1 v0.1：单机可用基线

```text
TUI/CLI ──native IPC──► apexd
                         ├── SQLite WAL + FTS5
                         ├── project files + shadow Git
                         ├── Provider adapters
                         ├── Tool supervisors
                         └── OS Credential Store
```

交付重点：单实例锁、目录布局、启动恢复、基础迁移、Pre-migration backup、TUI/CLI 协议协商和本机安全默认值。

### 31.2 v0.3：三端共享

```text
TUI ───────────────┐
Tauri shell/Vue ───┼── native protocol ──► apexd
Browser/Vue ─ Actix┘       loopback REST/WS
```

交付重点：Tauri shell、Actix Web Gateway、WebSocket 事件游标、三端会话共享、本机认证与协议协商、多 Provider 打包。

### 31.3 v0.5：编排与扩展接入

交付重点：Extension registry、MCP supervisor、Skill 加载、Memory 索引的目录与备份边界，以及 Write Claim 相关的工作区隔离部署形态。

### 31.4 v0.7：可靠性与运维面

交付重点：Projection rebuild、Outbox、启动恢复与 reconcile 的完整路径、Hook 部署约束、Observability 面板与支持包导出。

### 31.5 v1.0：可发布完整产品

```text
┌─────────────────────────────────────────────┐
│ Signed installer / updater / release channel │
└──────────────────────┬──────────────────────┘
                       ▼
┌─────────────────────────────────────────────┐
│ apexd                                      │
│ Core + Runtime + Storage + Event + Policy  │
│ Observability + Maintenance + Recovery     │
└──────┬───────────────┬───────────────┬──────┘
       ▼               ▼               ▼
   SQLite/FTS       Files/Git/Blob   Credential Store
       │
       ├── TUI/CLI
       ├── Tauri Desktop/Vue
       └── Actix Gateway/Web/Vue
```

v1.0 发布门槛：升级、备份、恢复、回滚、审计、支持包、磁盘保护、未知副作用恢复和 Secret 防泄漏测试全部通过。

### 31.6 v1.x：远程与灾备增强

按需增加：

- 远程 TLS 和组织身份；
- 外部 Backup/KMS/Vault；
- 跨设备 Portable Archive；
- 只读 standby 和受控故障转移；
- 多实例/多租户 Storage Port；
- 可选外部 telemetry，但保持本地事实和审计优先。

---

## 32. ADR 清单

实现前建议冻结：

1. `ADR-DEP-001`：单机单 Core、单 SQLite writer 与 single-instance lock；
2. `ADR-DEP-002`：Apex Home、项目目录和 Credential Store 的边界；
3. `ADR-DEP-003`：Native IPC、loopback fallback 和 handshake token；
4. `ADR-DEP-004`：Release Manifest、签名、SBOM 和撤销策略；
5. `ADR-DEP-005`：数据库 Migration forward-only 与降级策略；
6. `ADR-DEP-006`：Projection/FTS rebuild 与原子切换；
7. `ADR-DEP-007`：Tauri、TUI、Web 与 Core 的兼容窗口；
8. `ADR-DEP-008`：备份内容边界、加密和验证；
9. `ADR-DEP-009`：同库 Restore 与 Portable Import 的 ID 语义；
10. `ADR-DEP-010`：Credential metadata 与 Secret material 分离恢复；
11. `ADR-DEP-011`：单机故障恢复、远程 standby 和 fencing；
12. `ADR-DEP-012`：自动更新、灰度、回滚和发布中止门槛。

---

## 33. 验收标准

### 33.1 部署

- Windows、macOS、Linux 能按支持路径安装并启动 `apexd`；
- 默认仅本机暴露，Native IPC 优先、loopback 回退；
- 同一 Apex Home 不会启动两个 writer；
- TUI、Tauri、Web 均不能绕过 Core 访问数据库或执行工具；
- 目录、文件权限、符号链接和共享文件系统预检有效。

### 33.2 升级

- Release Manifest、签名、digest、schema、protocol 和 extension 兼容性可验证；
- 升级前自动或明确要求生成可验证备份；
- Migration 可观察、可恢复、可阻断，失败不静默丢数据；
- 活跃 Run、外部 operation、Write Claim 和 Credential lease 有安全处理；
- Projection/FTS 可重建，旧客户端在兼容窗口内能安全工作；
- 回滚不仅替换 binary，还验证数据库、Blob、Snapshot 和事件边界。

### 33.3 灾备

- 备份包含数据库、事件范围、Blob/Snapshot manifest 和版本信息；
- 恢复前显示影响范围、缺失对象、Credential rebind 和未知副作用；
- 同库恢复保持原 `event_store_id`，Portable Import 创建新 `event_store_id`；
- 恢复后完成 integrity、projection、Blob、审计和 readiness 检查；
- 任何未知外部副作用都不会自动重放；
- 定期恢复演练产生可查询报告和 Incident/Action Item。

### 33.4 安全

- 安装器、更新器、备份、日志、Crash、环境变量和支持包无 Secret 泄漏；
- 远程监听必须 TLS/身份/CSRF/CORS 预检通过；
- Credential Store 原文不进入普通发布物、数据库备份或项目归档；
- 高风险 Restore、Purge、Break-glass 均有二次确认和不可变审计；
- 供应链签名失败、撤销版本和 manifest 不匹配会 fail-closed。

---

## 附录 A：推荐服务命令

```text
apex daemon start
apex daemon stop
apex daemon status
apex daemon doctor
apex version --verify
apex migration status
apex migration preflight
apex migration run
apex backup create
apex backup list
apex backup verify
apex restore preflight
apex restore run
apex recovery status
apex projection status
apex projection rebuild
apex support-bundle create
apex maintenance list
apex maintenance cancel
```

命令行只负责调用稳定 Command/Query Port。`apex restore run`、`apex migration run`、`apex maintenance cancel` 等高风险命令必须经过 Core capability 和确认 token，不接受任意 SQL 或任意路径删除参数。

---

## 附录 B：推荐环境变量白名单

```text
APEX_HOME
APEX_CONFIG
APEX_LOG_LEVEL
APEX_LOG_FORMAT
APEX_PROFILE
APEX_LISTEN_MODE
APEX_LISTEN_ADDR
APEX_PROVIDER_DEFAULT
APEX_CREDENTIAL_BACKEND
APEX_DISABLE_EXTERNAL_TELEMETRY
APEX_UPDATE_CHANNEL
APEX_MIGRATION_MODE
```

白名单只定义变量名，不意味着变量值都安全。启动器对每个变量按类型、长度、敏感性和允许范围校验；未注册变量不进入诊断包。

---

## 附录 C：备份 Manifest 最小格式

```json
{
  "manifest_version": 1,
  "created_at": "2026-08-08T10:00:00Z",
  "created_by": "actor_...",
  "product_version": "0.5.0",
  "database_format": 5,
  "schema_revision": "2026_08_08_001",
  "event_store_id": "es_...",
  "event_seq_from": 1,
  "event_seq_to": 1842,
  "database_digest": "sha256:...",
  "blob_manifest_digest": "sha256:...",
  "snapshot_manifest_digest": "sha256:...",
  "projection_revisions": {},
  "credential_metadata_only": true,
  "redaction_policy_revision": "redaction-v6",
  "encryption": {
    "enabled": true,
    "key_ref": "keyring:apex-backup-key"
  },
  "artifact_digests": [],
  "signature": {
    "algorithm": "ed25519",
    "key_id": "backup-signing-key"
  }
}
```

---

## 附录 D：版本兼容矩阵模板

| 组件 | 当前版本 | 最低支持 | 最高支持 | 失败行为 |
|---|---:|---:|---:|---|
| `apexd` database format | 5 | 4 | 5 | 高版本拒绝写入 |
| Native protocol | v1 | v1 | v3 | 无交集拒绝连接 |
| REST/WS schema | `apex.v1` | v1 | v1 | 返回版本错误 |
| Event registry | events-v8 | events-v6 | events-v8 | 未知事件阻断投影 |
| Overview projection | v3 | v2 | v3 | refresh/adapter |
| Plugin API | v1 | v1 | v1 | 禁用不兼容插件 |
| Tauri shell | release-specific | previous stable | current | 只读/提示升级 |
| Credential metadata | v3 | v2 | v3 | 重新绑定或降级 |

实际版本发布时必须将模板转为机器可读 manifest，并纳入 CI 验证。

---

## 附录 E：Recovery 事件目录

```text
process.boot.started
process.preflight.completed
process.ready
process.degraded
process.quiescing
process.stopped
process.start_failed

upgrade.requested
upgrade.preflight.completed
upgrade.quiesced
upgrade.migration.started
upgrade.migration.completed
upgrade.validation.failed
upgrade.completed
upgrade.rollback.started
upgrade.rollback.completed

backup.requested
backup.started
backup.completed
backup.verification.completed
backup.verification.failed
restore.requested
restore.started
restore.validation.failed
restore.completed
portable_import.completed

database.integrity_failed
database.read_only_entered
database.watermark_crossed
blob.reference_missing
snapshot.manifest_mismatch
credential.rebind.required
```

---

## 附录 F：与其他详细设计的交叉约束

| 上游设计 | Deployment/DR 必须遵守 |
|---|---|
| 总体架构 | `apexd` 是唯一业务核心和 SQLite writer；三端为薄客户端 |
| 领域模型与事件规范 | Command 幂等、事件不可变、终态不可回退、未知副作用不盲重放 |
| API 与实时协议 | Hello/版本协商、cursor、reconnect、projection refresh、server restart |
| SQLite 数据模型 | WAL、单 writer、forward-only migration、backup API、只读隔离 |
| Agent Runtime | graceful quiesce、active operation reconcile、Supervisor 回收子进程 |
| Tool Gateway | operation_id、capability、外部效果状态和审计安全视图 |
| Rules/Gate | 升级/恢复后保留验证证据、Waiver 和 Skip Spec 语义 |
| Context/Checkpoint | Context/Checkpoint 可重建、敏感内容独立存储、恢复先事实后摘要 |
| Workspace/Snapshot | write intent、digest、Shadow Git、claim/fence 和文件边界 |
| Extension System | manifest、digest、registry generation、扩展能力不静默扩大 |
| Credential Governance | Secret material 与 metadata 分离；恢复后需要 rebind/lease 校验 |
| Observability | MaintenanceRun、Audit、Health、Alert、Incident 和支持包可观察可审计 |

---

## 附录 G：后续设计建议

至此，Apex 的主要架构主题已经覆盖：

1. 系统总体架构；
2. 领域模型与事件规范；
3. API 与实时事件协议；
4. SQLite 数据模型与迁移；
5. Agent Runtime 与 DAG 调度器；
6. Tool Gateway 与权限引擎；
7. Context 与 Checkpoint；
8. Workspace、Snapshot、Write Claim 与隔离工作区；
9. Rules 与 Verification Gate；
10. MCP、Skill、Hook 与 Plugin 扩展；
11. Credential 与敏感数据治理；
12. Observability、审计与运维控制面；
13. Deployment、升级与灾备。

下一阶段不再优先扩写横向架构，而应进入：

- 文档一致性审查与冲突清单；
- ADR 冻结；
- v0.5 可执行任务拆解；
- Cargo Workspace 与数据库 migration skeleton；
- Protocol/事件注册表代码生成；
- 最小可运行 `apexd`、TUI 和测试 fake adapter。
