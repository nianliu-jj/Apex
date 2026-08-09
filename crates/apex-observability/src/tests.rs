use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use super::*;

static NEXT_TEMP: AtomicU64 = AtomicU64::new(1);

fn temp_log_dir(test_name: &str) -> PathBuf {
    let sequence = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "apex-observability-{test_name}-{}-{sequence}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create test log directory");
    path
}

fn read_lines(runtime: &LogRuntime) -> Vec<String> {
    runtime.flush().expect("flush logs");
    fs::read_to_string(runtime.path())
        .expect("read log file")
        .lines()
        .map(ToOwned::to_owned)
        .collect()
}

#[test]
fn records_trace_id_thread_process_and_file_name_line() {
    let directory = temp_log_dir("metadata");
    let (dispatch, runtime) = build_file_logging(
        LogConfig::new("metadata-test")
            .with_directory(&directory)
            .with_source_root(env!("CARGO_MANIFEST_DIR")),
    )
    .expect("build file logging");

    let expected_line = line!() + 2;
    tracing::dispatcher::with_default(&dispatch, || {
        tracing::debug!(
            traceId = "550e8400-e29b-41d4-a716-446655440000",
            message_code = "test.progress",
            progress_current = 1_u64,
            progress_total = 3_u64,
            api_key = "must-not-leak",
            "metadata checkpoint"
        );
    });

    let records = read_lines(&runtime);
    let record = records.last().expect("one log record");
    assert_eq!(record.lines().count(), 1);
    assert!(record.contains("--[traceId: 550e8400-e29b-41d4-a716-446655440000]--"));
    assert!(record.contains("metadata checkpoint"));
    assert!(record.contains("[messageCode: test.progress]"));
    assert!(record.contains("[progressCurrent: 1]"));
    assert!(record.contains("[apiKey: [REDACTED]]"));
    let context_index = record
        .find("[messageCode: test.progress]")
        .expect("message code context");
    let message_index = record
        .find("metadata checkpoint")
        .expect("main log message");
    assert!(context_index < message_index);
    assert!(record.contains(&format!("tests.rs:{expected_line} :")));
    assert!(!record.contains("D:\\AiAgent\\"));
    assert!(!record.contains("must-not-leak"));
    assert_timestamp_prefix(record);

    runtime.shutdown().expect("shutdown log writer");
    fs::remove_dir_all(directory).expect("remove test logs");
}

#[test]
fn generates_uuid_trace_id_for_a_task_chain() {
    let directory = temp_log_dir("generated-trace");
    let (dispatch, runtime) =
        build_file_logging(LogConfig::new("trace-test").with_directory(&directory))
            .expect("build file logging");
    let tokio_runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build tokio runtime");

    tracing::dispatcher::with_default(&dispatch, || {
        tokio_runtime.block_on(scope_task(
            TaskContext::with_id("task-fixed-1", "checkpoint-builder"),
            async {
                tracing::info!("first trace event");
                tracing::info!("second trace event");
            },
        ));
    });

    let records = read_lines(&runtime);
    assert_eq!(records.len(), 2);
    let first_trace = trace_id_from_line(&records[0]);
    let second_trace = trace_id_from_line(&records[1]);
    assert!(is_uuid(&first_trace));
    assert_eq!(first_trace, second_trace);

    runtime.shutdown().expect("shutdown log writer");
    fs::remove_dir_all(directory).expect("remove test logs");
}

#[test]
fn escapes_multiline_values_into_one_physical_line() {
    let directory = temp_log_dir("single-line");
    let (dispatch, runtime) =
        build_file_logging(LogConfig::new("single-line-test").with_directory(&directory))
            .expect("build file logging");

    tracing::dispatcher::with_default(&dispatch, || {
        tracing::debug!(detail = "first\nsecond\r\nthird", "message\ncontinued");
    });

    let records = read_lines(&runtime);
    assert_eq!(records.len(), 1);
    assert!(records[0].contains("[detail: first\\nsecond\\r\\nthird] message\\ncontinued"));

    runtime.shutdown().expect("shutdown log writer");
    fs::remove_dir_all(directory).expect("remove test logs");
}

#[test]
fn records_tokio_coroutine_identity_before_message() {
    let directory = temp_log_dir("task");
    let (dispatch, runtime) = build_file_logging(
        LogConfig::new("task-test")
            .with_directory(&directory)
            .with_source_root(env!("CARGO_MANIFEST_DIR")),
    )
    .expect("build file logging");
    let tokio_runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build tokio runtime");

    tracing::dispatcher::with_default(&dispatch, || {
        tokio_runtime.block_on(scope_task(
            TaskContext::with_id("task-fixed-1", "checkpoint-builder"),
            async {
                tracing::debug!(
                    message_code = "test.task.progress",
                    progress_percent = 50_u64,
                    "task reached midpoint"
                );
            },
        ));
    });

    let records = read_lines(&runtime);
    let record = records.last().expect("task log record");
    assert!(record.contains("[coroutine: task-fixed-1/checkpoint-builder]"));
    assert!(record.contains("[progressPercent: 50]"));

    runtime.shutdown().expect("shutdown log writer");
    fs::remove_dir_all(directory).expect("remove test logs");
}

#[test]
fn hides_error_context_for_error_level() {
    let directory = temp_log_dir("exception");
    let (dispatch, runtime) =
        build_file_logging(LogConfig::new("exception-test").with_directory(&directory))
            .expect("build file logging");

    tracing::dispatcher::with_default(&dispatch, || {
        tracing::error!(
            error = "connection failed\nretry exhausted",
            "provider failed"
        );
    });

    let records = read_lines(&runtime);
    assert_eq!(records.len(), 1);
    assert!(records[0].contains("provider failed"));
    assert!(!records[0].contains("[exception: connection failed\\nretry exhausted]"));

    runtime.shutdown().expect("shutdown log writer");
    fs::remove_dir_all(directory).expect("remove test logs");
}

