//! Side-effecting work container returned from `init`/`update`.

use alloc::{boxed::Box, vec::Vec};
use core::{
    future::{Future, pending, poll_fn},
    pin::Pin,
    task::Poll,
};

use crate::application::BoxFuture;

/// A side-effecting future (or set of them) the runtime drives.
///
/// `Task` is the runtime's pending-work container: returned from
/// [`crate::Application::init`] and [`crate::Application::update`], merged into
/// the runtime's active task, and polled internally. Internally it stores a
/// flat list of boxed futures; constructors flatten nested batches on
/// insertion.
///
/// Constructors:
/// - Task::none - no work.
/// - Task::perform - drive a `Future<Output = Option<M>>`.
/// - Task::future - drive a `Future<Output = M>`.
/// - Task::batch - run multiple tasks concurrently.

pub struct Task<M> {
    pub(crate) futures: Vec<BoxFuture<Option<M>>>,
}

impl<M: 'static> Task<M> {
    /// No work.
    #[must_use]
    pub fn none() -> Self {
        Self {
            futures: Vec::new(),
        }
    }

    /// Run `fut`; produce its `Option<M>` result via `update`.
    #[must_use]
    pub fn perform<F>(fut: F) -> Self
    where
        F: Future<Output = Option<M>> + 'static,
    {
        let mut futures = Vec::with_capacity(1);
        futures.push(Box::pin(fut) as BoxFuture<Option<M>>);
        Self { futures }
    }

    /// Run `fut`; always produce its `M` result via `update`.
    #[must_use]
    pub fn future<F>(fut: F) -> Self
    where
        F: Future<Output = M> + 'static,
    {
        Self::perform(async move { Some(fut.await) })
    }

    /// Combine multiple tasks. All futures are driven concurrently;
    /// resulting messages flow through `update` independently. Nested
    /// batches are flattened.
    #[must_use]
    pub fn batch(tasks: impl IntoIterator<Item = Task<M>>) -> Self {
        let mut futures: Vec<BoxFuture<Option<M>>> = Vec::new();
        for task in tasks {
            futures.extend(task.futures);
        }

        Self { futures }
    }

    /// True if this task has no pending futures.
    pub(crate) fn is_empty(&self) -> bool {
        self.futures.is_empty()
    }

    /// Merge `other`'s futures into `self`.
    /// Equivalent to `*self = Task::batch([core::mem::take(self), other])`
    /// without reallocating.
    pub(crate) fn extend(&mut self, other: Task<M>) {
        self.futures.extend(other.futures)
    }

    /// Poll all pending futures concurrently; return the first to
    /// complete and remove it from the task. If empty, pends forever;
    /// save to use a `select` arm.
    pub(crate) async fn next(&mut self) -> Option<M> {
        if self.is_empty() {
            return pending::<Option<M>>().await;
        }

        poll_fn(|cx| {
            let mut idx = 0;
            while idx < self.futures.len() {
                match Pin::as_mut(&mut self.futures[idx]).poll(cx) {
                    Poll::Ready(result) => {
                        let _ = self.futures.swap_remove(idx);
                        return Poll::Ready(result);
                    }
                    Poll::Pending => idx += 1,
                }
            }

            Poll::Pending
        })
        .await
    }
}

impl<M> Default for Task<M> {
    fn default() -> Self {
        Self {
            futures: Vec::new(),
        }
    }
}
