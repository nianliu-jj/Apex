use std::fs::{File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use crate::{LogConfig, LogError, LogOverflowPolicy};

enum WriterCommand {
    Write(String),
    Flush(mpsc::Sender<Result<(), String>>),
    Shutdown(mpsc::Sender<Result<(), String>>),
}

#[derive(Default)]
struct SinkState {
    dropped_records: AtomicU64,
    last_error: Mutex<Option<String>>,
}

/// Cloneable producer side owned by the tracing layer.
#[derive(Clone)]
pub(crate) struct FileSink {
    sender: SyncSender<WriterCommand>,
    state: Arc<SinkState>,
    overflow_policy: LogOverflowPolicy,
}

impl FileSink {
    pub(crate) fn write_line(&self, line: String) {
        let result = match self.overflow_policy {
            LogOverflowPolicy::Block => self.sender.send(WriterCommand::Write(line)),
            LogOverflowPolicy::DropNewest => match self.sender.try_send(WriterCommand::Write(line))
            {
                Ok(()) => return,
                Err(TrySendError::Full(_)) => {
                    self.state.dropped_records.fetch_add(1, Ordering::Relaxed);
                    return;
                }
                Err(TrySendError::Disconnected(command)) => Err(mpsc::SendError(command)),
            },
        };

        if result.is_err() {
            self.state.dropped_records.fetch_add(1, Ordering::Relaxed);
            self.store_error("log writer channel disconnected".to_owned());
        }
    }

    fn store_error(&self, message: String) {
        if let Ok(mut error) = self.state.last_error.lock() {
            *error = Some(message);
        }
    }

    pub(crate) fn dropped_records(&self) -> u64 {
        self.state.dropped_records.load(Ordering::Relaxed)
    }

    pub(crate) fn last_error(&self) -> Option<String> {
        self.state.error_snapshot()
    }
}

impl SinkState {
    fn error_snapshot(&self) -> Option<String> {
        self.last_error.lock().ok().and_then(|error| error.clone())
    }
}

/// Lifecycle owner for the writer thread.
pub(crate) struct FileSinkGuard {
    sink: FileSink,
    worker: Option<JoinHandle<()>>,
    closed: bool,
}

impl FileSinkGuard {
    pub(crate) fn start(config: &LogConfig, path: &Path) -> Result<(FileSink, Self), LogError> {
        if config.queue_capacity == 0 {
            return Err(LogError::InvalidQueueCapacity);
        }

        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path)
            .map_err(|source| LogError::Io {
                path: path.to_path_buf(),
                source,
            })?;
        let (sender, receiver) = mpsc::sync_channel(config.queue_capacity);
        let state = Arc::new(SinkState::default());
        let worker_state = Arc::clone(&state);
        let flush_each_event = config.flush_each_event;
        let worker = thread::Builder::new()
            .name("apex-log-writer".to_owned())
            .spawn(move || writer_loop(file, receiver, worker_state, flush_each_event))
            .map_err(|source| LogError::Io {
                path: path.to_path_buf(),
                source,
            })?;

        let sink = FileSink {
            sender,
            state,
            overflow_policy: config.overflow_policy,
        };
        let guard = Self {
            sink: sink.clone(),
            worker: Some(worker),
            closed: false,
        };
        Ok((sink, guard))
    }

    pub(crate) fn flush(&self) -> Result<(), LogError> {
        if let Some(error) = self.sink.last_error() {
            return Err(LogError::WriterFailure(error));
        }
        let (reply_sender, reply_receiver) = mpsc::channel();
        self.sink
            .sender
            .send(WriterCommand::Flush(reply_sender))
            .map_err(|_| LogError::WriterUnavailable)?;
        reply_receiver
            .recv()
            .map_err(|_| LogError::WriterUnavailable)?
            .map_err(LogError::WriterFailure)
    }

    pub(crate) fn dropped_records(&self) -> u64 {
        self.sink.dropped_records()
    }

    pub(crate) fn shutdown(&mut self) -> Result<(), LogError> {
        if self.closed {
            return Ok(());
        }
        self.closed = true;

        let (reply_sender, reply_receiver) = mpsc::channel();
        let response = self
            .sink
            .sender
            .send(WriterCommand::Shutdown(reply_sender))
            .map_err(|_| LogError::WriterUnavailable)
            .and_then(|()| {
                reply_receiver
                    .recv()
                    .map_err(|_| LogError::WriterUnavailable)?
                    .map_err(LogError::WriterFailure)
            });

        let join_result = self.worker.take().map_or(Ok(()), |worker| {
            worker.join().map_err(|_| LogError::WriterPanicked)
        });
        response.and(join_result)
    }
}

impl Drop for FileSinkGuard {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

fn writer_loop(
    file: File,
    receiver: Receiver<WriterCommand>,
    state: Arc<SinkState>,
    flush_each_event: bool,
) {
    let mut writer = BufWriter::new(file);
    while let Ok(command) = receiver.recv() {
        match command {
            WriterCommand::Write(line) => {
                if let Err(error) = write_record(&mut writer, &line, flush_each_event) {
                    store_worker_error(&state, error);
                }
            }
            WriterCommand::Flush(reply) => {
                let result = writer.flush().map_err(|error| error.to_string());
                if let Err(message) = &result {
                    store_worker_error_message(&state, message.clone());
                }
                let _ = reply.send(result);
            }
            WriterCommand::Shutdown(reply) => {
                let result = writer.flush().map_err(|error| error.to_string());
                if let Err(message) = &result {
                    store_worker_error_message(&state, message.clone());
                }
                let _ = reply.send(result);
                break;
            }
        }
    }
}

fn write_record(
    writer: &mut BufWriter<File>,
    line: &str,
    flush_each_event: bool,
) -> io::Result<()> {
    writer.write_all(line.as_bytes())?;
    writer.write_all(b"\n")?;
    if flush_each_event {
        writer.flush()?;
    }
    Ok(())
}

fn store_worker_error(state: &SinkState, error: io::Error) {
    store_worker_error_message(state, error.to_string());
}

fn store_worker_error_message(state: &SinkState, message: String) {
    if let Ok(mut last_error) = state.last_error.lock() {
        *last_error = Some(message);
    }
}

pub(crate) fn prepare_log_path(config: &LogConfig, run_id: &str) -> Result<PathBuf, LogError> {
    let directory = if config.directory.is_absolute() {
        config.directory.clone()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(&config.directory)
    };
    std::fs::create_dir_all(&directory).map_err(|source| LogError::Io {
        path: directory.clone(),
        source,
    })?;
    let directory = directory.canonicalize().unwrap_or(directory);
    let component = sanitize_component(&config.component);
    Ok(directory.join(format!("{component}-{run_id}.log")))
}

fn sanitize_component(component: &str) -> String {
    let sanitized: String = component
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '-'
            }
        })
        .collect();
    if sanitized.is_empty() {
        "apex".to_owned()
    } else {
        sanitized
    }
}
