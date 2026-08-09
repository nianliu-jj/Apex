use std::path::{Path, PathBuf};

use tracing::Level;

/// File logging severity threshold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub(crate) const fn as_tracing(self) -> Level {
        match self {
            Self::Trace => Level::TRACE,
            Self::Debug => Level::DEBUG,
            Self::Info => Level::INFO,
            Self::Warn => Level::WARN,
            Self::Error => Level::ERROR,
        }
    }
}

/// Behavior when the bounded writer queue is full.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogOverflowPolicy {
    /// Wait until the file writer can accept the record. This is the default so
    /// execution-progress diagnostics are not silently lost.
    Block,
    /// Drop the newest record and increment the dropped-record counter.
    DropNewest,
}

/// Configuration for one process-run log file.
#[derive(Debug, Clone)]
pub struct LogConfig {
    pub(crate) component: String,
    pub(crate) directory: PathBuf,
    pub(crate) source_root: PathBuf,
    pub(crate) level: LogLevel,
    pub(crate) queue_capacity: usize,
    pub(crate) overflow_policy: LogOverflowPolicy,
    pub(crate) flush_each_event: bool,
}

impl LogConfig {
    /// Creates a configuration that writes one pattern-formatted line per event under `./logs`.
    #[must_use]
    pub fn new(component: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            directory: PathBuf::from("logs"),
            source_root: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            level: LogLevel::Debug,
            queue_capacity: 8_192,
            overflow_policy: LogOverflowPolicy::Block,
            flush_each_event: true,
        }
    }

    #[must_use]
    pub fn with_directory(mut self, directory: impl Into<PathBuf>) -> Self {
        self.directory = directory.into();
        self
    }

    #[must_use]
    pub fn with_source_root(mut self, source_root: impl Into<PathBuf>) -> Self {
        self.source_root = source_root.into();
        self
    }

    #[must_use]
    pub const fn with_level(mut self, level: LogLevel) -> Self {
        self.level = level;
        self
    }

    #[must_use]
    pub const fn with_queue_capacity(mut self, queue_capacity: usize) -> Self {
        self.queue_capacity = queue_capacity;
        self
    }

    #[must_use]
    pub const fn with_overflow_policy(mut self, overflow_policy: LogOverflowPolicy) -> Self {
        self.overflow_policy = overflow_policy;
        self
    }

    #[must_use]
    pub const fn with_flush_each_event(mut self, flush_each_event: bool) -> Self {
        self.flush_each_event = flush_each_event;
        self
    }

    #[must_use]
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    #[must_use]
    pub fn component(&self) -> &str {
        &self.component
    }
}
