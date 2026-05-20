//! Time-based subscription primitives.
//!
//! [`every`] is the common case — fire a message every N. It wraps the
//! built-in [`Tick`] recipe so users don't have to define their own.

use crate::application::{BoxFuture, Recipe, Subscription};
use alloc::boxed::Box;
use core::hash::{Hash, Hasher};
use embassy_time::{Duration, Timer};

/// Periodic-tick [`Recipe`]: emit `message.clone()` every `duration`.
///
/// Identity = `TypeId::of::<Tick<M>>()` ⊕ `Hash` of `duration` and
/// `message`, so two ticks with different durations or different
/// messages are distinct subscriptions.
///
/// Users usually call [`every`] rather than constructing this
/// directly.
pub struct Tick<M>
where
    M: Clone + 'static,
{
    /// Tick period.
    pub duration: Duration,
    /// Message to emit on each tick.
    pub message: M,
}

impl<M> Hash for Tick<M>
where
    M: Clone + Hash + 'static,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.duration.as_ticks().hash(state);
        self.message.hash(state);
    }
}

impl<M> Recipe for Tick<M>
where
    M: Clone + Hash + 'static,
{
    type Message = M;

    fn next(&mut self) -> Option<BoxFuture<M>> {
        let duration = self.duration;
        let message = self.message.clone();
        Some(Box::pin(async move {
            Timer::after(duration).await;
            message
        }))
    }
}

/// Emit `message` every `duration`.
///
/// Equivalent to `Subscription::from_recipe(Tick { duration, message })`.
///
/// ```ignore
/// fn subscription(&self) -> Subscription<Msg> {
///     if self.auto {
///         zest_core::time::every(Duration::from_secs(1), Msg::Tick)
///     } else {
///         Subscription::none()
///     }
/// }
/// ```
#[must_use]
pub fn every<M>(duration: Duration, message: M) -> Subscription<M>
where
    M: Clone + Hash + 'static,
{
    Subscription::from_recipe(Tick { duration, message })
}
