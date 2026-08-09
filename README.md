# Apex

> 一个以 **Spec 驱动开发（Spec-Driven Development）** 为核心、强调规范校验、权限控制与全链路可观测性的开源 AI 编程 Agent。
>
> 当前仓库处于 **v0.1 MVP 的基础设施与垂直切片建设阶段**。现阶段已落地 Rust workspace、结构化文件日志基础设施和可运行的日志示例，TUI、Agent Runtime、SQLite 事件存储、Provider、Tool Gateway 等能力按开发计划逐步实现。

## 项目定位

Apex 计划提供一个统一的本机 Agent 核心，并以 TUI、桌面端和 Web 端作为不同客户端形态。与直接修改代码的传统 Agent 流程不同，Apex 的目标是把需求、设计、任务拆解、实现、验证和审计组织为可追踪的工作流：

```text
需求
  → requirements.md
  → design.md
  → tasks.md
  → 用户确认
  → DAG 任务执行
  → 增量规范检查
  → Verification Gate
  → 验收报告
```

核心设计目标包括：

- **Spec 优先**：先形成可评审的需求、设计和任务文档，再进入实现阶段；
- **质量门强制执行**：通过 Spec 内嵌规则、PostToolUse 检查和 Verification Gate 降低规范偏离；
- **权限与审计**：工具调用、敏感数据和高风险操作具有明确的权限、确认与审计边界；
- **事件与事实优先**：UI 展示基于事件和投影，不以临时文本拼接冒充业务状态；
- **全链路可观测**：Skill、MCP、Tool、SubAgent、任务进度和运行日志都可以被追踪；
- **可恢复运行**：围绕 SQLite、Checkpoint 和 Workspace Snapshot 设计会话恢复与状态重建能力。

## 当前实现状态

### 已实现

- Rust workspace 与统一版本、edition、license 配置；
- `apex-observability` 本地结构化日志 crate；
- 每次进程运行生成独立日志文件；
- `tracing` 调用点采集真实源码文件名和行号；
- 本地时间、毫秒精度、日志级别、PID、线程身份和运行 ID；
- Tokio 逻辑任务的任务 ID、任务名称和继承式 `traceId`；
- 日志队列容量、阻塞/丢弃策略、flush、关闭和写入失败处理；
- 基础敏感字段脱敏以及单物理行日志输出；
- `apex-log-demo` 示例程序，可验证日志初始化、异步任务、线程和优雅关闭流程。

### 规划中

以下能力已在设计和开发计划中定义，尚不代表当前仓库全部可用：

- TUI 生命周期、输入模型和事件渲染；
- In-process Fake Application 与后续 `apexd` Native Transport；
- SQLite Event Store、Projection 和重启恢复；
- Project / Session / Run / Turn 生命周期；
- Provider 抽象与 Agent Loop；
- Permission、Approval、Tool Gateway 和基础文件/命令工具；
- Requirements / Design / Tasks / Implementation / Verification Spec 流程；
- Rules、PostToolUse 和 Verification Gate；
- Context Budget、Checkpoint、Workspace Snapshot 和 Write Claim；
- Skill、MCP、Hook、Plugin 扩展系统；
- Observability 运行面板以及 TUI、桌面端、Web 端共享会话。

详细阶段状态请参阅 [`docs/Apex—— v0.1 MVP逐功能可运行阶段计划.md`](docs/Apex——%20v0.1%20MVP逐功能可运行阶段计划.md)。

## 仓库结构

```text
Apex/
├── apps/
│   └── apex-log-demo/          # 日志基础设施可运行示例
├── crates/
│   └── apex-observability/     # 本地结构化日志与任务上下文
├── docs/                       # 需求、架构、领域模型和分阶段设计文档
├── logs/                       # 本地运行时生成的日志文件（不应提交）
├── Cargo.toml                  # Cargo workspace 配置
├── Cargo.lock                  # workspace 依赖锁定文件
├── rust-toolchain.toml         # 固定 Rust 工具链
└── README.md
```

## 技术栈

- **语言**：Rust 2024 Edition；
- **工具链**：Rust `1.96.1`，并启用 `rustfmt`、`clippy`；
- **异步运行时**：Tokio；
- **日志与诊断**：`tracing`、`tracing-subscriber`、`chrono`；
- **计划中的终端 UI**：`ratatui`、`crossterm`；
- **计划中的持久化**：`rusqlite` + SQLite WAL；
- **计划中的序列化**：`serde`、`serde_json`。

## 环境要求

- Git；
- Rust toolchain `1.96.1`，推荐通过仓库中的 `rust-toolchain.toml` 自动安装/选择；
- Windows、Linux 或 macOS 开发环境；
- 首次构建需要可访问 crates.io 的网络环境，以下载 Cargo 依赖。

检查工具链：

```bash
rustc --version
cargo --version
rustup show active-toolchain
```

## 快速开始

### 1. 构建整个 workspace

```bash
cargo build --workspace
```

### 2. 运行测试

```bash
cargo test --workspace
```

### 3. 运行日志示例

```bash
cargo run -p apex-log-demo
```

示例会：

