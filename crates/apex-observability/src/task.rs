use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::task::JoinHandle;

static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(1);
static NEXT_TRACE_ID: AtomicU64 = AtomicU64::new(1);

tokio::task_local! {
    static TASK_CONTEXT: TaskContext;
}

/// Logical coroutine/task identity and trace identity injected into every event
/// emitted in its scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskContext {
    id: String,
    name: String,
    trace_id: String,
}

impl TaskContext {
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self::with_trace_id(new_trace_id(), name)
    }

    #[must_use]
    pub fn with_id(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self::with_trace_id(new_trace_id(), name).with_task_id(id)
    }

    #[must_use]
    pub fn with_trace_id(trace_id: impl Into<String>, name: impl Into<String>) -> Self {
        let sequence = NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed);
        Self {
            id: format!("task-{}-{sequence}", std::process::id()),
            name: name.into(),
            trace_id: trace_id.into(),
        }
    }

    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// UUID identifying one complete task execution chain.
    #[must_use]
    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    fn with_task_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }
}

pub(crate) fn current_task_context() -> Option<TaskContext> {
    TASK_CONTEXT.try_with(Clone::clone).ok()
}

/// Runs a future inside an explicit logical task/coroutine scope.
pub async fn scope_task<F>(context: TaskContext, future: F) -> F::Output
where
    F: Future,
{
    TASK_CONTEXT.scope(context, future).await
}

/// Spawns a Tokio task with a stable task id/name and lifecycle progress records.
/// A child task inherits the current trace id so its complete execution chain can
/// be queried with one UUID.
pub fn spawn_logged<F>(name: impl Into<String>, future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    let trace_id =
        current_task_context().map_or_else(new_trace_id, |context| context.trace_id().to_owned());
    let context = TaskContext::with_trace_id(trace_id, name);
    tokio::spawn(TASK_CONTEXT.scope(context, async move {
        tracing::info!(
            message_code = "runtime.task.started",
            "logical task started"
        );
        let mut completion = TaskCompletionGuard { completed: false };
        let output = future.await;
        completion.completed = true;
        tracing::info!(
            message_code = "runtime.task.completed",
            "logical task completed"
        );
        output
    }))
}

fn new_trace_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0_u128, |duration| duration.as_nanos());
    let sequence = NEXT_TRACE_ID.fetch_add(1, Ordering::Relaxed) as u128;
    let pid = u128::from(std::process::id());
    let thread = format!("{:?}", std::thread::current().id());
    let thread_hash = thread.bytes().fold(0_u128, |hash, byte| {
        hash.wrapping_mul(257).wrapping_add(u128::from(byte))
    });
    let mut high = timestamp ^ sequence.rotate_left(17) ^ pid.rotate_left(41);
    let mut low = timestamp.rotate_left(53) ^ thread_hash ^ sequence.rotate_left(89);
    high ^= high >> 29;
    low ^= low >> 31;

    // RFC 4122 UUID variant and version bits. This is a process-local tracing
    // identity, not a credential or cryptographic nonce.
    high = (high & 0xffff_ffff_ffff_0fff) | 0x0000_0000_0000_4000;
    low = (low & 0x3fff_ffff_ffff_ffff) | 0x8000_0000_0000_0000;
    let compact = format!("{high:016x}{low:016x}", high = high, low = low);
    format!(
        "{}-{}-{}-{}-{}",
        &compact[0..8],
        &compact[8..12],
        &compact[12..16],
        &compact[16..20],
        &compact[20..32]
    )
}

struct TaskCompletionGuard {
    completed: bool,
}

impl Drop for TaskCompletionGuard {
    fn drop(&mut self) {
        if !self.completed {
            tracing::warn!(
                message_code = "runtime.task.interrupted",
                "logical task was cancelled or unwound"
            );
        }
    }
}
