# apex-observability

Apex v0.1 的本地单行文本日志基础设施。设计借鉴 FastLog 的 `record → formatter → sink` 分层，Rust 实现使用 `tracing` 捕获真实调用点，并由独立文件写入线程落盘。

## 输出格式

文件日志使用与 Spring Boot Console Log Pattern 对齐的位置式格式（文件中不输出 ANSI 颜色控制码）：

```text
%d{yyyy-MM-dd HH:mm:ss.SSS} %5p ${PID} --[traceId: %X{traceId}]-- [%15.15t] %-40.40logger{39} %M:%L : %m %wEx
```

实际记录示例：

```text
2026-08-09 10:00:22.523 DEBUG 33744 --[traceId: 550e8400-e29b-41d4-a716-446655440000]-- [           main] apex_log_demo                            main.rs:14 : [messageCode: apex.process.started] [runId: ...] Apex logging demo started
```

其中 Rust 没有 JVM `%M` 的稳定运行时方法名元数据，因此 `%M:%L` 使用更可靠的**真实源码文件名:代码行号**实现。`logger` 使用 `tracing` target；仅在 TRACE/DEBUG 级别，Tokio 协程身份、稳定事件码和其余结构化字段作为诊断上下文放置在主消息之前；INFO/WARN/ERROR 级别只输出主消息。缺失的 `traceId` 显示为空白；任务上下文存在时自动生成并继承 UUID。

## 每条记录

- 本地时间，毫秒精度；
- 固定宽度日志级别；
- 进程 PID；
- 任务链路 `traceId`；
- OS 线程名称（无名称时使用线程 ID）；
- logger/target；
- 真实源码文件和代码行号；
- 消息、异常信息；
- `runId`、`messageCode`、Tokio 协程 ID/名称和业务进度字段。

换行、回车、制表符和其他控制字符会转义，保证一条日志只占一个物理行。敏感字段名仍执行基础脱敏。

## 使用

```rust
let logging = apex_observability::init_file_logging(
    apex_observability::LogConfig::new("apex-tui")
        .with_directory("logs")
)?;

tracing::info!(
    traceId = "550e8400-e29b-41d4-a716-446655440000",
    message_code = "tui.bootstrap.progress",
    progress_current = 1_u64,
    progress_total = 3_u64,
    "terminal initialized"
);

let task = apex_observability::spawn_logged("event-loop", async move {
    tracing::debug!(message_code = "tui.event.received", "input received");
});
task.await?;
logging.shutdown()?;
```

所有 Apex Tokio 后台任务应通过 `spawn_logged` 或 `scope_task` 注册逻辑协程身份。日志是诊断数据，不是领域事实源。