1. 初始化 `apex-observability`；
2. 创建根任务上下文和 `traceId`；
3. 通过 `spawn_logged` 启动带逻辑身份的 Tokio 子任务；
4. 记录配置加载、运行时启动、健康检查和完成事件；
5. 将日志写入 `logs/` 下的独立文件；
6. flush 并关闭日志写入线程；
7. 在标准输出打印 `run_id`、`trace_id`、日志文件路径和丢弃记录数。

典型输出：

```text
run_id=20260809T091759220111Z-pid18952-r1
trace_id=550e8400-e29b-41d4-a716-446655440000
log_file=logs/apex-log-demo-20260809T091759220111Z-pid18952-r1.log
dropped_records=0
```

> `run_id`、PID、时间戳和 `trace_id` 会随每次运行变化。

## `apex-observability` 使用示例

```rust
use std::error::Error;

use apex_observability::{
    init_file_logging, scope_task, spawn_logged, LogConfig, LogLevel, TaskContext,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let logging = init_file_logging(
        LogConfig::new("my-component")
            .with_directory("logs")
            .with_level(LogLevel::Trace),
    )?;

    let root = TaskContext::new("root-run");
    scope_task(root, async {
        let worker = spawn_logged("worker", async {
            tracing::debug!(
                message_code = "worker.started",
                progress_current = 1_u64,
                progress_total = 1_u64,
                "worker is running"
            );
        });

        worker.await?;
        tracing::info!(message_code = "run.completed", "run completed");
        Ok::<(), Box<dyn Error>>(())
    })
    .await?;

    logging.flush()?;
    logging.shutdown()?;
    Ok(())
}
```

### 日志格式

当前文件日志使用单行、位置式格式，便于人类阅读和后续采集：

```text
2026-08-09 10:00:22.523 DEBUG 33744 --[traceId: ...]-- [           main] apex_log_demo                            main.rs:14 : [messageCode: apex.process.started] Apex logging demo started
```

日志包含本地时间、级别、PID、`traceId`、OS 线程、`tracing` target、源码位置、消息和结构化诊断字段。TRACE/DEBUG 级别会展示更多上下文；INFO/WARN/ERROR 主要展示业务消息。控制字符会被转义，以保证一条事件占用一个物理行。

`apex-observability` 的日志是诊断数据，不是领域事实源。未来领域状态、会话状态和任务状态应以事件存储及其投影为准。

## 常用质量检查

提交代码前建议执行：

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

自动修复格式：

```bash
cargo fmt --all
```

## 文档导航

- [需求分析](docs/Apex——%20需求分析文档.md)：产品定位、目标用户、核心场景和功能需求；
- [系统总体架构](docs/Apex——%20系统总体架构设计.md)：客户端、Core、Runtime、存储和扩展边界；
- [领域模型与事件规范](docs/Apex——%20领域模型与事件规范.md)：领域对象、事件和状态变更约束；
- [项目开发计划](docs/Apex——%20项目开发计划（从最小粒度TUI闭环开始）.md)：从最小 TUI 闭环到完整产品的垂直切片计划；
- [v0.1 MVP 阶段计划](docs/Apex——%20v0.1%20MVP逐功能可运行阶段计划.md)：S00～S24 的可运行阶段、验收标准和命令入口；
- [Agent Runtime 与 DAG 调度器设计](docs/Apex——%20Agent%20Runtime与DAG调度器详细设计.md)；
- [Tool Gateway 与权限引擎设计](docs/Apex——%20Tool%20Gateway与权限引擎详细设计.md)；
- [Rules 与 Verification Gate 设计](docs/Apex——%20Rules与Verification%20Gate详细设计.md)；
- [Context 与 Checkpoint 设计](docs/Apex——%20Context与Checkpoint系统详细设计.md)；
- [MCP、Skill、Hook 与 Plugin 设计](docs/Apex——%20MCP、Skill、Hook与Plugin扩展系统详细设计.md)；
- [Observability、审计与运维控制面设计](docs/Apex——%20Observability、审计与运维控制面详细设计.md)。

## 开发原则

1. **先垂直闭环，后横向扩展**：优先让用户能够启动、输入、执行、看到状态并恢复；
2. **TUI 是客户端，不是业务核心**：业务逻辑通过 Application/Core 边界实现；
3. **事实优先于显示**：显示层消费事件和查询投影；
4. **安全先于功能**：权限判定、敏感数据治理、高危操作确认和审计不可被临时绕过；
5. **质量门不可延期**：格式化、Clippy、测试和文档一致性从第一阶段开始执行。

## 版本与许可证

当前 workspace 版本为 `0.1.0`，仍处于早期开发阶段，API 和目录结构可能发生变化。

workspace 元数据声明采用 `MIT OR Apache-2.0` 双许可证。当前仓库尚未包含对应的许可证正文文件，首次公开发布前应补充 `LICENSE-MIT` 与 `LICENSE-APACHE`。

## 贡献

欢迎通过 Issue 或 Pull Request 参与设计和实现。提交改动时请：

- 明确改动属于哪个阶段或设计文档；
- 保持任务边界和写入路径清晰，避免跨模块隐式修改；
- 为新增行为补充测试和结构化日志；
- 执行 `cargo fmt --all -- --check`、Clippy 和 workspace 测试；
- 若改变架构、协议或领域约束，同步更新相关文档和 ADR。



