//! Apex local structured logging.
//!
//! The module follows a record/formatter/sink split inspired by FastLog while
//! using Rust's `tracing` call-site metadata. Every pattern-formatted record includes
//! local time, process id, OS thread identity, optional Tokio task identity,
//! run id, monotonic sequence, elapsed time, source file, and source line.

mod config;
mod error;
mod layer;
mod sink;
mod task;

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use chrono::Utc;
use tracing::Dispatch;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::prelude::*;
use tracing_subscriber::registry::Registry;

pub use config::{LogConfig, LogLevel, LogOverflowPolicy};
pub use error::LogError;
pub use task::{TaskContext, scope_task, spawn_logged};
pub use tracing::{debug, error, info, trace, warn};

use crate::layer::ApexFileLayer;
use crate::sink::{FileSinkGuard, prepare_log_path};

/// Owns one process-run file and its writer thread.
pub struct LogRuntime {
    path: std::path::PathBuf,
    run_id: String,
    guard: Option<FileSinkGuard>,
}

impl LogRuntime {
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub fn run_id(&self) -> &str {
        &self.run_id
    }

    /// Waits until all records submitted before the call are on disk.
    pub fn flush(&self) -> Result<(), LogError> {
        self.guard
            .as_ref()
            .ok_or(LogError::WriterUnavailable)?
            .flush()
    }

    #[must_use]
    pub fn dropped_records(&self) -> u64 {
        self.guard
            .as_ref()
            .map_or(0, FileSinkGuard::dropped_records)
    }

    /// Flushes the file and joins the writer thread.
    pub fn shutdown(mut self) -> Result<(), LogError> {
        let Some(mut guard) = self.guard.take() else {
            return Ok(());
        };
        guard.shutdown()
    }
}

/// Installs the process-global file logger.
///
/// Call this before starting the Tokio runtime's application tasks. A process
/// can install one global tracing subscriber; repeated initialization returns a
/// stable error instead of replacing the existing logger.
pub fn init_file_logging(config: LogConfig) -> Result<LogRuntime, LogError> {
    let (dispatch, runtime) = build_file_logging(config)?;
    if tracing::dispatcher::set_global_default(dispatch).is_err() {
        drop(runtime);
        return Err(LogError::GlobalSubscriberAlreadySet);
    }
    Ok(runtime)
}
fn build_file_logging(config: LogConfig) -> Result<(Dispatch, LogRuntime), LogError> {
    let run_id = new_run_id();
    let path = prepare_log_path(&config, &run_id)?;
    let (sink, guard) = FileSinkGuard::start(&config, &path)?;
    let layer = ApexFileLayer::new(
        config.component.clone(),
        run_id.clone(),
        config.source_root.clone(),
        sink,
    );
    let level = LevelFilter::from_level(config.level.as_tracing());
    let subscriber = Registry::default().with(level).with(layer);
    let dispatch = Dispatch::new(subscriber);
    let runtime = LogRuntime {
        path,
        run_id,
        guard: Some(guard),
    };
    Ok((dispatch, runtime))
}

fn new_run_id() -> String {
    static NEXT_RUN: AtomicU64 = AtomicU64::new(1);
    let timestamp = Utc::now().format("%Y%m%dT%H%M%S%6fZ").to_string();
    let sequence = NEXT_RUN.fetch_add(1, Ordering::Relaxed);
    format!("{timestamp}-pid{}-r{sequence}", std::process::id())
}

#[cfg(test)]
mod tests;
