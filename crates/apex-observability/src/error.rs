use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;
use std::path::PathBuf;

/// Errors that can occur while initializing or draining the local log sink.
#[derive(Debug)]
pub enum LogError {
    Io { path: PathBuf, source: io::Error },
    GlobalSubscriberAlreadySet,
    WriterUnavailable,
    WriterFailure(String),
    WriterPanicked,
    InvalidQueueCapacity,
}

impl Display for LogError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(formatter, "log I/O failed for {}: {source}", path.display())
            }
            Self::GlobalSubscriberAlreadySet => {
                formatter.write_str("the global tracing subscriber is already installed")
            }
            Self::WriterUnavailable => formatter.write_str("the log writer is unavailable"),
            Self::WriterFailure(message) => write!(formatter, "the log writer failed: {message}"),
            Self::WriterPanicked => formatter.write_str("the log writer thread panicked"),
            Self::InvalidQueueCapacity => {
                formatter.write_str("the log writer queue capacity must be greater than zero")
            }
        }
    }
}

impl Error for LogError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}
