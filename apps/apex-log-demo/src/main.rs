use std::error::Error;
use std::time::Duration;

use apex_observability::{
    LogConfig, LogLevel, TaskContext, init_file_logging, scope_task, spawn_logged,
};

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> Result<(), Box<dyn Error>> {
    let logging = init_file_logging(
        LogConfig::new("apex-log-demo")
            .with_directory("logs")
            .with_level(LogLevel::Trace),
    )?;
    let root_task = TaskContext::new("apex-log-demo-run");
    let trace_id = root_task.trace_id().to_owned();

    scope_task(root_task, async {
        tracing::info!(
            message_code = "apex.process.started",
            progress_stage = "logging-bootstrap",
            progress_current = 0_u64,
            progress_total = 3_u64,
            "Apex logging demo started"
        );

        let configuration = spawn_logged("configuration-loader", async {
            tracing::debug!(
                message_code = "apex.bootstrap.progress",
                progress_current = 1_u64,
                progress_total = 3_u64,
                "configuration loaded"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        });

        let runtime = spawn_logged("runtime-bootstrap", async {
            tracing::info!(
                message_code = "apex.bootstrap.progress",
                progress_current = 2_u64,
                progress_total = 3_u64,
                "runtime services initialized"
            );
            tokio::time::sleep(Duration::from_millis(10)).await;
        });

        configuration.await?;
        runtime.await?;

        let health_trace_id = trace_id.clone();
        let thread = std::thread::Builder::new()
            .name("apex-health-probe".to_owned())
            .spawn(move || {
                tracing::info!(
                    traceId = health_trace_id,
                    message_code = "apex.health.checked",
                    health = "ready",
                    "health probe completed"
                );
            })?;
        thread.join().map_err(|_| "health probe thread panicked")?;

        tracing::info!(
            message_code = "apex.process.ready",
            progress_stage = "logging-bootstrap",
            progress_current = 3_u64,
            progress_total = 3_u64,
            "Apex logging demo is ready"
        );

        tracing::info!(
            message_code = "apex.process.completed",
            progress_stage = "logging-bootstrap",
            progress_current = 3_u64,
            progress_total = 3_u64,
            "Apex logging demo completed"
        );

        Ok::<(), Box<dyn Error>>(())
    })
    .await?;

    logging.flush()?;
    let path = logging.path().to_path_buf();
    let run_id = logging.run_id().to_owned();
    let dropped = logging.dropped_records();
    logging.shutdown()?;

    println!("run_id={run_id}");
    println!("trace_id={trace_id}");
    println!("log_file={}", path.display());
    println!("dropped_records={dropped}");
    Ok(())
}