#[test]
fn shows_diagnostic_context_only_for_trace_and_debug() {
    let directory = temp_log_dir("diagnostic-level");
    let (dispatch, runtime) = build_file_logging(
        LogConfig::new("diagnostic-level-test")
            .with_directory(&directory)
            .with_level(LogLevel::Trace),
    )
    .expect("build file logging");

    tracing::dispatcher::with_default(&dispatch, || {
        tracing::trace!(
            message_code = "trace.context",
            detail = "trace-value",
            "trace message"
        );
        tracing::debug!(
            message_code = "debug.context",
            detail = "debug-value",
            "debug message"
        );
        tracing::info!(
            message_code = "info.context",
            detail = "info-value",
            "info message"
        );
        tracing::warn!(
            message_code = "warn.context",
            detail = "warn-value",
            "warn message"
        );
        tracing::error!(
            message_code = "error.context",
            detail = "error-value",
            "error message"
        );
    });

    let records = read_lines(&runtime);
    assert_eq!(records.len(), 5);
    for (record, (code, detail, message, shows_context)) in records.iter().zip([
        ("trace.context", "trace-value", "trace message", true),
        ("debug.context", "debug-value", "debug message", true),
        ("info.context", "info-value", "info message", false),
        ("warn.context", "warn-value", "warn message", false),
        ("error.context", "error-value", "error message", false),
    ]) {
        assert!(record.contains(message));
        if shows_context {
            assert!(record.contains(&format!("[messageCode: {code}]")));
            assert!(record.contains(&format!("[detail: {detail}]")));
        } else {
            assert!(!record.contains(&format!("[messageCode: {code}]")));
            assert!(!record.contains(&format!("[detail: {detail}]")));
        }
    }

    runtime.shutdown().expect("shutdown log writer");
    fs::remove_dir_all(directory).expect("remove test logs");
}

#[test]
fn creates_a_distinct_file_for_each_run() {
    let directory = temp_log_dir("run-file");
    let (_, first) = build_file_logging(LogConfig::new("apex").with_directory(&directory))
        .expect("build first logger");
    let (_, second) = build_file_logging(LogConfig::new("apex").with_directory(&directory))
        .expect("build second logger");

    assert_ne!(first.run_id(), second.run_id());
    assert_ne!(first.path(), second.path());
    assert!(first.path().exists());
    assert!(second.path().exists());

    first.shutdown().expect("shutdown first logger");
    second.shutdown().expect("shutdown second logger");
    fs::remove_dir_all(directory).expect("remove test logs");
}

#[test]
fn reports_log_startup_failure_without_panicking() {
    let directory = temp_log_dir("startup-failure");
    let blocking_file = directory.join("not-a-directory");
    fs::write(&blocking_file, "occupied").expect("create blocking file");

    let result = build_file_logging(
        LogConfig::new("failure-test").with_directory(blocking_file.join("logs")),
    );

    assert!(matches!(result, Err(LogError::Io { .. })));
    fs::remove_dir_all(directory).expect("remove test logs");
}

#[test]
fn records_interrupted_tokio_task() {
    let directory = temp_log_dir("task-interrupted");
    let (dispatch, runtime) = build_file_logging(
        LogConfig::new("task-interrupted-test")
            .with_directory(&directory)
            .with_source_root(env!("CARGO_MANIFEST_DIR")),
    )
    .expect("build file logging");
    let tokio_runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build tokio runtime");

    tracing::dispatcher::with_default(&dispatch, || {
        tokio_runtime.block_on(async {
            let task = spawn_logged("cancelled-worker", async {
                std::future::pending::<()>().await;
            });
            tokio::task::yield_now().await;
            task.abort();
            let _ = task.await;
        });
    });

    let records = read_lines(&runtime);
    assert!(
        records
            .iter()
            .any(|record| record.contains("logical task was cancelled or unwound"))
    );
    assert!(
        !records
            .iter()
            .any(|record| record.contains("[messageCode: runtime.task.interrupted]"))
    );

    runtime.shutdown().expect("shutdown log writer");
    fs::remove_dir_all(directory).expect("remove test logs");
}

fn assert_timestamp_prefix(record: &str) {
    let bytes = record.as_bytes();
    assert!(bytes.len() > 23);
    assert_eq!(bytes[4], b'-');
    assert_eq!(bytes[7], b'-');
    assert_eq!(bytes[10], b' ');
    assert_eq!(bytes[13], b':');
    assert_eq!(bytes[16], b':');
    assert_eq!(bytes[19], b'.');
    assert_eq!(bytes[23], b' ');
    assert!(
        record[..4]
            .chars()
            .all(|character| character.is_ascii_digit())
    );
}

fn trace_id_from_line(line: &str) -> String {
    let marker = "--[traceId: ";
    let start = line.find(marker).expect("trace id marker") + marker.len();
    let remainder = &line[start..];
    remainder
        .split("]--")
        .next()
        .unwrap_or(remainder)
        .to_owned()
}

fn is_uuid(value: &str) -> bool {
    value.len() == 36
        && value.chars().enumerate().all(|(index, character)| {
            matches!(index, 8 | 13 | 18 | 23) && character == '-'
                || !matches!(index, 8 | 13 | 18 | 23) && character.is_ascii_hexdigit()
        })
}
