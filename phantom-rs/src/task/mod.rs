// Cargo.toml dependencies (for reference):
// [dependencies]
// tokio = { version = "1.29", features = ["rt-multi-thread", "macros"] }
// tokio-util = "0.8"
// futures = "0.3"

use futures::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

/// An object‐safe trait for “something that can be cancelled and then awaited (joined)”.
///
/// - `cancel(&self)`: Issues a cancellation signal (e.g. via channel, `CancellationToken`, etc).
/// - `join(self)`: Consumes the task and returns a boxed Future you can `.await`.
///
/// Because `async fn` in traits is not directly object‐safe, we manually box the future.
pub trait CancellableTask: Send + 'static {
    /// Request cancellation (nonblocking). Implementers might, e.g., send on a channel
    /// or call `CancellationToken::cancel()`.
    fn cancel(&self);

    /// Consume `self` and return a boxed Future that resolves when the task is done.
    /// This must be object‐safe, so we return `Pin<Box<dyn Future<Output = ()> + Send>>`.
    fn join(self: Box<Self>) -> Pin<Box<dyn Future<Output = ()> + Send>>;
}

/// A concrete `CancellableTask` implementation built on Tokio’s `JoinHandle<()>` plus
/// a `CancellationToken`. When `cancel()` is called, we cancel the token;
/// the spawned task should be written to .await that token and exit early.
/// Then `join()` simply awaits the `JoinHandle`.
pub struct TokioTask {
    handle: JoinHandle<()>,
    token: CancellationToken,
}

impl TokioTask {
    /// Spawn a new Tokio task that listens for the provided `CancellationToken`.
    ///
    /// - `f` is the async work you want to do. Inside `f`, you should periodically
    ///   check `token.cancelled().await` or `.is_cancelled()` (depending on your logic)
    ///   to exit early if cancellation was requested.
    pub fn spawn<Fn, Fut>(block: Fn) -> Self
    where
        Fut: Future<Output = ()> + Send + 'static,
        Fn: FnOnce(CancellationToken) -> Fut + Send + 'static,
    {
        let token = CancellationToken::new();
        let f = block(token.clone());

        let inner_token = token.clone();
        let handle = tokio::spawn(async move {
            tokio::select! {
                _ = inner_token.cancelled() => {
                    // The token was cancelled—exit early.
                    // (You could do cleanup work here if needed, before returning.)
                }
                _ = f => {
                    // The inner future finished normally.
                }
            }
        });

        TokioTask { handle, token }
    }
}

impl CancellableTask for TokioTask {
    fn cancel(&self) {
        // Signal cancellation. The running task is listening on `self.token`.
        self.token.cancel();
    }

    fn join(self: Box<Self>) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(async move {
            // Await the JoinHandle to ensure the task has fully shut down.
            let _ = self.handle.await;
        })
    }
}

/// A “manager” that holds many `Box<dyn CancellableTask>`. Internally it uses
/// `Arc<Mutex<Vec<…>>>` so that any clone of `TaskManager` can add tasks or
/// later call `shutdown(&self)`. Because the `Vec` is wrapped in a `Mutex`,
/// you never need a `&mut self` to modify it—just `&self`.
#[derive(Clone)]
pub struct TaskManager {
    inner: Arc<Mutex<Vec<Box<dyn CancellableTask + Send>>>>,
}

impl TaskManager {
    /// Create a new, empty TaskManager.
    pub fn new() -> Self {
        TaskManager {
            inner: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Insert a new task into our list. This takes any boxed `CancellableTask`.
    /// Typically you’ll do something like:
    ///
    /// ```ignore
    /// let token = CancellationToken::new();
    /// let my_task = TokioTask::spawn(token.clone(), async move {
    ///     // … your async work here, checking token.cancelled().await …
    /// });
    /// manager.add_task(Box::new(my_task));
    /// ```
    pub fn add_task(&self, task: impl CancellableTask) {
        let mut guard = self.inner.lock().expect("Mutex poisoned");
        guard.push(Box::new(task));
    }

    /// Shut everything down. This takes all tasks out of the internal Vec,
    /// calls `cancel()` on each one, then `.await`s each `.join()`. Because
    /// we drain the Vec in one go, we never hold the `MutexGuard` across `.await`.
    pub async fn shutdown(&self) {
        // 1. Grab the lock and replace the Vec with an empty one, so we can drop the lock.
        let tasks_to_cancel: Vec<Box<dyn CancellableTask + Send>> = {
            let mut guard = self.inner.lock().expect("Mutex poisoned");
            // Use `std::mem::take` to replace `*guard` with a brand‐new Vec,
            // returning the old Vec. This ensures we do not hold the lock
            // while we `.await` on each task.
            std::mem::take(&mut *guard)
        };

        // 2. Cancel and join each task. We know `tasks_to_cancel` now owns all the tasks.
        for task in &tasks_to_cancel {
            task.cancel();
        }

        for task in tasks_to_cancel {
            task.join().await;
        }
        // At this point, all tasks have been signaled to cancel, and we have awaited them.
    }
}
